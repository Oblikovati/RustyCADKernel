//! `Prs3d_DimensionAspect` (the attributes of dimension presentations) and
//! `Prs3d_DimensionUnits` (the length and angle units of dimensions).
use super::basic_aspect::{Prs3dBasicAspect, dump_aspect_field};
use super::{
    Prs3dArrowAspect, Prs3dDimensionArrowOrientation, Prs3dDimensionTextHorizontalPosition,
    Prs3dDimensionTextVerticalPosition, Prs3dLineAspect, Prs3dTextAspect,
};
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::aspect::AspectTypeOfLine;
use rustycad_tkservice::graphic3d::{
    Graphic3dAspectsTextMembers as _, Graphic3dHorizontalTextAlignment,
    Graphic3dVerticalTextAlignment,
};
use rustycad_tkservice::json_dump::{class_begin, field_integer, field_real, field_unquoted};
use std::f64::consts::PI;
use std::io;

/// The length and angle units of dimensions: meters and radians by default.
///
/// Example: `let mut units = Prs3dDimensionUnits::default(); units.set_angle_units(&"deg".into());`.
#[derive(Clone, Debug, PartialEq)]
pub struct Prs3dDimensionUnits {
    length_units: AsciiString,
    angle_units: AsciiString,
}

impl Default for Prs3dDimensionUnits {
    fn default() -> Self {
        Self {
            length_units: AsciiString::from("m"),
            angle_units: AsciiString::from("rad"),
        }
    }
}

impl Prs3dDimensionUnits {
    /// Sets the angle units; `units.set_angle_units(&"deg".into())`.
    pub fn set_angle_units(&mut self, units: &AsciiString) {
        self.angle_units = units.clone();
    }
    /// The angle units; `units.get_angle_units()`.
    pub fn get_angle_units(&self) -> &AsciiString {
        &self.angle_units
    }
    /// Sets the length units; `units.set_length_units(&"mm".into())`.
    pub fn set_length_units(&mut self, units: &AsciiString) {
        self.length_units = units.clone();
    }
    /// The length units; `units.get_length_units()`.
    pub fn get_length_units(&self) -> &AsciiString {
        &self.length_units
    }
}

/// The value fields of a dimension aspect.
#[derive(Clone)]
struct Prs3dDimensionState {
    line_aspect: Handle<Prs3dLineAspect>,
    text_aspect: Handle<Prs3dTextAspect>,
    arrow_aspect: Handle<Prs3dArrowAspect>,
    value_string_format: AsciiString,
    extension_size: f64,
    arrow_tail_size: f64,
    arrow_orientation: Prs3dDimensionArrowOrientation,
    text_h_position: Prs3dDimensionTextHorizontalPosition,
    text_v_position: Prs3dDimensionTextVerticalPosition,
    to_display_units: bool,
    is_text_3d: bool,
    is_text_shaded: bool,
    is_arrows_3d: bool,
}

/// The attributes of dimension presentations.
///
/// Example: `let dim = Prs3dDimensionAspect::new()?; dim.set_common_color(red);`.
pub struct Prs3dDimensionAspect {
    transient: StandardTransient,
    state: SharedState<Prs3dDimensionState>,
}
rustycad_tkernel::impl_transient!(Prs3dDimensionAspect, "Prs3d_DimensionAspect", dyn Prs3dBasicAspect, transient,
    interfaces: [dyn Prs3dBasicAspect]);

/// The lawn green, non-zoomable, centered text of the default dimension aspect.
fn default_text() -> Prs3dTextAspect {
    let text = Prs3dTextAspect::new();
    if let Some(aspect) = text.aspect().get() {
        aspect.set_text_zoomable(false);
    }
    text.set_color(Color::named(NamedColor::LAWNGREEN));
    text.set_horizontal_justification(Graphic3dHorizontalTextAlignment::Center);
    text.set_vertical_justification(Graphic3dVerticalTextAlignment::Center);
    text
}

/// The lawn green arrow of 12 degrees and length 6 of the default dimension aspect.
fn default_arrow() -> Result<Prs3dArrowAspect, Failure> {
    let arrow = Prs3dArrowAspect::new()?;
    arrow.set_color(Color::named(NamedColor::LAWNGREEN));
    arrow.set_angle(PI * 12.0 / 180.0)?;
    arrow.set_length(6.0);
    Ok(arrow)
}

impl Prs3dDimensionAspect {
    /// The default: lawn green lines, text and arrows, fitted text and arrows, format "%g".
    /// Example: `Prs3dDimensionAspect::new()?`.
    pub fn new() -> Result<Self, Failure> {
        let line = Prs3dLineAspect::new(
            Color::named(NamedColor::LAWNGREEN),
            AspectTypeOfLine::Solid,
            1.0,
        )?;
        let state = Prs3dDimensionState {
            line_aspect: Handle::new(line),
            text_aspect: Handle::new(default_text()),
            arrow_aspect: Handle::new(default_arrow()?),
            value_string_format: AsciiString::from("%g"),
            extension_size: 6.0,
            arrow_tail_size: 6.0,
            arrow_orientation: Prs3dDimensionArrowOrientation::Fit,
            text_h_position: Prs3dDimensionTextHorizontalPosition::Fit,
            text_v_position: Prs3dDimensionTextVerticalPosition::Center,
            to_display_units: false,
            is_text_3d: false,
            is_text_shaded: false,
            is_arrows_3d: false,
        };
        Ok(Self {
            transient: StandardTransient::default(),
            state: SharedState::new(state),
        })
    }
    fn edit(&self, change: impl FnOnce(&mut Prs3dDimensionState)) {
        change(&mut self.state.write());
    }
    /// The line aspect; `dim.line_aspect()`.
    pub fn line_aspect(&self) -> Handle<Prs3dLineAspect> {
        self.state.read().line_aspect.clone()
    }
    /// Sets the line aspect; `dim.set_line_aspect(line)`.
    pub fn set_line_aspect(&self, aspect: Handle<Prs3dLineAspect>) {
        self.edit(|s| s.line_aspect = aspect);
    }
    /// The text aspect; `dim.text_aspect()`.
    pub fn text_aspect(&self) -> Handle<Prs3dTextAspect> {
        self.state.read().text_aspect.clone()
    }
    /// Sets the text aspect; `dim.set_text_aspect(text)`.
    pub fn set_text_aspect(&self, aspect: Handle<Prs3dTextAspect>) {
        self.edit(|s| s.text_aspect = aspect);
    }
    /// Whether the text is 3D; `dim.is_text_3d()`.
    pub fn is_text_3d(&self) -> bool {
        self.state.read().is_text_3d
    }
    /// Sets whether the text is 3D; `dim.make_text_3d(true)`.
    pub fn make_text_3d(&self, is_text_3d: bool) {
        self.edit(|s| s.is_text_3d = is_text_3d);
    }
    /// Whether the 3D text is shaded; `dim.is_text_shaded()`.
    pub fn is_text_shaded(&self) -> bool {
        self.state.read().is_text_shaded
    }
    /// Sets whether the 3D text is shaded; `dim.make_text_shaded(true)`.
    pub fn make_text_shaded(&self, is_text_shaded: bool) {
        self.edit(|s| s.is_text_shaded = is_text_shaded);
    }
    /// Whether the arrows are 3D; `dim.is_arrows_3d()`.
    pub fn is_arrows_3d(&self) -> bool {
        self.state.read().is_arrows_3d
    }
    /// Sets whether the arrows are 3D; `dim.make_arrows_3d(true)`.
    pub fn make_arrows_3d(&self, is_arrows_3d: bool) {
        self.edit(|s| s.is_arrows_3d = is_arrows_3d);
    }
    /// Whether units are displayed; `dim.is_units_displayed()`.
    pub fn is_units_displayed(&self) -> bool {
        self.state.read().to_display_units
    }
    /// Sets whether units are displayed; `dim.make_units_displayed(true)`.
    pub fn make_units_displayed(&self, is_displayed: bool) {
        self.edit(|s| s.to_display_units = is_displayed);
    }
    /// Sets the arrow orientation; `dim.set_arrow_orientation(Prs3dDimensionArrowOrientation::External)`.
    pub fn set_arrow_orientation(&self, orientation: Prs3dDimensionArrowOrientation) {
        self.edit(|s| s.arrow_orientation = orientation);
    }
    /// The arrow orientation; `dim.arrow_orientation()`.
    pub fn arrow_orientation(&self) -> Prs3dDimensionArrowOrientation {
        self.state.read().arrow_orientation
    }
    /// Sets the vertical text position; `dim.set_text_vertical_position(Prs3dDimensionTextVerticalPosition::Above)`.
    pub fn set_text_vertical_position(&self, position: Prs3dDimensionTextVerticalPosition) {
        self.edit(|s| s.text_v_position = position);
    }
    /// The vertical text position; `dim.text_vertical_position()`.
    pub fn text_vertical_position(&self) -> Prs3dDimensionTextVerticalPosition {
        self.state.read().text_v_position
    }
    /// Sets the horizontal text position; `dim.set_text_horizontal_position(Prs3dDimensionTextHorizontalPosition::Left)`.
    pub fn set_text_horizontal_position(&self, position: Prs3dDimensionTextHorizontalPosition) {
        self.edit(|s| s.text_h_position = position);
    }
    /// The horizontal text position; `dim.text_horizontal_position()`.
    pub fn text_horizontal_position(&self) -> Prs3dDimensionTextHorizontalPosition {
        self.state.read().text_h_position
    }
    /// The arrow aspect; `dim.arrow_aspect()`.
    pub fn arrow_aspect(&self) -> Handle<Prs3dArrowAspect> {
        self.state.read().arrow_aspect.clone()
    }
    /// Sets the arrow aspect; `dim.set_arrow_aspect(arrow)`.
    pub fn set_arrow_aspect(&self, aspect: Handle<Prs3dArrowAspect>) {
        self.edit(|s| s.arrow_aspect = aspect);
    }
    /// `SetCommonColor`: one color for the lines, the text and the arrows; `dim.set_common_color(red)`.
    pub fn set_common_color(&self, color: Color) {
        let state = self.state.read().clone();
        if let Some(line) = state.line_aspect.get() {
            line.set_color(color);
        }
        if let Some(text) = state.text_aspect.get() {
            text.set_color(color);
        }
        if let Some(arrow) = state.arrow_aspect.get() {
            arrow.set_color(color);
        }
    }
    /// Sets the extension size; `dim.set_extension_size(4.0)`.
    pub fn set_extension_size(&self, size: f64) {
        self.edit(|s| s.extension_size = size);
    }
    /// The extension size; `dim.extension_size()`.
    pub fn extension_size(&self) -> f64 {
        self.state.read().extension_size
    }
    /// Sets the arrow tail size; `dim.set_arrow_tail_size(4.0)`.
    pub fn set_arrow_tail_size(&self, size: f64) {
        self.edit(|s| s.arrow_tail_size = size);
    }
    /// The arrow tail size; `dim.arrow_tail_size()`.
    pub fn arrow_tail_size(&self) -> f64 {
        self.state.read().arrow_tail_size
    }
    /// Sets the printf format of the value; `dim.set_value_string_format(&"%.2f".into())`.
    pub fn set_value_string_format(&self, format: &AsciiString) {
        let format = format.clone();
        self.edit(|s| s.value_string_format = format);
    }
    /// The printf format of the value; `dim.value_string_format()`.
    pub fn value_string_format(&self) -> AsciiString {
        self.state.read().value_string_format.clone()
    }
}

impl Prs3dBasicAspect for Prs3dDimensionAspect {
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Prs3d_DimensionAspect")?;
        let s = self.state.read().clone();
        dump_aspect_field(stream, depth, "myLineAspect.get()", &s.line_aspect)?;
        dump_aspect_field(stream, depth, "myTextAspect.get()", &s.text_aspect)?;
        dump_aspect_field(stream, depth, "myArrowAspect.get()", &s.arrow_aspect)?;
        field_unquoted(
            stream,
            "myValueStringFormat",
            s.value_string_format.as_bytes(),
        )?;
        field_real(stream, "myExtensionSize", s.extension_size)?;
        field_real(stream, "myArrowTailSize", s.arrow_tail_size)?;
        let integers = [
            ("myArrowOrientation", s.arrow_orientation as i32),
            ("myTextHPosition", s.text_h_position as i32),
            ("myTextVPosition", s.text_v_position as i32),
            ("myToDisplayUnits", i32::from(s.to_display_units)),
            ("myIsText3d", i32::from(s.is_text_3d)),
            ("myIsTextShaded", i32::from(s.is_text_shaded)),
            ("myIsArrows3d", i32::from(s.is_arrows_3d)),
        ];
        integers
            .into_iter()
            .try_for_each(|(field, value)| field_integer(stream, field, value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkservice::graphic3d::Graphic3dAspectsMembers as _;

    #[test]
    fn units_default_to_meters_and_radians() {
        let mut units = Prs3dDimensionUnits::default();
        assert_eq!(units.get_length_units().as_bytes(), b"m");
        assert_eq!(units.get_angle_units().as_bytes(), b"rad");
        units.set_length_units(&AsciiString::from("mm"));
        units.set_angle_units(&AsciiString::from("deg"));
        assert_eq!(units.clone().get_length_units().as_bytes(), b"mm");
        assert_eq!(units.get_angle_units().as_bytes(), b"deg");
    }

    #[test]
    fn defaults_and_common_color() {
        let dim = Prs3dDimensionAspect::new().unwrap();
        assert_eq!(dim.arrow_aspect().angle(), PI * 12.0 / 180.0);
        assert_eq!(dim.arrow_aspect().length(), 6.0);
        assert!(!dim.text_aspect().aspect().is_text_zoomable());
        assert_eq!(dim.value_string_format().as_bytes(), b"%g");
        dim.set_common_color(Color::named(NamedColor::RED));
        assert_eq!(
            dim.line_aspect().aspect().interior_color(),
            Color::named(NamedColor::RED)
        );
        assert_eq!(
            dim.text_aspect().aspect().interior_color(),
            Color::named(NamedColor::RED)
        );
        let mut stream = DumpStream::default();
        dim.dump_json(&mut stream, 0).unwrap();
        assert!(
            stream
                .text()
                .contains("\"ValueStringFormat\": %g, \"ExtensionSize\": 6")
        );
    }

    #[test]
    fn flags_and_positions() {
        let dim = Prs3dDimensionAspect::new().unwrap();
        dim.make_text_3d(true);
        dim.make_text_shaded(true);
        dim.make_arrows_3d(true);
        dim.make_units_displayed(true);
        dim.set_arrow_orientation(Prs3dDimensionArrowOrientation::Internal);
        dim.set_text_vertical_position(Prs3dDimensionTextVerticalPosition::Below);
        dim.set_text_horizontal_position(Prs3dDimensionTextHorizontalPosition::Right);
        dim.set_extension_size(1.0);
        dim.set_arrow_tail_size(2.0);
        assert!(
            dim.is_text_3d()
                && dim.is_text_shaded()
                && dim.is_arrows_3d()
                && dim.is_units_displayed()
        );
        assert_eq!(
            dim.arrow_orientation(),
            Prs3dDimensionArrowOrientation::Internal
        );
        assert_eq!(
            dim.text_vertical_position(),
            Prs3dDimensionTextVerticalPosition::Below
        );
        assert_eq!(
            dim.text_horizontal_position(),
            Prs3dDimensionTextHorizontalPosition::Right
        );
        assert_eq!((dim.extension_size(), dim.arrow_tail_size()), (1.0, 2.0));
    }
}
