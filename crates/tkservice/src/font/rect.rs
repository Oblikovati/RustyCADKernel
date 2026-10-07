//! `Font_Rect`: a glyph or text rectangle, Y up.
use crate::json_dump::{class_begin, field_real};
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::standard::dump::DumpStream;
use std::io;

/// A rectangle by its four sides; `FontRect::default().width() == 0.0`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FontRect {
    /// `Left`.
    pub left: f32,
    /// `Right`.
    pub right: f32,
    /// `Top`.
    pub top: f32,
    /// `Bottom`.
    pub bottom: f32,
}

impl FontRect {
    /// The top-left corner; `rect.top_left()`.
    pub fn top_left(&self) -> Vec2<f32> {
        Vec2::new(self.left, self.top)
    }
    /// The top-right corner; `rect.top_right()`.
    pub fn top_right(&self) -> Vec2<f32> {
        Vec2::new(self.right, self.top)
    }
    /// The bottom-left corner; `rect.bottom_left()`.
    pub fn bottom_left(&self) -> Vec2<f32> {
        Vec2::new(self.left, self.bottom)
    }
    /// The bottom-right corner; `rect.bottom_right()`.
    pub fn bottom_right(&self) -> Vec2<f32> {
        Vec2::new(self.right, self.bottom)
    }
    /// `Right - Left`; `rect.width()`.
    pub fn width(&self) -> f32 {
        self.right - self.left
    }
    /// `Top - Bottom`; `rect.height()`.
    pub fn height(&self) -> f32 {
        self.top - self.bottom
    }
    /// `DumpJson`: the class name and the four sides; `rect.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, _depth: i32) -> io::Result<()> {
        class_begin(stream, "Font_Rect")?;
        field_real(stream, "Left", f64::from(self.left))?;
        field_real(stream, "Right", f64::from(self.right))?;
        field_real(stream, "Top", f64::from(self.top))?;
        field_real(stream, "Bottom", f64::from(self.bottom))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corners_and_sizes() {
        let rect = FontRect {
            left: 1.0,
            right: 4.0,
            top: 6.0,
            bottom: 2.0,
        };
        assert_eq!((rect.width(), rect.height()), (3.0, 4.0));
        assert_eq!(rect.bottom_right(), Vec2::new(4.0, 2.0));
        assert_eq!(rect.top_left(), Vec2::new(1.0, 6.0));
        let mut stream = DumpStream::default();
        rect.dump_json(&mut stream, -1).unwrap();
        assert_eq!(
            stream.text(),
            "\"className\": \"Font_Rect\", \"Left\": 1, \"Right\": 4, \"Top\": 6, \"Bottom\": 2"
        );
    }
}
