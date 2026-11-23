package app.uuob.kit

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.ui.draw.clip
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isCtrlPressed
import androidx.compose.ui.input.key.isShiftPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.onKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

/**
 * The keys every app answers to with a hardware keyboard (docs/design.md):
 * Ctrl+, Settings · Ctrl+N New · Ctrl+F Find · Ctrl+Alt+S Toggle sidebar ·
 * Ctrl+R Refresh · Ctrl+/ the list; plus the app's own (`extra`: name, key
 * label, the key, whether Ctrl is held, what it does).
 */
class Shortcuts(
  val settings: (() -> Unit)? = null,
  val new: Pair<String, () -> Unit>? = null,
  val find: (() -> Unit)? = null,
  val toggleSidebar: (() -> Unit)? = null,
  val refresh: (() -> Unit)? = null,
  val extra: List<Extra> = emptyList(),
) {
  class Extra(val title: String, val label: String, val key: Key, val ctrl: Boolean, val shift: Boolean = false, val action: () -> Unit)

  /** Every shortcut as (name, keys), for the list. */
  fun rows(): List<Pair<String, String>> = buildList {
    add("Command palette" to "Ctrl+K")
    if (settings != null) add("Settings" to "Ctrl+,")
    new?.let { add(it.first to "Ctrl+N or C") }
    if (find != null) add("Find" to "Ctrl+F or /")
    if (toggleSidebar != null) add("Toggle sidebar" to "Ctrl+Alt+S")
    if (refresh != null) add("Refresh" to "Ctrl+R")
    extra.forEach { add(it.title to it.label) }
    add("Close or cancel" to "Esc")
    add("Keyboard shortcuts" to "Ctrl+/ or ?")
  }

  /** Every action with its keys, for the palette. */
  fun actions(): List<Triple<String, String, () -> Unit>> = buildList {
    new?.let { add(Triple(it.first, "Ctrl+N  C", it.second)) }
    find?.let { add(Triple("Find", "Ctrl+F  /", it)) }
    extra.forEach { add(Triple(it.title, it.label, it.action)) }
    settings?.let { add(Triple("Settings", "Ctrl+,", it)) }
    toggleSidebar?.let { add(Triple("Toggle sidebar", "Ctrl+Alt+S", it)) }
    refresh?.let { add(Triple("Refresh", "Ctrl+R", it)) }
  }
}

/** Answers the shortcuts anywhere inside, and shows their list with Ctrl+/. */
@Composable
fun ShortcutsHost(shortcuts: Shortcuts, content: @Composable () -> Unit) {
  var listShown by remember { mutableStateOf(false) }
  var paletteShown by remember { mutableStateOf(false) }
  // Linear's single letters, only when no field took the key (C new, / find, ? the list).
  Box(Modifier.onKeyEvent { e ->
    if (e.type != KeyEventType.KeyDown || e.isCtrlPressed) return@onKeyEvent false
    val hit: (() -> Unit)? = when {
      e.key == Key.C -> shortcuts.new?.second
      e.key == Key.Slash && e.isShiftPressed -> ({ listShown = true })
      e.key == Key.Slash -> shortcuts.find
      else -> null
    }
    hit?.invoke()
    hit != null
  }.onPreviewKeyEvent { e ->
    if (e.type != KeyEventType.KeyDown) return@onPreviewKeyEvent false
    val ctrl = e.isCtrlPressed
    val hit: (() -> Unit)? = when {
      ctrl && e.key == Key.K -> ({ paletteShown = true })
      ctrl && e.key == Key.Comma -> shortcuts.settings
      ctrl && !e.isShiftPressed && e.key == Key.N -> shortcuts.new?.second
      ctrl && e.key == Key.F -> shortcuts.find
      ctrl && e.isAltPressedCompat() && e.key == Key.S -> shortcuts.toggleSidebar
      ctrl && e.key == Key.R -> shortcuts.refresh
      ctrl && (e.key == Key.Slash || (e.isShiftPressed && e.key == Key.Slash)) -> ({ listShown = true })
      else -> shortcuts.extra.firstOrNull { it.key == e.key && it.ctrl == ctrl && it.shift == e.isShiftPressed }?.action
    }
    hit?.invoke()
    hit != null
  }) { content() }
  if (paletteShown) CommandPalette(shortcuts.actions()) { paletteShown = false }
  if (listShown) {
    val rows = shortcuts.rows()
    DetailSheet("Keyboard shortcuts", { listShown = false }) {
      rows.forEachIndexed { i, (name, keys) ->
        IconRow(null, last = i == rows.size - 1) {
          Text(name, fontSize = 15.sp, modifier = Modifier.weight(1f))
          Text(keys, fontSize = 14.sp, color = Tokens.text2, modifier = Modifier.background(Tokens.surface2, RoundedCornerShape(6.dp)).padding(horizontal = 8.dp, vertical = 3.dp))
        }
      }
    }
  }
}

private fun androidx.compose.ui.input.key.KeyEvent.isAltPressedCompat() = (nativeKeyEvent.metaState and android.view.KeyEvent.META_ALT_ON) != 0

/** Linear's command palette: type to narrow every action down, tap one (or Enter for the first) to run it. */
@Composable
fun CommandPalette(actions: List<Triple<String, String, () -> Unit>>, onClose: () -> Unit) {
  var query by remember { mutableStateOf("") }
  val shown = actions.filter { query.isBlank() || it.first.contains(query.trim(), ignoreCase = true) }
  fun run(a: Triple<String, String, () -> Unit>) { onClose(); a.third() }
  WebSheet(onClose, padding = 12.dp) {
    WebField(query, { query = it }, placeholder = "Type a command…", keyboardOptions = androidx.compose.foundation.text.KeyboardOptions(imeAction = androidx.compose.ui.text.input.ImeAction.Go),
      keyboardActions = androidx.compose.foundation.text.KeyboardActions(onGo = { shown.firstOrNull()?.let(::run) }))
    shown.forEach { a ->
      androidx.compose.foundation.layout.Row(
        Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).clickable { run(a) }.padding(horizontal = 12.dp, vertical = 10.dp),
        verticalAlignment = androidx.compose.ui.Alignment.CenterVertically,
      ) {
        Text(a.first, fontSize = 15.sp, modifier = Modifier.weight(1f))
        Text(a.second, fontSize = 13.sp, color = Tokens.text2)
      }
    }
    if (shown.isEmpty()) Text("Nothing matches", color = Tokens.text3, modifier = Modifier.padding(16.dp))
  }
}
