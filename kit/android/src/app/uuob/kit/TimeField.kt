package app.uuob.kit

import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.ui.unit.sp
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.TextFieldValue
import app.uuob.client.friendlyTime
import kotlinx.coroutines.delay
import app.uuob.client.parseTime

/**
 * A time field that writes times the way the user chose (14:30 or 2:30 PM),
 * whatever the phone's own setting (ui/TimeField.svelte). Typing "1430",
 * "9", "9.15" or "2pm" works too; anything else goes back to the time it
 * had. `value` is "HH:MM"; with `optional`, emptying the field makes it null.
 */
@Composable
fun TimeField(
  value: String?,
  onChange: (String?) -> Unit,
  modifier: Modifier = Modifier,
  label: String? = null,
  placeholder: String = "",
  optional: Boolean = false,
) {
  val hour12 = UuobSettings.hour12
  fun shown() = value?.let { friendlyTime(it, hour12) } ?: ""
  var text by remember { mutableStateOf(TextFieldValue(shown())) }
  var editing by remember { mutableStateOf(false) }
  val focus = LocalFocusManager.current
  LaunchedEffect(value, hour12, editing) { if (!editing) text = TextFieldValue(shown()) }
  // Focused with all of it selected, so typing replaces the time. The tap
  // that focused it places its own cursor first, so this comes just after.
  LaunchedEffect(editing) {
    if (editing) {
      delay(60)
      text = text.copy(selection = TextRange(0, text.text.length))
    }
  }

  fun commit() {
    val t = parseTime(text.text)
    when {
      t != null -> if (t != value) onChange(t)
      optional && text.text.isBlank() -> if (value != null) onChange(null)
    }
    text = TextFieldValue(shown())
  }

  // Leaving the screen while typing keeps what was typed.
  val pending by rememberUpdatedState(if (editing) text.text else null)
  val finish by rememberUpdatedState(::commit)
  DisposableEffect(Unit) { onDispose { if (pending != null) finish() } }

  WebField(
    text, { text = it },
    modifier.onFocusChanged {
      if (it.isFocused && !editing) {
        editing = true
      } else if (!it.isFocused && editing) {
        editing = false
        commit()
      }
    },
    placeholder = placeholder,
    textStyle = androidx.compose.ui.text.TextStyle(fontSize = 15.sp),
    keyboardOptions = KeyboardOptions(keyboardType = if (hour12) KeyboardType.Text else KeyboardType.Decimal, imeAction = ImeAction.Done),
    keyboardActions = KeyboardActions(onDone = { focus.clearFocus() }),
  )
}
