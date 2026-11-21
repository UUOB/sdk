package app.uuob.kit

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsFocusedAsState
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.scale
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.PathParser
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties

/*
 * The web version's own building blocks (ui/tokens.css, ui/Icon.svelte),
 * drawn the same way in Compose: line icons on a 24 grid, pill buttons, grey
 * fields, the segmented control, the round check, the switch and the sheet
 * that rises over a dimmed screen. Every app's screens are made of these, so
 * they look as they did.
 */

/** The line icons of ui/Icon.svelte (24px grid, 1.75 stroke, round ends). */
object Icons24 {
  val paths = mapOf(
    "tasks" to "M9 6h11M9 12h11M9 18h11M4 6l1 1 2-2M4 12l1 1 2-2M4 18l1 1 2-2",
    "calendar" to "M4 7a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2zM4 10h16M8 3v4M16 3v4",
    "plus" to "M12 5v14M5 12h14",
    "trash" to "M4 7h16M10 11v6M14 11v6M6 7l1 12a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2l1-12M9 7V4h6v3",
    "link" to "M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1 1M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1-1",
    "close" to "M6 6l12 12M18 6L6 18",
    "check" to "M5 12l4 4 10-10",
    "download" to "M12 4v11M7 10l5 5 5-5M5 20h14",
    "copy" to "M9 9h10v10a1 1 0 0 1-1 1H10a1 1 0 0 1-1-1zM5 15V5a1 1 0 0 1 1-1h9",
    "lock" to "M6 11h12v9H6zM8 11V8a4 4 0 0 1 8 0v3",
    "chevron" to "M9 6l6 6-6 6",
    "back" to "M15 6l-6 6 6 6",
    "left" to "M15 6l-6 6 6 6",
    "right" to "M9 6l6 6-6 6",
    "edit" to "M4 20h4L19 9l-4-4L4 16zM13 7l4 4",
    "search" to "M11 4a7 7 0 1 1 0 14 7 7 0 0 1 0-14zM20 20l-4-4",
    "settings" to "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z",
    "sun" to "M12 8a4 4 0 1 1 0 8 4 4 0 0 1 0-8zM12 2v2M12 20v2M2 12h2M20 12h2M5 5l1.4 1.4M17.6 17.6 19 19M5 19l1.4-1.4M17.6 6.4 19 5",
    "list" to "M8 6h12M8 12h12M8 18h12M4 6h.01M4 12h.01M4 18h.01",
    "inbox" to "M4 13l2-8h12l2 8M4 13v6h16v-6M4 13h5l1 2h4l1-2h5",
    "clock" to "M12 3a9 9 0 1 1 0 18 9 9 0 0 1 0-18zM12 7v5l3 2",
    "menu" to "M4 7h16M4 12h16M4 17h16",
    "people" to "M9 11a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7zM2.5 20a6.5 6.5 0 0 1 13 0M16 4.2a3.5 3.5 0 0 1 0 6.6M18 14.3a6.5 6.5 0 0 1 3.5 5.7",
    "share" to "M12 15V3M8 7l4-4 4 4M7 11H5a1 1 0 0 0-1 1v8a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1v-8a1 1 0 0 0-1-1h-2",
    "photo" to "M4 6a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2zM4 16l4.5-4.5 3.5 3.5 3-3 5 5M9 9h.01",
    "play" to "M8 5l11 7-11 7z",
    "bell" to "M6 16V11a6 6 0 0 1 12 0v5l1.5 2h-15zM10 20.5a2 2 0 0 0 4 0",
    "place" to "M12 21s-7-6.2-7-11.5a7 7 0 0 1 14 0C19 14.8 12 21 12 21zM12 12a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5z",
    "video" to "M3 7a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2zM16 10l5-3v10l-5-3",
    "notes" to "M5 7h14M5 12h14M5 17h9",
    "at" to "M16 12a4 4 0 1 1-8 0 4 4 0 0 1 8 0zM16 12v1.5a2.5 2.5 0 0 0 5 0V12a9 9 0 1 0-3.5 7.1",
    "repeat" to "M17 2l4 4-4 4M3 11V9a3 3 0 013-3h15M7 22l-4-4 4-4M21 13v2a3 3 0 01-3 3H3",
  )
  private val parsed = HashMap<String, androidx.compose.ui.graphics.Path>()
  fun path(name: String) = parsed.getOrPut(name) { PathParser().parsePathString(paths[name] ?: "").toPath() }
}

/** An icon of ui/Icon.svelte. */
@Composable
fun Icon24(name: String, modifier: Modifier = Modifier, size: Dp = 22.dp, stroke: Float = 1.75f, tint: Color = LocalContentColor.current) {
  val path = Icons24.path(name)
  Canvas(modifier.size(size)) {
    val k = this.size.width / 24f
    scale(k, k, pivot = androidx.compose.ui.geometry.Offset.Zero) {
      drawPath(path, tint, style = Stroke(width = stroke, cap = StrokeCap.Round, join = StrokeJoin.Round))
    }
  }
}

/** Shrinks a little while pressed (`:active { transform: scale(0.97) }`). */
@Composable
internal fun pressScale(source: MutableInteractionSource, to: Float = 0.97f): Float {
  val pressed by source.collectIsPressedAsState()
  return animateFloatAsState(if (pressed) to else 1f, spring(stiffness = Spring.StiffnessMediumLow), label = "press").value
}

enum class BtnKind { Normal, Primary, Danger, Ghost }

/** `.btn`: a pill, 40 tall (50 with `lg`), with an icon before its text if given. */
@Composable
fun Btn(
  text: String,
  onClick: () -> Unit,
  modifier: Modifier = Modifier,
  kind: BtnKind = BtnKind.Normal,
  lg: Boolean = false,
  icon: String? = null,
  enabled: Boolean = true,
) {
  val source = remember { MutableInteractionSource() }
  val k = pressScale(source)
  val (bg, fg) = when (kind) {
    BtnKind.Primary -> MaterialTheme.colorScheme.primary to Color.White
    BtnKind.Danger -> MaterialTheme.colorScheme.error.copy(alpha = if (Colors.dark) 0.14f else 0.1f) to MaterialTheme.colorScheme.error
    BtnKind.Ghost -> Color.Transparent to MaterialTheme.colorScheme.primary
    BtnKind.Normal -> Tokens.surface2 to MaterialTheme.colorScheme.onSurface
  }
  Row(
    modifier.scale(k).graphicsLayer { alpha = if (enabled) 1f else 0.45f }
      .height(if (lg) 50.dp else 40.dp).clip(Pill.shape).background(bg)
      .clickable(source, null, enabled = enabled, onClick = onClick)
      .padding(horizontal = if (lg) 26.dp else 18.dp),
    verticalAlignment = Alignment.CenterVertically,
    horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally),
  ) {
    if (icon != null) Icon24(icon, size = 18.dp, stroke = 2f, tint = fg)
    Text(text, color = fg, fontSize = if (lg) 17.sp else 15.sp, fontWeight = FontWeight.SemiBold, maxLines = 1)
  }
}

/** `.icon-btn`: a round 36 button with a grey icon. */
@Composable
fun IconBtn(icon: String, onClick: () -> Unit, modifier: Modifier = Modifier, size: Dp = 22.dp, tint: Color = Tokens.text2, description: String? = null) {
  val source = remember { MutableInteractionSource() }
  Box(
    modifier.size(36.dp).clip(CircleShape).clickable(source, null, onClick = onClick),
    contentAlignment = Alignment.Center,
  ) { Icon24(icon, size = size, tint = tint) }
  @Suppress("UNUSED_EXPRESSION") description
}

/** `.field`: grey, a hairline, 46 tall, a blue ring while it has the focus. */
@Composable
fun WebField(
  value: TextFieldValue,
  onValueChange: (TextFieldValue) -> Unit,
  modifier: Modifier = Modifier,
  placeholder: String = "",
  singleLine: Boolean = true,
  minHeight: Dp = 46.dp,
  textStyle: TextStyle = TextStyle(fontSize = 17.sp),
  keyboardOptions: KeyboardOptions = KeyboardOptions.Default,
  keyboardActions: KeyboardActions = KeyboardActions.Default,
  border: Boolean = true,
) {
  val source = remember { MutableInteractionSource() }
  val focused by source.collectIsFocusedAsState()
  val ring by animateDpAsState(if (focused && border) 4.dp else 0.dp, tween(180), label = "ring")
  val edge by animateColorAsState(if (focused && border) MaterialTheme.colorScheme.primary else if (border) Tokens.line else Color.Transparent, tween(180), label = "edge")
  val fg = MaterialTheme.colorScheme.onSurface
  BasicTextField(
    value, onValueChange,
    modifier.fillMaxWidth().heightIn(min = minHeight),
    singleLine = singleLine,
    textStyle = textStyle.copy(color = fg),
    cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
    keyboardOptions = keyboardOptions, keyboardActions = keyboardActions,
    interactionSource = source,
    decorationBox = { inner ->
      Box(
        Modifier.fillMaxWidth().heightIn(min = minHeight)
          .then(if (ring > 0.dp) Modifier.border(ring, Tokens.accentSoft, RoundedCornerShape(12.dp + ring)) else Modifier)
          .padding(ring)
          .clip(Tokens.field)
          .background(if (!border) Color.Transparent else if (focused) MaterialTheme.colorScheme.surface else Tokens.surface2)
          .then(if (border || focused) Modifier.border(1.dp, edge, Tokens.field) else Modifier)
          .padding(horizontal = if (border) 14.dp else 2.dp, vertical = 12.dp),
        contentAlignment = if (singleLine) Alignment.CenterStart else Alignment.TopStart,
      ) {
        if (value.text.isEmpty()) Text(placeholder, Modifier.fillMaxWidth(), style = textStyle, color = Tokens.text3)
        inner()
      }
    },
  )
}

/** [WebField] for plain text. */
@Composable
fun WebField(
  value: String,
  onValueChange: (String) -> Unit,
  modifier: Modifier = Modifier,
  placeholder: String = "",
  singleLine: Boolean = true,
  minHeight: Dp = 46.dp,
  textStyle: TextStyle = TextStyle(fontSize = 17.sp),
  keyboardOptions: KeyboardOptions = KeyboardOptions.Default,
  keyboardActions: KeyboardActions = KeyboardActions.Default,
  border: Boolean = true,
) {
  var tf by remember { mutableStateOf(TextFieldValue(value)) }
  if (tf.text != value) tf = tf.copy(text = value)
  WebField(tf, { tf = it; if (it.text != value) onValueChange(it.text) }, modifier, placeholder, singleLine, minHeight, textStyle, keyboardOptions, keyboardActions, border)
}

/** `label.stack`: a small grey label over a field. */
@Composable
fun Stack(label: String, modifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
  Column(modifier, verticalArrangement = Arrangement.spacedBy(6.dp)) {
    Text(label, fontSize = 13.sp, fontWeight = FontWeight.Medium, color = Tokens.text2)
    content()
  }
}

/** `.check`: a ring, filled in its colour with a tick when on. */
@Composable
fun Check(on: Boolean, ring: Color?, onClick: () -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true) {
  val source = remember { MutableInteractionSource() }
  val k = pressScale(source, 0.85f)
  val color = ring ?: MaterialTheme.colorScheme.primary
  val fill by animateColorAsState(if (on) color else Color.Transparent, tween(120), label = "check")
  val edge = if (on) color else ring ?: Tokens.text3
  Box(
    modifier.scale(k).size(22.dp).graphicsLayer { alpha = if (enabled) 1f else 0.45f }
      .clip(CircleShape).background(fill).border(1.75.dp, edge, CircleShape)
      .clickable(source, null, enabled = enabled, onClick = onClick),
    contentAlignment = Alignment.Center,
  ) {
    if (on) Canvas(Modifier.size(22.dp)) {
      val u = size.width / 22f
      val p = androidx.compose.ui.graphics.Path().apply { moveTo(6.5f * u, 11f * u); lineTo(9.5f * u, 14f * u); lineTo(15.5f * u, 8f * u) }
      drawPath(p, Color.White, style = Stroke(2f * u, cap = StrokeCap.Square, join = StrokeJoin.Miter))
    }
  }
}

/** `.switch`: iOS's switch, green when on. */
@Composable
fun WebSwitch(on: Boolean, onChange: (Boolean) -> Unit, modifier: Modifier = Modifier) {
  val x by animateDpAsState(if (on) 20.dp else 0.dp, spring(dampingRatio = 0.85f, stiffness = Spring.StiffnessMedium), label = "knob")
  val bg by animateColorAsState(if (on) Colors.ok else if (Colors.dark) Color(0x24FFFFFF) else Color(0x24000000), tween(120), label = "track")
  // No ripple: it lingered over the track and made the switch feel slow.
  Box(modifier.size(51.dp, 31.dp).clip(Pill.shape).background(bg).clickable(remember { MutableInteractionSource() }, null) { onChange(!on) }) {
    Box(Modifier.offset(x = 2.dp + x, y = 2.dp).size(27.dp).shadow(3.dp, CircleShape).clip(CircleShape).background(Color.White))
  }
}

/** `.card` / `.group`: white, a hairline, rounded 18. */
@Composable
fun WebCard(modifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
  Column(modifier.clip(Tokens.card).background(MaterialTheme.colorScheme.surface).border(Tokens.border, Tokens.card), content = content)
}

/** `.section-label`. */
@Composable
fun SectionLabel(text: String, modifier: Modifier = Modifier) {
  Text(text.uppercase(), modifier, fontSize = 13.sp, fontWeight = FontWeight.SemiBold, color = Tokens.text3, letterSpacing = 0.5.sp)
}

/** `.large-title`. */
@Composable
fun LargeTitle(text: String, modifier: Modifier = Modifier, color: Color = MaterialTheme.colorScheme.onSurface) {
  Text(text, modifier, fontSize = 34.sp, lineHeight = 39.sp, fontWeight = FontWeight.Bold, letterSpacing = (-0.85).sp, color = color, maxLines = 1)
}

/** `.scrim` + `.sheet`: a card that pops up over the dimmed screen; a tap outside closes it. */
@Composable
fun WebSheet(onDismiss: () -> Unit, modifier: Modifier = Modifier, padding: Dp = 28.dp, content: @Composable ColumnScope.() -> Unit) {
  Dialog(onDismissRequest = onDismiss, properties = DialogProperties(usePlatformDefaultWidth = false, decorFitsSystemWindows = false)) {
    var shown by remember { mutableStateOf(false) }
    LaunchedEffect(Unit) { shown = true }
    val pop by animateFloatAsState(if (shown) 1f else 0f, spring(dampingRatio = 0.7f, stiffness = Spring.StiffnessMediumLow), label = "pop")
    Box(
      Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.45f * minOf(pop * 1.5f, 1f)))
        .clickable(remember { MutableInteractionSource() }, null, onClick = onDismiss)
        .safeDrawingPadding().imePadding().padding(16.dp),
      contentAlignment = Alignment.Center,
    ) {
      Column(
        modifier.widthIn(max = 460.dp).fillMaxWidth()
          .graphicsLayer { alpha = pop.coerceIn(0f, 1f); translationY = (1 - pop) * 12.dp.toPx(); scaleX = 0.97f + 0.03f * pop; scaleY = scaleX }
          .shadow(24.dp, RoundedCornerShape(26.dp))
          .clip(RoundedCornerShape(26.dp)).background(MaterialTheme.colorScheme.surface).border(Tokens.border, RoundedCornerShape(26.dp))
          .clickable(remember { MutableInteractionSource() }, null) {}
          .verticalScroll(rememberScrollState()).padding(padding),
        content = content,
      )
    }
  }
}

/** `.empty`: a glyph, a headline and a line. */
@Composable
fun Empty(glyph: String, headline: String, caption: String, modifier: Modifier = Modifier) {
  Column(modifier.fillMaxWidth().padding(horizontal = 24.dp, vertical = 64.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(8.dp)) {
    Text(glyph, fontSize = 44.sp)
    Text(headline, fontWeight = FontWeight.SemiBold, color = Tokens.text2)
    if (caption.isNotEmpty()) Text(caption, fontSize = 13.sp, color = Tokens.text2, textAlign = androidx.compose.ui.text.style.TextAlign.Center)
  }
}

/** A row with what fills it, left to right (`.row`: 56 tall, a line above). */
@Composable
fun WebRow(modifier: Modifier = Modifier, first: Boolean = false, content: @Composable RowScope.() -> Unit) {
  Column(modifier) {
    if (!first) Box(Modifier.fillMaxWidth().height(1.dp).background(Tokens.line))
    Row(Modifier.fillMaxWidth().heightIn(min = 52.dp).padding(horizontal = 14.dp, vertical = 8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp), content = content)
  }
}

@Composable
fun ProvideText(color: Color, content: @Composable () -> Unit) = CompositionLocalProvider(LocalContentColor provides color, content = content)

/**
 * A row as the event card draws them (and so the edit window and Settings):
 * a grey icon (or room for one), then the content, at least 46 high, a
 * hairline under it unless it is the last. No boxes: the lines are enough.
 */
@Composable
fun IconRow(icon: String?, modifier: Modifier = Modifier, last: Boolean = false, top: Boolean = false, content: @Composable RowScope.() -> Unit) {
  Column(modifier) {
    Row(
      Modifier.fillMaxWidth().heightIn(min = 46.dp).padding(horizontal = 2.dp, vertical = 6.dp),
      verticalAlignment = if (top) Alignment.Top else Alignment.CenterVertically,
      horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
      Box(Modifier.width(20.dp).then(if (top) Modifier.padding(top = 10.dp) else Modifier)) { if (icon != null) Icon24(icon, size = 20.dp, stroke = 1.8f, tint = Tokens.text2) }
      ProvideText(MaterialTheme.colorScheme.onSurface) { content() }
    }
    if (!last) Box(Modifier.fillMaxWidth().height(1.dp).background(Tokens.line))
  }
}

/** A small grey heading over a few rows ("Views"). */
@Composable
fun RowsLabel(text: String, modifier: Modifier = Modifier) {
  Text(text.uppercase(), modifier.padding(top = 18.dp, bottom = 2.dp), fontSize = 12.sp, fontWeight = FontWeight.SemiBold, letterSpacing = 0.5.sp, color = Tokens.text3)
}
