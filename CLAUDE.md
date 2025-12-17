# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

libremarkable is a Rust framework for developing applications on the reMarkable Paper Tablet. It provides low-level access to the eInk display (with partial refresh support), multitouch input, Wacom digitizer, and physical buttons. The project is semi-abandoned and needs modernization for production readiness.

**Target Platform**: reMarkable 1 (Gen1) and reMarkable 2 (Gen2) tablets, cross-compiled for `armv7-unknown-linux-gnueabihf` or `armv7-unknown-linux-musleabihf`.

## Build Commands

### Prerequisites Setup

1. Install the reMarkable toolchain:
   - Gen1 (rM1): Download from [Codex rM10x toolchain](https://storage.googleapis.com/remarkable-codex-toolchain/codex-x86_64-cortexa9hf-neon-rm10x-toolchain-3.1.2.sh)
   - Gen2 (rM2): Download from [Codex rM11x toolchain](https://storage.googleapis.com/remarkable-codex-toolchain/codex-x86_64-cortexa7hf-neon-rm11x-toolchain-3.1.2.sh)

2. Add Rust target: `rustup target add armv7-unknown-linux-gnueabihf`

3. Generate `.cargo/config`:
   ```bash
   # Source the toolchain environment first (example path):
   source /opt/codex/rm11x/3.1.15/environment-setup-cortexa7hf-neon-remarkable-linux-gnueabi

   # Generate config
   python3 gen_cargo_config.py
   ```

### Building

```bash
# Build library only
make library
# or: cargo build --release --target=armv7-unknown-linux-gnueabihf

# Build examples
make examples
# or: cargo build --examples --release --target=armv7-unknown-linux-gnueabihf

# Build everything
make all

# Build with runtime benchmarking enabled
make bench
```

### Alternative: Using `cross` (easier, no toolchain required)

```bash
cargo install cross

# Build demo with cross
make x-demo
# or: cross build --example demo --release --target=armv7-unknown-linux-musleabihf

# Build and deploy
make deploy-x-demo
```

### Testing

```bash
# Run tests (on host, not cross-compiled)
make test
# or: cargo test
```

### Deployment to Device

Device is expected at `10.11.99.1` with SSH key-based auth configured.

```bash
# Build and run demo (stops xochitl)
make run

# Deploy and run the demo
make deploy-demo

# Run live server example
make live

# Stop demo and restart xochitl
make start-xochitl

# Spy on xochitl with LD_PRELOAD
make spy-xochitl
```

Override device IP: `make run DEVICE_IP=192.168.1.100`

## Architecture Overview

### Core Components

The codebase is organized into several feature-gated modules:

1. **`framebuffer`** (`src/framebuffer/`) - eInk display control
   - `core.rs`: Main `Framebuffer` struct, device detection (Gen1 vs Gen2)
   - `io.rs`: Low-level framebuffer I/O implementation
   - `draw.rs`: Drawing primitives (lines, circles, rectangles)
   - `graphics.rs`: Higher-level drawing operations (text, images)
   - `refresh.rs`: Partial and full refresh implementations
   - `storage.rs`: Framebuffer state save/restore with compression
   - `swtfb_client.rs`: rm2fb client for Gen2 devices

2. **`input`** (`src/input/`) - Input device handling
   - `wacom.rs`: Wacom digitizer (stylus) events
   - `multitouch.rs`: Multitouch events
   - `gpio.rs`: Physical button events
   - `ev.rs`: epoll-based event loop for async input
   - `scan.rs`: Auto-detection of input devices

3. **`device`** (`src/device/`) - Hardware abstraction
   - Auto-detects Gen1 vs Gen2 from `/sys/devices/soc0/machine`
   - Provides device-specific paths, rotations, and inversions

4. **`appctx`** (`src/appctx.rs`) - Application framework (optional)
   - High-level `ApplicationContext` with UI element management
   - Quadtree-based active region tracking for touch zones
   - Optional Lua scripting support via `hlua` feature

5. **`ui_extensions`** (`src/ui_extensions/`) - UI helpers
   - `element.rs`: UI element wrappers and handlers
   - `luaext.rs`: Lua bindings for drawing operations

### Key Traits

The framebuffer module uses a trait-based design:

- **`FramebufferIO`**: Low-level pixel/frame read/write operations
- **`FramebufferDraw`**: Drawing primitives (lines, circles, text, images)
- **`FramebufferBase`**: Base configuration and device management
- **`FramebufferRefresh`**: Display refresh control (partial/full updates)

All traits are implemented by `core::Framebuffer`.

### Display Refresh System

The eInk display requires explicit refresh calls. Two update methods exist:

1. **Full Refresh** (`full_refresh`): Updates entire screen, high quality, slower
2. **Partial Refresh** (`partial_refresh`): Updates specific region, faster, may have ghosting

Partial refresh modes:
- `PartialRefreshMode::Wait`: Blocks until refresh completes
- `PartialRefreshMode::Async`: Returns immediately, check marker later
- `PartialRefreshMode::DryRun`: Test for collision without updating

Waveform modes control visual quality vs speed (defined in `common.rs`).

### Device Differences (Gen1 vs Gen2)

**Gen1 (reMarkable 1)**:
- Uses `/dev/fb0` directly via ioctl
- Standard Linux framebuffer interface

**Gen2 (reMarkable 2)**:
- Requires [rm2fb](https://github.com/ddvk/remarkable2-framebuffer) server
- Uses shared memory (`/dev/shm/swtfb.01`) via `SwtfbClient`
- Set `LIBREMARKABLE_FB_DISFAVOR_INTERNAL_RM2FB=1` to force old ioctl method

Device detection is automatic via `device::CURRENT_DEVICE`.

### Feature Flags

The project uses extensive feature gating (see `Cargo.toml`):

- `framebuffer-types`, `framebuffer`, `framebuffer-drawing`, `framebuffer-text-drawing`: Display features
- `input-types`, `input`: Input device support
- `image`: Image rendering support
- `battery`: Battery status reading
- `appctx`: High-level application context framework
- `enable-runtime-benchmarking`: Performance profiling macros
- `scan`: Device auto-detection

Default features enable most functionality.

### Important Implementation Details

1. **Cross-compilation is mandatory**: The library targets ARM devices, use `--target=armv7-unknown-linux-gnueabihf`

2. **Release builds strongly recommended**: Debug builds have ~70% CPU utilization idle; release builds have 0-2% CPU usage

3. **Input device rotation/inversion**: Each input device (Wacom, multitouch) has device-specific rotation and inversion (see `device::Device::get_*_placement()`)

4. **Update markers**: Refresh operations return markers for tracking completion via `wait_refresh_complete()`

5. **Color format**: Framebuffer uses RGB565 little-endian format (see `color` enum in `common.rs`)

6. **Thread safety**: `Framebuffer` and `ApplicationContext` implement `Send + Sync` with manual unsafe impls

## Current API Analysis: Problems and Best Practices

### Current API Surface Area Issues

The current API requires users to have deep knowledge of eInk hardware internals. Example of a typical refresh call:

```rust
// Current verbose API (7 parameters!)
fb.partial_refresh(
    &region,
    PartialRefreshMode::Async,
    waveform_mode::WAVEFORM_MODE_DU,
    display_temp::TEMP_USE_REMARKABLE_DRAW,
    dither_mode::EPDC_FLAG_USE_DITHERING_PASSTHROUGH,
    DRAWING_QUANT_BIT,
    false,
);
```

**Problems:**

1. **Over-exposure of hardware details**: Users must understand:
   - Waveform modes (DU, GC16, GC16_FAST, REAGL, etc.) - 12+ options
   - Display temperature settings - hardware calibration details
   - Dithering modes - pixel processing internals
   - Quantization bits - mysterious magic numbers (`0x7614_3b24`)
   - Update modes vs force_full_refresh boolean overlap

2. **No sensible defaults**: Every call requires all 7 parameters, even though 95% of use cases need only 2-3 variations

3. **Poor discoverability**: New users cannot easily determine which parameters to use

4. **No semantic naming**: Parameters don't indicate **when** to use specific values

5. **Positional parameters**: Easy to mix up the order, no compile-time help

6. **Inconsistent return types**: Some methods return `u32` markers, others return `mxcfb_rect` regions

### Library API Design Best Practices

**Principle: Progressive Disclosure** - Make simple things simple, complex things possible.

#### 1. Provide High-Level Semantic APIs

```rust
// Proposed: Simple API for common cases
fb.refresh_region(&region);  // Smart defaults
fb.refresh_region_fast(&region);  // Speed over quality
fb.refresh_region_quality(&region);  // Quality over speed
fb.refresh_full();  // Whole screen

// Advanced users can still access low-level control
fb.refresh_region_with(RefreshConfig {
    region: &region,
    mode: RefreshMode::Async,
    quality: Quality::Fast,  // Maps to waveform internally
    ..Default::default()
})
```

#### 2. Builder Pattern for Complex Operations

```rust
// Proposed: Builder pattern
fb.refresh()
    .region(&region)
    .fast()  // or .quality() or .balanced()
    .async_mode()
    .send();

// Advanced customization when needed
fb.refresh()
    .region(&region)
    .waveform(WaveformMode::DU)
    .temperature(DisplayTemp::Ambient)
    .send();
```

#### 3. Configuration Structs with Defaults

```rust
// Proposed: Named configuration
#[derive(Default)]
pub struct RefreshConfig {
    pub quality: Quality,  // enum: Fast, Balanced, High
    pub mode: RefreshMode,  // enum: Async, Blocking
    pub region: Option<mxcfb_rect>,  // None = full screen
    // Advanced options hidden unless needed
    pub advanced: AdvancedRefreshOptions,
}

impl Default for RefreshConfig {
    fn default() -> Self {
        Self {
            quality: Quality::Balanced,  // GC16_FAST + standard settings
            mode: RefreshMode::Async,
            region: None,
            advanced: Default::default(),
        }
    }
}
```

#### 4. Documentation-Driven Design

Users shouldn't need to read datasheets. Document **when** to use each option:

```rust
pub enum Quality {
    /// Fastest refresh, only black/white, use for drawing/writing
    /// Hardware: DU waveform mode
    Fast,

    /// Balanced speed/quality, good for UI updates, supports grayscale
    /// Hardware: GC16_FAST waveform mode
    Balanced,

    /// Highest quality, use for images/photos, slower
    /// Hardware: GC16 waveform mode
    High,

    /// Reduce ghosting after many fast refreshes
    /// Hardware: REAGL waveform mode
    AntiGhost,
}
```

#### 5. Separate Concerns: Drawing vs Refreshing

Consider whether drawing should auto-refresh or be explicit:

```rust
// Option A: Explicit refresh (current approach, gives more control)
let region = fb.draw_circle(pos, radius, color);
fb.refresh_region(&region);

// Option B: Auto-refresh with opt-out (more convenient)
fb.draw_circle(pos, radius, color);  // Auto-refreshes with smart defaults
fb.no_auto_refresh().draw_circle(pos, radius, color);  // Manual control

// Option C: Separate drawing context
let mut batch = fb.begin_draw();
batch.circle(pos, radius, color);
batch.line(p1, p2, color);
batch.finish_and_refresh();  // Batched refresh of entire region
```

#### 6. Type Safety Over Magic Numbers

```rust
// Current: Magic numbers
const DRAWING_QUANT_BIT: i32 = 0x7614_3b24;

// Proposed: Named constants with documentation
pub struct QuantizationProfile;
impl QuantizationProfile {
    /// Standard quantization for general drawing
    pub const DRAWING: i32 = 0x7614_3b24;
    /// Alternative quantization profile
    pub const DRAWING_ALT: i32 = 0x75e7_bb24;
    /// Use device default
    pub const DEFAULT: i32 = 0;
}

// Or better: hide entirely unless advanced user needs it
```

#### 7. Chainable, Fluent Interfaces

```rust
// Proposed: Fluent API
app.framebuffer()
    .clear()
    .draw_text("Hello", pos, font)
    .draw_circle(center, radius, color)
    .refresh_all()
    .wait();
```

### Recommended API Modernization Strategy

When redesigning the API:

1. **Keep low-level access**: Advanced users may need direct hardware control
2. **Add high-level layer**: 80% of users should never see `waveform_mode` enum
3. **Use semantic names**: "fast", "quality", "anti_ghost" instead of "DU", "GC16", "REAGL"
4. **Provide presets**: `RefreshPreset::Drawing`, `RefreshPreset::UI`, `RefreshPreset::Image`
5. **Builder pattern**: For operations with >3 parameters
6. **Comprehensive examples**: Show common patterns, not just API reference
7. **Type-safe APIs**: Use enums/structs instead of raw integers where possible

### Comparison: Good Library API Examples

**Good examples to study:**
- `image` crate: Simple by default, powerful with configuration structs
- `reqwest`: Builder pattern for HTTP requests with sensible defaults
- `clap`: Progressive disclosure (simple derive → advanced builder)

**Anti-patterns to avoid:**
- Exposing hardware registers directly (this crate currently does)
- Requiring knowledge of implementation details for basic usage
- Too many positional parameters (>3 is a code smell)
- Magic numbers without semantic meaning

## Development Notes for Modernization

When modernizing this codebase:

1. **Update dependencies**: Many crates are outdated (e.g., `image 0.23`, `evdev 0.12`)

2. **API redesign (PRIORITY)**: See "Current API Analysis" section above - the current API is too low-level and verbose for general use

3. **Error handling**: Many functions use `unwrap()` or return simple `Result<T, &'static str>` - should use proper error types

4. **Documentation**: Add comprehensive rustdoc comments to public APIs with usage examples

5. **Testing**: Add integration tests and mock framebuffer/input for unit tests

6. **Breaking changes needed**:
   - Add high-level refresh API with sensible defaults (see API analysis section)
   - `FramebufferBase::from_path()` should accept proper path types
   - Remove `unsafe impl Send/Sync` and use proper synchronization
   - Standardize return types (markers vs regions vs void)
   - Create proper error types instead of `&'static str`

7. **Gen2 setup requirement**: Document that rm2fb must be installed on reMarkable 2 devices (from [Toltec](https://toltec-dev.org/))

8. **Backward compatibility**: Consider feature flags: `low-level-api` for current API, default to high-level API

## Examples

- `examples/demo.rs`: Full-featured demonstration of all capabilities
- `examples/basic_draw.rs`: Simple drawing example
- `examples/input.rs`: Input device handling
- `examples/live.rs`: HTTP server streaming framebuffer
- `examples/spy.rs`: LD_PRELOAD library for ioctl monitoring
- `examples/screenshot.rs`: Capture framebuffer to file
