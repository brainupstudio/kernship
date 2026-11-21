---
name: Kernship
description: A macOS font manager whose navigation is signage, with one yellow rail and ink for everything else.
colors:
  dark-ground: "#101010"
  dark-panel: "#1C1C1B"
  dark-panel-hi: "#282826"
  dark-sel-fill: "#3A3A36"
  dark-field: "#161615"
  dark-menu: "#20201F"
  dark-rail: "#191918"
  dark-rail-ink: "#F2F1EC"
  dark-rail-dim: "#A6A59E"
  dark-rail-hover: "#282826"
  dark-rail-icon: "#FFD200"
  dark-band: "#FFD200"
  dark-band-ink: "#111111"
  dark-ink: "#F2F1EC"
  dark-dim: "#A6A59E"
  dark-faint: "#8C8B84"
  dark-rule: "#34342F"
  dark-rule-strong: "#5C5C55"
  dark-danger: "#FF7A6B"
  light-ground: "#FFFFFF"
  light-panel: "#F3F3F0"
  light-panel-hi: "#E8E8E4"
  light-sel-fill: "#D6D6D0"
  light-field: "#FFFFFF"
  light-menu: "#FFFFFF"
  light-rail: "#FFD200"
  light-rail-ink: "#111111"
  light-rail-dim: "#3D3300"
  light-rail-hover: "#F0C200"
  light-rail-icon: "#111111"
  light-band: "#111111"
  light-band-ink: "#FFD200"
  light-ink: "#111111"
  light-dim: "#55554F"
  light-faint: "#6C6C65"
  light-rule: "#D8D8D3"
  light-rule-strong: "#96968F"
  light-danger: "#BE281C"
typography:
  headline:
    fontFamily: "Familjen Grotesk, SF Pro, Helvetica, sans-serif"
    fontSize: "18px"
    fontWeight: 700
  title:
    fontFamily: "Familjen Grotesk, SF Pro, Helvetica, sans-serif"
    fontSize: "16px"
    fontWeight: 700
  section:
    fontFamily: "Familjen Grotesk, SF Pro, Helvetica, sans-serif"
    fontSize: "14px"
    fontWeight: 700
  field:
    fontFamily: "Familjen Grotesk, SF Pro, Helvetica, sans-serif"
    fontSize: "14px"
    fontWeight: 400
  body:
    fontFamily: "Familjen Grotesk, SF Pro, Helvetica, sans-serif"
    fontSize: "13px"
    fontWeight: 400
  small:
    fontFamily: "Familjen Grotesk, SF Pro, Helvetica, sans-serif"
    fontSize: "11.5px"
    fontWeight: 400
  label:
    fontFamily: "Familjen Grotesk, SF Pro, Helvetica, sans-serif"
    fontSize: "13px"
    fontWeight: 500
  rail-row:
    fontFamily: "Familjen Grotesk, SF Pro, Helvetica, sans-serif"
    fontSize: "13.5px"
    fontWeight: 500
  count:
    fontFamily: "Familjen Grotesk, SF Pro, Helvetica, sans-serif"
    fontSize: "13px"
    fontWeight: 700
  action:
    fontFamily: "Familjen Grotesk, SF Pro, Helvetica, sans-serif"
    fontSize: "13.5px"
    fontWeight: 700
  group-label:
    fontFamily: "Familjen Grotesk, SF Pro, Helvetica, sans-serif"
    fontSize: "12px"
    fontWeight: 700
  status-word:
    fontFamily: "Familjen Grotesk, SF Pro, Helvetica, sans-serif"
    fontSize: "11.5px"
    fontWeight: 700
rounded:
  none: "0px"
  chip: "4px"
  control: "5px"
  card: "6px"
  float: "8px"
spacing:
  xs: "6px"
  sm: "8px"
  md: "12px"
  lg: "16px"
  pane: "18px"
components:
  rail:
    backgroundColor: "{colors.dark-rail}"
    textColor: "{colors.dark-rail-ink}"
    width: "210px"
    rounded: "{rounded.none}"
  rail-light:
    backgroundColor: "{colors.light-rail}"
    textColor: "{colors.light-rail-ink}"
    width: "210px"
    rounded: "{rounded.none}"
  rail-row:
    textColor: "{colors.dark-rail-ink}"
    typography: "{typography.rail-row}"
    height: "32px"
    rounded: "{rounded.none}"
  rail-row-hover:
    backgroundColor: "{colors.dark-rail-hover}"
  rail-row-current:
    backgroundColor: "{colors.dark-band}"
    textColor: "{colors.dark-band-ink}"
    height: "32px"
    rounded: "{rounded.none}"
  rail-row-current-light:
    backgroundColor: "{colors.light-band}"
    textColor: "{colors.light-band-ink}"
    height: "32px"
    rounded: "{rounded.none}"
  rail-pictogram:
    backgroundColor: "{colors.dark-rail-icon}"
    textColor: "{colors.dark-rail}"
    size: "20px"
    rounded: "{rounded.chip}"
  sign-band:
    backgroundColor: "{colors.dark-ink}"
    textColor: "{colors.dark-ground}"
    typography: "{typography.label}"
    height: "32px"
    rounded: "{rounded.control}"
    padding: "0 12px"
  tool-button:
    backgroundColor: "{colors.dark-panel}"
    textColor: "{colors.dark-ink}"
    size: "32px"
    rounded: "{rounded.control}"
  tool-button-hover:
    backgroundColor: "{colors.dark-panel-hi}"
  tool-button-on:
    backgroundColor: "{colors.dark-ink}"
    textColor: "{colors.dark-ground}"
  button-primary:
    backgroundColor: "{colors.dark-ink}"
    textColor: "{colors.dark-ground}"
    typography: "{typography.action}"
    rounded: "{rounded.control}"
    height: "28px"
    padding: "0 14px"
  button-plain:
    backgroundColor: "{colors.dark-ground}"
    textColor: "{colors.dark-ink}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    height: "28px"
    padding: "0 14px"
  button-plain-hover:
    backgroundColor: "{colors.dark-panel-hi}"
  button-danger:
    backgroundColor: "{colors.dark-ground}"
    textColor: "{colors.dark-danger}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    height: "28px"
    padding: "0 14px"
  toggle:
    textColor: "{colors.dark-ink}"
    rounded: "{rounded.control}"
    height: "24px"
    padding: "0 6px"
  toggle-on:
    backgroundColor: "{colors.dark-ink}"
    textColor: "{colors.dark-ground}"
  font-card:
    backgroundColor: "{colors.dark-panel}"
    textColor: "{colors.dark-ink}"
    rounded: "{rounded.card}"
  font-card-hover:
    backgroundColor: "{colors.dark-panel-hi}"
  font-card-selected:
    backgroundColor: "{colors.dark-ground}"
    textColor: "{colors.dark-ink}"
    rounded: "{rounded.card}"
  list-row-selected:
    backgroundColor: "{colors.dark-sel-fill}"
    textColor: "{colors.dark-ink}"
    rounded: "{rounded.chip}"
  chip:
    backgroundColor: "{colors.dark-panel}"
    textColor: "{colors.dark-ink}"
    rounded: "{rounded.chip}"
    height: "24px"
    padding: "0 9px"
  chip-on:
    backgroundColor: "{colors.dark-ink}"
    textColor: "{colors.dark-ground}"
  search-field:
    backgroundColor: "{colors.dark-field}"
    textColor: "{colors.dark-ink}"
    typography: "{typography.field}"
    rounded: "{rounded.control}"
    height: "32px"
  checkbox:
    backgroundColor: "{colors.dark-field}"
    size: "16px"
    rounded: "{rounded.chip}"
  checkbox-on:
    backgroundColor: "{colors.dark-ink}"
    textColor: "{colors.dark-ground}"
  menu:
    backgroundColor: "{colors.dark-menu}"
    textColor: "{colors.dark-ink}"
    typography: "{typography.body}"
    rounded: "{rounded.float}"
    width: "236px"
    padding: "5px"
  menu-row-hover:
    backgroundColor: "{colors.dark-ink}"
    textColor: "{colors.dark-ground}"
    height: "24px"
  preview-bar:
    backgroundColor: "{colors.dark-ground}"
    textColor: "{colors.dark-ink}"
    rounded: "{rounded.float}"
    height: "75px"
    width: "520px"
  toast:
    backgroundColor: "{colors.dark-ink}"
    textColor: "{colors.dark-ground}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    height: "30px"
    padding: "0 14px"
---

# Design System: Kernship

## Overview

**Creative North Star: "Yellow Wayfinding"**

Kernship borrows the grammar of airport sign systems: navigation is signage. One rail at the left edge tells you where you are, and its yellow is spent on nothing else. Everything to the right of the rail is ink on ground: flat grey panels, ink frames, and an ink fill for whatever is switched on. The fonts are the content, so the interface carries no second accent and no decoration that could compete with a specimen.

The system ships in two appearances that are one design inverted, not two themes. In light the rail is a saturated yellow sign with black lettering and the current source is a black band; in dark the rail is near-black with off-white lettering, yellow pictograms and status marks, and the current source is a full yellow band. The direction was chosen by the user from a dealt hand (catalog world `wayfinding-cartography-signage-terminal-yellow-wayfinding`) and evolved at their request with the dark appearance. The tokens below are read from the shipped source, `src/theme.rs`.

State is never left to colour. A font's activation is a shape plus a word, a selected card is a heavy ink frame, and a control that is on is an ink fill with ground lettering. The system refuses the graphite window with a system-blue accent.

**Key Characteristics:**
- One yellow rail, 210 pt wide, running to the window edges; yellow appears nowhere else.
- Ink on ground everywhere else: flat panels, 1 pt hairlines, ink frames, ink fills.
- Status is a shape plus a word: filled disc, half disc, disc in ring, empty ring, open ring.
- The current source is an inverted band that slides between rail rows.
- A sign band in the toolbar reads like a direction sign: source, arrow, selection, count.
- One typeface, Familjen Grotesk, in three weights; sizes sit between 11.5 and 18 pt.
- Every state change fades over 200 ms; nothing bounces or scales.

## Colors

Two neutral scales and one signal yellow; each role exists once per appearance, and the app blends between the two over 200 ms when the appearance changes.

### Primary
- **Signal Yellow** (`light-rail`, `dark-rail-icon`, `dark-band`, `light-band-ink`): the rail's colour. In light it is the rail's ground; in dark it is the rail's pictogram squares, the status marks of the activation tally and the current-source band. It never leaves the rail.
- **Pressed Yellow** (`light-rail-hover`): the light rail under the pointer.
- **Sign Brown** (`light-rail-dim`): group labels on the light rail ("Libraries", "Tags", "Activation"); in dark they take Rail Dim, a neutral grey.
- **Rail Hover** (`dark-rail-hover`): the dark rail under the pointer, a neutral near-black.

### Neutral
- **Ground** (`dark-ground`, `light-ground`): the window behind the fonts, and the lettering on any ink fill.
- **Panel** (`dark-panel`, `light-panel`): font cards, toolbar controls, the right pane, the list header.
- **Panel High** (`dark-panel-hi`, `light-panel-hi`): a panel under the pointer, quiet chips, segmented tracks.
- **Selection Grey** (`dark-sel-fill`, `light-sel-fill`): selected rows of a list or table, and selected text in a field.
- **Field** (`dark-field`, `light-field`): text fields, code, the empty checkbox.
- **Menu** (`dark-menu`, `light-menu`): menus, popovers, dialogs, the settings window.
- **Rail Black** (`dark-rail`): the dark rail's ground, parted from the fonts by one hairline.
- **Rail Ink** (`dark-rail-ink`, `light-rail-ink`): names and chevrons on the rail, off-white in dark, near-black in light. Numbers on the rail take Rail Ink at 70% opacity.
- **Rail Icon** (`dark-rail-icon`, `light-rail-icon`): pictogram squares and the tally's status marks; yellow in dark, near-black in light.
- **Ink** (`dark-ink`, `light-ink`, `dark-band-ink`, `light-rail-ink`, `light-band`): lettering, frames, and the fill of everything that is on. Off-white in dark, near-black in light.
- **Dim** (`dark-dim`, `light-dim`): secondary lettering, the Off ring and word, hints, idle icons.
- **Faint** (`dark-faint`, `light-faint`): disabled lettering and tertiary notes.
- **Rule** (`dark-rule`, `light-rule`): 1 pt hairlines between rows and sections.
- **Rule Strong** (`dark-rule-strong`, `light-rule-strong`): the frame of fields, popovers and windows, slider tracks.

### Tertiary
- **Danger** (`dark-danger`, `light-danger`): the frame and lettering of a button that deletes, and error text. It is never a fill.

### Named Rules
**The Rail Only Rule.** Yellow belongs to the rail and to nothing else. A yellow button, badge, link or focus ring anywhere right of the rail is a defect.

**The Ink Is On Rule.** What is on is an ink fill with ground lettering: the active toolbar button, the current view, the tab, the sample in use, a ticked checkbox, the menu row under the pointer, the primary button. Nothing that is off may take an ink fill.

**The Shape And Word Rule.** State is never carried by colour. A font's status is a mark and its word (Active filled disc, Temporary half disc, Auto disc in ring, Off empty ring, Downloading open ring), drawn in ink.

## Typography

**Display Font:** Familjen Grotesk Bold (the macOS system font stands behind it for scripts it lacks)
**Body Font:** Familjen Grotesk Regular and Medium (same fallback)

**Character:** One grotesque in three weights does every job, the way a sign system uses one alphabet. Weight, not size, separates what you read from what you count: names are Medium, numbers and states that are on are Bold. The family is SIL OFL and compiled into the binary from `assets/fonts`.

### Hierarchy
- **Headline** (Bold, 18 pt): the title of an empty view ("No Fonts").
- **Title** (Bold, 16 pt): the family name at the head of the Info pane; egui's heading style.
- **Section** (Bold, 14 pt): family names above their cards, icon set names, settings headings and the settings title.
- **Field** (Regular, 14 pt): typed text in the search field and the preview-text field.
- **Body** (Regular, 13 pt): menus, list cells, notes. List secondaries drop to 12.5 pt.
- **Small** (Regular, 11.5 pt): the Off status word, minor annotations.
- **Label** (Medium, 13 pt): plain buttons, the sign band's place and family, toasts, chips. Tabs and sample chips use 12.5 pt; card names use 12 pt.
- **Rail Row** (Medium, 13.5 pt): source names on the rail.
- **Count** (Bold, 13 pt): numbers on the rail and the count at the right of the sign band. The tally at the rail's foot uses 12.5 pt. On the rail, counts sit at 70% opacity so names lead.
- **Action** (Bold, 13.5 pt): the lettering of a primary button.
- **Group Label** (Bold, 12 pt, sentence case, no tracking): "Libraries", "Tags", "Activation" on the rail, and section heads in the Convert pane, each sitting above a group it names.
- **Status Word** (Bold, 11.5 pt when on; Regular in Dim when Off): the word beside a status mark; list column heads use the same size in Medium, Bold when sorted.

### Named Rules
**The One Alphabet Rule.** The interface is set in Familjen Grotesk only. Other faces appear solely as content: specimens, and the letter samples that stand for a font category.

**The Bold Counts Rule.** Numbers and live states are Bold; names are Medium; prose is Regular. No uppercase styling, no letter-spacing.

## Layout

A fixed shell of edge-to-edge regions. The rail is 210 pt wide by default (resizable from 170 to 380 pt) and runs the full height with no inset. The toolbar is one 52 pt strip whose controls are all 32 pt high: size slider, sign band (it takes the remaining width and is dropped below 150 pt), style bar, Grid/List, Convert, Info, search (up to 260 pt). The right pane is 260 to 520 pt wide with an 18 by 16 pt inset. The preview bar floats over the content, centred near its foot, 520 by 75 pt.

Rail rows are 32 pt high; each nesting level indents 14 pt; a 20 pt pictogram square sits before the name and the number is right-aligned 12 pt from the edge. Names too long for their row lose their middle, never their end. The activation tally occupies the last 104 pt of the rail under a rail-ink rule and is dropped when the rail is shorter than 420 pt.

The grid's rhythm is 12 pt: 12 pt between cards and 12 pt side padding, three or four cards across at the default size. Widgets are spaced 8 by 6 pt; dialogs and windows have a 16 pt margin; menus a 5 pt margin with 24 pt rows. Density is that of a working tool: nothing is padded for show.

## Elevation & Depth

Flat by default. Depth is tonal: ground, then panel, then panel-high on hover, with 1 pt hairlines where two regions of the same tone meet. Only layers that genuinely float over content carry a shadow, and that shadow is a soft black ambient blur, never a hard offset.

### Shadow Vocabulary
- **Menu** (`0 6px 18px` black at alpha 110/255 in dark, 40/255 in light): context menus and dropdowns.
- **Window** (`0 10px 30px` black at alpha 120/255 in dark, 45/255 in light): dialogs and the settings window.
- **Popover** (`0 8px 24px` black at alpha 90/255): the search filter panel.
- **Preview bar** (an 18 pt blur of black at alpha 60/255, offset 8 pt down): the floating preview-text bar.

### Named Rules
**The Float Only Rule.** A shadow means "this is above the fonts and will go away or move": menus, popovers, dialogs, the preview bar. Cards, panes, the rail and the toolbar never cast one.

## Shapes

Small, consistent radii that step with the size of the object: square for the rail and its band (0), 4 pt for chips, checkboxes and pictogram squares, 5 pt for buttons, fields and toolbar controls, 6 pt for font cards and icon cells, 8 pt for anything that floats (menus, popovers, windows, the preview bar). Menu rows highlight with a 6 pt radius.

Lines are 1 pt hairlines. The two heavier strokes have fixed meanings: a 2.5 pt ink frame inside a card means selected, and the keyboard focus ring is a 2 pt ground gap plus a 2 pt ink ring outside the control. Pictograms and toolbar icons are drawn with 1.25 to 1.5 pt strokes, about 13 pt across; there is no icon font and no image set. Status marks are discs and rings of 4.5 to 5 pt radius.

## Components

### Rail
- **Character:** the sign. Edge to edge, no radius, no inset.
- **Row:** 32 pt; disclosure chevron, 20 pt pictogram square (4 pt radius) filled with Rail Icon and its glyph drawn in the rail colour, name in Medium 13.5, number in Bold 13 at 70% opacity at the right.
- **Hover:** the rail-hover fill across the full width.
- **Current source:** the inverted band, full width and square; lettering, square and glyph take the band's ink. The band glides to the new row in 160 ms on cubic-bezier(0.05, 0.7, 0.1, 1), fast out and soft in, and does not animate while scrolling.
- **Keyboard focus:** a 2 pt ring in the row's own lettering colour, inset 2 pt.
- **Tally:** at the foot, under a rail-ink hairline: Active, Temporary, Auto, each with its status mark in Rail Icon, word, and Bold count at 70% opacity.
- **Not downloaded:** a download arrow replaces the number.

### Sign Band
A 32 pt ink bar (5 pt radius) in the toolbar with ground lettering: place in Medium, then a drawn arrow and the family, then the style or "n fonts selected" in Bold; at the right, after a faint divider, the count in Bold ("7,986 styles", "… , 12 active"). A part that does not fit is left out with its arrow; the place always stays.

### Buttons
- **Shape:** 5 pt radius, 28 pt high, at least 72 pt wide (label plus 28 pt).
- **Primary:** ink fill, ground lettering in Bold 13.5; the one action a pane or dialog exists for. Full-width in a pane it is 38 pt high. Hover mixes the fill 35% toward Dim.
- **Plain:** ground fill, 1 pt ink frame, ink lettering in Medium 13; hover fills with Panel High. Full-width in a pane it is 32 pt high.
- **Danger:** as Plain, with frame and lettering in Danger.
- **Disabled:** 40% strength; no hover.

### Toolbar Buttons and Segments
- **Tool button:** a 32 pt Panel square (5 pt radius) with a stroked icon; Panel High on hover; ink fill with a ground icon when its pane or bar is showing.
- **Grid/List:** a 68 pt Panel track with an ink block that slides between the two halves in 200 ms.
- **Settings segments:** a Panel High track with an ink knob, 118 pt per option, Medium 13.
- **Size slider:** a Panel pill with a 2 pt track (Rule Strong, ink up to the knob) and a 14 pt ink knob.

### Toggles and Chips
- **Toggle (tabs, alignment):** a word 24 pt high; transparent at rest, Panel High on hover, ink fill with ground lettering when on.
- **Sample chip:** 24 pt, 4 pt radius, Panel; ink fill when its text is the preview text.
- **Filter chip and joined segments:** 26 pt, 4 pt radius, Panel High; Selection Grey on hover; ink fill when on.

### Font Cards and Icon Cells
- **Corner Style:** 6 pt.
- **Background:** Panel; Panel High on hover; no border and no shadow. Icon cells sit bare on Ground and take Panel on hover.
- **Header:** the status mark (5 pt radius) at the left, which is also the activation switch; the name in Medium 12; at the right the status word, Bold in ink when on and Regular in Dim when Off; a filled ink star when starred.
- **Selected:** the fill returns to Ground and a 2.5 pt ink frame is drawn inside the card.
- **Family header above cards:** name in Bold 14, style count in Dim at the right.

### List and Table Rows
- **Header:** 26 pt Panel strip with a hairline under it; column names in Medium 11.5 Dim, the sorted column in Bold ink.
- **Rows:** 4 pt radius; Panel High on hover; Selection Grey when selected. Status mark at the left, 4.5 pt radius.

### Inputs / Fields
- **Search:** 32 pt, Field fill, 1 pt Rule Strong frame, 5 pt radius, a stroked magnifier in Dim. On focus the frame thickens to 2 pt and turns ink. When filters are on, the magnifier sits on a 22 pt ink square.
- **Checkbox:** 16 pt, 4 pt radius; off is Field with a 1.5 pt ink frame; on is an ink fill with a 2 pt ground tick.
- **Text cursor and selection:** a 1.5 pt ink caret; selected text keeps its ink on Selection Grey.

### Menus and Popovers
- **Menu:** Menu fill, 1 pt Rule Strong frame, 8 pt radius, 236 pt wide (submenus 180 pt), 24 pt rows in Regular 13 with tick, icon and label columns; shortcuts in Dim at the right.
- **Row under the pointer:** an ink band (6 pt radius) with ground lettering.
- **Separator:** a Rule hairline inset 10 pt.
- **Search filter panel and settings window:** the same Menu fill, Rule Strong frame and 8 pt radius.

### Preview Bar
A floating bar, Ground fill with a 1 pt ink frame and 8 pt radius: a frameless text field above, a row of sample chips below.

### Toasts
A 30 pt ink pill (5 pt radius) with ground lettering in Medium 13, centred over the content; it rises 8 pt and fades in and out over 200 ms.

### Status Marks
Five marks drawn in ink, each always shown with its word where space allows: **Active** filled disc, **Temporary** half disc in a ring, **Auto** disc in a ring, **Off** empty ring (in Dim), **Downloading** open ring. The same marks stand as menu icons for Activate, Activate Temporarily and Deactivate.

## Do's and Don'ts

### Do:
- **Do** keep yellow on the rail: its ground in light, its pictograms, tally marks and current-source band in dark.
- **Do** show what is on as an ink fill with ground lettering, in both appearances.
- **Do** mark a selected card or icon cell with a 2.5 pt ink frame on Ground, and a selected list row with Selection Grey.
- **Do** pair every status mark with its word (Active, Temporary, Auto, Off, Downloading) wherever there is room.
- **Do** give each pane or dialog one primary ink-filled button, and frame every other action.
- **Do** set the interface in Familjen Grotesk Regular, Medium and Bold, with numbers and live states in Bold.
- **Do** fade every hover, selection and appearance change over 200 ms.
- **Do** draw icons with 1.25 to 1.5 pt strokes in the colour of the lettering beside them.
- **Do** show the keyboard focus ring (2 pt ground gap, 2 pt ink ring) for keyboard movement only.
- **Do** define every new colour role twice, once in each appearance.

### Don't:
- **Don't** use yellow for a button, link, badge, selection or focus ring outside the rail.
- **Don't** introduce a second accent colour or a system-blue highlight.
- **Don't** signal selection or state with hue alone.
- **Don't** fill a surface with Danger; it is a frame and lettering only.
- **Don't** give cards, panes, the rail or the toolbar a shadow or a border at rest.
- **Don't** set interface lettering in a face other than Familjen Grotesk.
- **Don't** let the interface compete with a specimen: no gradients, textures or decoration on the ground behind the fonts.
