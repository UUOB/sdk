package app.uuob.kit

import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextFieldColors
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.getValue
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

/** `.field`: grey and borderless until it has the focus, then a blue edge. */
@Composable
fun fieldColors(): TextFieldColors = OutlinedTextFieldDefaults.colors(
  unfocusedContainerColor = Tokens.surface2,
  focusedContainerColor = MaterialTheme.colorScheme.surface,
  disabledContainerColor = Tokens.surface2,
  unfocusedBorderColor = Color.Transparent,
  focusedBorderColor = MaterialTheme.colorScheme.primary,
  unfocusedPlaceholderColor = Tokens.text3,
  focusedPlaceholderColor = Tokens.text3,
)

/** A text field in UUOB's style. */
@Composable
fun Field(
  value: String,
  onValueChange: (String) -> Unit,
  modifier: Modifier = Modifier,
  placeholder: String = "",
  singleLine: Boolean = true,
  minLines: Int = 1,
  textStyle: TextStyle = MaterialTheme.typography.bodyLarge,
  keyboardOptions: KeyboardOptions = KeyboardOptions.Default,
  keyboardActions: KeyboardActions = KeyboardActions.Default,
) {
  OutlinedTextField(
    value, onValueChange, modifier,
    placeholder = { Text(placeholder, style = textStyle) },
    singleLine = singleLine, minLines = minLines, textStyle = textStyle,
    keyboardOptions = keyboardOptions, keyboardActions = keyboardActions,
    shape = Tokens.field, colors = fieldColors(),
  )
}

/**
 * A scrolling wheel of numbers (`values`), the middle one chosen, with a label
 * under it: days, hours, minutes. Any app can use it for a duration.
 */
@androidx.compose.runtime.Composable
fun NumberWheel(values: List<Int>, value: Int, label: String, onChange: (Int) -> Unit, modifier: Modifier = Modifier) {
  val rowH = 40.dp
  val start = values.indexOf(value).coerceAtLeast(0)
  val state = androidx.compose.foundation.lazy.rememberLazyListState(start)
  val fling = androidx.compose.foundation.gestures.snapping.rememberSnapFlingBehavior(state)
  val centre by androidx.compose.runtime.remember { androidx.compose.runtime.derivedStateOf {
    val info = state.layoutInfo
    val mid = (info.viewportStartOffset + info.viewportEndOffset) / 2
    info.visibleItemsInfo.minByOrNull { kotlin.math.abs(it.offset + it.size / 2 - mid) }?.index?.minus(1) ?: start
  } }
  androidx.compose.runtime.LaunchedEffect(state.isScrollInProgress) {
    if (!state.isScrollInProgress) values.getOrNull(centre)?.let { if (it != value) onChange(it) }
  }
  androidx.compose.foundation.layout.Column(modifier, horizontalAlignment = androidx.compose.ui.Alignment.CenterHorizontally) {
    androidx.compose.foundation.layout.Box(androidx.compose.ui.Modifier.height(rowH * 3).fillMaxWidth()) {
      androidx.compose.foundation.layout.Box(
        androidx.compose.ui.Modifier.align(androidx.compose.ui.Alignment.Center).fillMaxWidth().height(rowH)
          .background(Tokens.surface2, Tokens.field),
      )
      androidx.compose.foundation.lazy.LazyColumn(androidx.compose.ui.Modifier.fillMaxWidth(), state = state, flingBehavior = fling,
        horizontalAlignment = androidx.compose.ui.Alignment.CenterHorizontally) {
        // A blank row above and below, so the first and last can sit in the middle.
        item { androidx.compose.foundation.layout.Spacer(androidx.compose.ui.Modifier.height(rowH)) }
        items(values.size) { i ->
          val on = i == centre
          androidx.compose.foundation.layout.Box(androidx.compose.ui.Modifier.height(rowH).fillMaxWidth(), contentAlignment = androidx.compose.ui.Alignment.Center) {
            Text("${values[i]}", fontSize = if (on) 22.sp else 18.sp, fontWeight = if (on) FontWeight.SemiBold else FontWeight.Normal,
              color = if (on) MaterialTheme.colorScheme.onSurface else Tokens.text3)
          }
        }
        item { androidx.compose.foundation.layout.Spacer(androidx.compose.ui.Modifier.height(rowH)) }
      }
    }
    Text(label, fontSize = 13.sp, color = Tokens.text2, modifier = androidx.compose.ui.Modifier.padding(top = 6.dp))
  }
}
