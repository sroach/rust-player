use dioxus::prelude::*;
use shared::RadioStation;
use crate::app::PlaybackContext;

fn get_base_url() -> String {
    if let Some(window) = web_sys::window() {
        let loc = window.location();
        if let (Ok(protocol), Ok(host)) = (loc.protocol(), loc.host()) {
            return format!("{}{}", protocol, host);
        }
    }
    "http://localhost:3001".to_string()
}

fn get_initial(label: &str) -> String {
    label.chars().next().unwrap_or('R').to_uppercase().to_string()
}

#[component]
pub fn StationList() -> Element {
    let mut playback = use_context::<PlaybackContext>();
    let stations = use_signal::<Vec<RadioStation>>(Vec::new);
    let loading = use_signal(|| true);
    let error = use_signal::<Option<String>>(|| None);

    // Fetch stations on mount
    use_effect({
        let stations = stations.clone();
        let loading = loading.clone();
        let error = error.clone();
        move || {
            let mut stations = stations.clone();
            let mut loading = loading.clone();
            let mut error = error.clone();
            dioxus::prelude::spawn(async move {
                let url = format!("{}/api/stations", get_base_url());
                match reqwest::get(&url).await {
                    Ok(response) => {
                        match response.text().await {
                            Ok(text) => {
                                let parsed: Vec<RadioStation> = serde_json::from_str(&text).unwrap_or_default();
                                stations.set(parsed);
                                loading.set(false);
                            }
                            Err(e) => {
                                loading.set(false);
                                error.set(Some(format!("Failed to read response: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        loading.set(false);
                        error.set(Some(format!("Failed to load stations: {}", e)));
                    }
                }
            });
        }
    });

    let mut select_station = move |station: RadioStation| {
        playback.with_mut(|state| {
            state.current_station = Some(station.clone());
            state.is_playing = true;
        });
    };

    rsx! {
        div { class: "station-list", role: "region", aria_label: "Station list",
            if loading() {
                div { class: "loading", role: "status", "Loading stations..." }
            } else if let Some(err) = error() {
                div { class: "error", role: "alert", "{err}" }
            } else if stations.read().is_empty() {
                div { class: "empty", "No stations available" }
            } else {
                div { class: "stations-grid", role: "list",
                    for station in stations.iter() {
                        {
                            let is_current = playback.read().current_station.as_ref().map(|c| c.index == station.index).unwrap_or(false);
                            let station_clone = station.clone();
                            let station_clone_key = station.clone();
                            let initial = get_initial(&station.label);
                            rsx! {
                                div {
                                    key: "{station.index}",
                                    class: if is_current { "station-card active" } else { "station-card" },
                                    role: "listitem",
                                    tabindex: "0",
                                    aria_label: format!("{} radio station", station.label),
                                    onclick: move |_| select_station(station_clone.clone()),
                                    onkeydown: move |ev| {
                                        let key_str = format!("{}", ev.key());
                                        if key_str == "Enter" || key_str == " " {
                                            ev.prevent_default();
                                            select_station(station_clone_key.clone());
                                        }
                                    },
                                    div { class: "station-icon", "{initial}" }
                                    div { class: "station-info",
                                        h3 { "{station.label}" }
                                        div { class: "station-genres",
                                            for genre in station.type_.iter() {
                                                span { class: "genre-tag", "{genre}" }
                                            }
                                        }
                                    }
                                    if is_current {
                                        div { class: "playing-indicator", "Playing" }
                                    } else {
                                        div { class: "play-hint", "Click to play" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
