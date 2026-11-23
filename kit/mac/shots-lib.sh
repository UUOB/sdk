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

# Clicks the app's first button (or menu button) whose title, description, hint
# or text inside is the text (with ~ first: contains it).
click() {
  local r
  r=$(osascript - "$app_pid" "$1" <<'EOF' 2>>"$out/clicks.txt"
on run argv
  set pid to (item 1 of argv) as integer
  set t to item 2 of argv
  set part to t starts with "~"
  if part then set t to text 2 thru -1 of t
  tell application "System Events" to tell (first process whose unix id is pid)
    set frontmost to true
    repeat with e in (entire contents of front window)
      try
        if role of e is in {"AXButton", "AXMenuButton", "AXPopUpButton"} then
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
            repeat with c in (entire contents of e)
              try
                set end of ws to value of c
              end try
            end repeat
          end try
          repeat with w in ws
            if w is not missing value then
              if (part and (w as text) contains t) or ((w as text) is t) then
                click e
                return "ok"
              end if
            end if
          end repeat
        end if
      end try
    end repeat
  end tell
  return "missing"
end run
EOF
  )
  sleep 1.8
  echo "$mode $1 -> $r" >> "$out/clicks.txt"
  [ "$r" = ok ] || { echo "  no button: $1"; [ -s "$out/ax.txt" ] || ax > "$out/ax.txt"; return 1; }
}

# What the app shows to accessibility (role, name, description of each element), to see why a click missed.
ax() {
  osascript - "$app_pid" <<'EOF' 2>&1
on run argv
  set out to ""
  tell application "System Events" to tell (first process whose unix id is ((item 1 of argv) as integer))
    set out to out & "process: " & name & ", windows: " & (count of windows) & linefeed
    repeat with e in (entire contents of front window)
      try
        set out to out & (role of e) & " | " & (name of e as text) & " | " & (description of e as text) & linefeed
      end try
    end repeat
  end tell
  return out
end run
EOF
}

# Moves the pointer to x,y on the screen (for a hover hint), or right-clicks
# there (`pointer x y right`), with a tiny Swift tool.
pointer() {
  local tool="$RUNNER_TEMP/pointer"
  [ -x "$tool" ] || {
    cat > "$RUNNER_TEMP/pointer.swift" <<'EOF'
import CoreGraphics
let a = CommandLine.arguments
let p = CGPoint(x: Double(a[1])!, y: Double(a[2])!)
CGWarpMouseCursorPosition(p)
CGEvent(mouseEventSource: nil, mouseType: .mouseMoved, mouseCursorPosition: p, mouseButton: .left)?.post(tap: .cghidEventTap)
if a.count > 3 && a[3] == "right" {
    CGEvent(mouseEventSource: nil, mouseType: .rightMouseDown, mouseCursorPosition: p, mouseButton: .right)?.post(tap: .cghidEventTap)
    CGEvent(mouseEventSource: nil, mouseType: .rightMouseUp, mouseCursorPosition: p, mouseButton: .right)?.post(tap: .cghidEventTap)
}
EOF
    swiftc -O "$RUNNER_TEMP/pointer.swift" -o "$tool" 2>/dev/null
  }
  "$tool" "$@"; sleep 1.6
}

# The centre of the app's first button with this name or description, as "x y".
centre() {
  osascript - "$app_pid" "$1" <<'EOF' 2>/dev/null
on run argv
  tell application "System Events" to tell (first process whose unix id is ((item 1 of argv) as integer))
    repeat with e in (entire contents of front window)
      try
        if role of e is in {"AXButton", "AXMenuButton"} and ((name of e is (item 2 of argv)) or (description of e is (item 2 of argv))) then
          set {x, y} to position of e
          set {w, h} to size of e
          return ((x + w / 2) as integer as text) & " " & ((y + h / 2) as integer as text)
        end if
      end try
    end repeat
  end tell
  return ""
end run
EOF
}

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
  "$exe" >>"$out/app.log" 2>&1 & app_pid=$!
  sleep 7; deny; sleep 1
  if click "Connect to UUOB" 2>/dev/null; then
    sleep 2
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
