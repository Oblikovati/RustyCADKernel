//! Surface bounds, protected copies, and shell composition in native order.
use super::{FaceDivider, ShapeExtendStatus};
use crate::{
    shape_analysis::ShapeAnalysis,
    shape_fix::{FixRoot, ShapeFixComposeShell},
    shape_upgrade::{SplitSurface, WireDivider},
};
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_tools::ReShape,
    topexp::Explorer,
    topods::{self, Face},
};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::{geom::GeomSurface, top_abs::ShapeEnum};
use rustycad_tkmath::toploc::Location;

pub(super) fn split_surface(
    tool: &(impl FaceDivider + ?Sized),
    area: f64,
) -> Result<bool, Failure> {
    let splitter = tool.split_surface_tool().cast::<dyn SplitSurface>();
    let Some(splitter) = splitter.get() else {
        return Ok(false);
    };
    let Some(face) = result_face(tool)? else {
        return Ok(false);
    };
    split_supported_face(tool, splitter, &face, area)
}

fn split_supported_face(
    tool: &(impl FaceDivider + ?Sized),
    splitter: &dyn SplitSurface,
    face: &Face,
    area: f64,
) -> Result<bool, Failure> {
    let (surface, location) = BRepTool::surface_with_location(face)?;
    let Some(bounds) = finite_face_bounds(face)? else {
        return Ok(false);
    };
    splitter.init_surface_bounds(&surface, extended_bounds(&surface, bounds), area)?;
    splitter.perform(tool.face_divide_data().with(|fields| fields.segment))?;
    if !splitter.status_flag(ShapeExtendStatus::Done) {
        return Ok(false);
    }
    if splitter.status_flag(ShapeExtendStatus::Done3) {
        protect_vertices(tool, face)?;
    }
    compose_surface_result(tool, splitter, face, &location)?;
    Ok(true)
}

type FaceUvBounds = (f64, f64, f64, f64);
fn finite_face_bounds(face: &Face) -> Result<Option<FaceUvBounds>, Failure> {
    let bounds = ShapeAnalysis::get_face_uv_bounds(face)?;
    if [bounds.0, bounds.1, bounds.2, bounds.3]
        .into_iter()
        .any(precision::is_infinite)
    {
        return Ok(None);
    }
    Ok(Some(bounds))
}

fn result_face(tool: &(impl FaceDivider + ?Sized)) -> Result<Option<Face>, Failure> {
    let result = tool.face_divide_data().with(|fields| fields.result.clone());
    if result.is_null() || result.shape_type()? != ShapeEnum::Face {
        tool.face_divide_data().add_status(ShapeExtendStatus::Fail3);
        return Ok(None);
    }
    Ok(Some(topods::face(&result)?))
}

fn extended_bounds(
    surface: &Handle<dyn GeomSurface>,
    bounds: (f64, f64, f64, f64),
) -> (f64, f64, f64, f64) {
    let natural = surface.bounds();
    let u = extend_axis(
        (bounds.0, bounds.1),
        (natural.0, natural.1),
        surface.is_u_periodic(),
    );
    let v = extend_axis(
        (bounds.2, bounds.3),
        (natural.2, natural.3),
        surface.is_v_periodic(),
    );
    (u.0, u.1, v.0, v.1)
}

fn extend_axis(mut bounds: (f64, f64), natural: (f64, f64), periodic: bool) -> (f64, f64) {
    if periodic {
        return bounds;
    }
    let extension = (bounds.1 - bounds.0) * 0.01;
    if bounds.0 > natural.0 {
        bounds.0 -= native_minimum(extension, bounds.0 - natural.0);
    }
    if bounds.1 < natural.1 {
        bounds.1 += native_minimum(extension, natural.1 - bounds.1);
    }
    bounds
}

fn protect_vertices(tool: &(impl FaceDivider + ?Sized), face: &Face) -> Result<(), Failure> {
    let context = tool.face_divide_base().context();
    let mut vertices = Explorer::new(face, ShapeEnum::Vertex, ShapeEnum::Shape)?;
    while vertices.more() {
        let vertex = vertices.current();
        if !context.is_recorded(vertex) {
            context.replace(vertex, &vertex.empty_copied()?)?;
        }
        vertices.next()?;
    }
    Ok(())
}

fn compose_surface_result(
    tool: &(impl FaceDivider + ?Sized),
    splitter: &dyn SplitSurface,
    face: &Face,
    location: &Location,
) -> Result<(), Failure> {
    let composer = configured_composer(tool, splitter, face, location)?;
    composer.perform()?;
    if composer.status(ShapeExtendStatus::Fail) || !composer.status(ShapeExtendStatus::Done) {
        tool.face_divide_data().add_status(ShapeExtendStatus::Fail2);
    }
    tool.face_divide_data()
        .with_mut(|fields| fields.result = composer.result());
    tool.face_divide_data().add_status(ShapeExtendStatus::Done2);
    Ok(())
}

fn configured_composer(
    tool: &(impl FaceDivider + ?Sized),
    splitter: &dyn SplitSurface,
    face: &Face,
    location: &Location,
) -> Result<ShapeFixComposeShell, Failure> {
    let base = tool.face_divide_base();
    let composer = ShapeFixComposeShell::new();
    composer.init(
        &splitter.result_surfaces(),
        location,
        face,
        base.precision(),
    )?;
    composer.set_context(&base.context());
    composer.set_max_tolerance(base.max_tolerance());
    if !tool.wire_divide_tool().is_null() {
        let wire = tool.wire_divide_tool().cast::<dyn WireDivider>();
        composer.set_transfer_param_tool(&wire.transfer_param_tool().cast());
    }
    Ok(composer)
}

fn native_minimum(first: f64, second: f64) -> f64 {
    // std::min retains its first operand for unordered comparisons.
    if second < first { second } else { first }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extension_preserves_periodic_and_native_unordered_comparisons() {
        assert_eq!(extend_axis((1., 2.), (0., 3.), true), (1., 2.));
        assert_eq!(extend_axis((1., 2.), (0., 3.), false), (0.99, 2.01));
        assert_eq!(extend_axis((0., 3.), (0., 3.), false), (0., 3.));
        assert!(native_minimum(f64::NAN, 1.).is_nan());
        assert_eq!(native_minimum(1., f64::NAN), 1.);
    }
}
