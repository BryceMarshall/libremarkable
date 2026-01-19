# E-ink Sweep Animation Experiments

## Goal
Create a test application for reMarkable 2 that demonstrates different transition patterns when switching between images. This evaluates perceived quality vs. full-screen refresh for menu transitions and overlay effects.

## Technical Context

**Device**: reMarkable 2 (1404×1872, RGB565 e-ink)
**Framework**: libremarkable (examples available in repo)
**Display system**: Partial refresh capability via `mxcfb_rect` regions

### Refresh Characteristics
- Full refresh: ~1000ms, visible flash, clean result
- Partial refresh: ~200-300ms per region, no flash, can ghost
- Fast waveform (DU): ~100-200ms, optimized for binary content
- Quality waveform (GC16): ~300ms, better grayscale

## Implementation Structure

### Core Components

1. **Image Buffer Management**
   - Load 2-3 test images (simple geometric patterns work well)
   - Store in framebuffer-compatible format
   - Pre-calculate tile divisions

2. **Tile System**
   - Divide screen into configurable tiles (suggest 64×64px or 128×128px)
   - Each pattern generates ordering of tiles to refresh
   - Adjustable delay between tile refreshes (10-100ms)

3. **Pattern Implementations**

```rust
// Pattern trait for consistency
trait TransitionPattern {
    fn tile_order(&self, screen_width: u32, screen_height: u32, 
                   tile_size: u32) -> Vec<mxcfb_rect>;
}
```

### Transition Patterns to Implement

#### 1. Row-Based Sweep
- Top to bottom or bottom to top
- Refresh one horizontal strip at a time
- Natural reading direction

#### 2. Column-Based Sweep  
- Left to right or right to left
- Refresh one vertical strip at a time
- Good for side panel transitions

#### 3. Diagonal Sweep
- Top-left to bottom-right
- Progress along diagonal lines
- Calculate tiles by (x + y) sum

#### 4. Center Outward
- Start from screen center
- Expand in concentric rectangles
- Calculate by distance from center point

#### 5. Random Pattern
- Pseudo-random tile order (use seeded RNG for reproducibility)
- Creates organic, less mechanical feel
- Might reduce perception of "loading"

#### 6. Arbitrary Point Outward
- User specifies origin point (x, y)
- Tiles ordered by distance from origin
- Good for tap-to-expand effects

#### 7. Tap Halo Effect
- Concentric rings from tap point
- Outer rings refresh first (faster waveform)
- Inner rings refresh last (quality waveform)
- Creates "fading in" perception

### Refresh Strategy

```rust
// Pseudo-code structure
fn apply_transition(
    framebuffer: &mut Framebuffer,
    old_image: &Image,
    new_image: &Image,
    pattern: &impl TransitionPattern,
    tile_size: u32,
    delay_ms: u64,
    waveform: waveform_mode
) {
    // Write new image to framebuffer (no refresh yet)
    write_image_to_fb(framebuffer, new_image);
    
    // Get tile order from pattern
    let tiles = pattern.tile_order(WIDTH, HEIGHT, tile_size);
    
    // Refresh each tile with delay
    for (i, tile) in tiles.iter().enumerate() {
        framebuffer.partial_refresh(
            tile,
            PartialRefreshMode::Async,
            waveform,
            display_temp::TEMP_USE_AMBIENT,
            dither_mode::EPDC_FLAG_USE_DITHERING_PASSTHROUGH,
            0,
            false
        );
        
        std::thread::sleep(Duration::from_millis(delay_ms));
    }
}
```

### Test Sequence

Create a demonstration that cycles through patterns:

1. Start with simple geometric image (white background, black shapes)
2. Transition to next image using pattern #1 (row-based)
3. Wait 2 seconds
4. Transition back using pattern #2 (column-based)
5. Continue cycling through all 7 patterns
6. Repeat with different tile sizes (32px, 64px, 128px)
7. Repeat with different delays (10ms, 50ms, 100ms)

### Test Images

Simple test images to generate:
- **Image A**: White background with black circle in center
- **Image B**: White background with black square grid
- **Image C**: Gradient from white to black (horizontal bars)

Use simple drawing primitives from libremarkable's FramebufferDraw trait.

## Parameters to Experiment With

### Tile Size
- Small (32×32): Smoother animation, more overhead
- Medium (64×64): Good balance
- Large (128×128): Faster completion, chunkier effect

### Inter-tile Delay
- 0ms: Maximum speed, may feel jarring
- 10-20ms: Perceptible sweep, still fast
- 50-100ms: Deliberate animation effect

### Waveform Choice
- DU (1): Fastest, binary content only, ghosting likely
- GC16 (2): Quality grayscale, slower
- Mixed: Fast outer tiles, quality inner tiles (for halo effect)

## Expected Behavior

### Row/Column Sweeps
- Should feel like "wiping" across screen
- Users can track progress visually
- Good for showing "something is happening"

### Center Outward
- Draws attention to focal point
- Good for modal dialogs or alerts
- May feel slower due to smaller initial tiles

### Random
- Less predictable, might feel broken or buggy
- Could reduce "loading bar" psychology
- Test user perception carefully

### Tap Halo
- Most interactive feeling
- Reinforces tap location
- Fading effect from fast outer → quality inner waveforms

## Success Criteria

Application should:
1. Successfully compile for ARM using cross-compilation workflow
2. Run on reMarkable 2 without crashing
3. Demonstrate all 7 transition patterns
4. Allow easy parameter adjustment (tile size, delay, waveform)
5. Show clear visual difference between patterns

## Reference Code Locations

Look at libremarkable examples for:
- Framebuffer initialization: `examples/demo.rs` or similar
- Drawing primitives: `FramebufferDraw` trait
- Refresh methods: `FramebufferRefresh` trait
- Input handling (if adding tap-to-trigger): `input` module

## Build and Deploy

```bash
# Cross-compile using Docker
docker run --rm -v $(PWD):/work -w /work \
    dockcross/linux-armv7 \
    bash -c "cargo build --release --target armv7-unknown-linux-gnueabihf"

# Deploy to device
rsync -avz target/armv7-unknown-linux-gnueabihf/release/sweep-animations \
    root@10.11.99.1:/home/root/

# Run on device (stop xochitl first)
ssh root@10.11.99.1 "systemctl stop xochitl && \
    /home/root/sweep-animations && \
    systemctl start xochitl"
```

## Notes for Implementation

- Use `PartialRefreshMode::Async` to avoid blocking between tiles
- Consider tracking markers and waiting for completion if ghosting becomes problematic
- Start with larger tiles and slower delays to make patterns clearly visible
- Add keyboard/touch input to cycle through patterns manually
- Log timing information to measure actual refresh performance
- Consider implementing a "comparison mode" showing split-screen of two patterns side-by-side
