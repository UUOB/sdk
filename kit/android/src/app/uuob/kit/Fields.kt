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
