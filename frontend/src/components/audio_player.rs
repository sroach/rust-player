use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use crate::app::PlaybackContext;
use crate::components::SpectrumVisualizer;

fn get_audio_element() -> Option<web_sys::HtmlMediaElement> {
    let window = web_sys::window()?;
    let doc = window.document()?;
    let el = doc.get_element_by_id("radio-audio")?;
    el.dyn_ref::<web_sys::HtmlMediaElement>().map(|r| r.clone())
}

// Convert a direct stream URL to a proxied URL for same-origin Web Audio API access
fn get_stream_url(url: &str) -> String {
    let encoded: String = url::form_urlencoded::byte_serialize(url.as_bytes())
        .map(|s| s.to_string())
        .collect();
    format!("/api/stream?url={}", encoded)
}

fn get_initial(label: &str) -> String {
    label.chars().next().unwrap_or('R').to_uppercase().to_string()
}

#[component]
pub fn AudioPlayer() -> Element {
    let mut playback = use_context::<PlaybackContext>();
    let mut volume = use_signal(|| 0.8_f64);

    // Play/pause when playback state changes
    use_effect(move || {
        let playing = playback.read().is_playing;
        let station = playback.read().current_station.clone();
        let vol = volume();

        if let Some(ref station) = station {
            if let Some(audio) = get_audio_element() {
                // Set crossOrigin before loading the source - required for Web Audio API
                audio.set_cross_origin(Some("anonymous"));
                audio.set_src(&get_stream_url(&station.url));
                audio.set_volume(vol);

                if playing {
                    let _ = audio.play();
                } else {
                    let _ = audio.pause();
                }
            }
        }
    });

    // Update volume when it changes
    use_effect(move || {
        let vol = volume();
        if let Some(audio) = get_audio_element() {
            audio.set_volume(vol);
        }
    });

    let current_station = playback.read().current_station.clone();
    let is_playing = playback.read().is_playing;

    rsx! {
        div { class: "audio-player", role: "region", aria_label: "Audio player",
            if let Some(ref station) = current_station {
                div { class: "player-info",
                    div { class: "player-icon", "{get_initial(&station.label)}" }
                    div { class: "player-details",
                        h3 { "{station.label}" }
                        div { class: "player-genres",
                            for genre in station.type_.iter() {
                                span { class: "genre-tag", "{genre}" }
                            }
                        }
                    }
                }
            } else {
                div { class: "player-empty",
                    p { "Select a station to start listening" }
                }
            }

            if current_station.is_some() {
                div { class: "player-controls",
                    button {
                        class: "play-button",
                        aria_label: if is_playing { "Pause" } else { "Play" },
                        onclick: move |_| {
                            playback.with_mut(|state| {
                                state.is_playing = !state.is_playing;
                            });
                        },
                        if is_playing {
                            "Pause"
                        } else {
                            "Play"
                        }
                    }

                    // Spectrum visualizer
                    SpectrumVisualizer {}

                    div { class: "volume-control",
                        span { class: "volume-icon", aria_hidden: "true", "🔊" }
                        input {
                            id: "volume",
                            r#type: "range",
                            min: "0",
                            max: "1",
                            step: "0.01",
                            value: "{volume}",
                            aria_label: "Volume",
                            oninput: move |ev| {
                                if let Ok(val) = ev.value().parse::<f64>() {
                                    volume.set(val);
                                }
                            },
                        }
                    }
                }
            }

            // Hidden audio element
            audio {
                id: "radio-audio",
                class: "radio-audio",
                style: "display: none;",
                crossorigin: "anonymous",
            }
        }
    }
}
