# Inspection, 23 November 2026: UUOB

UUOB after moving onto the kit (UUOB `harmonize` branch, kit 9f17a79),
compared with 0.4.2 on each platform, size and theme. The kit findings it
turned up are in `inspection-2026-11-23.md` and fixed in the kit.

| | Linux | Android phone | Mac |
|---|---|---|---|
| **Sizes** | 1100 × 760, 800 × 700, 420 × 700 | 1080 × 2400, and at 1.5× text | 1000 × 676, 380 × 480 |
| **Themes** | light, dark | light, dark | light, dark (small window dark) |
| **How** | Docker image, demo data, keys and clicks | emulator, demo data, taps by text | GitHub runner, demo data, keys (`mac/shots.sh`) |

Every page, the share offer, Add and its choices, the palette and the
shortcut list were seen on all three. On Android also every sheet (codes,
rename, your name, an app's and a person's details). The Mac sheets that open
only from a button (rename, codes) were not reached: the runner's UI scripting
doesn't find SwiftUI buttons by name. They use the same kit sheets as Linux,
where they were seen.

## Fixed in UUOB

| Where | What was wrong | Now |
|---|---|---|
| All, Settings | Long explanations under Start at login, Apps can connect and Copies, and a paragraph under Storage devices | Start at login alone; the AI server address; "How many storage devices keep each thing"; no paragraph |
| Linux, Network and Settings | The rename and remove buttons were wide grey pills (UUOB's page button style caught them) | The kit's round icon button |
| Linux, Access | Rows said "Calendar · App", with where it runs on a third line | The coloured badge and "This device" beside the name, as on the Mac and Android |
| Android, Add | "A person" opened an older sheet with a paragraph and three large buttons | Add lists "A person: show my code" and "A person: enter their code", as on the Mac and Linux |
| Android, Add → An AI | A paragraph | One line: where to do it |
| Android, a person's codes | Paragraphs that named an old path (Access → Add → A person → Show my code) | The Mac's lines, with today's names |
| Mac | Icon-only buttons (rename, remove, copy, disconnect, the graph's nodes) read as "button" to VoiceOver | They read their tooltip |
| Mac, Linux | A page picked in the sidebar over a narrow window left it open | It closes (kit `uuobClosePeek`, `closer()`) |

## Found in UUOB, fixed in the kit

- Mac: a stretched segmented control cut "People" to "Pe…" at 380 wide.
- Mac: C, / and ? did nothing (no keyboard focus). Seen working on the runner.
- Mac, Linux, Android: the palette listed Settings twice (UUOB's Settings page and ⌘,). Seen fixed on the Mac; Linux and Android after the next kit commit.
- Mac: in a 380 window the sheets and the palette stuck out past the window. Seen fitting on the runner.
- Android: the + had no name for TalkBack.

## Seen and left as they are

- **Android, 1.5× text:** a device's name is cut short ("Alex's Lap…") next to its "This device" badge. The full name is in its sheet.
- **Linux, Android:** the share offer has no Later button; Esc or Back is Later. The Mac shows Later, since a Mac sheet has no other way out.
- **All:** "The name people you link with see" stays under your name: one short line that says who sees it.
