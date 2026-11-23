# How UUOB's apps look and behave

One design for UUOB and every app, on the Mac (SwiftUI), Linux (GTK 4 and
Libadwaita) and Android (Compose). Calendar's event card and editor are the
model. The kit in `kit/` builds these pieces, so an app rarely needs its own:

| Piece | Mac (`kit/mac`) | Linux (`kit/linux`) | Android (`kit/android`) |
|---|---|---|---|
| Sidebar, rail, drawer | `SidebarScaffold`, `SidebarRow`, `RailButton` | `SidebarScaffold`, `sidebar_row`, `rail_button` | `SidebarScaffold`, `SidebarRow`, `RailButton` |
| Page top | `Toolbar`, `PlusButton`, `titleText` | `toolbar`, `plus_button` | `Toolbar`, `PlusButton` |
| Detail card | `DetailSheet` | `detail_sheet` | `DetailSheet` |
| Editor | `EditSheet` | `edit_sheet` | `EditSheet` |
| Rows | `IconRow`, `ItemRow`, `RowsLabel` | `icon_row`, `item_row`, `rows_label` | `IconRow`, `ItemRow`, `RowsLabel` |
| Empty | `EmptyState` | `empty_state` | `EmptyState` |
| Settings | `SettingsSheet` | `settings_dialog` | `SettingsSheet` |
| Tints | `tint(kind)` | `tint(kind)` | `tint(kind)` |
| Shortcuts and palette | `UuobCommands`, `.uuobKeys`, `CommandPalette` | `Shortcuts::install`, `command_palette` | `ShortcutsHost`, `CommandPalette` |

## Measures

- **Spacing:** 4, 8, 12, 14, 18, 22, 28. Sheets have 22 inside, pages 28 at the sides (14 in a narrow window).
- **Type:**

  | Use | Size |
  |---|---|
  | Page title | 34 bold (26 narrow, 28 on phones) |
  | Sheet title | 22 semibold |
  | Row text | 15 |
  | List item | 16 |
  | Grey lines | 13 |
  | Section labels | 12 semibold, capitals |

  A grey part of a title (a year, a count) is the same size, regular weight.
- **Corners:**

  | Element | Radius |
  |---|---|
  | Sheets | 26 (Android) or the system's |
  | Cards | 18 |
  | Fields and segmented track | 12 / 10 |
  | Rows and rail tiles | 10 / 12 |
  | Events | 7 |
  | Buttons | Pill |

- **Icons:**

  | Use | Size |
  |---|---|
  | In rows | 19–20 |
  | Rail | 20 in a 40 tile |
  | Icon buttons | 20 in a 36 circle |
  | Inside text | 11–12 |
  | Stroke | 1.75–1.8 (2.2 on +) |

- **Rows:** icon rows at least 44 (46 on Android), sidebar rows 36 (40), list items about 46. A hairline under each row except the last.
- **Sheets:** 460 wide, at most 640 tall, as tall as their content.

## Colours and tints

- **Neutrals:** light and dark greys from `Look` / `Tokens` (`bg`, `elevated`, `surface2`, `rail`, `line`, `text2`, `text3`), and one blue accent. Text always follows light and dark.
- **The eight calendar colours** (blue, red, orange, yellow, green, teal, purple, pink) are for things the user names: calendars, lists, apps.
- **The colour a list or calendar gives:** it tints everything on its page, including the +, ticks and field edges (Tasks).
- **Item tints**, quiet shades of the item's colour (`tint(kind)`):

  | Kind | Fill | Edge | Whole item |
  |---|---|---|---|
  | Plain | 24% | 100% | 100% |
  | All-day | 22% | 100% | 100% |
  | Joinable (a meeting link) | +8%, and a camera by the time | 100% | 100% |
  | Repeating | −9%, arrows by the time | 55% | 100% |
  | Past | — | — | 60% |
  | Declined | — | — | 45%, struck through |
  | Tentative ("maybe") | Stripes | — | — |
  | Not answered | Outline only | — | — |

## Sidebar

Every app with more than one place has one: Calendar's calendars, Tasks' lists, Photos' albums, UUOB's pages.

- **Expanded:** 240 wide, icons or colour dots with names, small grey section labels.
- **Rail:** 64 wide, icons only in 40 tiles, names as tooltips. It shows by itself below 600 wide, or when folded with the toggle at the bottom of the sidebar (⌃⌘S). The choice is remembered on that device.
- **In a narrow window:** the toggle opens the expanded sidebar over the content.
- **Phones:** a drawer, hidden until the menu button in the toolbar or a swipe from the left edge. It looks like the expanded sidebar. UUOB keeps its three bottom tabs instead.

## Sheets

- **Detail card:** the title, its actions (Edit…) and ✕ on the top line, then lines and icon rows.
- **Editor:** the title editable at the top with ✕ and a line under it (what it belongs to), then icon rows with flat controls: no boxes, the hairlines are enough.
  - The bottom has Delete on the left; Cancel (only for something new) and Save/Add on the right, all small.
  - Esc cancels, ↩ in the title or ⌘↩ anywhere saves.
- **Settings:** the same rows. All-your-apps choices first, then the app's own. The version is a small grey line at the end. There are no explanatory notes.
- Always a way out: ✕ or Cancel, Esc, a tap outside on phones.

## Text

- Never a letter a line: shorten with "…", drop the less important part (the weekday, the time), or show less (fewer days, the rail).
- No paragraphs that explain the obvious. A warning or a question stays ("This can't be undone", "Only this one, or all of them?").
- Empty places say what's there in a few words, with at most one button.

## Keyboard shortcuts

Linear is the model:
- **Every action is in a command palette** (⌘K / Ctrl+K). Type a few letters and press ↩. Each action shows its shortcut there.
- **Single letters work wherever no field is typing:** C for new, / to find, ? for the list.
- **Tooltips and menus name their shortcut.**
- **Esc always leaves** a sheet, the palette or a menu.

Computers (⌘ on the Mac, Ctrl on Linux), and Android with a keyboard:

| What | Mac | Linux / Android |
|---|---|---|
| Command palette | ⌘K | Ctrl+K |
| Settings | ⌘, | Ctrl+, |
| New (event, task, photos, device) | ⌘N or C | Ctrl+N or C |
| Find | ⌘F or / | Ctrl+F or / |
| Toggle sidebar | ⌃⌘S | Ctrl+Alt+S |
| Refresh | ⌘R | Ctrl+R |
| Save in an editor | ⌘↩ | Ctrl+Enter |
| Close or cancel | Esc, ⌘W | Esc |
| Delete what's selected | ⌘⌫ | Delete |
| Keyboard shortcuts | ⌘? or ? | Ctrl+? or ? (Ctrl+/ on Android) |
| Pages or views | ⌘1–4 | Ctrl+1–4 |

- **Calendar:** ← → previous and next and T today, where no field is typing; ⌘← ⌘→ ⌘T in the menu (Alt+← Alt+→ Ctrl+T on Linux).
- **Photos:** ← → previous and next, Space opens, ⌘S save, ⌘C copy, ⌘I info, ⌫ delete.

On the Mac they are in the menu bar (Settings in the app menu). On Linux they are window actions with accelerators, and a list. On Android they work with a hardware keyboard, and Ctrl+/ lists them.
