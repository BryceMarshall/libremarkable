//! Sweep Animation Demo
//!
//! Demonstrates sweep transitions between two images using touch controls.
//! Tap anywhere on the screen to trigger a sweep transition to the next image.
//!
//! Uses a "strip sweep" pattern where thin horizontal bands span the full width,
//! revealing all columns simultaneously as the sweep progresses top-to-bottom.

use libremarkable::cgmath;
use libremarkable::framebuffer::common::{color, mxcfb_rect};
use libremarkable::framebuffer::core::Framebuffer;
use libremarkable::framebuffer::{FramebufferDraw, FramebufferRefresh, FramebufferRefreshExt};
use libremarkable::image;
use libremarkable::input::{ev::EvDevContext, InputDevice, InputEvent};

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

/// Tile size in pixels (used for tile-based sweep - alternative pattern)
#[allow(dead_code)]
const TILE_SIZE: u32 = 64;

/// Strip height for the strip sweep effect - thin horizontal bands
const STRIP_HEIGHT: u32 = 8;

/// Delay between strip refreshes in milliseconds
const DELAY_MS: u64 = 2;

/// Demo region where images are displayed
const DEMO_REGION: mxcfb_rect = mxcfb_rect {
    top: 200,
    left: 100,
    width: 800,
    height: 600,
};

/// Track which image is currently displayed (false = image A, true = image B)
static SHOWING_IMAGE_B: AtomicBool = AtomicBool::new(false);

/// Generates tiles for a row-based sweep (top to bottom, left to right within each row)
#[allow(dead_code)]
fn generate_row_tiles(region: &mxcfb_rect, tile_size: u32) -> Vec<mxcfb_rect> {
    let mut tiles = Vec::new();

    let mut y = region.top;
    while y < region.top + region.height {
        let tile_height = (region.top + region.height - y).min(tile_size);

        let mut x = region.left;
        while x < region.left + region.width {
            let tile_width = (region.left + region.width - x).min(tile_size);

            tiles.push(mxcfb_rect {
                top: y,
                left: x,
                width: tile_width,
                height: tile_height,
            });

            x += tile_size;
        }
        y += tile_size;
    }

    tiles
}

/// Applies a row-based sweep transition by refreshing tiles sequentially with delays
#[allow(dead_code)]
fn apply_row_sweep<FB: FramebufferRefresh>(
    fb: &FB,
    region: &mxcfb_rect,
    tile_size: u32,
    delay_ms: u64,
) {
    let tiles = generate_row_tiles(region, tile_size);

    for tile in tiles {
        fb.refresh_fast(&tile);
        if delay_ms > 0 {
            thread::sleep(Duration::from_millis(delay_ms));
        }
    }
}

/// Generates horizontal strips that span the full width of the region.
/// Each strip reveals a portion of ALL tiles in that row simultaneously.
fn generate_horizontal_strips(region: &mxcfb_rect, strip_height: u32) -> Vec<mxcfb_rect> {
    let mut strips = Vec::new();

    let mut y = region.top;
    while y < region.top + region.height {
        let height = (region.top + region.height - y).min(strip_height);

        strips.push(mxcfb_rect {
            top: y,
            left: region.left,
            width: region.width,
            height,
        });

        y += strip_height;
    }

    strips
}

/// Applies a strip sweep transition - reveals thin horizontal bands across the full width.
/// This creates a smooth top-to-bottom wipe effect where all columns update simultaneously.
fn apply_strip_sweep<FB: FramebufferRefresh>(
    fb: &FB,
    region: &mxcfb_rect,
    strip_height: u32,
    delay_ms: u64,
) {
    let strips = generate_horizontal_strips(region, strip_height);

    for strip in strips {
        fb.refresh_fast(&strip);
        if delay_ms > 0 {
            thread::sleep(Duration::from_millis(delay_ms));
        }
    }
}

/// Draws image A (Rust logo) scaled to fit the demo region
fn draw_image_a(fb: &mut Framebuffer, region: &mxcfb_rect) {
    // Clear the region first
    fb.fill_rect(
        cgmath::Point2 {
            x: region.left as i32,
            y: region.top as i32,
        },
        cgmath::Vector2 {
            x: region.width,
            y: region.height,
        },
        color::WHITE,
    );

    // Load and draw the Rust logo
    let img = image::load_from_memory(include_bytes!("../assets/rustlang.png")).unwrap();
    let rgb_img = img.to_rgb8();

    // Center the image in the region
    let img_width = rgb_img.width();
    let img_height = rgb_img.height();
    let x_offset = region.left as i32 + (region.width as i32 - img_width as i32) / 2;
    let y_offset = region.top as i32 + (region.height as i32 - img_height as i32) / 2;

    fb.draw_image(&rgb_img, cgmath::Point2 { x: x_offset, y: y_offset });
}

/// Draws image B (colorspace test image) scaled to fit the demo region
fn draw_image_b(fb: &mut Framebuffer, region: &mxcfb_rect) {
    // Clear the region first
    fb.fill_rect(
        cgmath::Point2 {
            x: region.left as i32,
            y: region.top as i32,
        },
        cgmath::Vector2 {
            x: region.width,
            y: region.height,
        },
        color::WHITE,
    );

    // Load and draw the colorspace image
    let img = image::load_from_memory(include_bytes!("../assets/colorspace.png")).unwrap();
    let rgb_img = img.to_rgb8();

    // Center the image in the region
    let img_width = rgb_img.width();
    let img_height = rgb_img.height();
    let x_offset = region.left as i32 + (region.width as i32 - img_width as i32) / 2;
    let y_offset = region.top as i32 + (region.height as i32 - img_height as i32) / 2;

    fb.draw_image(&rgb_img, cgmath::Point2 { x: x_offset, y: y_offset });
}

/// Draw instructions text
fn draw_instructions(fb: &mut Framebuffer) {
    let instructions = "Tap anywhere to trigger sweep transition";
    fb.draw_text(
        cgmath::Point2 { x: 100.0, y: 100.0 },
        instructions,
        40.0,
        color::BLACK,
        false,
    );

    let exit_msg = "Press Ctrl+C to exit";
    fb.draw_text(
        cgmath::Point2 { x: 100.0, y: 150.0 },
        exit_msg,
        30.0,
        color::GRAY(128),
        false,
    );
}

fn main() {
    env_logger::init();

    // Create framebuffer
    let mut fb = Framebuffer::new();

    // Clear screen
    fb.clear();

    // Draw instructions
    draw_instructions(&mut fb);

    // Draw initial image (Image A - Rust logo)
    draw_image_a(&mut fb, &DEMO_REGION);

    // Draw border around demo region
    fb.draw_rect(
        cgmath::Point2 {
            x: DEMO_REGION.left as i32 - 2,
            y: DEMO_REGION.top as i32 - 2,
        },
        cgmath::Vector2 {
            x: DEMO_REGION.width + 4,
            y: DEMO_REGION.height + 4,
        },
        2,
        color::BLACK,
    );

    // Full refresh to show initial state
    fb.full_refresh(
        libremarkable::framebuffer::common::waveform_mode::WAVEFORM_MODE_GC16,
        libremarkable::framebuffer::common::display_temp::TEMP_USE_AMBIENT,
        libremarkable::framebuffer::common::dither_mode::EPDC_FLAG_USE_DITHERING_PASSTHROUGH,
        0,
        true,
    );

    println!("Sweep Demo started. Tap screen to trigger transitions.");

    // Set up input handling
    let (input_tx, input_rx) = std::sync::mpsc::channel::<InputEvent>();

    // Spawn input handling thread
    EvDevContext::new(InputDevice::Multitouch, input_tx).start();

    // Main event loop
    for event in input_rx.iter() {
        match event {
            InputEvent::MultitouchEvent { event } => {
                if let libremarkable::input::MultitouchEvent::Press { finger: _ } = event {
                    // Toggle image on touch
                    let showing_b = SHOWING_IMAGE_B.load(Ordering::Relaxed);

                    if showing_b {
                        // Switch to Image A
                        draw_image_a(&mut fb, &DEMO_REGION);
                    } else {
                        // Switch to Image B
                        draw_image_b(&mut fb, &DEMO_REGION);
                    }

                    // Apply strip sweep transition - reveals horizontal bands across full width
                    apply_strip_sweep(&fb, &DEMO_REGION, STRIP_HEIGHT, DELAY_MS);

                    // Toggle state
                    SHOWING_IMAGE_B.store(!showing_b, Ordering::Relaxed);

                    println!(
                        "Transition complete. Now showing: {}",
                        if !showing_b { "Image B (colorspace)" } else { "Image A (Rust logo)" }
                    );
                }
            }
            _ => {}
        }
    }
}
