#!/usr/bin/env bash
# Helpers for an app's Mac screenshots on GitHub's runner (its mac/shots.sh
# sources this): a UUOB with made-up demo data, the app connected to it, and
# the app driven by keys and button names through System Events, so the app
# itself needs no screenshot code. The app's script sets `name` (the process),
# `exe` (its binary), `out` and `mode`, then calls these.
#   UUOB=<uuob binary> DATA=<seeded data dir> before sourcing.

se() { osascript -e "tell application \"System Events\" to $1" >/dev/null 2>&1 || true; }
esc() { se 'key code 53'; sleep 1.2; }
cmd() { se "keystroke \"$1\" using {command down}"; sleep 1.5; }
key() { se "keystroke \"$1\""; sleep 1.2; }
enter() { se 'key code 36'; sleep 1.5; }
shot() { screencapture -x "$out/$mode-$1.png"; }

# The runner's local network prompt (not the app's): Don't Allow, in whichever process shows it.
deny() {
  osascript >/dev/null 2>&1 <<'EOF' || true
tell application "System Events"
  repeat with p in (every process)
    try
      repeat with w in (windows of p)
        repeat with b in (buttons of w)
          set n to name of b
          if n starts with "Don" and n ends with "Allow" then click b
        end repeat
      end repeat
    end try
  end repeat
end tell
EOF
}

# The app's UI, walked level by level (macOS's "entire contents" comes back
# empty for a SwiftUI window): `ui click|centre <text>` clicks a button (or
# menu button, segment, check) whose title, description, hint or text inside is
# the text (~ first: contains it), or gives its centre as "x y"; `ui dump`
# lists what accessibility sees, to show why a click missed.
ui() {
  local script="$RUNNER_TEMP/ui.applescript"
  [ -f "$script" ] || cat > "$script" <<'EOF'
on matches(e, t, part)
  tell application "System Events"
    set ws to {}
    try
      set end of ws to name of e
    end try
    try
      set end of ws to description of e
    end try
    try
      set end of ws to value of attribute "AXHelp" of e
    end try
    try
      set end of ws to value of e
    end try
  end tell
  repeat with w in ws
    try
      if w is not missing value then
        if (part and (w as text) contains t) or ((w as text) is t) then return true
      end if
    end try
  end repeat
  return false
end matches

on findButton(el, t, part, depth)
  if depth > 30 then return missing value
  tell application "System Events"
    set kids to {}
    try
      set kids to UI elements of el
    end try
    repeat with k in kids
      set r to ""
      try
        set r to role of k
      end try
      if r is in {"AXButton", "AXMenuButton", "AXPopUpButton", "AXRadioButton", "AXCheckBox"} then
        if my matches(k, t, part) then return k
        try
          repeat with c in (UI elements of k)
            if my matches(c, t, part) then return k
          end repeat
        end try
      end if
      set f to my findButton(k, t, part, depth + 1)
      if f is not missing value then return f
    end repeat
  end tell
  return missing value
end findButton

on dump(el, depth)
  set out to ""
  if depth > 30 then return out
  tell application "System Events"
    set kids to {}
    try
      set kids to UI elements of el
    end try
    repeat with k in kids
      set line_ to ""
      try
        set line_ to (role of k)
      end try
      try
        set line_ to line_ & " | " & (name of k as text)
      end try
      try
        set line_ to line_ & " | " & (description of k as text)
      end try
      set pad to ""
      repeat depth times
        set pad to pad & "  "
      end repeat
      set out to out & pad & line_ & linefeed
      -- A button: every attribute it has, to see where its label went.
      try
        if role of k is "AXButton" then
          repeat with atr in (attributes of k)
            try
              set out to out & pad & "    @" & (name of atr) & " = " & (value of atr as text) & linefeed
            end try
          end repeat
        end if
      end try
      set out to out & my dump(k, depth + 1)
    end repeat
  end tell
  return out
end dump

on run argv
  set pid to (item 1 of argv) as integer
  set what to item 2 of argv
  tell application "System Events"
    set p to first process whose unix id is pid
    set frontmost of p to true
    set w to front window of p
  end tell
  if what is "dump" then return dump(w, 0)
  set t to item 3 of argv
  set part to t starts with "~"
  if part then set t to text 2 thru -1 of t
  set b to findButton(w, t, part, 0)
  if b is missing value then return ""
  tell application "System Events"
    if what is "click" then
      click b
      return "ok"
    end if
    set {x, y} to position of b
    set {bw, bh} to size of b
    return ((x + bw / 2) as integer as text) & " " & ((y + bh / 2) as integer as text)
  end tell
end run
EOF
  osascript "$script" "$app_pid" "$@" 2>>"$out/clicks.txt"
}

# Where the text shows on the screen, as "x y" (macOS's own text recognition,
# Vision): for buttons whose label accessibility doesn't give (as on GitHub's
# runner). With several, the one nearest the middle of the screen (a sheet's).
seen() {
  local tool="$RUNNER_TEMP/seen"
  [ -x "$tool" ] || {
    cat > "$RUNNER_TEMP/seen.swift" <<'EOF'
import AppKit
import Vision
let args = CommandLine.arguments
var want = args[2].lowercased()
let part = want.hasPrefix("~")
if part { want.removeFirst() }
guard let image = NSImage(contentsOfFile: args[1]), let cg = image.cgImage(forProposedRect: nil, context: nil, hints: nil),
      let screen = NSScreen.main?.frame.size else { exit(1) }
let request = VNRecognizeTextRequest()
request.recognitionLevel = .accurate
request.usesLanguageCorrection = false
try? VNImageRequestHandler(cgImage: cg).perform([request])
var best: (CGFloat, CGPoint)?
for o in request.results ?? [] {
    guard let c = o.topCandidates(1).first else { continue }
    let s = c.string.lowercased()
    guard part ? s.contains(want) : (s == want || s.hasPrefix(want + " ") || s.hasSuffix(" " + want)) else { continue }
    var box = o.boundingBox
    // Just the words asked for, when the line holds more.
    if let r = c.string.range(of: want, options: .caseInsensitive), let b = try? c.boundingBox(for: r)?.boundingBox { box = b }
    let p = CGPoint(x: box.midX * screen.width, y: (1 - box.midY) * screen.height)
    let d = hypot(p.x - screen.width / 2, p.y - screen.height / 2)
    if best == nil || d < best!.0 { best = (d, p) }
}
guard let (_, p) = best else { exit(1) }
print(Int(p.x), Int(p.y))
EOF
    swiftc -O "$RUNNER_TEMP/seen.swift" -o "$tool" 2>>"$out/clicks.txt"
  }
  screencapture -x "$RUNNER_TEMP/seen.png"
  "$tool" "$RUNNER_TEMP/seen.png" "$1"
}

click() {
  local r; r=$(ui click "$1")
  # Not by name: by where its words show.
  if [ "$r" != ok ]; then
    local at; at=$(seen "$1") && [ -n "$at" ] && { pointer $at left; r="ok (seen at $at)"; }
  fi
  sleep 1.8
  echo "$mode $1 -> ${r:-missing}" >> "$out/clicks.txt"
  case $r in ok*) ;; *) echo "  no button: $1"; [ -s "$out/ax.txt" ] || ui dump > "$out/ax.txt"; return 1 ;; esac
}

# Moves the pointer to x,y on the screen (for a hover hint), and clicks there
# (`pointer x y left|right`) or scrolls (`pointer x y scroll -10`: down), with a
# tiny Swift tool: real mouse events, as a person's.
pointer() {
  local tool="$RUNNER_TEMP/pointer"
  [ -x "$tool" ] || {
    cat > "$RUNNER_TEMP/pointer.swift" <<'EOF'
import CoreGraphics
let a = CommandLine.arguments
let p = CGPoint(x: Double(a[1])!, y: Double(a[2])!)
CGWarpMouseCursorPosition(p)
CGEvent(mouseEventSource: nil, mouseType: .mouseMoved, mouseCursorPosition: p, mouseButton: .left)?.post(tap: .cghidEventTap)
if a.count > 3 && a[3] == "left" {
    CGEvent(mouseEventSource: nil, mouseType: .leftMouseDown, mouseCursorPosition: p, mouseButton: .left)?.post(tap: .cghidEventTap)
    CGEvent(mouseEventSource: nil, mouseType: .leftMouseUp, mouseCursorPosition: p, mouseButton: .left)?.post(tap: .cghidEventTap)
}
if a.count > 4 && a[3] == "scroll" {
    CGEvent(scrollWheelEvent2Source: nil, units: .line, wheelCount: 1, wheel1: Int32(a[4])!, wheel2: 0, wheel3: 0)?.post(tap: .cghidEventTap)
}
if a.count > 3 && a[3] == "right" {
    CGEvent(mouseEventSource: nil, mouseType: .rightMouseDown, mouseCursorPosition: p, mouseButton: .right)?.post(tap: .cghidEventTap)
    CGEvent(mouseEventSource: nil, mouseType: .rightMouseUp, mouseCursorPosition: p, mouseButton: .right)?.post(tap: .cghidEventTap)
}
EOF
    swiftc -O "$RUNNER_TEMP/pointer.swift" -o "$tool" 2>/dev/null
  }
  "$tool" "$@"; sleep 1.6
}

# The centre of the app's first button with this text, as "x y".
centre() { ui centre "$1"; }

# Rests the pointer on a button until its hint shows.
hover() {
  local at; at=$(centre "$1")
  [ -n "$at" ] || { echo "  no button to hover: $1"; return 1; }
  pointer $at; sleep 1
}

# A System Events command for the app's process (found by its pid).
win() { se "tell (first process whose unix id is $app_pid) to $1"; }

# UUOB running with the demo data, for the app to connect to.
uuob_up() {
  UUOB_DATA_DIR="$DATA" "$UUOB" run --name "Demo Mac" >"$out/uuob.log" 2>&1 &
  uuob_pid=$!
  sleep 4
}
uuob_down() { kill "$uuob_pid" 2>/dev/null; wait "$uuob_pid" 2>/dev/null; }

# Starts the app at the usual size; the first time, connects it (Connect, then allow the ask).
app_up() {
  # Through Launch Services (`open`), as a person starts it: started bare, an
  # app's window shows nothing inside it to accessibility.
  local full; full="$(cd "$(dirname "$exe")" && pwd)/$(basename "$exe")"
  local log="$out/app.log"; case $log in /*) ;; *) log="$PWD/$log" ;; esac
  open -n -a "${full%/Contents/MacOS/*}" --stdout "$log" --stderr "$log"
  sleep 7
  # Its process, by the full path it runs from (the list shows full paths).
  app_pid=$(pgrep -n -f "$full")
  echo "app $full: pid ${app_pid:-none}" >> "$out/clicks.txt"
  # Ask for the full accessibility tree: SwiftUI fills in buttons' labels only
  # when it believes an assistive app is reading it (as VoiceOver does).
  osascript -e "tell application \"System Events\" to tell (first process whose unix id is $app_pid)" \
    -e 'try' -e 'set value of attribute "AXEnhancedUserInterface" to true' -e 'end try' \
    -e 'try' -e 'set value of attribute "AXManualAccessibility" to true' -e 'end try' -e 'end tell' >>"$out/clicks.txt" 2>&1
  sleep 2
  deny; sleep 1
  if click "Connect to UUOB" 2>/dev/null; then
    sleep 3
    # Without the UUOB app on this Mac, Connect asks macOS for uuob:// (Cancel)
    # and the app offers its code field: a code from `uuob app-code`, then Enter.
    if click "Cancel"; then
      local code; code=$(UUOB_DATA_DIR="$DATA" "$UUOB" app-code | head -1)
      click "~amber-river" && { key "$code"; enter; sleep 4; }
    fi
    local id
    id=$(UUOB_DATA_DIR="$DATA" "$UUOB" asks | grep -oE "uuob allow [a-z0-9]+" | head -1 | cut -d" " -f3)
    [ -n "$id" ] && UUOB_DATA_DIR="$DATA" "$UUOB" allow "$id" >/dev/null
    sleep 5
  fi
  win "set position of front window to {12, 31}"
  win "set size of front window to {${1:-1000}, ${2:-680}}"
  sleep 1.5
}
app_down() { kill "$app_pid" 2>/dev/null; wait "$app_pid" 2>/dev/null; sleep 1; }
