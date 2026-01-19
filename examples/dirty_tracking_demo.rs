//! Dirty Region Tracking Demo
//!
//! Demonstrates the v1.1 dirty region tracking feature by:
//! 1. Drawing several shapes
//! 2. Showing which regions were automatically tracked
//! 3. Refreshing only the dirty regions

use libremarkable::appctx::ApplicationContext;
use libremarkable::framebuffer::common::color;
use libremarkable::framebuffer::{FramebufferDraw, FramebufferRefreshExt};

fn main() {
    let mut app = ApplicationContext::default();

    println!("=== Dirty Region Tracking Demo ===");
    println!("This demo shows automatic dirty region tracking (v1.1)\n");

    // Clear screen
    app.clear(true);

    let fb = app.get_framebuffer_ref();

    // Draw several shapes without manual refresh
    println!("Drawing shapes...");

    fb.draw_circle((200, 200).into(), 50, color::BLACK);
    fb.draw_circle((400, 400).into(), 75, color::BLACK);
    fb.fill_circle((600, 200).into(), 40, color::GRAY(128));
    fb.draw_line((100, 500).into(), (700, 500).into(), 3, color::BLACK);

    // Check dirty tracker
    #[cfg(feature = "framebuffer-dirty-tracking")]
    {
        let tracker = fb.dirty_tracker();

        if tracker.is_dirty() {
            println!("\n✓ Dirty regions detected!");
            println!("  Individual regions tracked: {}", tracker.get_regions().len());

            if let Some(merged) = tracker.get_merged() {
                println!("  Merged bounding box:");
                println!("    Position: ({}, {})", merged.left, merged.top);
                println!("    Size: {}x{}", merged.width, merged.height);
                println!("    Area: {} pixels", merged.width * merged.height);

                // Refresh only the dirty region (instead of full screen)
                println!("\nRefreshing dirty region...");
                fb.refresh_balanced(&merged);
            }

            // Show individual regions
            println!("\nIndividual dirty regions:");
            for (i, rect) in tracker.get_regions().iter().enumerate() {
                println!("  Region {}: pos=({},{}) size={}x{} area={}",
                    i + 1,
                    rect.left, rect.top,
                    rect.width, rect.height,
                    rect.width * rect.height
                );
            }
        } else {
            println!("\n✗ No dirty regions detected (feature may be disabled)");
        }
    }

    #[cfg(not(feature = "framebuffer-dirty-tracking"))]
    {
        println!("\n⚠ Dirty tracking feature is DISABLED");
        println!("  To enable: build with --features framebuffer-dirty-tracking");
        println!("  Falling back to full screen refresh...");
        fb.refresh_balanced(&libremarkable::framebuffer::common::mxcfb_rect {
            top: 0,
            left: 0,
            width: 1404,
            height: 1872,
        });
    }

    println!("\n=== Demo Complete ===");
    println!("The screen should now show 3 circles and 1 line.");
    println!("Press Ctrl+C to exit.");

    // Keep running so user can see the result
    std::thread::sleep(std::time::Duration::from_secs(10));
}
