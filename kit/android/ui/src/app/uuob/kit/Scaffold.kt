package app.uuob.kit

import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectHorizontalDragGestures
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.json.JSONObject

// The frames every app is built in, taken from Calendar's (docs/design.md):
// the sidebar (a drawer on phones), the toolbar, the detail card and the
// editor, the empty state, list rows and the quiet tints.

/**
 * True inside a SidebarScaffold that shows its sidebar as a drawer (a phone):
 * the page then has a menu button to open it. Measured on the whole screen,
 * not the page, so a tablet with its sidebar showing has none.
 */
val LocalDrawerMode = androidx.compose.runtime.compositionLocalOf { false }

/** Whether the phone's drawer is open, for the menu button that opens it. */
class DrawerState {
  var open by mutableStateOf(false)
}

/**
 * The sidebar and the rest of the screen. On a phone (narrower than 600) the
 * sidebar is a drawer, hidden until the menu button or a swipe from the left
 * edge opens it. On a tablet it is expanded (240) or folded to a rail (64)
 * with the toggle at its bottom, remembered on this device. `footer` (Settings,
 * a note) shares the toggle's line in the expanded sidebar, left of it, and
 * ends the phone's drawer.
 */
@Composable
fun SidebarScaffold(
  drawer: DrawerState,
  sidebar: @Composable ColumnScope.() -> Unit,
  rail: @Composable ColumnScope.() -> Unit,
  footer: @Composable RowScope.() -> Unit = {},
  content: @Composable () -> Unit,
) = SidebarScaffold(drawer.open, { drawer.open = it }, sidebar = sidebar, rail = rail, footer = footer, content = content)

/** The same, with the app keeping whether the drawer is open (`drawerOpen`, `onDrawer`). */
@Composable
fun SidebarScaffold(
  drawerOpen: Boolean,
  onDrawer: (Boolean) -> Unit,
  sidebar: @Composable ColumnScope.() -> Unit,
  rail: @Composable ColumnScope.() -> Unit,
  footer: @Composable RowScope.() -> Unit = {},
  content: @Composable () -> Unit,
) {
  val prefs = LocalContext.current.getSharedPreferences("uuob.kit", 0)
  var folded by remember { mutableStateOf(prefs.getBoolean("sidebar.folded", false)) }
  BoxWithConstraints(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background)) {
    val phone = maxWidth < 600.dp
    if (phone) {
      Box(Modifier.fillMaxSize().pointerInput(Unit) {
        // A swipe from the left edge opens the drawer.
        var fromEdge = false
        detectHorizontalDragGestures(onDragStart = { fromEdge = it.x < 24.dp.toPx() }) { _, dx -> if (fromEdge && dx > 12) onDrawer(true) }
      }) { androidx.compose.runtime.CompositionLocalProvider(LocalDrawerMode provides true) { content() } }
      BackHandler(drawerOpen) { onDrawer(false) }
      AnimatedVisibility(drawerOpen, enter = fadeIn(), exit = fadeOut()) {
        Box(Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.35f)).clickable(remember { MutableInteractionSource() }, null) { onDrawer(false) })
      }
      AnimatedVisibility(drawerOpen, enter = slideInHorizontally { -it }, exit = slideOutHorizontally { -it }) {
        Column(Modifier.width(300.dp).fillMaxHeight().shadow(16.dp).background(MaterialTheme.colorScheme.surface).safeDrawingPadding().padding(horizontal = 12.dp, vertical = 16.dp)) {
          sidebar()
          Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, content = footer)
        }
      }
    } else {
      Row(Modifier.fillMaxSize()) {
        Column(Modifier.width(if (folded) 64.dp else 240.dp).fillMaxHeight().background(MaterialTheme.colorScheme.surface).safeDrawingPadding().padding(horizontal = if (folded) 8.dp else 12.dp, vertical = 16.dp)) {
          Column(Modifier.weight(1f), horizontalAlignment = if (folded) Alignment.CenterHorizontally else Alignment.Start, content = if (folded) rail else sidebar)
          Row(Modifier.fillMaxWidth(), horizontalArrangement = if (folded) Arrangement.Center else Arrangement.End, verticalAlignment = Alignment.CenterVertically) {
            if (!folded) Row(Modifier.weight(1f), verticalAlignment = Alignment.CenterVertically, content = footer)
            IconBtn("sidebar", { folded = !folded; prefs.edit().putBoolean("sidebar.folded", folded).apply() }, size = 18.dp, description = if (folded) "Show sidebar" else "Hide sidebar")
          }
        }
        Box(Modifier.width(1.dp).fillMaxHeight().background(Tokens.line))
        Box(Modifier.weight(1f).fillMaxHeight()) { content() }
      }
    }
  }
}

/** A row of the sidebar: `leading` (an icon, a dot, a check), the name, `trailing`; the chosen one raised. */
@Composable
fun SidebarRow(
  title: String,
  selected: Boolean = false,
  onClick: () -> Unit,
  leading: @Composable () -> Unit = {},
  trailing: @Composable RowScope.() -> Unit = {},
) {
  Row(
    Modifier.fillMaxWidth().heightIn(min = 40.dp).clip(RoundedCornerShape(10.dp))
      .background(if (selected) MaterialTheme.colorScheme.onSurface.copy(alpha = 0.08f) else Color.Transparent)
      .clickable(onClick = onClick).padding(horizontal = 10.dp),
    verticalAlignment = Alignment.CenterVertically,
    horizontalArrangement = Arrangement.spacedBy(10.dp),
  ) {
    Box(Modifier.width(22.dp), contentAlignment = Alignment.Center) { leading() }
    Text(title, Modifier.weight(1f), fontSize = 15.sp, fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Medium, maxLines = 1, overflow = TextOverflow.Ellipsis)
    ProvideText(Tokens.text2) { trailing() }
  }
}

/** An icon of the rail: 40 square, a soft tile when chosen. */
@Composable
fun RailButton(description: String, selected: Boolean = false, onClick: () -> Unit, mark: @Composable () -> Unit) {
  Box(
    Modifier.padding(vertical = 2.dp).size(40.dp).clip(RoundedCornerShape(12.dp))
      .background(if (selected) MaterialTheme.colorScheme.onSurface.copy(alpha = 0.08f) else Color.Transparent)
      .clickable(onClickLabel = description, onClick = onClick)
      // Read out by TalkBack: the icon alone says nothing.
      .semantics { contentDescription = description; role = Role.Button; this.selected = selected },
    contentAlignment = Alignment.Center,
  ) { ProvideText(if (selected) MaterialTheme.colorScheme.onSurface else Tokens.text2) { mark() } }
}

/** A rail button with an icon. */
@Composable
fun RailIcon(icon: String, description: String, selected: Boolean = false, onClick: () -> Unit) =
  RailButton(description, selected, onClick) { Icon24(icon, size = 20.dp, tint = if (selected) MaterialTheme.colorScheme.onSurface else Tokens.text2) }

/** The round blue + (New event, New task, Add photos). */
@Composable
fun PlusButton(onClick: () -> Unit, description: String = "New") {
  Box(
    Modifier.size(36.dp).clip(CircleShape).background(MaterialTheme.colorScheme.primary).clickable(onClickLabel = description, onClick = onClick)
      .semantics { contentDescription = description; role = Role.Button },
    contentAlignment = Alignment.Center,
  ) {
    Icon24("plus", size = 20.dp, stroke = 2.2f, tint = Color.White)
  }
}

/**
 * The top of a page: the menu button on a phone (opens the drawer), the title,
 * the page's controls, the round +. The title gives way first.
 */
@Composable
fun Toolbar(
  title: androidx.compose.ui.text.AnnotatedString,
  compact: Boolean = false,
  menu: (() -> Unit)? = null,
  add: (() -> Unit)? = null,
  /** What the + adds, for TalkBack ("New task", "Add photos"). */
  addDescription: String = "New",
  controls: @Composable RowScope.() -> Unit = {},
) {
  Row(Modifier.fillMaxWidth().padding(bottom = if (compact) 12.dp else 18.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
    if (menu != null) IconBtn("menu", menu, description = "Menu")
    Text(title, Modifier.weight(1f), fontSize = if (compact) 28.sp else 34.sp, letterSpacing = (-0.8).sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
    controls()
    if (add != null) PlusButton(add, addDescription)
  }
}

/** The detail card (Calendar's event card): the title with its `actions` and ✕, then the app's rows. */
@Composable
fun DetailSheet(title: String, onClose: () -> Unit, actions: @Composable RowScope.() -> Unit = {}, content: @Composable ColumnScope.() -> Unit) {
  WebSheet(onClose, padding = 22.dp) {
    Row(verticalAlignment = Alignment.Top) {
      Text(title, Modifier.weight(1f).padding(top = 4.dp), fontSize = 22.sp, fontWeight = FontWeight.SemiBold, letterSpacing = (-0.4).sp)
      actions()
      IconBtn("close", onClose, size = 19.dp, description = "Close")
    }
    content()
  }
}

/**
 * The editor (Calendar's event editor): the title big and editable with ✕,
 * a line under it (`under`), the app's icon rows, then Delete on the left and
 * Save on the right.
 */
@Composable
fun EditSheet(
  title: String,
  onTitle: (String) -> Unit,
  placeholder: String = "Title",
  isNew: Boolean = false,
  canSave: Boolean = true,
  onCancel: () -> Unit,
  onSave: () -> Unit,
  onDelete: (() -> Unit)? = null,
  /** Something was changed: a tap outside or Back asks before it goes. */
  dirty: Boolean = false,
  under: @Composable RowScope.() -> Unit = {},
  content: @Composable ColumnScope.() -> Unit,
) {
  var asking by remember { mutableStateOf(false) }
  // A tap outside, or Back: gone at once unless that would lose changes.
  WebSheet({ if (dirty) asking = true else onCancel() }, padding = 22.dp) {
    if (asking) {
      Text("Discard changes?", fontSize = 20.sp, fontWeight = FontWeight.SemiBold)
      Text("What you changed here isn’t saved.", fontSize = 15.sp, color = Tokens.text2, modifier = Modifier.padding(top = 6.dp))
      Row(Modifier.fillMaxWidth().padding(top = 18.dp), horizontalArrangement = Arrangement.spacedBy(10.dp, Alignment.End)) {
        Btn("Keep editing", { asking = false })
        Btn("Discard", onCancel, kind = BtnKind.Danger)
      }
      return@WebSheet
    }
    Row(verticalAlignment = Alignment.CenterVertically) {
      WebField(title, onTitle, Modifier.weight(1f), placeholder = placeholder, minHeight = 48.dp, border = false, textStyle = TextStyle(fontSize = 22.sp, fontWeight = FontWeight.SemiBold))
      IconBtn("close", onCancel, size = 19.dp, description = "Cancel")
    }
    Row(Modifier.padding(start = 2.dp, bottom = 6.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(7.dp), content = under)
    content()
    Row(Modifier.fillMaxWidth().padding(top = 18.dp), horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.CenterVertically) {
      if (onDelete != null && !isNew) Btn("Delete", onDelete, kind = BtnKind.Danger)
      Spacer(Modifier.weight(1f))
      if (isNew) Btn("Cancel", onCancel)
      Btn(if (isNew) "Add" else "Save", onSave, kind = BtnKind.Primary, enabled = canSave)
    }
  }
}

/** An empty list or library: an icon in a soft tile, a line, maybe one button. */
@Composable
fun EmptyState(icon: String, headline: String, modifier: Modifier = Modifier, color: Color = Tokens.text3, action: Pair<String, () -> Unit>? = null) {
  Column(modifier.fillMaxWidth().padding(vertical = 56.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(10.dp)) {
    Box(Modifier.size(60.dp).clip(RoundedCornerShape(18.dp)).background(Tokens.surface2), contentAlignment = Alignment.Center) { Icon24(icon, size = 28.dp, tint = color) }
    Text(headline, fontSize = 17.sp, fontWeight = FontWeight.SemiBold, color = Tokens.text2)
    if (action != null) Btn(action.first, action.second, Modifier.padding(top = 8.dp), kind = BtnKind.Primary)
  }
}

/** A row of a list: `leading` (a check, a dot), a title with an optional grey line, `trailing`, a hairline under it. */
@Composable
fun ItemRow(
  title: String,
  modifier: Modifier = Modifier,
  detail: String = "",
  done: Boolean = false,
  last: Boolean = false,
  leading: @Composable () -> Unit = {},
  trailing: @Composable RowScope.() -> Unit = {},
) {
  Column(modifier) {
    Row(Modifier.fillMaxWidth().padding(horizontal = 10.dp, vertical = 10.dp), verticalAlignment = Alignment.Top, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
      leading()
      Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
        Text(title, fontSize = 16.sp, color = if (done) Tokens.text3 else MaterialTheme.colorScheme.onSurface, textDecoration = if (done) TextDecoration.LineThrough else null, maxLines = 2, overflow = TextOverflow.Ellipsis)
        if (detail.isNotEmpty()) Text(detail, fontSize = 13.sp, color = Tokens.text2, maxLines = 1, overflow = TextOverflow.Ellipsis)
      }
      trailing()
    }
    if (!last) Box(Modifier.fillMaxWidth().padding(horizontal = 10.dp).height(1.dp).background(Tokens.line))
  }
}

/** What kind of item something is, for a quiet shade of its colour. */
data class TintKind(
  val repeating: Boolean = false,
  val joinable: Boolean = false,
  val allDay: Boolean = false,
  val past: Boolean = false,
  val declined: Boolean = false,
  val tentative: Boolean = false,
) {
  companion object {
    /** From the core's fields: repeats, join, past, answer, solid / row (all-day). */
    fun of(e: JSONObject) = TintKind(
      repeating = e.optBoolean("repeats"), joinable = e.optBoolean("join"), allDay = e.optBoolean("solid") || e.optBoolean("row"),
      past = e.optBoolean("past"), declined = e.optString("answer") == "no", tentative = e.optString("answer") == "maybe",
    )
  }
}

/** How much of the colour fills the item, how strong its edge is, how faded it is. */
data class Tint(val fill: Float, val edge: Float, val fade: Float)

/** A meeting to join a little deeper, a repeating one lighter with a softer edge, all-day a touch lighter, past and declined faded. */
fun tint(k: TintKind): Tint {
  var fill = if (k.allDay) 0.22f else 0.24f
  if (k.joinable) fill += 0.08f else if (k.repeating) fill -= 0.09f
  return Tint(fill, if (k.repeating) 0.55f else 1f, if (k.declined) 0.45f else if (k.past) 0.6f else 1f)
}
