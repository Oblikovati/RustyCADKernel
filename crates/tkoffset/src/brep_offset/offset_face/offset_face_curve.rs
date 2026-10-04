//! Exact 3D curves for elementary-surface p-curves in `BRepOffset_Offset::Init(Face)`.
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::Edge,
};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::{adaptor2d::Adaptor2dCurve2d, geom2d_adaptor::Geom2dAdaptorCurve};
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dSurface,
    geom::{GeomCircle, GeomCurve},
    geom_adaptor::GeomAdaptorSurface,
};
use rustycad_tkmath::{
    elslib,
    geom_abs::{CurveType, SurfaceType},
    gp::{Axis1, Circle3, Direction2, StandardDirection2},
};

/// Stores the exact circle for a cylinder's V-isoparametric line, as native ComputeCurve3d does.
pub(super) fn build_cylinder_v_iso(
    edge: &Edge,
    source: &Edge,
    face: &rustycad_tkbrep::topods::Face,
) -> Result<bool, Failure> {
    let pcurve = BRepTool::curve_on_face(edge, face)?;
    let curve2d = Geom2dAdaptorCurve::with_range(&pcurve.pcurve, pcurve.first, pcurve.last)?;
    let curve2d: Handle<dyn Adaptor2dCurve2d> = Handle::new(curve2d).cast();
    if curve2d.get_type()? != CurveType::Line {
        return Ok(false);
    }
    let line = curve2d.line()?;
    let x_axis = Direction2::from_standard(StandardDirection2::X);
    if !line.direction().is_parallel(x_axis, precision::angular()) {
        return Ok(false);
    }
    let surface = BRepTool::surface(face)?;
    let adaptor = GeomAdaptorSurface::with_surface(&surface)?;
    if adaptor.get_type()? != SurfaceType::Cylinder {
        return Ok(false);
    }
    store_iso_circle(
        edge,
        source,
        adaptor.cylinder()?,
        line.location(),
        line.direction().is_opposite(x_axis, precision::angular()),
    )?;
    Ok(true)
}

fn store_iso_circle(
    edge: &Edge,
    source: &Edge,
    cylinder: rustycad_tkmath::gp::Cylinder3,
    uv: rustycad_tkmath::gp::Point2,
    reverse: bool,
) -> Result<(), Failure> {
    let frame = cylinder.position();
    let mut circle: Circle3 = elslib::cylinder_v_iso(frame, cylinder.radius(), uv.y());
    circle.rotate(Axis1::new(frame.location(), frame.direction()), uv.x())?;
    if reverse {
        let mut position = circle.position();
        position.set_direction(position.direction().reversed())?;
        circle.set_position(position);
    }
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomCircle::from_circ(circle)).cast();
    BRepBuilder::default().update_edge_curve(edge, &curve, BRepTool::edge_tolerance(source)?)
}
