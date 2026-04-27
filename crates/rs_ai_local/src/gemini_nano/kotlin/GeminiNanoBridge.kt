/**
 * Kotlin bridge for Android's Gemini Nano Prompt API.
 *
 * This interface must be implemented by your Android app and passed to Rust
 * via JNI. The Rust side will call these methods through the JNI bridge.
 *
 * # Example implementation
 *
 * ```kotlin
 * import android.content.Context
 * import com.google.android.gms.common.ConnectionResult
 * import com.google.android.gms.common.GoogleApiAvailability
 * import com.google.ai.edge.aicore.GenerativeModel
 * import com.google.ai.edge.aicore.generationConfig
 * import kotlinx.coroutines.Dispatchers
 * import kotlinx.coroutines.withContext
 *
 * class GeminiNanoBridgeImpl(private val context: Context) : GeminiNanoBridge {
 *
 *     private var model: GenerativeModel? = null
 *     private val sessions = mutableMapOf<String, Chat>()
 *
 *     override fun isAvailable(): Boolean {
 *         return GoogleApiAvailability.getInstance()
 *             .isGooglePlayServicesAvailable(context) == ConnectionResult.SUCCESS
 *     }
 *
 *     override fun downloadState(): Int {
 *         // Check if model is downloaded via AI Core
 *         return when (AiCoreStatus.getStatus(context)) {
 *             AiCoreStatus.DOWNLOADED -> 2
 *             AiCoreStatus.DOWNLOADING -> 1
 *             else -> 0
 *         }
 *     }
 *
 *     override fun requestDownload(): Boolean {
 *         // Trigger model download via AI Core
 *         return AiCoreManager.requestDownload(context)
 *     }
 *
 *     override fun capabilities(): Map<String, Boolean> {
 *         return mapOf(
 *             "textGeneration" to true,
 *             "summarization" to false,
 *             "rewriting" to false
 *         )
 *     }
 *
 *     override fun generate(prompt: String, configJson: String): String {
 *         return runBlocking(Dispatchers.IO) {
 *             val config = parseConfig(configJson)
 *             val model = getOrCreateModel(config)
 *             val response = model.generateContent(prompt)
 *             response.text ?: ""
 *         }
 *     }
 *
 *     override fun createSession(configJson: String): String {
 *         val config = parseConfig(configJson)
 *         val model = getOrCreateModel(config)
 *         val sessionId = UUID.randomUUID().toString()
 *         sessions[sessionId] = model.startChat()
 *         return sessionId
 *     }
 *
 *     override fun sendMessage(sessionId: String, message: String): String {
 *         val chat = sessions[sessionId]
 *             ?: throw IllegalArgumentException("Session not found: $sessionId")
 *         return runBlocking(Dispatchers.IO) {
 *             val response = chat.sendMessage(message)
 *             response.text ?: ""
 *         }
 *     }
 *
 *     override fun closeSession(sessionId: String): Boolean {
 *         sessions.remove(sessionId)
 *         return true
 *     }
 *
 *     private fun parseConfig(configJson: String): GenerationConfig {
 *         val json = JSONObject(configJson)
 *         return generationConfig {
 *             temperature = json.optDouble("temperature", 0.7).toFloat()
 *             topK = json.optInt("topK", 40)
 *             maxOutputTokens = json.optInt("maxTokens", 1024)
 *         }
 *     }
 *
 *     private fun getOrCreateModel(config: GenerationConfig): GenerativeModel {
 *         return model ?: GenerativeModel.from(
 *             modelName = "gemini-nano",
 *             config = config
 *         ).also { model = it }
 *     }
 * }
 * ```
 *
 * # Building
 *
 * Add to your `build.gradle.kts`:
 *
 * ```kotlin
 * dependencies {
 *     implementation("com.google.ai.edge.aicore:aicore:0.1.0")
 * }
 * ```
 *
 * # Usage from Rust
 *
 * ```rust,no_run
 * use std::sync::Arc;
 * use jni::JavaVM;
 * use rs_ai_gemini_nano::{GeminiNanoProvider, JniGeminiNanoBridge};
 *
 * let vm = /* get JavaVM from Android */;
 * let bridge = /* create Kotlin bridge instance */;
 * let jni_bridge = Arc::new(JniGeminiNanoBridge::new(Arc::new(vm), bridge));
 * let provider = GeminiNanoProvider::new(jni_bridge);
 * let model = provider.model();
 * ```
 */
interface GeminiNanoBridge {
    /** Returns true if Gemini Nano is available on this device. */
    fun isAvailable(): Boolean

    /**
     * Returns the current download state:
     * - 0 = NotDownloaded
     * - 1 = Downloading
     * - 2 = Downloaded
     * - 3 = Failed
     */
    fun downloadState(): Int

    /** Requests model download. Returns true if the request was accepted. */
    fun requestDownload(): Boolean

    /** Returns a map of capability names to availability. */
    fun capabilities(): Map<String, Boolean>

    /**
     * Generates text from a prompt.
     * @param prompt The input prompt
     * @param configJson JSON string with NanoSessionConfig fields
     * @return The generated text
     */
    fun generate(prompt: String, configJson: String): String

    /**
     * Creates a new multi-turn session.
     * @param configJson JSON string with NanoSessionConfig fields
     * @return A unique session identifier
     */
    fun createSession(configJson: String): String

    /**
     * Sends a message in an existing session.
     * @param sessionId The session identifier
     * @param message The message to send
     * @return The model's response
     */
    fun sendMessage(sessionId: String, message: String): String

    /**
     * Closes a session and frees resources.
     * @param sessionId The session identifier
     * @return true if the session was closed successfully
     */
    fun closeSession(sessionId: String): Boolean
}
