// ---------------------------------------------------------------------------
// WASM-only helpers that call into JS.
// These are NOT unit-tested on the host target; they rely on js_sys::Date.
// ---------------------------------------------------------------------------

/// Return the current UTC time as an ISO-8601 string, e.g. "2026-06-15T12:34:56.789Z".
pub fn now_iso() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::new_0()
            .to_iso_string()
            .as_string()
            .unwrap_or_else(|| "1970-01-01T00:00:00.000Z".to_string())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Fallback for host compilation (tests). Should not be called in tests.
        "1970-01-01T00:00:00.000Z".to_string()
    }
}

/// Generate a unique-enough ID based on the current timestamp (ms since epoch)
/// plus a thread-local counter so rapid calls don't collide.
pub fn new_id() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        use std::cell::Cell;
        thread_local! {
            static COUNTER: Cell<u32> = const { Cell::new(0) };
        }
        let ts = js_sys::Date::now() as u64;
        let count = COUNTER.with(|c| {
            let v = c.get();
            c.set(v.wrapping_add(1));
            v
        });
        format!("{}-{}", ts, count)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Fallback for host compilation (tests). Should not be called in tests.
        "host-stub-id".to_string()
    }
}
