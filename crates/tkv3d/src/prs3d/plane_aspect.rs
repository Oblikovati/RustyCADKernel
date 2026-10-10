//! `Prs3d_PlaneAspect`: the edges, isoparameters and arrows of a plane presentation.
use super::Prs3dLineAspect;
use super::basic_aspect::{Prs3dBasicAspect, dump_aspect_field};
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkservice::aspect::AspectTypeOfLine;
use rustycad_tkservice::json_dump::{class_begin, field_integer, field_real};
use std::f64::consts::PI;
use std::io;

/// The numeric and boolean settings of a plane aspect.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Prs3dPlaneSettings {
    /// `myArrowsLength`.
    pub arrows_length: f64,
    /// `myArrowsSize`.
    pub arrows_size: f64,
    /// `myArrowsAngle`.
    pub arrows_angle: f64,
    /// `myPlaneXLength`.
    pub plane_x_length: f64,
    /// `myPlaneYLength`.
    pub plane_y_length: f64,
    /// `myIsoDistance`.
    pub iso_distance: f64,
    /// `myDrawCenterArrow`.
    pub draw_center_arrow: bool,
    /// `myDrawEdgesArrows`.
    pub draw_edges_arrows: bool,
    /// `myDrawEdges`.
    pub draw_edges: bool,
    /// `myDrawIso`.
    pub draw_iso: bool,
}

impl Default for Prs3dPlaneSettings {
    fn default() -> Self {
        Self {
            arrows_length: 0.02,
            arrows_size: 0.1,
            arrows_angle: PI / 8.0,
            plane_x_length: 1.0,
            plane_y_length: 1.0,
            iso_distance: 0.5,
            draw_center_arrow: false,
            draw_edges_arrows: false,
            draw_edges: true,
            draw_iso: false,
        }
    }
}

/// The attributes of a plane presentation.
///
/// Example: `let plane = Prs3dPlaneAspect::new()?; plane.set_plane_length(2.0, 3.0);`.
pub struct Prs3dPlaneAspect {
    transient: StandardTransient,
    edges_aspect: Handle<Prs3dLineAspect>,
    iso_aspect: Handle<Prs3dLineAspect>,
    arrow_aspect: Handle<Prs3dLineAspect>,
    settings: SharedState<Prs3dPlaneSettings>,
}
rustycad_tkernel::impl_transient!(Prs3dPlaneAspect, "Prs3d_PlaneAspect", dyn Prs3dBasicAspect, transient,
    interfaces: [dyn Prs3dBasicAspect]);

/// A solid line aspect of a named color and width.
fn solid_line(name: NamedColor, width: f64) -> Result<Handle<Prs3dLineAspect>, Failure> {
    Ok(Handle::new(Prs3dLineAspect::new(
        Color::named(name),
        AspectTypeOfLine::Solid,
        width,
    )?))
}

impl Prs3dPlaneAspect {
    /// Green edges, gray75 isoparameters of width 0.5, peach-puff arrows; `Prs3dPlaneAspect::new()?`.
    pub fn new() -> Result<Self, Failure> {
        Ok(Self {
            transient: StandardTransient::default(),
            edges_aspect: solid_line(NamedColor::GREEN, 1.0)?,
            iso_aspect: solid_line(NamedColor::GRAY75, 0.5)?,
            arrow_aspect: solid_line(NamedColor::PEACHPUFF, 1.0)?,
            settings: SharedState::new(Prs3dPlaneSettings::default()),
        })
    }
    /// The edges aspect; `plane.edges_aspect()`.
    pub fn edges_aspect(&self) -> Handle<Prs3dLineAspect> {
        self.edges_aspect.clone()
    }
    /// The isoparameters aspect; `plane.iso_aspect()`.
    pub fn iso_aspect(&self) -> Handle<Prs3dLineAspect> {
        self.iso_aspect.clone()
    }
    /// The arrows aspect; `plane.arrow_aspect()`.
    pub fn arrow_aspect(&self) -> Handle<Prs3dLineAspect> {
        self.arrow_aspect.clone()
    }
    /// A copy of the settings; `plane.settings().iso_distance`.
    pub fn settings(&self) -> Prs3dPlaneSettings {
        *self.settings.read()
    }
    fn edit(&self, change: impl FnOnce(&mut Prs3dPlaneSettings)) {
        change(&mut self.settings.write());
    }
    /// Sets the arrows length; `plane.set_arrows_length(0.1)`.
    pub fn set_arrows_length(&self, length: f64) {
        self.edit(|s| s.arrows_length = length);
    }
    /// The arrows length; `plane.arrows_length()`.
    pub fn arrows_length(&self) -> f64 {
        self.settings().arrows_length
    }
    /// Sets the arrows size; `plane.set_arrows_size(0.2)`.
    pub fn set_arrows_size(&self, size: f64) {
        self.edit(|s| s.arrows_size = size);
    }
    /// The arrows size; `plane.arrows_size()`.
    pub fn arrows_size(&self) -> f64 {
        self.settings().arrows_size
    }
    /// Sets the arrows angle; `plane.set_arrows_angle(0.3)`.
    pub fn set_arrows_angle(&self, angle: f64) {
        self.edit(|s| s.arrows_angle = angle);
    }
    /// The arrows angle; `plane.arrows_angle()`.
    pub fn arrows_angle(&self) -> f64 {
        self.settings().arrows_angle
    }
    /// Sets whether the center arrow is drawn; `plane.set_display_center_arrow(true)`.
    pub fn set_display_center_arrow(&self, to_draw: bool) {
        self.edit(|s| s.draw_center_arrow = to_draw);
    }
    /// Whether the center arrow is drawn; `plane.display_center_arrow()`.
    pub fn display_center_arrow(&self) -> bool {
        self.settings().draw_center_arrow
    }
    /// Sets whether the edge arrows are drawn; `plane.set_display_edges_arrows(true)`.
    pub fn set_display_edges_arrows(&self, to_draw: bool) {
        self.edit(|s| s.draw_edges_arrows = to_draw);
    }
    /// Whether the edge arrows are drawn; `plane.display_edges_arrows()`.
    pub fn display_edges_arrows(&self) -> bool {
        self.settings().draw_edges_arrows
    }
    /// Sets whether the edges are drawn; `plane.set_display_edges(false)`.
    pub fn set_display_edges(&self, to_draw: bool) {
        self.edit(|s| s.draw_edges = to_draw);
    }
    /// Whether the edges are drawn; `plane.display_edges()`.
    pub fn display_edges(&self) -> bool {
        self.settings().draw_edges
    }
    /// Sets whether the isoparameters are drawn; `plane.set_display_iso(true)`.
    pub fn set_display_iso(&self, to_draw: bool) {
        self.edit(|s| s.draw_iso = to_draw);
    }
    /// Whether the isoparameters are drawn; `plane.display_iso()`.
    pub fn display_iso(&self) -> bool {
        self.settings().draw_iso
    }
    /// Sets the plane lengths; `plane.set_plane_length(2.0, 3.0)`.
    pub fn set_plane_length(&self, lx: f64, ly: f64) {
        self.edit(|s| (s.plane_x_length, s.plane_y_length) = (lx, ly));
    }
    /// The X length; `plane.plane_x_length()`.
    pub fn plane_x_length(&self) -> f64 {
        self.settings().plane_x_length
    }
    /// The Y length; `plane.plane_y_length()`.
    pub fn plane_y_length(&self) -> f64 {
        self.settings().plane_y_length
    }
    /// Sets the distance between isoparameters; `plane.set_iso_distance(0.25)`.
    pub fn set_iso_distance(&self, distance: f64) {
        self.edit(|s| s.iso_distance = distance);
    }
    /// The distance between isoparameters; `plane.iso_distance()`.
    pub fn iso_distance(&self) -> f64 {
        self.settings().iso_distance
    }
}

impl Prs3dBasicAspect for Prs3dPlaneAspect {
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Prs3d_PlaneAspect")?;
        dump_aspect_field(stream, depth, "myEdgesAspect.get()", &self.edges_aspect)?;
        dump_aspect_field(stream, depth, "myIsoAspect.get()", &self.iso_aspect)?;
        dump_aspect_field(stream, depth, "myArrowAspect.get()", &self.arrow_aspect)?;
        let s = self.settings();
        let reals = [
            ("myArrowsLength", s.arrows_length),
            ("myArrowsSize", s.arrows_size),
            ("myArrowsAngle", s.arrows_angle),
            ("myPlaneXLength", s.plane_x_length),
            ("myPlaneYLength", s.plane_y_length),
            ("myIsoDistance", s.iso_distance),
        ];
        for (field, value) in reals {
            field_real(stream, field, value)?;
        }
        let flags = [
            ("myDrawCenterArrow", s.draw_center_arrow),
            ("myDrawEdgesArrows", s.draw_edges_arrows),
            ("myDrawEdges", s.draw_edges),
            ("myDrawIso", s.draw_iso),
        ];
        flags
            .into_iter()
            .try_for_each(|(field, value)| field_integer(stream, field, i32::from(value)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_setters() {
        let plane = Prs3dPlaneAspect::new().unwrap();
        assert_eq!(plane.settings(), Prs3dPlaneSettings::default());
        assert_eq!(plane.iso_aspect().aspect().width(), 0.5);
        plane.set_plane_length(2.0, 3.0);
        plane.set_iso_distance(0.25);
        plane.set_arrows_length(1.0);
        plane.set_arrows_size(2.0);
        plane.set_arrows_angle(0.3);
        plane.set_display_center_arrow(true);
        plane.set_display_edges_arrows(true);
        plane.set_display_edges(false);
        plane.set_display_iso(true);
        assert_eq!((plane.plane_x_length(), plane.plane_y_length()), (2.0, 3.0));
        assert_eq!((plane.iso_distance(), plane.arrows_length()), (0.25, 1.0));
        assert_eq!((plane.arrows_size(), plane.arrows_angle()), (2.0, 0.3));
        assert!(plane.display_center_arrow() && plane.display_edges_arrows());
        assert!(!plane.display_edges() && plane.display_iso());
    }
}
