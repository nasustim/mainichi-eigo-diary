use yew::{component, html, Callback, Html, Properties};

use crate::model::DiaryEntry;

/// Maximum characters to show in the body snippet.
const SNIPPET_LEN: usize = 80;

/// Props for the `EntryList` component.
#[derive(Properties, PartialEq)]
pub struct EntryListProps {
    pub entries: Vec<DiaryEntry>,
    /// Called with the entry id when the user clicks an entry (to edit it).
    pub on_select: Callback<String>,
    /// Called with the entry id when the user clicks the delete button.
    pub on_delete: Callback<String>,
}

fn snippet(body: &str) -> String {
    if body.len() <= SNIPPET_LEN {
        body.to_string()
    } else {
        format!("{}…", &body[..SNIPPET_LEN])
    }
}

/// Renders the list of diary entries with select and delete actions.
#[component]
pub fn EntryList(props: &EntryListProps) -> Html {
    if props.entries.is_empty() {
        return html! {
            <p class="empty-message">{ "No entries yet. Write your first diary entry above!" }</p>
        };
    }

    html! {
        <ul class="entry-list">
            { for props.entries.iter().map(|entry| {
                let id = entry.id.clone();
                let id_del = entry.id.clone();
                let on_select = props.on_select.clone();
                let on_delete = props.on_delete.clone();

                html! {
                    <li key={entry.id.clone()} class="entry-item">
                        <div
                            class="entry-body"
                            onclick={Callback::from(move |_| on_select.emit(id.clone()))}
                        >
                            <span class="entry-date">{ &entry.created_at }</span>
                            <p class="entry-snippet">{ snippet(&entry.body) }</p>
                        </div>
                        <button
                            class="btn-danger"
                            onclick={Callback::from(move |_| on_delete.emit(id_del.clone()))}
                        >
                            { "Delete" }
                        </button>
                    </li>
                }
            })}
        </ul>
    }
}
