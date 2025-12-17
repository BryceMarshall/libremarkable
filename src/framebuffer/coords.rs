use cgmath::{Point2, Vector2};

/// Extension trait for converting coordinate types commonly used with drawing
///
/// This trait provides safe conversion from various coordinate types to `Point2<i32>`,
/// which is used by drawing operations. Returns `None` if the conversion would overflow.
///
/// # Examples
///
/// ```
/// use cgmath::Point2;
/// use libremarkable::framebuffer::coords::ToDrawCoords;
///
/// let point_u32 = Point2::new(100u32, 200u32);
/// let point_i32 = point_u32.to_draw_point().unwrap();
/// assert_eq!(point_i32, Point2::new(100i32, 200i32));
///
/// let point_f32 = Point2::new(100.7f32, 200.3f32);
/// let point_i32 = point_f32.to_draw_point().unwrap();
/// assert_eq!(point_i32, Point2::new(100i32, 200i32));
/// ```
pub trait ToDrawCoords {
    /// Convert to Point2<i32> for drawing operations
    /// Returns None if conversion would overflow/underflow
    fn to_draw_point(self) -> Option<Point2<i32>>;
}

/// Extension trait for converting to region coordinate types
///
/// This trait provides safe conversion from various coordinate types to `Point2<u32>`,
/// which is used by region operations (mxcfb_rect). Returns `None` if the conversion
/// would overflow or if values are negative.
///
/// # Examples
///
/// ```
/// use cgmath::Point2;
/// use libremarkable::framebuffer::coords::ToRegionCoords;
///
/// let point_i32 = Point2::new(100i32, 200i32);
/// let point_u32 = point_i32.to_region_point().unwrap();
/// assert_eq!(point_u32, Point2::new(100u32, 200u32));
///
/// // Negative coordinates return None
/// let point_neg = Point2::new(-10i32, 200i32);
/// assert!(point_neg.to_region_point().is_none());
/// ```
pub trait ToRegionCoords {
    /// Convert to Point2<u32> for region operations
    /// Returns None if conversion would overflow or if values are negative
    fn to_region_point(self) -> Option<Point2<u32>>;
}

/// Extension trait for vector conversions to drawing coordinates
pub trait ToDrawVector {
    /// Convert to Vector2<i32> for drawing operations
    fn to_draw_vec(self) -> Option<Vector2<i32>>;
}

/// Extension trait for vector conversions to region coordinates
pub trait ToRegionVector {
    /// Convert to Vector2<u32> for region operations
    fn to_region_vec(self) -> Option<Vector2<u32>>;
}

// Point2<u32> -> Point2<i32>
impl ToDrawCoords for Point2<u32> {
    #[inline]
    fn to_draw_point(self) -> Option<Point2<i32>> {
        Some(Point2 {
            x: i32::try_from(self.x).ok()?,
            y: i32::try_from(self.y).ok()?,
        })
    }
}

// Point2<i32> -> Point2<u32>
impl ToRegionCoords for Point2<i32> {
    #[inline]
    fn to_region_point(self) -> Option<Point2<u32>> {
        Some(Point2 {
            x: u32::try_from(self.x).ok()?,
            y: u32::try_from(self.y).ok()?,
        })
    }
}

// Point2<f32> -> Point2<i32>
impl ToDrawCoords for Point2<f32> {
    #[inline]
    fn to_draw_point(self) -> Option<Point2<i32>> {
        Some(Point2 {
            x: self.x as i32,
            y: self.y as i32,
        })
    }
}

// Point2<f32> -> Point2<u32>
impl ToRegionCoords for Point2<f32> {
    #[inline]
    fn to_region_point(self) -> Option<Point2<u32>> {
        if self.x < 0.0 || self.y < 0.0 {
            return None;
        }
        Some(Point2 {
            x: self.x as u32,
            y: self.y as u32,
        })
    }
}

// Point2<u16> -> Point2<i32> (for input device coordinates)
impl ToDrawCoords for Point2<u16> {
    #[inline]
    fn to_draw_point(self) -> Option<Point2<i32>> {
        Some(Point2 {
            x: i32::from(self.x),
            y: i32::from(self.y),
        })
    }
}

// Point2<u16> -> Point2<u32> (for input device coordinates)
impl ToRegionCoords for Point2<u16> {
    #[inline]
    fn to_region_point(self) -> Option<Point2<u32>> {
        Some(Point2 {
            x: u32::from(self.x),
            y: u32::from(self.y),
        })
    }
}

// Vector conversions
impl ToDrawVector for Vector2<u32> {
    #[inline]
    fn to_draw_vec(self) -> Option<Vector2<i32>> {
        Some(Vector2 {
            x: i32::try_from(self.x).ok()?,
            y: i32::try_from(self.y).ok()?,
        })
    }
}

impl ToRegionVector for Vector2<i32> {
    #[inline]
    fn to_region_vec(self) -> Option<Vector2<u32>> {
        Some(Vector2 {
            x: u32::try_from(self.x).ok()?,
            y: u32::try_from(self.y).ok()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u32_to_i32_conversion() {
        let p = Point2::new(100u32, 200u32);
        let converted = p.to_draw_point().unwrap();
        assert_eq!(converted, Point2::new(100i32, 200i32));
    }

    #[test]
    fn i32_to_u32_conversion() {
        let p = Point2::new(100i32, 200i32);
        let converted = p.to_region_point().unwrap();
        assert_eq!(converted, Point2::new(100u32, 200u32));
    }

    #[test]
    fn i32_negative_to_u32_fails() {
        let p = Point2::new(-10i32, 200i32);
        assert!(p.to_region_point().is_none());
    }

    #[test]
    fn f32_to_i32_truncates() {
        let p = Point2::new(100.7f32, 200.3f32);
        let converted = p.to_draw_point().unwrap();
        assert_eq!(converted, Point2::new(100i32, 200i32));
    }

    #[test]
    fn f32_to_u32_truncates() {
        let p = Point2::new(100.7f32, 200.3f32);
        let converted = p.to_region_point().unwrap();
        assert_eq!(converted, Point2::new(100u32, 200u32));
    }

    #[test]
    fn f32_negative_to_u32_fails() {
        let p = Point2::new(-10.5f32, 200.0f32);
        assert!(p.to_region_point().is_none());
    }

    #[test]
    fn u16_to_i32_conversion() {
        let p = Point2::new(100u16, 200u16);
        let converted = p.to_draw_point().unwrap();
        assert_eq!(converted, Point2::new(100i32, 200i32));
    }

    #[test]
    fn u16_to_u32_conversion() {
        let p = Point2::new(100u16, 200u16);
        let converted = p.to_region_point().unwrap();
        assert_eq!(converted, Point2::new(100u32, 200u32));
    }

    #[test]
    fn vector_u32_to_i32_conversion() {
        let v = Vector2::new(100u32, 200u32);
        let converted = v.to_draw_vec().unwrap();
        assert_eq!(converted, Vector2::new(100i32, 200i32));
    }

    #[test]
    fn vector_i32_to_u32_conversion() {
        let v = Vector2::new(100i32, 200i32);
        let converted = v.to_region_vec().unwrap();
        assert_eq!(converted, Vector2::new(100u32, 200u32));
    }

    #[test]
    fn vector_i32_negative_to_u32_fails() {
        let v = Vector2::new(-10i32, 200i32);
        assert!(v.to_region_vec().is_none());
    }
}
