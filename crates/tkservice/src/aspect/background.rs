//! Window backgrounds: a plain color (`Aspect_Background`) and a two-color gradient
//! (`Aspect_GradientBackground`).
use super::AspectGradientFillMethod;
use crate::json_dump::{class_begin, field_color, field_integer};
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::dump::DumpStream;
use std::io;
use std::ops::{Deref, DerefMut};

/// A window background color; `AspectBackground::default().color()` is Matra gray.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AspectBackground {
    /// `MyColor`: the background color.
    pub(crate) color: Color,
}
impl Default for AspectBackground {
    /// The Matra gray background of `Aspect_Background()`.
    fn default() -> Self {
        Self::new(Color::named(NamedColor::MATRAGRAY))
    }
}
impl AspectBackground {
    /// A background of `color`; `AspectBackground::new(Color::named(NamedColor::BLACK))`.
    pub fn new(color: Color) -> Self {
        Self { color }
    }
    /// Changes the color; `background.set_color(Color::named(NamedColor::WHITE))`.
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    /// The color; `AspectBackground::default().color()`.
    pub fn color(&self) -> Color {
        self.color
    }
    /// `DumpJson`: the class name, then the color; `background.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Aspect_Background")?;
        field_color(stream, depth, "&MyColor", self.color)
    }
}

/// A two-color gradient background; `AspectGradientBackground::default()` is black, no gradient.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AspectGradientBackground {
    /// The `Aspect_Background` base, holding the first color.
    base: AspectBackground,
    /// `MyColor2`: the second color.
    color2: Color,
    /// `MyGradientMethod`: how the two colors blend.
    gradient_method: AspectGradientFillMethod,
}
impl Default for AspectGradientBackground {
    /// Black to black with no fill method, as `Aspect_GradientBackground()`.
    fn default() -> Self {
        let black = Color::named(NamedColor::BLACK);
        Self::new(black, black, AspectGradientFillMethod::None)
    }
}
impl Deref for AspectGradientBackground {
    type Target = AspectBackground;
    fn deref(&self) -> &AspectBackground {
        &self.base
    }
}
impl DerefMut for AspectGradientBackground {
    fn deref_mut(&mut self) -> &mut AspectBackground {
        &mut self.base
    }
}
impl AspectGradientBackground {
    /// A gradient from `color1` to `color2`; the native default method is
    /// [`AspectGradientFillMethod::Horizontal`].
    ///
    /// Example: `AspectGradientBackground::new(red, blue, AspectGradientFillMethod::Vertical)`.
    pub fn new(color1: Color, color2: Color, method: AspectGradientFillMethod) -> Self {
        Self {
            base: AspectBackground::new(color1),
            color2,
            gradient_method: method,
        }
    }
    /// Changes both colors and the method; `gradient.set_colors(red, blue, method)`.
    pub fn set_colors(&mut self, color1: Color, color2: Color, method: AspectGradientFillMethod) {
        self.base.set_color(color1);
        self.color2 = color2;
        self.gradient_method = method;
    }
    /// The two colors (the C++ output references); `let (first, second) = gradient.colors();`.
    pub fn colors(&self) -> (Color, Color) {
        (self.base.color(), self.color2)
    }
    /// The fill method; `gradient.bg_gradient_fill_method()`.
    pub fn bg_gradient_fill_method(&self) -> AspectGradientFillMethod {
        self.gradient_method
    }
    /// `DumpJson`: class name, both colors (keys `aColor` -> `Color`, `MyColor2`) and the method.
    ///
    /// Example: `gradient.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Aspect_GradientBackground")?;
        field_color(stream, depth, "&aColor", self.base.color())?;
        field_color(stream, depth, "&MyColor2", self.color2)?;
        field_integer(stream, "MyGradientMethod", self.gradient_method as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_background_defaults_to_matra_gray() {
        let mut background = AspectBackground::default();
        assert_eq!(background.color(), Color::named(NamedColor::MATRAGRAY));
        background.set_color(Color::named(NamedColor::WHITE));
        assert_eq!(background.color(), Color::named(NamedColor::WHITE));
    }

    #[test]
    fn gradient_keeps_both_colors_and_the_method() {
        let black = Color::named(NamedColor::BLACK);
        let red = Color::named(NamedColor::RED);
        let mut gradient = AspectGradientBackground::default();
        assert_eq!(gradient.colors(), (black, black));
        assert_eq!(
            gradient.bg_gradient_fill_method(),
            AspectGradientFillMethod::None
        );
        gradient.set_colors(red, black, AspectGradientFillMethod::Corner3);
        assert_eq!(gradient.colors(), (red, black));
        assert_eq!(gradient.color(), red);
        assert_eq!(
            gradient.bg_gradient_fill_method(),
            AspectGradientFillMethod::Corner3
        );
    }

    #[test]
    fn gradient_dump_lists_colors_and_method() {
        let black = Color::named(NamedColor::BLACK);
        let gradient =
            AspectGradientBackground::new(black, black, AspectGradientFillMethod::Vertical);
        let mut stream = DumpStream::default();
        gradient.dump_json(&mut stream, -1).unwrap();
        assert_eq!(
            stream.text(),
            "\"className\": \"Aspect_GradientBackground\", \"Color\": {\"RGB\": [0, 0, 0]}, \
             \"Color2\": {\"RGB\": [0, 0, 0]}, \"GradientMethod\": 2"
        );
    }
}
