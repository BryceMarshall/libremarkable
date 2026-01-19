use crate::framebuffer::common::*;
use crate::framebuffer::{FramebufferRefresh, PartialRefreshMode};

/// Builder for configuring framebuffer refresh operations
///
/// This builder provides a fluent interface for configuring display refreshes,
/// allowing you to use high-level quality presets or fine-tune individual
/// hardware parameters as needed.
///
/// # Examples
///
/// Simple quality-based refresh:
/// ```no_run
/// use libremarkable::framebuffer::core::Framebuffer;
/// use libremarkable::framebuffer::refresh_builder::FramebufferRefreshExt;
/// use libremarkable::framebuffer::common::{RefreshQuality, mxcfb_rect};
///
/// let framebuffer = Framebuffer::new();
/// let rect = mxcfb_rect { top: 0, left: 0, width: 100, height: 100 };
///
/// // Use quality preset
/// framebuffer.refresh()
///     .region(&rect)
///     .quality(RefreshQuality::Fast)
///     .send();
/// ```
///
/// Advanced customization with explicit hardware parameters:
/// ```no_run
/// use libremarkable::framebuffer::core::Framebuffer;
/// use libremarkable::framebuffer::refresh_builder::FramebufferRefreshExt;
/// use libremarkable::framebuffer::common::*;
///
/// let framebuffer = Framebuffer::new();
/// let rect = mxcfb_rect { top: 0, left: 0, width: 100, height: 100 };
///
/// framebuffer.refresh()
///     .region(&rect)
///     .waveform(waveform_mode::WAVEFORM_MODE_DU)
///     .temperature(display_temp::TEMP_USE_REMARKABLE_DRAW)
///     .dithering(dither_mode::EPDC_FLAG_EXP1)
///     .quantization(DRAWING_QUANT_BIT)
///     .wait()
///     .send();
/// ```
pub struct RefreshBuilder<'fb, FB: FramebufferRefresh> {
    framebuffer: &'fb FB,
    region: Option<&'fb mxcfb_rect>,
    waveform_mode: waveform_mode,
    temperature: display_temp,
    dither_mode: dither_mode,
    quant_bit: i32,
    wait_completion: bool,
    force_full: bool,
}

impl<'fb, FB: FramebufferRefresh> RefreshBuilder<'fb, FB> {
    /// Create a new refresh builder with default parameters
    ///
    /// Defaults to Balanced quality (GC16_FAST waveform) with async execution
    pub(crate) fn new(framebuffer: &'fb FB) -> Self {
        // Default to Balanced quality
        let (waveform, temp, dither, quant) = RefreshQuality::Balanced.to_hardware_params();

        RefreshBuilder {
            framebuffer,
            region: None,
            waveform_mode: waveform,
            temperature: temp,
            dither_mode: dither,
            quant_bit: quant,
            wait_completion: false,
            force_full: false,
        }
    }

    /// Set the region to refresh (required for partial refresh)
    ///
    /// If no region is set, a full screen refresh will be performed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use libremarkable::framebuffer::core::Framebuffer;
    /// # use libremarkable::framebuffer::refresh_builder::FramebufferRefreshExt;
    /// # use libremarkable::framebuffer::common::mxcfb_rect;
    /// # let framebuffer = Framebuffer::new();
    /// let rect = mxcfb_rect { top: 0, left: 0, width: 100, height: 100 };
    /// framebuffer.refresh().region(&rect).send();
    /// ```
    #[inline]
    pub fn region(mut self, region: &'fb mxcfb_rect) -> Self {
        self.region = Some(region);
        self
    }

    /// Set quality preset (replaces waveform, temperature, dither, quant)
    ///
    /// Using a quality preset is the recommended way to configure refreshes
    /// for most use cases. This sets all hardware parameters to optimal values
    /// for the specified quality level.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use libremarkable::framebuffer::core::Framebuffer;
    /// # use libremarkable::framebuffer::refresh_builder::FramebufferRefreshExt;
    /// # use libremarkable::framebuffer::common::{RefreshQuality, mxcfb_rect};
    /// # let framebuffer = Framebuffer::new();
    /// # let rect = mxcfb_rect::default();
    /// // Fast refresh for drawing
    /// framebuffer.refresh().region(&rect).quality(RefreshQuality::Fast).send();
    ///
    /// // High quality for final rendering
    /// framebuffer.refresh().region(&rect).quality(RefreshQuality::High).send();
    /// ```
    #[inline]
    pub fn quality(mut self, quality: RefreshQuality) -> Self {
        let (waveform, temp, dither, quant) = quality.to_hardware_params();
        self.waveform_mode = waveform;
        self.temperature = temp;
        self.dither_mode = dither;
        self.quant_bit = quant;
        self
    }

    /// Set waveform mode (overrides quality preset)
    ///
    /// Advanced users can set the waveform mode directly for fine-grained control.
    /// This overrides any previous quality preset or waveform setting.
    #[inline]
    pub fn waveform(mut self, mode: waveform_mode) -> Self {
        self.waveform_mode = mode;
        self
    }

    /// Set temperature mode (overrides quality preset)
    ///
    /// Advanced users can set the temperature mode directly.
    /// This overrides any previous quality preset or temperature setting.
    #[inline]
    pub fn temperature(mut self, temp: display_temp) -> Self {
        self.temperature = temp;
        self
    }

    /// Set dithering mode (overrides quality preset)
    ///
    /// Advanced users can set the dithering mode directly.
    /// This overrides any previous quality preset or dithering setting.
    #[inline]
    pub fn dithering(mut self, mode: dither_mode) -> Self {
        self.dither_mode = mode;
        self
    }

    /// Set quantization bits (overrides quality preset)
    ///
    /// Advanced users can set the quantization bits directly.
    /// This overrides any previous quality preset or quantization setting.
    #[inline]
    pub fn quantization(mut self, quant: i32) -> Self {
        self.quant_bit = quant;
        self
    }

    /// Wait for refresh to complete (blocking)
    ///
    /// By default, refreshes are asynchronous. Call this method to block
    /// until the refresh operation completes.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use libremarkable::framebuffer::core::Framebuffer;
    /// # use libremarkable::framebuffer::refresh_builder::FramebufferRefreshExt;
    /// # use libremarkable::framebuffer::common::mxcfb_rect;
    /// # let framebuffer = Framebuffer::new();
    /// # let rect = mxcfb_rect::default();
    /// // Refresh and wait for completion
    /// framebuffer.refresh().region(&rect).wait().send();
    /// ```
    #[inline]
    pub fn wait(mut self) -> Self {
        self.wait_completion = true;
        self
    }

    /// Force full refresh mode (rare use case)
    ///
    /// This forces a full refresh even when a partial refresh region is specified.
    /// Rarely needed in practice.
    #[inline]
    pub fn force_full(mut self) -> Self {
        self.force_full = true;
        self
    }

    /// Execute the refresh operation
    ///
    /// Returns the update marker which can be used to track refresh completion
    /// with `wait_refresh_complete()`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use libremarkable::framebuffer::core::Framebuffer;
    /// # use libremarkable::framebuffer::refresh_builder::FramebufferRefreshExt;
    /// # use libremarkable::framebuffer::common::mxcfb_rect;
    /// # let framebuffer = Framebuffer::new();
    /// # let rect = mxcfb_rect::default();
    /// let marker = framebuffer.refresh().region(&rect).send();
    /// // Later: framebuffer.wait_refresh_complete(marker);
    /// ```
    #[inline]
    pub fn send(self) -> u32 {
        match self.region {
            Some(region) => {
                let mode = if self.wait_completion {
                    PartialRefreshMode::Wait
                } else {
                    PartialRefreshMode::Async
                };

                self.framebuffer.partial_refresh(
                    region,
                    mode,
                    self.waveform_mode,
                    self.temperature,
                    self.dither_mode,
                    self.quant_bit,
                    self.force_full,
                )
            }
            None => {
                // Full screen refresh
                self.framebuffer.full_refresh(
                    self.waveform_mode,
                    self.temperature,
                    self.dither_mode,
                    self.quant_bit,
                    self.wait_completion,
                )
            }
        }
    }
}

/// Extension trait adding ergonomic refresh methods to FramebufferRefresh
///
/// This trait is automatically implemented for all types that implement
/// `FramebufferRefresh`, providing convenient shorthand methods for common
/// refresh operations.
///
/// # Examples
///
/// ```no_run
/// use libremarkable::framebuffer::core::Framebuffer;
/// use libremarkable::framebuffer::refresh_builder::FramebufferRefreshExt;
/// use libremarkable::framebuffer::common::mxcfb_rect;
///
/// let framebuffer = Framebuffer::new();
/// let rect = mxcfb_rect { top: 0, left: 0, width: 100, height: 100 };
///
/// // Simple convenience methods
/// framebuffer.refresh_fast(&rect);      // Fast refresh
/// framebuffer.refresh_balanced(&rect);  // Balanced refresh
/// framebuffer.refresh_quality(&rect);   // High quality refresh
/// framebuffer.clear_screen();           // Full screen clear
///
/// // Or use the builder for customization
/// framebuffer.refresh()
///     .region(&rect)
///     .quality(RefreshQuality::Fast)
///     .wait()
///     .send();
/// ```
pub trait FramebufferRefreshExt: FramebufferRefresh {
    /// Create a refresh builder with sensible defaults
    ///
    /// Returns a `RefreshBuilder` that defaults to Balanced quality and
    /// async execution. Use the builder methods to customize as needed.
    #[inline]
    fn refresh(&self) -> RefreshBuilder<'_, Self>
    where
        Self: Sized,
    {
        RefreshBuilder::new(self)
    }

    /// Convenience: Fast refresh (async)
    ///
    /// Performs a fast refresh using DU waveform mode, suitable for drawing
    /// and high-frequency updates. Returns immediately without waiting.
    ///
    /// Equivalent to:
    /// ```no_run
    /// # use libremarkable::framebuffer::core::Framebuffer;
    /// # use libremarkable::framebuffer::refresh_builder::FramebufferRefreshExt;
    /// # use libremarkable::framebuffer::common::{RefreshQuality, mxcfb_rect};
    /// # let framebuffer = Framebuffer::new();
    /// # let region = &mxcfb_rect::default();
    /// framebuffer.refresh().region(region).quality(RefreshQuality::Fast).send();
    /// ```
    #[inline]
    fn refresh_fast(&self, region: &mxcfb_rect) -> u32
    where
        Self: Sized,
    {
        self.refresh()
            .region(region)
            .quality(RefreshQuality::Fast)
            .send()
    }

    /// Convenience: Balanced refresh (async)
    ///
    /// Performs a balanced-quality refresh using GC16_FAST waveform, suitable
    /// for UI updates and text. Returns immediately without waiting.
    ///
    /// Equivalent to:
    /// ```no_run
    /// # use libremarkable::framebuffer::core::Framebuffer;
    /// # use libremarkable::framebuffer::refresh_builder::FramebufferRefreshExt;
    /// # use libremarkable::framebuffer::common::{RefreshQuality, mxcfb_rect};
    /// # let framebuffer = Framebuffer::new();
    /// # let region = &mxcfb_rect::default();
    /// framebuffer.refresh().region(region).quality(RefreshQuality::Balanced).send();
    /// ```
    #[inline]
    fn refresh_balanced(&self, region: &mxcfb_rect) -> u32
    where
        Self: Sized,
    {
        self.refresh()
            .region(region)
            .quality(RefreshQuality::Balanced)
            .send()
    }

    /// Convenience: High quality refresh (async)
    ///
    /// Performs a high-quality refresh using GC16 waveform, suitable for
    /// images and final rendering. Returns immediately without waiting.
    ///
    /// Equivalent to:
    /// ```no_run
    /// # use libremarkable::framebuffer::core::Framebuffer;
    /// # use libremarkable::framebuffer::refresh_builder::FramebufferRefreshExt;
    /// # use libremarkable::framebuffer::common::{RefreshQuality, mxcfb_rect};
    /// # let framebuffer = Framebuffer::new();
    /// # let region = &mxcfb_rect::default();
    /// framebuffer.refresh().region(region).quality(RefreshQuality::High).send();
    /// ```
    #[inline]
    fn refresh_quality(&self, region: &mxcfb_rect) -> u32
    where
        Self: Sized,
    {
        self.refresh()
            .region(region)
            .quality(RefreshQuality::High)
            .send()
    }

    /// Convenience: Full screen clear
    ///
    /// Performs a full screen refresh using INIT waveform to clear ghosting.
    /// This will cause a visible flash but completely resets the display.
    ///
    /// Equivalent to:
    /// ```no_run
    /// # use libremarkable::framebuffer::core::Framebuffer;
    /// # use libremarkable::framebuffer::refresh_builder::FramebufferRefreshExt;
    /// # use libremarkable::framebuffer::common::RefreshQuality;
    /// # let framebuffer = Framebuffer::new();
    /// framebuffer.refresh().quality(RefreshQuality::Clear).send();
    /// ```
    #[inline]
    fn clear_screen(&self) -> u32
    where
        Self: Sized,
    {
        self.refresh().quality(RefreshQuality::Clear).send()
    }
}

/// Blanket implementation of FramebufferRefreshExt for all FramebufferRefresh types
impl<T: FramebufferRefresh> FramebufferRefreshExt for T {}
