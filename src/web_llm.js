// Web LLM wrapper — loaded as a wasm-bindgen JS snippet.
// Pinned to exact CDN version: @mlc-ai/web-llm@0.2.84
import * as webllm from "https://esm.run/@mlc-ai/web-llm@0.2.84";

/**
 * Create an MLC engine for the given model.
 * @param {string} modelId  - The WebLLM model ID string.
 * @param {Function} progressCb - Called with (progress: number, text: string).
 * @returns {Promise<*>} Resolves to the engine instance.
 */
export async function createEngine(modelId, progressCb) {
    const engine = await webllm.CreateMLCEngine(modelId, {
        initProgressCallback: (report) => {
            progressCb(report.progress, report.text);
        },
    });
    return engine;
}

/**
 * Proofread the given diary text using the loaded engine.
 * Returns ONLY the corrected text (no markdown fences, no explanation).
 * @param {*} engine - The engine returned by createEngine.
 * @param {string} text - The diary entry text to proofread.
 * @param {string} systemPrompt - The system prompt (owned by the Rust layer).
 * @returns {Promise<string>} The corrected text.
 */
export async function proofread(engine, text, systemPrompt) {
    const reply = await engine.chat.completions.create({
        messages: [
            { role: "system", content: systemPrompt },
            { role: "user", content: text },
        ],
    });
    return reply.choices[0].message.content ?? "";
}

/**
 * Returns true if WebGPU is available in this browser environment.
 * @returns {boolean}
 */
export function isWebGpuAvailable() {
    return typeof navigator !== "undefined" && !!navigator.gpu;
}
