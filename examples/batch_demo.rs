//! Batch Operations Demo
//!
//! Demonstrates the v1.1 batch operations API by showing three different patterns:
//! 1. Auto-refresh batch (RAII pattern)
//! 2. Manual batch with deferred refresh
//! 3. Multi-region refresh for non-rectangular shapes

use libremarkable::appctx::ApplicationContext;
use libremarkable::framebuffer::common::{color, RefreshQuality};
use libremarkable::framebuffer::{FramebufferDraw, FramebufferBatchExt};

fn main() {
    let mut app = ApplicationContext::default();

    println!("=== Batch Operations Demo ===");
    println!("This demo shows the v1.1 batch operations API\\n");

    // Clear screen
    app.clear(true);

    let fb = app.get_framebuffer_ref();

    // Demo 1: Auto-refresh batch (simplest usage)
    println!("Demo 1: Auto-refresh batch");
    println!("  Drawing 3 circles with automatic refresh on scope exit...");

    {
        let mut batch = fb.batch();
        batch.draw_circle((200, 200).into(), 50, color::BLACK);
        batch.draw_circle((400, 200).into(), 50, color::BLACK);
        batch.draw_circle((600, 200).into(), 50, color::BLACK);

        println!("  Dirty regions tracked: {}", batch.dirty_regions().len());
        if let Some(merged) = batch.merged_region() {
            println!("  Merged region: pos=({},{}) size={}x{}",
                merged.left, merged.top, merged.width, merged.height);
        }
    } // Auto-refresh happens here

    println!("  ✓ Batch auto-refreshed\\n");

    std::thread::sleep(std::time::Duration::from_secs(2));

    // Demo 2: Manual batch with quality control
    println!("Demo 2: Manual batch with Fast quality");
    println!("  Drawing 2 lines with deferred refresh...");

    {
        let mut batch = fb.batch_quality(RefreshQuality::Fast).defer_refresh();

        batch.draw_line((100, 400).into(), (700, 400).into(), 3, color::BLACK);
        batch.draw_line((100, 500).into(), (700, 500).into(), 3, color::BLACK);

        println!("  Dirty regions tracked: {}", batch.dirty_regions().len());

        // Manual flush
        if let Some(marker) = batch.flush() {
            println!("  ✓ Manually flushed (marker: {})\\n", marker);
        }
    }

    std::thread::sleep(std::time::Duration::from_secs(2));

    // Demo 3: Multi-region refresh (for widely separated shapes)
    println!("Demo 3: Multi-region refresh");
    println!("  Drawing 2 widely separated circles...");

    {
        let mut batch = fb.batch().defer_refresh();

        batch.draw_circle((150, 700).into(), 40, color::GRAY(128));
        batch.draw_circle((650, 700).into(), 40, color::GRAY(128));

        let regions = batch.dirty_regions().to_vec();
        println!("  Individual regions: {}", regions.len());

        // Calculate area saved by using multi-region instead of merged
        if let Some(merged) = batch.merged_region() {
            let merged_area = merged.width * merged.height;
            let individual_area: u32 = regions.iter()
                .map(|r| r.width * r.height)
                .sum();

            println!("  Merged area: {} pixels", merged_area);
            println!("  Individual area: {} pixels", individual_area);
            println!("  Area saved: {} pixels ({:.1}%)",
                merged_area - individual_area,
                100.0 * (merged_area - individual_area) as f32 / merged_area as f32);

            // Use multi-region refresh
            let markers = batch.flush_multi(&regions);
            println!("  ✓ Refreshed {} regions (markers: {:?})\\n", markers.len(), markers);
        }
    }

    println!("=== Demo Complete ===");
    println!("The screen should show:");
    println!("  - Top: 3 circles in a row (auto-refresh)");
    println!("  - Middle: 2 horizontal lines (manual refresh)");
    println!("  - Bottom: 2 gray circles (multi-region refresh)");
    println!("\\nPress Ctrl+C to exit.");

    // Keep running so user can see the result
    std::thread::sleep(std::time::Duration::from_secs(10));
}
