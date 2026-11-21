---
version: 1
slug: "src-ui-main-rs"
primary_target: "src/ui_main.rs"
related_targets: ["src/theme.rs","src/ui_sidebar.rs","src/ui_webfont.rs","src/ui_panels.rs","src/ui_search.rs","src/ui_list.rs","src/ui_settings.rs","src/ui_icons.rs"]
---

# Kernship main window (all views)

Scope: the whole app UI drawn in `src/theme.rs` and `src/ui_*.rs`. Mode: Operate. Platform: native macOS, egui.
Audience and job: a mixed studio team browsing, activating and converting fonts for the web. Structure is fixed (sidebar, toolbar, grid/list, preview bar, right pane, search panel, settings); only the look changes. Both appearances ship. Interface font: Familjen Grotesk (user's choice), bundled in the binary.

## Direction contract

THESIS: Navigation is signage. One yellow rail tells you where you are, and yellow is used for nothing else. Refuses the graphite window with a system-blue accent.

OWN-WORLD: Light: saturated yellow rail (#FFD200) with black type and pictograms on black squares; white ground, warm-grey panels, black frames. Dark: near-black rail with yellow type, the current source as a full yellow band; near-black ground, off-white ink. Selection is a heavy ink frame, never a colour. Status is a shape (filled, half, ringed, empty) plus a word.

STORY: The user always knows where they are and what state each font is in, then acts: activate, organise, convert.

FIRST VIEWPORT: 210 pt rail at left with counts always visible, bold, right aligned. Toolbar: size slider, a sign band naming source and count, Grid/List, Web/Info, search. Font cards as flat panels, three or four across. Floating preview bar. Right pane with Convert as the one filled ink button at its foot.

SIGNATURE: the rail's current-source band inverts (black on yellow, yellow on black) and slides between rows; the toolbar sign band reads as a direction sign: source, arrow, selection, count.

FORM: challenger dealt by the roll and chosen by the user, evolved at their request with a dark appearance. seed:3c2ed88f3d977cba

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance
