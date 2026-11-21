//! Draws preview text with the font's own outlines, in background threads, into images
//! the UI shows as textures. Only what is on screen is requested.
use ab_glyph::{Font, FontArc, FontVec, Glyph, PxScale, ScaleFont};
use parking_lot::{Condvar, Mutex};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RenderKey {
    pub font_id: String,
    pub text: String,
    /// Size in physical pixels.
    pub px: u32,
    /// Widest image in physical pixels (text is cut there, as on the cards).
    pub max_w: u32,
    /// Letter spacing in hundredths of an em.
    pub spacing: i32,
}

pub struct Job {
    pub key: RenderKey,
    /// Resolves the font file; for Google Fonts this may download it first.
    pub locate: Box<dyn FnOnce() -> Option<(PathBuf, u32)> + Send>,
}

pub enum Output {
    Image(egui::ColorImage),
    Failed,
    /// Scrolled out of view before its turn: not drawn (asked again if it comes back).
    Skipped,
}

struct Shared {
    queue: Mutex<VecDeque<Job>>,
    cv: Condvar,
    done: Mutex<Vec<(RenderKey, Output)>>,
    fonts: Mutex<HashMap<(PathBuf, u32), FontArc>>,
    /// The UI frame counter and the last frame each pending preview was still on screen.
    frame: std::sync::atomic::AtomicU64,
    wanted: Mutex<HashMap<RenderKey, u64>>,
}

pub struct Renderer {
    shared: Arc<Shared>,
    pending: HashSet<RenderKey>,
    pub textures: HashMap<RenderKey, Option<(egui::TextureHandle, egui::Vec2)>>,
    order: VecDeque<RenderKey>,
    born: HashMap<RenderKey, f64>,
}

const MAX_TEXTURES: usize = 600;
const MAX_QUEUE: usize = 160;

impl Renderer {
    pub fn new(ctx: &egui::Context) -> Self {
        let shared = Arc::new(Shared {
            queue: Mutex::new(VecDeque::new()),
            cv: Condvar::new(),
            done: Mutex::new(Vec::new()),
            fonts: Mutex::new(HashMap::new()),
            frame: std::sync::atomic::AtomicU64::new(0),
            wanted: Mutex::new(HashMap::new()),
        });
        let workers = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).clamp(2, 6);
        for _ in 0..workers {
            let s = shared.clone();
            let ctx = ctx.clone();
            std::thread::spawn(move || loop {
                let job = {
                    let mut q = s.queue.lock();
                    while q.is_empty() {
                        s.cv.wait(&mut q);
                    }
                    // Newest first: what the person scrolled to most recently.
                    q.pop_back().unwrap()
                };
                let key = job.key.clone();
                let now = s.frame.load(std::sync::atomic::Ordering::Relaxed);
                let last = s.wanted.lock().get(&key).copied().unwrap_or(0);
                if last + 3 < now {
                    s.done.lock().push((key, Output::Skipped));
                    continue;
                }
                let out = match (job.locate)() {
                    Some((path, idx)) => match load_font(&s, &path, idx) {
                        Some(f) => {
                            let text = if key.text == crate::ui_main::ICON_MARK {
                                codepoints(&f, 40)
                                    .into_iter()
                                    .filter(|c| (*c as u32) >= 0xE000)
                                    .take(16)
                                    .flat_map(|c| [c, ' '])
                                    .collect::<String>()
                            } else {
                                key.text.clone()
                            };
                            rasterize(&f, &text, key.px as f32, key.max_w, key.spacing as f32 / 100.0)
                            .map(Output::Image)
                            .unwrap_or(Output::Failed)
                        }
                        None => Output::Failed,
                    },
                    None => Output::Failed,
                };
                s.done.lock().push((key, out));
                ctx.request_repaint();
            });
        }
        Self { shared, pending: HashSet::new(), textures: HashMap::new(), order: VecDeque::new(), born: HashMap::new() }
    }

    /// Returns the texture for `key`, queueing it when missing. `None` while it is being drawn.
    pub fn get(&mut self, key: &RenderKey, locate: impl FnOnce() -> Option<(PathBuf, u32)> + Send + 'static)
        -> Option<Option<(egui::TextureHandle, egui::Vec2)>> {
        if let Some(t) = self.textures.get(key) {
            return Some(t.clone());
        }
        let frame = self.shared.frame.load(std::sync::atomic::Ordering::Relaxed);
        self.shared.wanted.lock().insert(key.clone(), frame);
        if !self.pending.contains(key) {
            self.pending.insert(key.clone());
            let mut q = self.shared.queue.lock();
            q.push_back(Job { key: key.clone(), locate: Box::new(locate) });
            while q.len() > MAX_QUEUE {
                if let Some(old) = q.pop_front() {
                    self.pending.remove(&old.key);
                }
            }
            self.shared.cv.notify_one();
        }
        None
    }

    /// 0→1 over 200 ms after a preview arrives, so it fades in instead of popping.
    pub fn fade_in(&self, key: &RenderKey, ctx: &egui::Context) -> f32 {
        let now = ctx.input(|i| i.time);
        let t = self.born.get(key).map(|b| ((now - b) as f32 / crate::theme::ANIM).clamp(0.0, 1.0)).unwrap_or(1.0);
        if t < 1.0 {
            ctx.request_repaint();
        }
        t
    }

    pub fn collect(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        let frame = self.shared.frame.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        if frame.is_multiple_of(120) {
            self.shared.wanted.lock().retain(|_, f| *f + 60 > frame);
        }
        let done: Vec<_> = std::mem::take(&mut *self.shared.done.lock());
        for (key, out) in done {
            self.pending.remove(&key);
            let entry = match out {
                Output::Skipped => continue,
                Output::Image(img) => {
                    let size = egui::vec2(img.size[0] as f32, img.size[1] as f32);
                    let tex = ctx.load_texture(
                        format!("pv-{}-{}", key.font_id, key.px),
                        img,
                        egui::TextureOptions::LINEAR,
                    );
                    Some((tex, size))
                }
                Output::Failed => None,
            };
            self.textures.insert(key.clone(), entry);
            self.born.insert(key.clone(), now);
            self.order.push_back(key);
        }
        while self.order.len() > MAX_TEXTURES {
            if let Some(k) = self.order.pop_front() {
                self.textures.remove(&k);
                self.born.remove(&k);
            }
        }
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn forget_font(&mut self, font_id: &str) {
        self.textures.retain(|k, _| k.font_id != font_id);
    }

    pub fn font(&self, path: &PathBuf, idx: u32) -> Option<FontArc> {
        load_font(&self.shared, path, idx)
    }
}

fn load_font(s: &Shared, path: &PathBuf, idx: u32) -> Option<FontArc> {
    if let Some(f) = s.fonts.lock().get(&(path.clone(), idx)) {
        return Some(f.clone());
    }
    let data = std::fs::read(path).ok()?;
    let f = FontArc::new(FontVec::try_from_vec_and_index(data, idx).ok()?);
    let mut map = s.fonts.lock();
    if map.len() > 300 {
        map.clear();
    }
    map.insert((path.clone(), idx), f.clone());
    Some(f)
}

/// Lays out one line of text with the font's advances and kerning, and paints its coverage.
pub fn rasterize(font: &FontArc, text: &str, px: f32, max_w: u32, spacing_em: f32) -> Option<egui::ColorImage> {
    let scale = PxScale::from(px);
    let sf = font.as_scaled(scale);
    let ascent = sf.ascent();
    let descent = sf.descent();
    let height = (ascent - descent).ceil().max(1.0) as usize + 2;
    let mut x = 0.0f32;
    let mut prev = None;
    let mut glyphs: Vec<Glyph> = Vec::new();
    for ch in text.chars() {
        if ch.is_control() {
            continue;
        }
        let id = sf.glyph_id(ch);
        if let Some(p) = prev {
            x += sf.kern(p, id);
        }
        if x > max_w as f32 {
            break;
        }
        glyphs.push(id.with_scale_and_position(scale, ab_glyph::point(x, ascent + 1.0)));
        x += sf.h_advance(id) + spacing_em * px;
        prev = Some(id);
    }
    let width = (x.ceil() as usize + 2).clamp(1, max_w.max(1) as usize);
    let mut cov = vec![0f32; width * height];
    let mut drew = false;
    for g in glyphs {
        if let Some(o) = font.outline_glyph(g) {
            let b = o.px_bounds();
            o.draw(|gx, gy, c| {
                let px = b.min.x as i32 + gx as i32;
                let py = b.min.y as i32 + gy as i32;
                if px >= 0 && py >= 0 && (px as usize) < width && (py as usize) < height {
                    let i = py as usize * width + px as usize;
                    cov[i] = (cov[i] + c).min(1.0);
                    drew = true;
                }
            });
        }
    }
    if !drew {
        return None;
    }
    let pixels = cov
        .iter()
        .map(|c| {
            let a = (c * 255.0).round() as u8;
            egui::Color32::from_rgba_premultiplied(a, a, a, a)
        })
        .collect();
    Some(egui::ColorImage { size: [width, height], pixels })
}

/// Codepoints a font maps, for glyph grids and icon fonts.
pub fn codepoints(font: &FontArc, limit: usize) -> Vec<char> {
    let mut v: Vec<char> = font
        .codepoint_ids()
        .map(|(_, c)| c)
        .filter(|c| !c.is_control() && !c.is_whitespace())
        .collect();
    v.sort();
    v.dedup();
    v.truncate(limit);
    v
}
