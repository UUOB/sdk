package app.uuob.kit

import android.Manifest
import android.app.Activity
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import androidx.core.app.NotificationCompat

/**
 * What UUOB hands an app on Android, checked, and the app's alerts shown as
 * its own notifications. UUOB keeps the time (it runs all along and learns of
 * changes made on other devices); when an alert is due it sends the app
 * `app.uuob.SHOW_ALERT` with `key` (tells alerts apart), `title` and `text`.
 *
 *   class AlertReceiver : BroadcastReceiver() {
 *     override fun onReceive(ctx: Context, i: Intent) = UuobAlerts.show(ctx, i, R.drawable.ic_notify, MainActivity::class.java)
 *   }
 */
object UuobAlerts {
  /** Did UUOB send this? Its `from` extra is a PendingIntent, and Android itself
   *  says which app created that one. Only apps the user allowed in UUOB are ever
   *  given one, so every other app is kept out. */
  @Suppress("DEPRECATION")
  fun fromUuob(intent: Intent): Boolean = intent.getParcelableExtra<PendingIntent>("from")?.creatorPackage == "app.uuob"

  /** Shows the alert in `intent` as this app's notification; tapping it opens `open`. */
  fun show(ctx: Context, intent: Intent, smallIcon: Int, open: Class<out Activity>) {
    if (intent.action != "app.uuob.SHOW_ALERT" || !fromUuob(intent)) return
    val key = intent.getStringExtra("key") ?: return
    val nm = ctx.getSystemService(NotificationManager::class.java)
    nm.createNotificationChannel(NotificationChannel("alerts", "Alerts", NotificationManager.IMPORTANCE_HIGH))
    val tap = PendingIntent.getActivity(
      ctx, key.hashCode(), Intent(ctx, open).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK),
      PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
    )
    val n = NotificationCompat.Builder(ctx, "alerts")
      .setSmallIcon(smallIcon)
      .setContentTitle(intent.getStringExtra("title") ?: "")
      .apply { intent.getStringExtra("text")?.takeIf { it.isNotEmpty() }?.let { setContentText(it) } }
      .setCategory(NotificationCompat.CATEGORY_REMINDER)
      .setPriority(NotificationCompat.PRIORITY_HIGH)
      .setContentIntent(tap)
      .setAutoCancel(true)
      .build()
    nm.notify(key.hashCode(), n)
  }

  /** Android 13+ shows notifications only once allowed: asks, once. */
  fun askPermission(activity: Activity) {
    if (Build.VERSION.SDK_INT >= 33 && activity.checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED) {
      activity.requestPermissions(arrayOf(Manifest.permission.POST_NOTIFICATIONS), 1)
    }
  }
}
