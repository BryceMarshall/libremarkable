//! Data Collector Example
//!
//! A simple app that captures handwriting strokes and submits them to a backend via HTTP POST.
//! Supports random prompt generation from dictionary words with configurable punctuation.
//!
//! Usage:
//!   BACKEND_URL=http://your-server/api/samples ./data_collector
//!
//! Environment variables:
//!   BACKEND_URL              - Required. The URL to POST sample data to.
//!   DATA_COLLECTION_API_KEY  - Required. API key for authentication.
//!   DEVICE_ID                - Optional. Override the device identifier.
//!   PROMPT_TEXT              - Optional. Manual prompt (only used with PROMPT_SOURCE=manual)
//!
//! Prompt generation variables:
//!   PROMPT_SOURCE            - "random" | "file" | "manual" (default: "random")
//!   WORDS_FILE               - Path to external word list file
//!   PROMPT_WEIGHT_SINGLE     - Weight for single words (default: 40)
//!   PROMPT_WEIGHT_PHRASE     - Weight for phrases (default: 40)
//!   PROMPT_WEIGHT_SENTENCE   - Weight for sentences (default: 20)
//!   PHRASE_MIN_WORDS         - Min words in phrase (default: 2)
//!   PHRASE_MAX_WORDS         - Max words in phrase (default: 5)
//!   SENTENCE_MIN_WORDS       - Min words in sentence (default: 3)
//!   SENTENCE_MAX_WORDS       - Max words in sentence (default: 8)
//!   BASELINE_ENABLED         - "true" | "false" (default: "true")
//!   BASELINE_Y_OFFSET        - Pixels from canvas bottom (default: 100)
//!   RANDOM_SEED              - Optional seed for reproducibility

mod prompt_generator;
use prompt_generator::{GeneratedPrompt, PromptGenerator, PromptMetadata};

use libremarkable::appctx::ApplicationContext;
use libremarkable::framebuffer::cgmath;
use libremarkable::framebuffer::common::*;
use libremarkable::framebuffer::FramebufferDraw;
use libremarkable::framebuffer::FramebufferRefreshExt;
use libremarkable::input::{InputEvent, MultitouchEvent, WacomEvent, WacomPen};
use libremarkable::ui_extensions::element::{
    UIConstraintRefresh, UIElement, UIElementHandle, UIElementWrapper,
};

use once_cell::sync::Lazy;
use serde::Serialize;

use std::env;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// A single point in a stroke with position, pressure, and timestamp
#[derive(Debug, Clone, Serialize)]
struct StrokePoint {
    x: f32,
    y: f32,
    p: f32,  // pressure normalized 0.0-1.0
    t: u64,  // timestamp in milliseconds since epoch
}

/// The complete sample data matching the backend schema
#[derive(Debug, Serialize)]
struct SampleData {
    prompt_text: String,
    strokes_json: Vec<Vec<StrokePoint>>,
    device_id: String,
    device_created_at: String,  // ISO 8601 format
    prompt_metadata: PromptMetadata,
}

/// Canvas region for drawing (main area of the screen)
const CANVAS_REGION: mxcfb_rect = mxcfb_rect {
    top: 200,
    left: 50,
    height: 1500,
    width: 1304,
};

/// Baseline guide color (light gray)
const BASELINE_COLOR: color = color::GRAY(180);
/// Baseline guide thickness in pixels
const BASELINE_THICKNESS: u32 = 2;

// Global state for stroke collection
static STROKES: Lazy<Mutex<Vec<Vec<StrokePoint>>>> = Lazy::new(|| Mutex::new(Vec::new()));
static CURRENT_STROKE: Lazy<Mutex<Vec<StrokePoint>>> = Lazy::new(|| Mutex::new(Vec::new()));
static PEN_DOWN: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));
static LAST_POINT: Lazy<Mutex<Option<cgmath::Point2<i32>>>> = Lazy::new(|| Mutex::new(None));

// Global state for prompt generation
static PROMPT_GENERATOR: Lazy<Mutex<PromptGenerator>> =
    Lazy::new(|| Mutex::new(PromptGenerator::from_env()));
static CURRENT_PROMPT: Lazy<Mutex<Option<GeneratedPrompt>>> = Lazy::new(|| Mutex::new(None));

/// Get current timestamp in milliseconds since Unix epoch
fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Get device ID from /etc/machine-id or environment variable
fn get_device_id() -> String {
    if let Ok(id) = env::var("DEVICE_ID") {
        return id;
    }

    fs::read_to_string("/etc/machine-id")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown-device".to_string())
}

/// Get the backend URL from environment variable
fn get_backend_url() -> Option<String> {
    env::var("BACKEND_URL").ok()
}


fn get_api_key() -> Option<String> {
    env::var("DATA_COLLECTION_API_KEY").ok()
}

/// Draw baseline guide on the canvas if enabled
fn draw_baseline(app: &mut ApplicationContext<'_>) {
    let prompt = CURRENT_PROMPT.lock().unwrap();
    if let Some(ref p) = *prompt {
        if p.metadata.baseline_visible {
            let y_offset = p.metadata.baseline_y.unwrap_or(100);
            let baseline_y = CANVAS_REGION.top + CANVAS_REGION.height - y_offset;

            let fb = app.get_framebuffer_ref();
            let start = cgmath::Point2 {
                x: (CANVAS_REGION.left + 10) as i32,
                y: baseline_y as i32,
            };
            let end = cgmath::Point2 {
                x: (CANVAS_REGION.left + CANVAS_REGION.width - 10) as i32,
                y: baseline_y as i32,
            };
            fb.draw_line(start, end, BASELINE_THICKNESS, BASELINE_COLOR);
        }
    }
}

/// Advance to the next prompt and update the display
fn advance_prompt(app: &mut ApplicationContext<'_>) {
    // Generate next prompt
    let prompt = {
        let mut gen = PROMPT_GENERATOR.lock().unwrap();
        gen.next()
    };

    println!(
        "Prompt #{}: \"{}\" (type: {:?})",
        prompt.metadata.prompt_index, prompt.text, prompt.prompt_type
    );

    // Store current prompt
    {
        let mut current = CURRENT_PROMPT.lock().unwrap();
        *current = Some(prompt.clone());
    }

    // Update prompt display element
    if let Some(elem) = app.get_element_by_name("prompt") {
        let mut wrapper = elem.write();
        if let UIElement::Text { ref mut text, .. } = wrapper.inner {
            *text = format!("Write: {}", prompt.text);
        }
    }
    app.draw_element("prompt");
}

/// Clear button handler - resets the drawing canvas
fn on_clear(app: &mut ApplicationContext<'_>, _element: UIElementHandle) {
    // Clear stroke data
    {
        let mut strokes = STROKES.lock().unwrap();
        strokes.clear();
    }
    {
        let mut current = CURRENT_STROKE.lock().unwrap();
        current.clear();
    }
    {
        let mut last = LAST_POINT.lock().unwrap();
        *last = None;
    }

    // Clear the canvas region on screen
    let fb = app.get_framebuffer_ref();
    fb.fill_rect(
        CANVAS_REGION.top_left().cast().unwrap(),
        CANVAS_REGION.size().cast().unwrap().into(),
        color::WHITE,
    );

    // Redraw baseline if enabled
    draw_baseline(app);

    fb.refresh_balanced(&CANVAS_REGION);

    println!("Canvas cleared");
}

/// Submit button handler - POSTs collected data to backend
fn on_submit(app: &mut ApplicationContext<'_>, _element: UIElementHandle) {
    let backend_url = match get_backend_url() {
        Some(url) => url,
        None => {
            println!("ERROR: BACKEND_URL environment variable not set!");
            return;
        }
    };

    let api_key = match get_api_key() {
        Some(api_key) => api_key,
        None => {
            println!("ERROR: DATA_COLLECTION_API_KEY environment variable not set!");
            return;
        }
    };

    const recog_url: &str = "http://10.0.0.101:8081/recognition";

    // Finalize any current stroke
    {
        let mut current = CURRENT_STROKE.lock().unwrap();
        if !current.is_empty() {
            let mut strokes = STROKES.lock().unwrap();
            strokes.push(current.drain(..).collect());
        }
    }

    // Collect the stroke data
    let strokes_data: Vec<Vec<StrokePoint>> = {
        let strokes = STROKES.lock().unwrap();
        strokes.clone()
    };

    if strokes_data.is_empty() {
        println!("No strokes to submit");
        return;
    }

    // Get current prompt info
    let (prompt_text, prompt_metadata) = {
        let current = CURRENT_PROMPT.lock().unwrap();
        match &*current {
            Some(p) => (p.text.clone(), p.metadata.clone()),
            None => (
                "unknown".to_string(),
                PromptMetadata::default(),
            ),
        }
    };

    // Build the sample data
    let sample = SampleData {
        prompt_text,
        strokes_json: strokes_data,
        device_id: get_device_id(),
        device_created_at: chrono::Utc::now().to_rfc3339(),
        prompt_metadata,
    };

    println!("Submitting {} strokes to {}", sample.strokes_json.len(), backend_url);

    // Make HTTP POST request (in a separate thread to avoid blocking UI)
    std::thread::spawn(move || {
        match ureq::post(&backend_url)
            .set("Content-Type", "application/json")
            .set("X-API-Key", &api_key)
            .send_json(&sample)
        {
            Ok(response) => {
                println!("Success! Status: {}", response.status());
            }
            Err(e) => {
                println!("Error submitting data: {}", e);
            }
        }

        match ureq::post(&recog_url)
            .set("Content-Type", "application/json")
            .send_json(&sample)
        {
            Ok(response) => {
                println!("Recognized text: {:?}", response)
            }
            Err(e) => {
                println!("Error recognizing data: {}", e)
            }
        }
    });

    // Advance to next prompt
    advance_prompt(app);

    // Clear canvas (this will also redraw the baseline)
    on_clear(app, _element);
    println!("Data submitted, advanced to next prompt");
}

/// Handle Wacom stylus input for drawing
fn on_wacom_input(app: &mut ApplicationContext<'_>, event: WacomEvent) {
    match event {
        WacomEvent::Draw { position, pressure, .. } => {
            // Only draw within the canvas region
            if !CANVAS_REGION.contains_point(&position.cast().unwrap()) {
                return;
            }

            let point = StrokePoint {
                x: position.x,
                y: position.y,
                p: pressure as f32 / 4096.0,  // Normalize pressure (max is 4096)
                t: now_millis(),
            };

            // Add point to current stroke
            {
                let mut current = CURRENT_STROKE.lock().unwrap();
                current.push(point);
            }

            // Draw the stroke segment
            let fb = app.get_framebuffer_ref();
            let current_pos: cgmath::Point2<i32> = position.cast().unwrap();

            let mut last = LAST_POINT.lock().unwrap();
            if let Some(prev_pos) = *last {
                // Draw line from previous point to current
                let rect = fb.draw_line(
                    prev_pos,
                    current_pos,
                    2,  // Line width
                    color::BLACK,
                );
                fb.refresh_fast(&rect);
            } else {
                // First point - draw a small circle
                let rect = fb.draw_circle(current_pos.cast().unwrap(), 2, color::BLACK);
                fb.refresh_fast(&rect);
            }

            *last = Some(current_pos);
        }

        WacomEvent::InstrumentChange { pen, state } => {
            match pen {
                WacomPen::Touch => {
                    if state {
                        // Pen touched down - start new stroke
                        PEN_DOWN.store(true, Ordering::Relaxed);
                    } else {
                        // Pen lifted - finalize current stroke
                        PEN_DOWN.store(false, Ordering::Relaxed);

                        let mut current = CURRENT_STROKE.lock().unwrap();
                        if !current.is_empty() {
                            let mut strokes = STROKES.lock().unwrap();
                            strokes.push(current.drain(..).collect());
                            println!("Stroke completed. Total strokes: {}", strokes.len());
                        }

                        // Clear last point for next stroke
                        let mut last = LAST_POINT.lock().unwrap();
                        *last = None;
                    }
                }
                _ => {}
            }
        }

        WacomEvent::Hover { .. } => {
            // Clear last point when hovering to prevent connecting separate strokes
            if !PEN_DOWN.load(Ordering::Relaxed) {
                let mut last = LAST_POINT.lock().unwrap();
                *last = None;
            }
        }

        _ => {}
    }
}

fn main() {
    env_logger::init();

    // Check for required environment variable
    if get_backend_url().is_none() {
        eprintln!("WARNING: BACKEND_URL not set. Submit will fail.");
        eprintln!("Usage: BACKEND_URL=http://your-server/api/samples ./data_collector");
    }

    if get_api_key().is_none() {
        eprintln!("WARNING: DATA_COLLECTION_API_KEY not set. Submit will fail.");
        eprintln!("Usage: DATA_COLLECTION_API_KEY=<key>");
    }

    let mut app = ApplicationContext::default();
    app.clear(true);

    // Set canvas bounds for metadata (used for normalizing stroke coordinates)
    {
        let mut gen = PROMPT_GENERATOR.lock().unwrap();
        gen.set_canvas_bounds(
            CANVAS_REGION.left,
            CANVAS_REGION.top,
            CANVAS_REGION.width,
            CANVAS_REGION.height,
        );
    }

    // Generate first prompt
    let first_prompt = {
        let mut gen = PROMPT_GENERATOR.lock().unwrap();
        gen.next()
    };
    println!(
        "Prompt #{}: \"{}\" (type: {:?})",
        first_prompt.metadata.prompt_index, first_prompt.text, first_prompt.prompt_type
    );

    // Store current prompt
    {
        let mut current = CURRENT_PROMPT.lock().unwrap();
        *current = Some(first_prompt.clone());
    }

    // Add prompt text at the top
    app.add_element(
        "prompt",
        UIElementWrapper {
            position: cgmath::Point2 { x: 50, y: 50 },
            refresh: UIConstraintRefresh::Refresh,
            inner: UIElement::Text {
                foreground: color::BLACK,
                text: format!("Write: {}", first_prompt.text),
                scale: 50.0,
                border_px: 0,
            },
            ..Default::default()
        },
    );

    // Add instructions
    app.add_element(
        "instructions",
        UIElementWrapper {
            position: cgmath::Point2 { x: 50, y: 120 },
            refresh: UIConstraintRefresh::Refresh,
            inner: UIElement::Text {
                foreground: color::BLACK,
                text: "Draw with stylus, then tap Submit".to_string(),
                scale: 35.0,
                border_px: 0,
            },
            ..Default::default()
        },
    );

    // Draw the canvas border
    app.add_element(
        "canvas_border",
        UIElementWrapper {
            position: CANVAS_REGION.top_left().cast().unwrap(),
            refresh: UIConstraintRefresh::Refresh,
            inner: UIElement::Region {
                size: CANVAS_REGION.size().cast().unwrap(),
                border_px: 2,
                border_color: color::BLACK,
            },
            ..Default::default()
        },
    );

    // Clear button (bottom left)
    app.add_element(
        "clear_button",
        UIElementWrapper {
            position: cgmath::Point2 { x: 100, y: 1750 },
            refresh: UIConstraintRefresh::Refresh,
            onclick: Some(on_clear),
            inner: UIElement::Text {
                foreground: color::BLACK,
                text: "Clear".to_string(),
                scale: 60.0,
                border_px: 5,
            },
            ..Default::default()
        },
    );

    // Submit button (bottom right)
    app.add_element(
        "submit_button",
        UIElementWrapper {
            position: cgmath::Point2 { x: 1100, y: 1750 },
            refresh: UIConstraintRefresh::Refresh,
            onclick: Some(on_submit),
            inner: UIElement::Text {
                foreground: color::BLACK,
                text: "Submit".to_string(),
                scale: 60.0,
                border_px: 5,
            },
            ..Default::default()
        },
    );

    // Draw all elements
    app.draw_elements();

    // Draw baseline if enabled
    draw_baseline(&mut app);
    app.get_framebuffer_ref().refresh_balanced(&CANVAS_REGION);

    println!("Data Collector started");
    println!("Backend URL: {:?}", get_backend_url());
    println!("Device ID: {}", get_device_id());
    println!("Baseline enabled: {}", first_prompt.metadata.baseline_visible);

    // Start event loop
    app.start_event_loop(true, true, false, |ctx, evt| {
        match evt {
            InputEvent::WacomEvent { event } => on_wacom_input(ctx, event),
            InputEvent::MultitouchEvent { event } => {
                // Multitouch is handled automatically for button clicks via onclick handlers
                match event {
                    MultitouchEvent::Press { .. } => {}
                    _ => {}
                }
            }
            _ => {}
        }
    });
}
