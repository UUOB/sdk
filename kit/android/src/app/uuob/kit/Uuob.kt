package app.uuob.kit

import android.app.Activity
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.net.Uri
import app.uuob.client.UuobClient
import app.uuob.client.UuobException
import app.uuob.client.UuobProver
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.launch
import org.json.JSONArray
import org.json.JSONObject
import org.json.JSONTokener

/**
 * The UUOB SDK for an Android app (client/, through UniFFI): one
 * per process. The app has no backend of its own; its data lives in UUOB.
 *
 *   Uuob.init(ctx, app)
 *   Uuob.client.put("photos", month, null, record.toString())
 *   Uuob.events.collect { e -> if (e.optString("type") == "changed") reload() }
 */
object Uuob {
  /** Lives as long as the process. */
  val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
  lateinit var client: UuobClient
    private set
  lateinit var app: AppInfo
    private set

  private val changes = MutableSharedFlow<JSONObject>(extraBufferCapacity = 64)
  /** {"type":"changed","container":…}, {"type":"state",…}, {"type":"asking",…}, {"type":"settings",…}. */
  val events: SharedFlow<JSONObject> = changes

  fun init(ctx: Context, app: AppInfo) {
    if (::client.isInitialized) return
    this.app = app
    client = UuobClient(app.id, app.name, ctx.packageName, ctx.filesDir.absolutePath)
    client.setProver(Prover(ctx.applicationContext))
    scope.launch {
      while (true) changes.emit(JSONObject(client.nextEvent()))
    }
    UuobSettings.start()
  }

  /** Brings UUOB to the front at `uuob://<what>` (connect, share, start). False if it isn't installed. */
  fun open(ctx: Context, what: String): Boolean = try {
    ctx.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse("uuob://$what")).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
    true
  } catch (_: Exception) {
    false
  }

  /** Shares a container: UUOB comes to the front, where the user picks people. */
  suspend fun share(ctx: Context, container: String) {
    if (!client.remote()) open(ctx, "share")
    client.share(container)
  }
}

/**
 * Another app could listen on UUOB's port while UUOB is not running and
 * collect what apps send there. So the SDK sends nothing to the port until
 * the server there answers a random nonce as UUOB answers it here: an ordered
 * broadcast to app.uuob (its ProveReceiver), which only the real UUOB
 * receives. Called by the SDK off the main thread; it waits for the answer.
 */
private class Prover(private val ctx: Context) : UuobProver {
  override fun prove(nonce: String): String? {
    val done = CountDownLatch(1)
    var answer: String? = null
    val intent = Intent("app.uuob.PROVE").setPackage("app.uuob").putExtra("nonce", nonce).addFlags(Intent.FLAG_INCLUDE_STOPPED_PACKAGES)
    val receiver = object : BroadcastReceiver() {
      override fun onReceive(c: Context, i: Intent) {
        answer = resultData
        done.countDown()
      }
    }
    ctx.sendOrderedBroadcast(intent, null, receiver, null, Activity.RESULT_OK, null, null)
    done.await(10, TimeUnit.SECONDS)
    return answer
  }
}

/** A call that failed: `detail` is for the user. */
val Throwable.userMessage: String
  get() = (this as? UuobException.Failed)?.detail ?: message ?: "Something went wrong"

/** `unreachable`, `unauthorized`, `cancelled`, … ("" for other failures). */
val Throwable.code: String
  get() = (this as? UuobException.Failed)?.code ?: ""

fun json(text: String?): Any? = text?.let { JSONTokener(it).nextValue().takeIf { v -> v != JSONObject.NULL } }
fun JSONArray?.objects(): List<JSONObject> = if (this == null) emptyList() else (0 until length()).mapNotNull { optJSONObject(it) }
fun objects(text: String?): List<JSONObject> = (json(text) as? JSONArray).objects()
fun JSONObject.text(key: String): String? = if (isNull(key)) null else optString(key)
fun params(vararg pairs: Pair<String, Any?>) = JSONObject().apply { pairs.forEach { (k, v) -> put(k, v ?: JSONObject.NULL) } }
