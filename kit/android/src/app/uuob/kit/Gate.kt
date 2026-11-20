package app.uuob.kit

import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.scale
import androidx.compose.ui.graphics.drawscope.translate
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.LifecycleResumeEffect
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

/**
 * Who the app is, for UUOB and for its connect screen. `noun`: what the app
 * keeps, as in "your photos". `updateFeed`/`updateKey`: where the app's own
 * updates are published and the key they are signed with (empty: none).
 */
data class AppInfo(
  val id: String,
  val name: String,
  val noun: String,
  val icon: ImageVector,
  val color: Color,
  val version: String,
  val updateFeed: String = "",
  val updateKey: String = "",
)

private enum class Step { Checking, Unreachable, Connect, Waiting, Ready }

/** A code is a number and three words, like 42-amber-river-falcon. */
private fun codeOk(code: String) = code.split(Regex("[^a-zA-Z0-9]+")).count { it.isNotEmpty() } == 4

/**
 * The app until it is connected to UUOB, then `content`. The same in every
 * app (ui/Gate.svelte): UUOB isn't found → install it or use a code from
 * another device; found → Connect, and the user taps Allow in UUOB.
 */
@Composable
fun Gate(updater: Updater, content: @Composable () -> Unit) {
  val app = Uuob.app
  val client = Uuob.client
  val ctx = LocalContext.current
  val scope = rememberCoroutineScope()
  // Connected before: show the app at once (from cache) and check in the background.
  var step by remember { mutableStateOf(if (client.hasToken()) Step.Ready else Step.Checking) }
  var offline by remember { mutableStateOf(false) }
  var asking by remember { mutableStateOf(false) }
  var code by remember { mutableStateOf("") }
  var useCode by remember { mutableStateOf(false) }
  var busy by remember { mutableStateOf(false) }
  var error by remember { mutableStateOf("") }
  var started by remember { mutableStateOf(false) }
  var connecting by remember { mutableStateOf<Job?>(null) }

  suspend fun check() {
    val s = client.check()
    if (step == Step.Waiting) return
    // UUOB isn't running (e.g. after a restart): start it once; it steps aside at once.
    if (s == "unreachable" && !started) {
      started = true
      Uuob.open(ctx, "start")
    }
    if (s == "unreachable" && step == Step.Ready) {
      offline = true
      return
    }
    step = when (s) {
      "connected" -> Step.Ready
      "disconnected" -> Step.Connect
      else -> Step.Unreachable
    }
    if (step == Step.Ready) offline = false
  }

  fun connect(withCode: Boolean) {
    error = ""
    val from = step
    if (!withCode) {
      step = Step.Waiting
      Uuob.open(ctx, "connect")
    }
    busy = withCode
    connecting = scope.launch {
      try {
        client.connect(if (withCode) code.trim() else null)
        step = Step.Ready
        offline = false
        code = ""
      } catch (e: Exception) {
        if (e.code == "cancelled") {
          step = Step.Connect
        } else {
          error = e.userMessage
          // Connected before (or refused): only a code from UUOB connects it now.
          if (!withCode && e.code == "permission_denied") useCode = true
          // A refused code leaves the user where they typed it.
          step = if (e.code == "unreachable") Step.Unreachable else if (withCode) from else Step.Connect
        }
      } finally {
        busy = false
      }
    }
  }

  LaunchedEffect(Unit) {
    launch {
      Uuob.events.collect { e ->
        when (e.optString("type")) {
          "state" -> {
            val s = e.optString("state")
            if (s == "disconnected" && step == Step.Ready) step = Step.Connect
            offline = s == "unreachable" && step == Step.Ready
          }
          "asking" -> asking = e.optBoolean("asking")
        }
      }
    }
    check()
    while (true) {
      delay(3000)
      if ((step == Step.Unreachable || offline) && !busy) check()
    }
  }
  // Back in front: a frozen stream starts afresh, and maybe there is an update.
  LifecycleResumeEffect(Unit) {
    client.resume()
    updater.check()
    onPauseOrDispose {}
  }

  if (step == Step.Ready) {
    Box(Modifier.fillMaxSize()) {
      content()
      val note = if (offline) "Waiting for UUOB. Your changes are kept and sent when it’s back." else if (asking) "Answer in UUOB on one of your devices." else null
      if (note != null) {
        Surface(
          shape = RoundedCornerShape(22.dp), tonalElevation = 6.dp, shadowElevation = 6.dp,
          modifier = Modifier.align(Alignment.BottomCenter).safeDrawingPadding().padding(16.dp),
        ) {
          Text(note, Modifier.padding(horizontal = 18.dp, vertical = 10.dp), color = if (offline) Colors.warn else MaterialTheme.colorScheme.onSurface, textAlign = TextAlign.Center)
        }
      }
    }
    return
  }
  if (step == Step.Checking) return

  Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background).safeDrawingPadding(), contentAlignment = Alignment.Center) {
    Column(
      Modifier.widthIn(max = 420.dp).fillMaxWidth().verticalScroll(rememberScrollState()).padding(horizontal = 24.dp, vertical = 32.dp),
      horizontalAlignment = Alignment.CenterHorizontally,
    ) {
      Row(Modifier.align(Alignment.End)) { UpdateButton(updater) }
      AppAndUuob(app, busy = step == Step.Waiting)
      Spacer(Modifier.height(28.dp))
      val codeForm: @Composable () -> Unit = {
        Row(Modifier.fillMaxWidth().padding(top = 18.dp), horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.CenterVertically) {
          OutlinedTextField(
            code, { code = it }, Modifier.weight(1f), singleLine = true,
            placeholder = { Text("42-amber-river-falcon") },
            keyboardOptions = KeyboardOptions(capitalization = KeyboardCapitalization.None, autoCorrectEnabled = false, imeAction = ImeAction.Go),
            keyboardActions = KeyboardActions(onGo = { if (codeOk(code) && !busy) connect(true) }),
          )
          Button(onClick = { connect(true) }, enabled = codeOk(code) && !busy, modifier = Modifier.height(56.dp)) { Text(if (busy) "Connecting…" else "Connect") }
        }
      }
      when (step) {
        Step.Unreachable -> {
          Title("${app.name} needs UUOB")
          Lead("Install UUOB on this device and open it.")
          Looking("Looking for UUOB…")
          OutlinedButton(onClick = { Uuob.open(ctx, "connect") }, Modifier.fillMaxWidth().height(52.dp)) { Text("Open UUOB") }
          HorizontalDivider(Modifier.padding(top = 32.dp, bottom = 28.dp))
          Text("Or use UUOB on another device", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold)
          codeForm()
          Error(error)
        }
        Step.Waiting -> {
          Title("Allow it in UUOB")
          Looking("Waiting for your answer…")
          OutlinedButton(onClick = { client.cancelConnect(); connecting?.cancel(); step = Step.Connect }, Modifier.fillMaxWidth().height(52.dp)) { Text("Cancel") }
        }
        else -> {
          Title("Connect ${app.name}")
          Row(Modifier.padding(top = 22.dp, bottom = 14.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.size(8.dp).clip(CircleShape).background(Colors.ok))
            Text("  UUOB is running on ${client.device() ?: "this device"}", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
          }
          Button(onClick = { connect(false) }, Modifier.fillMaxWidth().height(52.dp)) { Text("Connect to UUOB") }
          Error(error)
          if (useCode) {
            codeForm()
          } else {
            TextButton(onClick = { useCode = true }, Modifier.padding(top = 18.dp)) { Text("Use a code instead") }
          }
        }
      }
    }
  }
}

@Composable
private fun Title(text: String) =
  Text(text, style = MaterialTheme.typography.headlineMedium, fontWeight = FontWeight.Bold, textAlign = TextAlign.Center)

@Composable
private fun Lead(text: String) =
  Text(text, Modifier.padding(top = 12.dp), style = MaterialTheme.typography.bodyLarge, color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = TextAlign.Center)

@Composable
private fun Error(text: String) {
  if (text.isNotEmpty()) Text(text, Modifier.padding(top = 12.dp), color = MaterialTheme.colorScheme.error, textAlign = TextAlign.Center)
}

@Composable
private fun Looking(text: String) =
  Row(Modifier.padding(top = 22.dp, bottom = 14.dp), verticalAlignment = Alignment.CenterVertically) {
    CircularProgressIndicator(Modifier.size(14.dp), strokeWidth = 2.dp)
    Text("  $text", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
  }

/** The app's tile, three dots, UUOB's logo. The dots blink while waiting. */
@Composable
private fun AppAndUuob(app: AppInfo, busy: Boolean) {
  val blink = rememberInfiniteTransition(label = "dots")
  Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(14.dp)) {
    Box(Modifier.size(68.dp).clip(RoundedCornerShape(19.dp)).background(app.color), contentAlignment = Alignment.Center) {
      Icon(app.icon, null, Modifier.size(32.dp), tint = Color.White)
    }
    Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
      repeat(3) { i ->
        val a by blink.animateFloat(0.25f, 1f, infiniteRepeatable(tween(600, delayMillis = i * 200), RepeatMode.Reverse), label = "dot$i")
        Box(Modifier.size(6.dp).alpha(if (busy) a else 1f).clip(CircleShape).background(if (busy) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant))
      }
    }
    UuobLogo(68.dp)
  }
}

/** UUOB's logo (Mesh), drawn as ui/Logo.svelte draws it. */
@Composable
fun UuobLogo(size: Dp) {
  Canvas(Modifier.size(size).clip(RoundedCornerShape(size * 0.28f)).background(Color.Black)) {
    val k = this.size.width / 1024f
    val w = Color.White
    scale(k * 1.08f, pivot = Offset(this.size.width / 2, this.size.height / 2)) {
      translate((this.size.width - 1024f) / 2, (this.size.height - 1024f) / 2) {
        fun line(x1: Float, y1: Float, x2: Float, y2: Float, sw: Float) = drawLine(w, Offset(x1, y1), Offset(x2, y2), sw, StrokeCap.Round)
        fun box(x: Float, y: Float, bw: Float, bh: Float, r: Float) = drawRoundRect(w, Offset(x, y), Size(bw, bh), CornerRadius(r), style = Stroke(29.9f))
        line(430.5f, 429.3f, 349.1f, 579.2f, 22f); drawCircle(w, 34f, Offset(389.8f, 504.3f))
        line(593.5f, 429.3f, 672.2f, 574.3f, 22f); drawCircle(w, 34f, Offset(632.8f, 501.8f))
        line(435.5f, 724.4f, 582.9f, 724.4f, 22f); drawCircle(w, 34f, Offset(509.2f, 724.4f))
        box(411.2f, 192.4f, 201.6f, 134.4f, 19.6f); line(374.8f, 360.4f, 649.2f, 360.4f, 29.9f)
        box(208.7f, 612.4f, 123.2f, 224f, 28f); drawCircle(w, 15.4f, Offset(270.3f, 788.8f))
        box(647.3f, 623.6f, 212.8f, 86.8f, 19.6f); box(647.3f, 738.4f, 212.8f, 86.8f, 19.6f)
        drawCircle(w, 12.6f, Offset(812.5f, 667f)); drawCircle(w, 12.6f, Offset(812.5f, 781.8f))
      }
    }
  }
}
