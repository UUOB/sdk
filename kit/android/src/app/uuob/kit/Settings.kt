package app.uuob.kit

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.height
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.sp
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.ui.draw.scale
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import org.json.JSONObject

/**
 * The settings that belong to all of the user's apps (kept in UUOB, so a
 * change in one app reaches every app on every device): whether times are
 * written 14:30 or 2:30 PM, and whether weeks start on Sunday. Compose state,
 * so a screen that reads them follows a change at once.
 */
object UuobSettings {
  var hour12 by mutableStateOf(false)
    private set
  var sundayFirst by mutableStateOf(false)
    private set

  private fun apply(s: JSONObject) {
    hour12 = s.optBoolean("hour12", hour12)
    sundayFirst = s.optBoolean("sunday_first", sundayFirst)
  }

  /** Called by Uuob.init: what was seen last, then UUOB's, and again whenever something changes. */
  internal fun start() {
    apply(JSONObject(Uuob.client.cachedSettings()))
    Uuob.scope.launch { refresh() }
    Uuob.scope.launch {
      Uuob.events.collect { e ->
        when (e.optString("type")) {
          "settings" -> e.optJSONObject("settings")?.let { apply(it) }
          "changed" -> refresh()
          "state" -> if (e.optString("state") == "connected") refresh()
        }
      }
    }
  }

  suspend fun refresh() = apply(JSONObject(Uuob.client.settings()))

  /** Changes some of them, e.g. `{"hour12": true}`; shown at once. */
  fun set(change: JSONObject) {
    apply(change)
    Uuob.scope.launch { Uuob.client.setSettings(change.toString()) }
  }
}

/**
 * Settings, the same sheet in every app, in the event card's style: rows
 * with an icon and a hairline between them, no boxes. The choices for all
 * apps come first, the app's own groups (`content`) after them, and the
 * version is a small line at the end.
 */
@Composable
fun SettingsSheet(onClose: () -> Unit, week: Boolean = false, content: @Composable ColumnScope.() -> Unit = {}) {
  val app = Uuob.app
  WebSheet(onClose, padding = 22.dp) {
    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
      Text("Settings", fontSize = 22.sp, fontWeight = FontWeight.SemiBold, letterSpacing = (-0.4).sp, modifier = Modifier.weight(1f))
      IconBtn("close", onClose, size = 19.dp)
    }
    SettingsGroup("All your apps") {
      SettingsRow("Time", icon = "clock", last = !week) {
        Segmented(listOf("14:30", "2:30 PM"), if (UuobSettings.hour12) 1 else 0) { UuobSettings.set(JSONObject().put("hour12", it == 1)) }
      }
      if (week) {
        SettingsRow("Week starts", icon = "calendar", last = true) {
          Segmented(listOf("Monday", "Sunday"), if (UuobSettings.sundayFirst) 1 else 0) { UuobSettings.set(JSONObject().put("sunday_first", it == 1)) }
        }
      }
    }
    content()
    Text("${app.name} ${app.version}", fontSize = 12.sp, color = Tokens.text3, modifier = Modifier.fillMaxWidth().padding(top = 22.dp), textAlign = androidx.compose.ui.text.style.TextAlign.Center)
  }
}

/** A few rows under a small grey heading (`foot` is no longer shown: the rows say enough). */
@Composable
fun SettingsGroup(label: String, @Suppress("UNUSED_PARAMETER") foot: String? = null, rows: @Composable ColumnScope.() -> Unit) {
  RowsLabel(label)
  Column(Modifier.fillMaxWidth(), content = rows)
}

/** A row of Settings: an icon (or room for one), its name, a control on the right, a hairline under it unless `last`. */
@Composable
fun SettingsRow(label: String, @Suppress("UNUSED_PARAMETER") first: Boolean = true, icon: String? = null, last: Boolean = false, trailing: @Composable () -> Unit) {
  IconRow(icon, last = last) {
    Text(label, fontSize = 15.sp, modifier = Modifier.weight(1f))
    trailing()
  }
}

/** A row of choices, one of them on (`.segmented`: the chosen one raised on a grey track). */
@Composable
fun Segmented(options: List<String>, selected: Int, modifier: Modifier = Modifier, fill: Boolean = false, compact: Boolean = false, onSelect: (Int) -> Unit) {
  // Moves at once: the screen behind it may take a moment to follow.
  var shown by remember(selected) { mutableStateOf(selected) }
  Row(
    modifier.clip(RoundedCornerShape(10.dp)).background(Tokens.surface2).padding(if (compact) 2.dp else 3.dp),
    horizontalArrangement = Arrangement.spacedBy(2.dp),
  ) {
    options.forEachIndexed { i, label ->
      val on = i == shown
      val source = remember { MutableInteractionSource() }
      Box(
        // `fill`: the choices share the width evenly (many of them on a phone).
        (if (fill) Modifier.weight(1f) else Modifier).scale(pressScale(source, 0.96f)).shadow(if (on) 1.5.dp else 0.dp, RoundedCornerShape(8.dp))
          .clip(RoundedCornerShape(8.dp))
          .background(if (on) MaterialTheme.colorScheme.surface else Color.Transparent)
          .clickable(source, null) { if (i != shown) { shown = i; onSelect(i) } }
          // `compact` (a phone's view switch): just enough room around each name.
          .padding(horizontal = if (fill) 4.dp else if (compact) 10.dp else 14.dp, vertical = if (compact) 5.dp else 6.dp),
        contentAlignment = Alignment.Center,
      ) {
        Text(label, fontSize = if (compact || (fill && options.size > 4)) 13.sp else 14.sp, fontWeight = FontWeight.Medium, color = if (on) MaterialTheme.colorScheme.onSurface else Tokens.text2, maxLines = 1, softWrap = false)
      }
    }
  }
}
