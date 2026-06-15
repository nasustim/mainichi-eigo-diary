mod app;
mod components;
mod model;
mod portability;
mod proofread;
mod storage;
mod util;
mod web_llm;

fn main() {
    yew::Renderer::<app::App>::new().render();
}

#[cfg(test)]
mod tests {
    /// Smoke test — ensures the host-target build compiles and basic arithmetic works.
    #[test]
    fn smoke() {
        assert_eq!(2 + 2, 4);
    }
}
