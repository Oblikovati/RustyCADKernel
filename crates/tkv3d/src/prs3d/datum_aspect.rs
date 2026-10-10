//! `Prs3d_DatumAspect`: the line, shading, text, point and arrow aspects and the attributes
//! of a datum (trihedron) presentation.
use super::basic_aspect::{Prs3dBasicAspect, dump_aspect_field, prs3d_copy_aspects};
use super::{
    Prs3dArrowAspect, Prs3dDatumAttribute, Prs3dDatumAxes, Prs3dDatumParts, Prs3dLineAspect,
    Prs3dPointAspect, Prs3dShadingAspect, Prs3dTextAspect,
};
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkservice::aspect::{AspectTypeOfFacingModel, AspectTypeOfLine, AspectTypeOfMarker};
use rustycad_tkservice::graphic3d::Graphic3dAspectsMembers as _;
use rustycad_tkservice::json_dump::{class_begin, field_integer};
use std::io;

/// The settable fields of a datum aspect.
#[derive(Clone)]
struct Prs3dDatumState {
    text_aspects: [Handle<Prs3dTextAspect>; Prs3dDatumParts::NB],
    point_aspect: Handle<Prs3dPointAspect>,
    arrow_aspect: Handle<Prs3dArrowAspect>,
    attributes: [f64; Prs3dDatumAttribute::NB],
    axes: Prs3dDatumAxes,
    to_draw_labels: bool,
    to_draw_arrows: bool,
}

/// The attributes of a datum presentation.
///
/// Example: `let datum = Prs3dDatumAspect::new()?; datum.set_axis_length(10.0, 20.0, 30.0);`.
pub struct Prs3dDatumAspect {
    transient: StandardTransient,
    shaded_aspects: [Handle<Prs3dShadingAspect>; Prs3dDatumParts::NB],
    line_aspects: [Handle<Prs3dLineAspect>; Prs3dDatumParts::NB],
    state: SharedState<Prs3dDatumState>,
}
rustycad_tkernel::impl_transient!(Prs3dDatumAspect, "Prs3d_DatumAspect", dyn Prs3dBasicAspect, transient,
    interfaces: [dyn Prs3dBasicAspect]);

/// The default axis length, the same as in the interactive context.
const DEFAULT_AXIS_LENGTH: f64 = 100.0;

/// The default attributes: three axes of 100 and the shading proportions.
fn default_attributes() -> [f64; Prs3dDatumAttribute::NB] {
    let mut attributes = [0.0; Prs3dDatumAttribute::NB];
    attributes[Prs3dDatumAttribute::XAxisLength as usize] = DEFAULT_AXIS_LENGTH;
    attributes[Prs3dDatumAttribute::YAxisLength as usize] = DEFAULT_AXIS_LENGTH;
    attributes[Prs3dDatumAttribute::ZAxisLength as usize] = DEFAULT_AXIS_LENGTH;
    attributes[Prs3dDatumAttribute::ShadingTubeRadiusPercent as usize] = 0.02;
    attributes[Prs3dDatumAttribute::ShadingConeRadiusPercent as usize] = 0.04;
    attributes[Prs3dDatumAttribute::ShadingConeLengthPercent as usize] = 0.1;
    attributes[Prs3dDatumAttribute::ShadingOriginRadiusPercent as usize] = 0.015;
    attributes[Prs3dDatumAttribute::ShadingNumberOfFacettes as usize] = 12.0;
    attributes
}

/// The part aspects of the constructor: a line (none for the origin) and a shading aspect
/// per drawn part, in the arrow color for the arrows.
type PartAspects = (
    [Handle<Prs3dShadingAspect>; Prs3dDatumParts::NB],
    [Handle<Prs3dLineAspect>; Prs3dDatumParts::NB],
);

fn part_aspects(default_color: Color, arrow_color: Color) -> Result<PartAspects, Failure> {
    let mut shaded: [Handle<Prs3dShadingAspect>; Prs3dDatumParts::NB] = Default::default();
    let mut lines: [Handle<Prs3dLineAspect>; Prs3dDatumParts::NB] = Default::default();
    for part in Prs3dDatumParts::DRAWN {
        let is_arrow = (Prs3dDatumParts::XArrow..=Prs3dDatumParts::ZArrow).contains(&part);
        let color = if is_arrow { arrow_color } else { default_color };
        if part != Prs3dDatumParts::Origin {
            let line = Prs3dLineAspect::new(color, AspectTypeOfLine::Solid, 1.0)?;
            lines[part as usize] = Handle::new(line);
        }
        let shading = Prs3dShadingAspect::new()?;
        shading.set_color(color, AspectTypeOfFacingModel::BothSide)?;
        shaded[part as usize] = Handle::new(shading);
    }
    Ok((shaded, lines))
}

impl Prs3dDatumAspect {
    /// The default: three light steel blue axes of length 100 with labels and arrows.
    /// Example: `Prs3dDatumAspect::new()?`.
    pub fn new() -> Result<Self, Failure> {
        let default_color = Color::named(NamedColor::LIGHTSTEELBLUE4);
        let point_aspect = Prs3dPointAspect::new(AspectTypeOfMarker::Empty, default_color, 1.0)?;
        let arrow_aspect = Handle::new(Prs3dArrowAspect::new()?);
        let arrow_color = arrow_aspect.aspect().interior_color();
        let (shaded_aspects, line_aspects) = part_aspects(default_color, arrow_color)?;
        let mut text_aspects: [Handle<Prs3dTextAspect>; Prs3dDatumParts::NB] = Default::default();
        for part in [
            Prs3dDatumParts::XAxis,
            Prs3dDatumParts::YAxis,
            Prs3dDatumParts::ZAxis,
        ] {
            text_aspects[part as usize] = Handle::new(Prs3dTextAspect::new());
        }
        let state = Prs3dDatumState {
            text_aspects,
            point_aspect: Handle::new(point_aspect),
            arrow_aspect,
            attributes: default_attributes(),
            axes: Prs3dDatumAxes::XYZAxes,
            to_draw_labels: true,
            to_draw_arrows: true,
        };
        Ok(Self {
            transient: StandardTransient::default(),
            shaded_aspects,
            line_aspects,
            state: SharedState::new(state),
        })
    }
    /// The line aspect of a part (null for the origin and `None`); `datum.line_aspect(Prs3dDatumParts::XAxis)`.
    pub fn line_aspect(&self, part: Prs3dDatumParts) -> Handle<Prs3dLineAspect> {
        self.line_aspects[part as usize].clone()
    }
    /// The shading aspect of a part; `datum.shading_aspect(Prs3dDatumParts::Origin)`.
    pub fn shading_aspect(&self, part: Prs3dDatumParts) -> Handle<Prs3dShadingAspect> {
        self.shaded_aspects[part as usize].clone()
    }
    /// The text aspect of an axis (null for other parts); `datum.text_aspect(Prs3dDatumParts::YAxis)`.
    pub fn text_aspect(&self, part: Prs3dDatumParts) -> Handle<Prs3dTextAspect> {
        self.state.read().text_aspects[part as usize].clone()
    }
    /// Sets one text aspect for the three axes; `datum.set_text_aspect(text)`.
    pub fn set_text_aspect(&self, aspect: Handle<Prs3dTextAspect>) {
        let mut state = self.state.write();
        for part in [
            Prs3dDatumParts::XAxis,
            Prs3dDatumParts::YAxis,
            Prs3dDatumParts::ZAxis,
        ] {
            state.text_aspects[part as usize] = aspect.clone();
        }
    }
    /// The deprecated `TextAspect()`: the X axis text aspect; `datum.text_aspect_x()`.
    pub fn text_aspect_x(&self) -> Handle<Prs3dTextAspect> {
        self.text_aspect(Prs3dDatumParts::XAxis)
    }
    /// The origin point aspect; `datum.point_aspect()`.
    pub fn point_aspect(&self) -> Handle<Prs3dPointAspect> {
        self.state.read().point_aspect.clone()
    }
    /// Sets the origin point aspect; `datum.set_point_aspect(point)`.
    pub fn set_point_aspect(&self, aspect: Handle<Prs3dPointAspect>) {
        self.state.write().point_aspect = aspect;
    }
    /// The arrow aspect; `datum.arrow_aspect()`.
    pub fn arrow_aspect(&self) -> Handle<Prs3dArrowAspect> {
        self.state.read().arrow_aspect.clone()
    }
    /// Sets the arrow aspect; `datum.set_arrow_aspect(arrow)`.
    pub fn set_arrow_aspect(&self, aspect: Handle<Prs3dArrowAspect>) {
        self.state.write().arrow_aspect = aspect;
    }
    /// Whether a part is drawn with the current axes and arrow flag; `datum.draw_datum_part(Prs3dDatumParts::XOYAxis)`.
    pub fn draw_datum_part(&self, part: Prs3dDatumParts) -> bool {
        let state = self.state.read();
        datum_part_drawn(state.axes as i32, state.to_draw_arrows, part)
    }
    /// Sets the drawn axes; `datum.set_draw_datum_axes(Prs3dDatumAxes::XYAxes)`.
    pub fn set_draw_datum_axes(&self, axes: Prs3dDatumAxes) {
        self.state.write().axes = axes;
    }
    /// The drawn axes; `datum.datum_axes()`.
    pub fn datum_axes(&self) -> Prs3dDatumAxes {
        self.state.read().axes
    }
    /// An attribute value; `datum.attribute(Prs3dDatumAttribute::XAxisLength)`.
    pub fn attribute(&self, kind: Prs3dDatumAttribute) -> f64 {
        self.state.read().attributes[kind as usize]
    }
    /// Sets an attribute value; `datum.set_attribute(Prs3dDatumAttribute::ShadingNumberOfFacettes, 24.0)`.
    pub fn set_attribute(&self, kind: Prs3dDatumAttribute, value: f64) {
        self.state.write().attributes[kind as usize] = value;
    }
    /// The length of an axis part, 0 for the other parts; `datum.axis_length(Prs3dDatumParts::ZAxis)`.
    pub fn axis_length(&self, part: Prs3dDatumParts) -> f64 {
        let kind = match part {
            Prs3dDatumParts::XAxis => Prs3dDatumAttribute::XAxisLength,
            Prs3dDatumParts::YAxis => Prs3dDatumAttribute::YAxisLength,
            Prs3dDatumParts::ZAxis => Prs3dDatumAttribute::ZAxisLength,
            _ => return 0.0,
        };
        self.attribute(kind)
    }
    /// Sets the three axis lengths; `datum.set_axis_length(1.0, 2.0, 3.0)`.
    pub fn set_axis_length(&self, l1: f64, l2: f64, l3: f64) {
        let mut state = self.state.write();
        state.attributes[Prs3dDatumAttribute::XAxisLength as usize] = l1;
        state.attributes[Prs3dDatumAttribute::YAxisLength as usize] = l2;
        state.attributes[Prs3dDatumAttribute::ZAxisLength as usize] = l3;
    }
    /// Whether labels are drawn; `datum.to_draw_labels()`.
    pub fn to_draw_labels(&self) -> bool {
        self.state.read().to_draw_labels
    }
    /// Sets whether labels are drawn; `datum.set_draw_labels(false)`.
    pub fn set_draw_labels(&self, to_draw: bool) {
        self.state.write().to_draw_labels = to_draw;
    }
    /// `SetToDrawLabels`, the same as `SetDrawLabels`; `datum.set_to_draw_labels(false)`.
    pub fn set_to_draw_labels(&self, to_draw: bool) {
        self.set_draw_labels(to_draw);
    }
    /// Whether arrows are drawn; `datum.to_draw_arrows()`.
    pub fn to_draw_arrows(&self) -> bool {
        self.state.read().to_draw_arrows
    }
    /// Sets whether arrows are drawn; `datum.set_draw_arrows(false)`.
    pub fn set_draw_arrows(&self, to_draw: bool) {
        self.state.write().to_draw_arrows = to_draw;
    }
    /// `ArrowPartForAxis`: the arrow part of an axis or arrow part, else `None`.
    /// Example: `Prs3dDatumAspect::arrow_part_for_axis(Prs3dDatumParts::YAxis) == Prs3dDatumParts::YArrow`.
    pub fn arrow_part_for_axis(part: Prs3dDatumParts) -> Prs3dDatumParts {
        match part {
            Prs3dDatumParts::XArrow | Prs3dDatumParts::XAxis => Prs3dDatumParts::XArrow,
            Prs3dDatumParts::YArrow | Prs3dDatumParts::YAxis => Prs3dDatumParts::YArrow,
            Prs3dDatumParts::ZArrow | Prs3dDatumParts::ZAxis => Prs3dDatumParts::ZArrow,
            _ => Prs3dDatumParts::None,
        }
    }
    /// `CopyAspectsFrom`: the flags and the Graphic3d aspects of every sub-aspect.
    /// Example: `datum.copy_aspects_from(&other)`.
    pub fn copy_aspects_from(&self, other: &Prs3dDatumAspect) {
        let source = other.state.read().clone();
        let target = {
            let mut state = self.state.write();
            state.to_draw_arrows = source.to_draw_arrows;
            state.to_draw_labels = source.to_draw_labels;
            state.clone()
        };
        prs3d_copy_aspects(
            &*target.point_aspect.aspect(),
            &*source.point_aspect.aspect(),
        );
        prs3d_copy_aspects(
            &*target.arrow_aspect.aspect(),
            &*source.arrow_aspect.aspect(),
        );
        for part in [
            Prs3dDatumParts::XAxis,
            Prs3dDatumParts::YAxis,
            Prs3dDatumParts::ZAxis,
        ] {
            let index = part as usize;
            prs3d_copy_aspects(
                &*target.text_aspects[index].aspect(),
                &*source.text_aspects[index].aspect(),
            );
        }
        for part in Prs3dDatumParts::DRAWN {
            self.copy_part_aspects(other, part as usize);
        }
    }
    fn copy_part_aspects(&self, other: &Prs3dDatumAspect, index: usize) {
        if let (Some(line), Some(source)) = (
            self.line_aspects[index].get(),
            other.line_aspects[index].get(),
        ) {
            prs3d_copy_aspects(&*line.aspect(), &*source.aspect());
        }
        if let (Some(shading), Some(source)) = (
            self.shaded_aspects[index].get(),
            other.shaded_aspects[index].get(),
        ) {
            prs3d_copy_aspects(&*shading.aspect(), &*source.aspect());
        }
    }
}

/// `DrawDatumPart` over the axes mask and the arrow flag.
fn datum_part_drawn(axes: i32, to_draw_arrows: bool, part: Prs3dDatumParts) -> bool {
    let has = |mask: Prs3dDatumAxes| axes & mask as i32 != 0;
    match part {
        Prs3dDatumParts::Origin => true,
        Prs3dDatumParts::XAxis => has(Prs3dDatumAxes::XAxis),
        Prs3dDatumParts::XArrow => has(Prs3dDatumAxes::XAxis) && to_draw_arrows,
        Prs3dDatumParts::YAxis => has(Prs3dDatumAxes::YAxis),
        Prs3dDatumParts::YArrow => has(Prs3dDatumAxes::YAxis) && to_draw_arrows,
        Prs3dDatumParts::ZAxis => has(Prs3dDatumAxes::ZAxis),
        Prs3dDatumParts::ZArrow => has(Prs3dDatumAxes::ZAxis) && to_draw_arrows,
        Prs3dDatumParts::XOYAxis => has(Prs3dDatumAxes::XAxis) && has(Prs3dDatumAxes::YAxis),
        Prs3dDatumParts::YOZAxis => has(Prs3dDatumAxes::YAxis) && has(Prs3dDatumAxes::ZAxis),
        Prs3dDatumParts::XOZAxis => has(Prs3dDatumAxes::XAxis) && has(Prs3dDatumAxes::ZAxis),
        Prs3dDatumParts::None => false,
    }
}

impl Prs3dBasicAspect for Prs3dDatumAspect {
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Prs3d_DatumAspect")?;
        let state = self.state.read().clone();
        dump_aspect_field(stream, depth, "myPointAspect.get()", &state.point_aspect)?;
        dump_aspect_field(stream, depth, "myArrowAspect.get()", &state.arrow_aspect)?;
        for aspect in &self.shaded_aspects {
            dump_aspect_field(stream, depth, "aShadingAspect.get()", aspect)?;
        }
        for aspect in &self.line_aspects {
            dump_aspect_field(stream, depth, "aLineAspect.get()", aspect)?;
        }
        for aspect in &state.text_aspects[1..=3] {
            dump_aspect_field(stream, depth, "aTextAspect.get()", aspect)?;
        }
        field_integer(stream, "myAxes", state.axes as i32)?;
        field_integer(stream, "myToDrawLabels", i32::from(state.to_draw_labels))?;
        field_integer(stream, "myToDrawArrows", i32::from(state.to_draw_arrows))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_follow_the_native_constructor() {
        let datum = Prs3dDatumAspect::new().unwrap();
        assert!(datum.line_aspect(Prs3dDatumParts::Origin).is_null());
        assert!(!datum.line_aspect(Prs3dDatumParts::XOZAxis).is_null());
        assert!(datum.line_aspect(Prs3dDatumParts::None).is_null());
        assert!(datum.text_aspect(Prs3dDatumParts::Origin).is_null());
        assert!(!datum.text_aspect(Prs3dDatumParts::ZAxis).is_null());
        assert_eq!(datum.axis_length(Prs3dDatumParts::YAxis), 100.0);
        assert_eq!(datum.axis_length(Prs3dDatumParts::Origin), 0.0);
        assert_eq!(
            datum.attribute(Prs3dDatumAttribute::ShadingNumberOfFacettes),
            12.0
        );
        let arrow_color = datum.arrow_aspect().aspect().interior_color();
        let x_arrow = datum
            .line_aspect(Prs3dDatumParts::XArrow)
            .aspect()
            .interior_color();
        assert_eq!(x_arrow, arrow_color);
    }

    #[test]
    fn drawn_parts_follow_axes_and_arrows() {
        let datum = Prs3dDatumAspect::new().unwrap();
        datum.set_draw_datum_axes(Prs3dDatumAxes::XZAxes);
        assert!(datum.draw_datum_part(Prs3dDatumParts::XOZAxis));
        assert!(!datum.draw_datum_part(Prs3dDatumParts::XOYAxis));
        assert!(datum.draw_datum_part(Prs3dDatumParts::ZArrow));
        datum.set_draw_arrows(false);
        assert!(!datum.draw_datum_part(Prs3dDatumParts::ZArrow));
        assert!(datum.draw_datum_part(Prs3dDatumParts::Origin));
        assert!(!datum.draw_datum_part(Prs3dDatumParts::None));
        assert_eq!(
            Prs3dDatumAspect::arrow_part_for_axis(Prs3dDatumParts::ZAxis),
            Prs3dDatumParts::ZArrow
        );
        assert_eq!(
            Prs3dDatumAspect::arrow_part_for_axis(Prs3dDatumParts::Origin),
            Prs3dDatumParts::None
        );
    }

    #[test]
    fn copy_aspects_from_copies_flags_and_colors() {
        let source = Prs3dDatumAspect::new().unwrap();
        source.set_draw_labels(false);
        source
            .line_aspect(Prs3dDatumParts::YAxis)
            .set_color(Color::named(NamedColor::RED));
        let target = Prs3dDatumAspect::new().unwrap();
        target.copy_aspects_from(&source);
        assert!(!target.to_draw_labels());
        let color = target
            .line_aspect(Prs3dDatumParts::YAxis)
            .aspect()
            .interior_color();
        assert_eq!(color, Color::named(NamedColor::RED));
    }
}
