//! This example is a very basic drawing application. Marker can draw on the
//! screen (without tilt or pressure sensitivity) and Marker Plus can use the
//! eraser end to erase.
//!
//! Drawing is done in the framebuffer without any caching, so it's not possible
//! to save the results to file, zoom or pan, etc. There are also no GUI
//! elements or interactivity other than the pen.
//!
//! The new event loop design makes this type of application very easy to make.
//!
//! This example demonstrates the v1.0 simplified refresh API.

use libremarkable::appctx::ApplicationContext;
use libremarkable::framebuffer::common::color;
use libremarkable::framebuffer::{FramebufferDraw, FramebufferRefreshExt};
use libremarkable::input::{InputEvent, WacomEvent, WacomPen};

fn main() {
    let mut app = ApplicationContext::default();

    let mut tool_pen = false;
    let mut tool_rubber = false;

    app.clear(true);
    app.start_event_loop(true, false, false, |ctx, event| {
        if let InputEvent::WacomEvent { event } = event {
            match event {
                // The pen can have any number of attributes assigned to it at a
                // time. For example, when drawing with the tip of the Marker,
                // both the ToolPen and Touch attributes are applied. When
                // drawing with the eraser end of the Marker Plus, the
                // ToolRubber attribute is applied instead of ToolPen.
                //
                // The Tool attributes are mutually exclusive in practice, but
                // the protocol technically allows them to overlap. The Touch,
                // Stylus and Stylus2 events correspond to touching the display,
                // pressing the first button and pressing the second button
                // respectively. Markers don't have buttons but some Wacom pens
                // do.
                WacomEvent::InstrumentChange { pen, state } => {
                    eprintln!("pen {:?} state {}", pen, state);

                    let ptr = match pen {
                        WacomPen::ToolPen => Some(&mut tool_pen),
                        WacomPen::ToolRubber => Some(&mut tool_rubber),
                        _ => None,
                    };

                    if let Some(ptr) = ptr {
                        *ptr = state;
                    }
                }

                WacomEvent::Draw { position, .. } => {
                    eprintln!("drawing at {:?}", position);

                    let fb = ctx.get_framebuffer_ref();

                    let radcolor = if tool_rubber {
                        // 32 is about (>=) the physical radius of the eraser
                        (32, color::WHITE)
                    } else {
                        (4, color::BLACK)
                    };

                    let region = fb.fill_circle(
                        (position.x.floor() as i32, position.y.floor() as i32).into(),
                        radcolor.0,
                        radcolor.1,
                    );

                    // NEW v1.0 API: Simple refresh using quality presets
                    // This replaces the verbose 7-parameter partial_refresh call
                    // refresh_fast() uses DU waveform mode optimized for drawing
                    fb.refresh_fast(&region);
                }

                _ => {}
            }
        }
    });
}
