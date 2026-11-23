# Inspection, 23 November 2026

Calendar, Tasks and Photos after moving onto the kit (`docs/design.md`),
looked at on each platform, size and theme. UUOB itself is inspected by its
own session, in [inspection-2026-11-23-uuob.md](inspection-2026-11-23-uuob.md); the kit
findings from it are fixed here too.

| | Linux | Android phone | Android tablet | Mac |
|---|---|---|---|---|
| **Sizes** | 1300 × 900, 560 × 900 | 1080 × 2400 (411 dp) | 1600 × 2560 (800 dp) | builds only |
| **Themes** | dark, light | dark, light | dark | — |

The Mac versions of all three compile and are signed, on GitHub and on a
Mac over SSH (`mac/remote-build.sh`). They were not run, so nothing on the Mac
was seen on screen, including the keys and the command palette.

## Fixed

| Where | What was wrong | Now |
|---|---|---|
| Linux, all apps | Esc didn't close the command palette (the search field kept it) | Esc closes it |
| Linux, Calendar | Ctrl+E in the event card only focused the pencil | It opens the editor; the tooltip names the key |
| Linux, Photos | In a narrow window the grid sized itself to the window, not the room beside the rail, and was cut off | Three columns that fit beside the rail |
| Linux, kit | The empty state's icon tile stretched to the page's height | A 60-point tile |
| Linux, Android, Mac: Photos | "Recently deleted" was cut short when chosen, next to its count | No count on that row (the toolbar shows it) |
| Android, all apps | On a tablet, Tasks and Photos showed the menu button next to a sidebar that was already there | The kit says when the sidebar is a drawer (`LocalDrawerMode`); only then is there a menu button |
| Android, Calendar | The menu button sat on the second row, unlike Tasks and Photos | Before the title on phones, none on tablets |
| Android, Photos | The title sat lower than in the other apps, and the toolbar had a double gap under it | Same top and gap as Tasks |
| Android, Tasks | The time in the editor sat at the far right, and the colour dots wrapped to a second line | At the start, like the date; one line of dots |
| Mac, kit | A stretched segmented control kept 14-point margins, so "People" became "Pe…" in UUOB | 6-point margins when stretched |
| Mac, kit | C, / and ? did nothing: the page they listened on never had the keyboard focus (found by UUOB's screenshots on GitHub) | They're read from the window's key events, only with no modifier, outside sheets and text fields. Seen working in UUOB on GitHub's Mac runner |
| Mac, Linux, Android: kit | The palette and the shortcut list showed Settings twice when an app also had a Settings page | One row per name |
| Mac, kit | In a 380-point window the sheets (460) and the palette (520) stuck out past the window | Never wider than the window less 24 |
| Android, kit | TalkBack read nothing for the + and the rail's icons | They're read as buttons with their names ("Add photos", "New event", a list's name) |
| Android | Photos' title was larger than Tasks' and Calendar's at the same width, and Calendar's sat a little lower | The same size and top in all three |

## Seen and left as they are

- **Linux, Calendar:** in a week with the sidebar open, a short event in a narrow column shows "Desi…" on one line. It is the one-line rule for thin events and reads better than the old letter-per-line wrapping.
- **Linux, all apps:** after a key press, GTK draws a focus ring on the first button of the rail. It's GTK's keyboard focus indicator and goes away with the mouse.
- **Linux, Calendar:** the palette names the view keys "View 1"–"View 4", because the views can change while the keys stay. The Mac names them after the views.
- **Android, Calendar:** on a phone the main screen looks as it did. The calendars moved from a bottom sheet to the side drawer, like every app's.
- **Photos:** there are no albums or "Shared with me": Photos keeps its photos in month containers, and no shared ones exist yet.

## Not checked

- The Mac on screen (see above).
- Android tablet in light mode, and Android tablet with the sidebar folded to the rail.
