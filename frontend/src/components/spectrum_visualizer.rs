use std::cell::RefCell;
use std::rc::Rc;
use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::AudioContext;

/// Number of frequency bars in the spectrum display
const BAR_COUNT: usize = 64;

/// Spectrum visualizer that connects to the audio element via Web Audio API
/// and renders real-time frequency bars on a canvas.
#[component]
pub fn SpectrumVisualizer() -> Element {
    let is_playing = use_context::<crate::app::PlaybackContext>().read().is_playing;

    use_effect(move || {
        if is_playing {
            setup_analyser();
        }
    });

    rsx! {
        canvas {
            id: "spectrum-canvas",
            class: "spectrum-canvas",
            width: "256",
            height: "64",
            aria_hidden: "true",
        }
    }
}

/// Set up the Web Audio API analyser node and start the render loop.
/// This function uses a static variable to ensure the analyser is only created once,
/// since create_media_element_source can only be called once per audio element.
fn setup_analyser() {
    use std::sync::OnceLock;

    static INIT: OnceLock<()> = OnceLock::new();

    // Only initialize once - create_media_element_source can only be called once per element
    INIT.set(()).ok();

    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };

    // Get the audio element
    let audio = match get_audio_element() {
        Some(a) => a,
        None => return,
    };

    // Create or resume AudioContext
    let audio_context = match AudioContext::new() {
        Ok(ctx) => ctx,
        Err(_) => return,
    };

    // Resume the audio context - browsers suspend by default until user gesture
    let _ = audio_context.resume();

    // Create a media element source node (can only be done once per element)
    let source = match audio_context.create_media_element_source(&audio) {
        Ok(src) => src,
        Err(_) => {
            // Source already created, just get the analyser from the stored state
            return;
        }
    };

    // Create analyser node
    let analyser = match web_sys::AnalyserNode::new(&audio_context) {
        Ok(a) => a,
        Err(_) => return,
    };

    // Configure the analyser
    analyser.set_fft_size(256);
    analyser.set_smoothing_time_constant(0.8);

    // Connect: source -> analyser -> destination (speakers)
    let _ = source.connect_with_audio_node(&analyser);
    let destination = audio_context.destination();
    let _ = analyser.connect_with_audio_node(&destination);

    // Get the canvas
    let doc = match window.document() {
        Some(d) => d,
        None => return,
    };

    let canvas = match doc.get_element_by_id("spectrum-canvas") {
        Some(c) => c,
        None => return,
    };

    let canvas = match canvas.dyn_ref::<web_sys::HtmlCanvasElement>() {
        Some(c) => c.clone(),
        None => return,
    };

    let context = match canvas.get_context("2d") {
        Ok(ctx) => ctx,
        Err(_) => return,
    };

    let ctx = match context.and_then(|c| c.dyn_ref::<web_sys::CanvasRenderingContext2d>().map(|r| r.clone())) {
        Some(c) => c,
        None => return,
    };

    // Start the animation loop using requestAnimationFrame
    let buffer_length = analyser.frequency_bin_count() as usize;
    let data = Rc::new(RefCell::new(vec![0u8; buffer_length]));
    let ctx_rc = Rc::new(RefCell::new(ctx));
    let analyser_rc = Rc::new(RefCell::new(analyser));
    let window_rc = Rc::new(RefCell::new(window.clone()));

    // Create a self-referencing closure using Rc
    let closure_rc: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));

    {
        let data = Rc::clone(&data);
        let ctx = Rc::clone(&ctx_rc);
        let analyser = Rc::clone(&analyser_rc);
        let window = Rc::clone(&window_rc);
        let closure_for_raf = Rc::clone(&closure_rc);

        let c = Closure::wrap(Box::new(move |_time: f64| {
            // Get fresh frequency data from analyser
            analyser.borrow().get_byte_frequency_data(&mut data.borrow_mut());

            // Draw the frame
            draw_frame(&ctx.borrow(), &data.borrow(), buffer_length);

            // Request next frame
            let win = window.borrow();
            if let Some(ref c) = *closure_for_raf.borrow() {
                let _ = win.request_animation_frame(c.as_ref().unchecked_ref());
            }
        }) as Box<dyn FnMut(f64)>);

        closure_rc.borrow_mut().replace(c);
    }

    // Start the animation loop
    {
        let win = window_rc.borrow();
        if let Some(ref c) = *closure_rc.borrow() {
            let _ = win.request_animation_frame(c.as_ref().unchecked_ref());
        }
    } // Drop the borrow here

    // Keep all references alive for the animation loop
    Box::leak(Box::new((closure_rc, data, ctx_rc, analyser_rc, window_rc)));
}

/// Get the hidden audio element.
fn get_audio_element() -> Option<web_sys::HtmlMediaElement> {
    let window = web_sys::window()?;
    let doc = window.document()?;
    let el = doc.get_element_by_id("radio-audio")?;
    el.dyn_ref::<web_sys::HtmlMediaElement>().map(|r| r.clone())
}

/// Draw a single frame of the spectrum visualization.
fn draw_frame(
    ctx: &web_sys::CanvasRenderingContext2d,
    data: &[u8],
    buffer_length: usize,
) {
    let width = 256;
    let height = 64;

    // Clear the canvas with a dark background
    ctx.set_fill_style_str("rgba(15, 23, 42, 0.9)");
    ctx.fill_rect(0.0, 0.0, width as f64, height as f64);

    // Calculate bar width and spacing
    let bar_width = 3.0;
    let gap = 1.0;
    let step = buffer_length / BAR_COUNT;

    for i in 0..BAR_COUNT {
        let value = data[i * step] as f64;
        let max_height = height as f64 - 4.0; // Leave 2px padding top and bottom
        let bar_height = (value / 255.0 * max_height).max(1.0);

        let x = i as f64 * (bar_width + gap);
        let y = height as f64 - bar_height;

        // Color interpolation: blue (#3B82F6) at low frequencies, purple (#8B5CF6) at high
        let intensity = value / 255.0;
        let r = (59.0 + (139.0 - 59.0) * intensity) as u8;
        let g = (130.0 + (92.0 - 130.0) * intensity) as u8;
        let b = (246.0 + (246.0 - 246.0) * intensity) as u8;

        ctx.set_fill_style_str(&format!(
            "rgba({}, {}, {}, {})",
            r, g, b,
            0.6 + 0.4 * intensity // More intense bars are more opaque
        ));

        // Draw rounded bar
        draw_rounded_bar(ctx, x, y, bar_width, bar_height, 2.0);
    }
}

/// Draw a rounded rectangle bar.
fn draw_rounded_bar(
    ctx: &web_sys::CanvasRenderingContext2d,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radius: f64,
) {
    let r = radius.min(height / 2.0).min(width / 2.0);

    ctx.begin_path();
    ctx.move_to(x + r, y);
    ctx.line_to(x + width - r, y);
    ctx.arc_to(x + width, y, x + width, y + r, r);
    ctx.line_to(x + width, y + height);
    ctx.line_to(x, y + height);
    ctx.line_to(x, y + r);
    ctx.arc_to(x, y, x + r, y, r);
    ctx.close_path();
    ctx.fill();
}
