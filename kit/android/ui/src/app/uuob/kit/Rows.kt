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

// Settings-style rows and the segmented control: UI only, for UUOB and the apps.

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
