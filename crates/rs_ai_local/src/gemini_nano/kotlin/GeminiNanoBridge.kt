/**
 * Kotlin helper for Android's ML Kit GenAI Prompt API (Gemini Nano on-device).
 *
 * This is a companion helper called from Rust via JNI. Include this class
 * in your Android app's source tree so the Rust JNI bridge can find it.
 *
 * # Using from Rust
 *
 * The Rust `JniGeminiNanoBridge` calls:
 * - `GenAI.getClient().isAvailable()` directly via JNI
 * - `GeminiNanoBridge.generate(requestJson)` via this helper
 *
 * # Adding to your Android app
 *
 * Place this file somewhere in your app's Kotlin source (e.g.,
 * `app/src/main/java/com/rs_ai/GeminiNanoBridge.kt`) and add the dependency:
 *
 * ```kotlin
 * // build.gradle.kts
 * dependencies {
 *     implementation("com.google.mlkit:genai-prompt:1.0.0")
 * }
 * ```
 */
package com.rs_ai

import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.util.Base64
import com.google.mlkit.genai.prompt.GenAI
import com.google.mlkit.genai.prompt.GenerateContentRequest
import com.google.mlkit.genai.prompt.GenerationConfig
import com.google.mlkit.genai.prompt.ImagePart
import com.google.mlkit.genai.prompt.TextPart
import org.json.JSONArray
import org.json.JSONObject
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

/**
 * Static helper for the ML Kit GenAI Prompt API, called from Rust via JNI.
 */
object GeminiNanoBridge {

    /**
     * Generate content from a JSON-serialized request.
     *
     * Request JSON format:
     * ```json
     * {
     *   "parts": [
     *     {"text": "Hello"},
     *     {"base64": "...", "mime_type": "image/jpeg"}
     *   ],
     *   "config": {
     *     "temperature": 0.2,
     *     "candidate_count": 1,
     *     "max_output_tokens": 100
     *   }
     * }
     * ```
     *
     * Response JSON format:
     * ```json
     * {"text": "Generated response text"}
     * ```
     */
    @JvmStatic
    fun generate(requestJson: String): String {
        val request = JSONObject(requestJson)

        // Parse config
        val genConfig = GenerationConfig().apply {
            val cfg = request.optJSONObject("config")
            if (cfg != null) {
                if (cfg.has("temperature")) {
                    temperature = cfg.getDouble("temperature").toFloat()
                }
                if (cfg.has("candidate_count")) {
                    candidateCount = cfg.getInt("candidate_count")
                }
                if (cfg.has("max_output_tokens")) {
                    maxOutputTokens = cfg.getInt("max_output_tokens")
                }
            }
        }

        // Build content parts
        val partsArray: JSONArray = request.optJSONArray("parts") ?: JSONArray()
        val contentParts = mutableListOf<Any>()

        for (i in 0 until partsArray.length()) {
            val part = partsArray.getJSONObject(i)
            when {
                part.has("text") -> {
                    contentParts.add(TextPart(part.getString("text")))
                }
                part.has("base64") -> {
                    val base64 = part.getString("base64")
                    val mimeType = part.optString("mime_type", "image/jpeg")
                    val bytes = Base64.decode(base64, Base64.DEFAULT)
                    val bitmap = BitmapFactory.decodeByteArray(bytes, 0, bytes.size)
                    if (bitmap != null) {
                        contentParts.add(ImagePart(bitmap, mimeType))
                    }
                }
            }
        }

        val contentRequest = GenerateContentRequest(
            parts = contentParts,
            config = genConfig
        )

        // The ML Kit API uses listeners, so we block with a latch
        val latch = CountDownLatch(1)
        var resultText = ""
        var errorText: String? = null

        GenAI.getClient().generateContent(contentRequest)
            .addOnSuccessListener { response ->
                resultText = response.text ?: ""
                latch.countDown()
            }
            .addOnFailureListener { exception ->
                errorText = exception.message ?: "Unknown error in ML Kit API"
                latch.countDown()
            }

        // Wait up to 30 seconds
        val finished = latch.await(30, TimeUnit.SECONDS)

        if (!finished) {
            throw RuntimeException("ML Kit generateContent timed out after 30s")
        }

        if (errorText != null) {
            throw RuntimeException(errorText)
        }

        return JSONObject().apply {
            put("text", resultText)
        }.toString()
    }
}
