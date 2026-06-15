use yew::{component, html, Html};

#[component]
fn App() -> Html {
    html! {
        <main>
            <h1>{ "Mainichi Eigo Diary" }</h1>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        assert_eq!(2 + 2, 4);
    }
}
