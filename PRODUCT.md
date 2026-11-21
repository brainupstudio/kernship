# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

<!-- Platform note: Kernship is a native macOS desktop app drawn with Rust + eframe/egui (immediate-mode, custom-painted). The schema has no desktop value; `web` is the closest of the four and only means "not iOS/Android". No HTML/CSS ships. -->

## Users

A mixed studio team (confirmed 2026-11-20): designers who activate fonts for Adobe, Figma, Sketch and other design apps during daily client work, and web designers/developers who take the chosen fonts and deliver them as web fonts. Both share the same font library.

## Product Purpose

Kernship is a macOS font manager: it browses, organises, previews and activates fonts (libraries, sets, tags, smart filters, Google Fonts, icon sets, auto-activation for design apps) and converts fonts for the web.

Success for the current redesign (confirmed): the app looks like a finished, professional product with its own recognisable identity, not a generic template.

## Positioning

A font manager where the hand-off to the web is built in: any font in the library, including Google Fonts styles, becomes WOFF2/WOFF/TTF/EOT with a ready `@font-face` stylesheet, subsetting and a demo page, without leaving the app or installing anything.

## Operating Context

- Runs beside design apps all day; fonts are activated by clicking a status dot, by shortcut, or automatically when a document needs them.
- Typical session: pick a source in the sidebar, set preview text and size, scan many families, select, then activate, organise (sets, tags, star) or convert for the web.
- macOS only; dark and light appearance, following the system or forced.

## Capabilities and Constraints

- Rust, eframe/egui 0.31, everything custom-painted with egui's painter; icons are drawn with strokes, there is no icon font or image set.
- The interface is set in Familjen Grotesk (SIL OFL), compiled into the binary; the macOS system font stands behind it for scripts it lacks.
- Structure is fixed for the redesign (confirmed): left sidebar, toolbar, font grid / list, floating preview-text bar, right pane (Info / Convert for Web), search filter panel, settings window. Only the look may change: colour, typography, components, details.
- Every existing function and shortcut stays.
- Terminology in use: Library, Set, Tag, Smart Filter, Activate / Deactivate / Activate Temporarily, Starred, Convert for Web.
- `kernship --screenshot out.png` (with `KERNSHIP_SHOT_VIEW`) renders views headlessly for checking.

## Brand Commitments

- The name **Kernship** is binding (confirmed).
- Logo, logo colours and the tagline "A command centre for your font galaxy" are provisional and may be rethought later (confirmed). They are not constraints on the interface.

## Evidence on Hand

- Working app with real data: Google Fonts catalog (about 7,986 styles), system fonts, icon sets.
- Baseline screenshots of the current look in `shots/before/` (gitignored).
- No testimonials, pricing, customers or benchmarks exist; none may be invented.

## Product Principles

1. The fonts are the content; the interface never competes with a specimen.
2. Status is always readable at a glance: active, temporarily active, inactive, downloading.
3. The road from "this font" to "web-ready files" is short and explains itself.
4. Familiar structure, own character: the layout of a desktop font manager, with an identity of its own.
5. Native manners: macOS conventions for menus, shortcuts, selection and windows.

## Accessibility & Inclusion

No formal standard was set. Working target adopted for the redesign (assumption, not confirmed): WCAG AA contrast for text and controls in both appearances, and state never carried by colour alone.
