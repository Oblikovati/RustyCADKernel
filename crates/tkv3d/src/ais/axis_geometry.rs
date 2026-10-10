//! Native cached endpoints and aspect identity; setters intentionally do not schedule redisplay.
use super::{AisAxis, AisTypeOfAxis};
use crate::{prs_mgr::*, prs3d::*};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::GeomLine;
use rustycad_tkmath::gp::{Axis1, Point3, Xyz};
use rustycad_tkservice::aspect::AspectTypeOfLine;
pub(super) const AXIS_PARTS: [Prs3dDatumParts; 3] = [
    Prs3dDatumParts::XAxis,
    Prs3dDatumParts::YAxis,
    Prs3dDatumParts::ZAxis,
];
impl AisAxis {
    pub(super) fn initialize_axis_line(&self) -> Result<(), Failure> {
        self.attributes()
            .set_line_aspect(Handle::new(Prs3dLineAspect::new(
                Color::named(NamedColor::RED),
                AspectTypeOfLine::Dotdash,
                1.,
            )?));
        Ok(())
    }
    pub(super) fn initialize_axis_datum(&self) -> Result<(), Failure> {
        let datum = Handle::new(Prs3dDatumAspect::new()?);
        let length = self.units.write().any_to_ls(100., b"mm").unwrap_or(0.1);
        datum.set_axis_length(length, length, length);
        for part in AXIS_PARTS {
            datum
                .line_aspect(part)
                .set_color(Color::named(NamedColor::TURQUOISE));
        }
        self.attributes().set_datum_aspect(datum);
        Ok(())
    }
    pub(super) fn initialize_axis_ray(&self, axis: &Axis1, length: f64) -> Result<(), Failure> {
        if length <= 0. && length != -1. {
            return Err(Failure::with_type(
                "Standard_NumericError",
                &format!("received axis length {length}; expected positive length or -1"),
                "",
            ));
        }
        let length = if length == -1. {
            self.units.write().any_to_ls(250000., b"mm")?
        } else {
            length
        };
        self.assign_ray_geometry(axis, length);
        self.set_infinite_state(true);
        let datum = Handle::new(Prs3dDatumAspect::new()?);
        datum.set_draw_labels(false);
        self.attributes().set_datum_aspect(datum);
        let line = Handle::new(Prs3dLineAspect::new(
            Color::named(NamedColor::RED),
            AspectTypeOfLine::Solid,
            1.,
        )?);
        self.attributes().set_line_aspect(line.clone());
        self.geometry.write().line = line;
        Ok(())
    }
    fn assign_ray_geometry(&self, axis: &Axis1, length: f64) {
        let mut g = self.geometry.write();
        g.component = Handle::new(GeomLine::new(*axis));
        g.kind = AisTypeOfAxis::ZAxis;
        g.xyz = true;
        g.direction = axis.direction();
        g.length = length;
        g.first = axis.location();
        g.last = Point3::from(g.first.xyz() + g.direction.xyz() * length);
    }
    pub(super) fn refresh_line_endpoints(&self) -> Result<(), Failure> {
        let axis = self.component().position();
        let length = self.units.write().any_to_ls(250000., b"mm")?;
        let offset = axis.direction().xyz() * length;
        let mut g = self.geometry.write();
        g.first = Point3::from(axis.location().xyz() + offset);
        g.last = Point3::from(axis.location().xyz() - offset);
        Ok(())
    }
    pub(super) fn compute_axis_fields(&self) -> Result<(), Failure> {
        let mut g = self.geometry.write();
        let frame = g.placement.ax2()?;
        let datum = self.attributes().datum_aspect();
        g.first = frame.location();
        let mut offset = Xyz::default();
        if let Some((part, direction, label)) = axis_member(g.kind, &frame) {
            g.length = datum.axis_length(part);
            g.direction = direction;
            g.line = datum.line_aspect(part);
            g.label = label;
            offset = direction.xyz();
        }
        g.component = Handle::new(GeomLine::new(Axis1::new(g.first, g.direction)));
        g.last = Point3::from(g.first.xyz() + offset * g.length);
        self.set_infinite_state(true);
        Ok(())
    }
}
fn axis_member(
    kind: AisTypeOfAxis,
    frame: &rustycad_tkmath::gp::Axis2,
) -> Option<(
    Prs3dDatumParts,
    rustycad_tkmath::gp::Direction3,
    &'static str,
)> {
    match kind {
        AisTypeOfAxis::XAxis => Some((Prs3dDatumParts::XAxis, frame.x_direction(), "X")),
        AisTypeOfAxis::YAxis => Some((Prs3dDatumParts::YAxis, frame.y_direction(), "Y")),
        AisTypeOfAxis::ZAxis => Some((Prs3dDatumParts::ZAxis, frame.direction(), "Z")),
        AisTypeOfAxis::Unknown => None,
    }
}
