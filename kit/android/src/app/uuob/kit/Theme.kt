package app.uuob.kit

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.MaterialExpressiveTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.MotionScheme
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

/**
 * UUOB's design system (ui/tokens.css), the same in every app: calm greys,
 * white cards with a hairline, one blue accent, the calendar colours. It
 * follows the phone's light and dark mode, as the web version did.
 */
object Colors {
  /** Set by AppTheme: the light and dark palettes differ a little, as in tokens.css. */
  internal var dark = false
  private val light = mapOf(
    "blue" to Color(0xFF007AFF), "red" to Color(0xFFFF3B30), "orange" to Color(0xFFFF9500), "yellow" to Color(0xFFFFCC00),
    "green" to Color(0xFF34C759), "teal" to Color(0xFF30B0C7), "purple" to Color(0xFFAF52DE), "pink" to Color(0xFFFF2D55),
  )
  private val night = mapOf(
    "blue" to Color(0xFF0A84FF), "red" to Color(0xFFFF453A), "orange" to Color(0xFFFF9F0A), "yellow" to Color(0xFFFFD60A),
    "green" to Color(0xFF30D158), "teal" to Color(0xFF40C8E0), "purple" to Color(0xFFBF5AF2), "pink" to Color(0xFFFF375F),
  )
  /** A calendar colour by the name a record gives it ("blue" when unknown). */
  fun of(name: String?): Color = (if (dark) night else light).let { it[name] ?: it.getValue("blue") }
  val named: Map<String, Color> get() = if (dark) night else light
  val ok get() = of("green")
  val warn get() = of("orange")
}

/** The tokens the screens use beyond Material's colour roles. */
object Tokens {
  val radius = 12.dp
  val radiusLg = 18.dp
  val field = RoundedCornerShape(12.dp)
  val card = RoundedCornerShape(18.dp)
  /** `--line`: a card's hairline. */
  val line: Color @Composable get() = if (Colors.dark) Color(0x14FFFFFF) else Color(0x14000000)
  /** `--surface-2`: fields, the segmented control, secondary buttons. */
  val surface2: Color @Composable get() = if (Colors.dark) Color(0xFF26262A) else Color(0xFFF0F0F3)
  /** `--accent-soft`. */
  val accentSoft: Color @Composable get() = MaterialTheme.colorScheme.primary.copy(alpha = if (Colors.dark) 0.16f else 0.12f)
  val text2: Color @Composable get() = MaterialTheme.colorScheme.onSurfaceVariant
  val text3: Color @Composable get() = MaterialTheme.colorScheme.outline
  val border: BorderStroke @Composable get() = BorderStroke(1.dp, line)
}

/** `.btn`: a pill, `.btn.lg` 50 tall. */
object Pill {
  val shape = RoundedCornerShape(999.dp)
  val padding = PaddingValues(horizontal = 22.dp, vertical = 0.dp)
  @Composable fun secondary() = ButtonDefaults.buttonColors(containerColor = Tokens.surface2, contentColor = MaterialTheme.colorScheme.onSurface)
  @Composable fun danger() = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.error.copy(alpha = 0.12f), contentColor = MaterialTheme.colorScheme.error)
}

/** UUOB's light and dark palettes (tokens.css), whatever the phone's wallpaper. */
@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun AppTheme(@Suppress("UNUSED_PARAMETER") accent: Color, content: @Composable () -> Unit) {
  val dark = isSystemInDarkTheme()
  Colors.dark = dark
  val colors = if (dark) {
    darkColorScheme(
      primary = Color(0xFF0A84FF), onPrimary = Color.White, primaryContainer = Color(0x290A84FF), onPrimaryContainer = Color(0xFFF5F5F7),
      secondaryContainer = Color(0x1AFFFFFF), onSecondaryContainer = Color(0xFFF5F5F7),
      background = Color(0xFF0B0B0D), onBackground = Color(0xFFF5F5F7),
      surface = Color(0xFF161618), onSurface = Color(0xFFF5F5F7), onSurfaceVariant = Color(0x9EF5F5F7),
      surfaceContainerLowest = Color(0xFF0B0B0D), surfaceContainerLow = Color(0xFF161618), surfaceContainer = Color(0xFF1C1C1F),
      surfaceContainerHigh = Color(0xFF1C1C1F), surfaceContainerHighest = Color(0xFF26262A),
      outline = Color(0x61F5F5F7), outlineVariant = Color(0x14FFFFFF),
      error = Color(0xFFFF453A), onError = Color.White,
    )
  } else {
    lightColorScheme(
      primary = Color(0xFF007AFF), onPrimary = Color.White, primaryContainer = Color(0x1F007AFF), onPrimaryContainer = Color(0xFF1D1D1F),
      secondaryContainer = Color(0x14000000), onSecondaryContainer = Color(0xFF1D1D1F),
      background = Color(0xFFF5F5F7), onBackground = Color(0xFF1D1D1F),
      surface = Color(0xFFFFFFFF), onSurface = Color(0xFF1D1D1F), onSurfaceVariant = Color(0x9E1D1D1F),
      surfaceContainerLowest = Color.White, surfaceContainerLow = Color.White, surfaceContainer = Color.White,
      surfaceContainerHigh = Color.White, surfaceContainerHighest = Color(0xFFF0F0F3),
      outline = Color(0x611D1D1F), outlineVariant = Color(0x14000000),
      error = Color(0xFFFF3B30), onError = Color.White,
    )
  }
  val base = Typography()
  val type = base.copy(
    displaySmall = TextStyle(fontSize = 34.sp, lineHeight = 39.sp, fontWeight = FontWeight.Bold, letterSpacing = (-0.8).sp),
    headlineLarge = TextStyle(fontSize = 34.sp, lineHeight = 39.sp, fontWeight = FontWeight.Bold, letterSpacing = (-0.8).sp),
    headlineSmall = TextStyle(fontSize = 22.sp, lineHeight = 28.sp, fontWeight = FontWeight.SemiBold, letterSpacing = (-0.4).sp),
    titleLarge = TextStyle(fontSize = 22.sp, lineHeight = 28.sp, fontWeight = FontWeight.SemiBold, letterSpacing = (-0.4).sp),
    bodyLarge = base.bodyLarge.copy(fontSize = 17.sp, letterSpacing = (-0.2).sp),
    bodyMedium = base.bodyMedium.copy(fontSize = 15.sp, letterSpacing = (-0.1).sp),
    bodySmall = base.bodySmall.copy(fontSize = 13.sp, letterSpacing = 0.sp),
    labelMedium = TextStyle(fontSize = 13.sp, fontWeight = FontWeight.SemiBold, letterSpacing = 0.5.sp),
    labelLarge = base.labelLarge.copy(fontSize = 15.sp, fontWeight = FontWeight.SemiBold, letterSpacing = 0.sp),
  )
  MaterialExpressiveTheme(colorScheme = colors, typography = type, motionScheme = MotionScheme.expressive(), content = content)
}
