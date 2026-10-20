use dioxus::prelude::*;
use shared::RadioStation;
use crate::components::{StationList, AudioPlayer};

#[derive(Clone, Default)]
pub struct PlaybackState {
    pub current_station: Option<RadioStation>,
    pub is_playing: bool,
}

pub type PlaybackContext = Signal<PlaybackState>;

#[component]
pub fn App() -> Element {
    use_context_provider(|| {
        Signal::new(PlaybackState::default())
    });

    use_effect(|| {
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            let _ = doc.set_title("Caribbean Radio Player");
        }
    });

    rsx! {
        div { class: "app", role: "main",
            header { class: "header",
                div { class: "header-icon", "📻" }
                h1 { "Caribbean Radio" }
                p { class: "subtitle", "Live streaming from the islands" }
            }

            main { class: "main-content",
                StationList {}
                AudioPlayer {}
            }
        }
    }
}
