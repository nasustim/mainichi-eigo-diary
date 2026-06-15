mod app;
mod components;
mod model;
mod storage;
mod util;

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
