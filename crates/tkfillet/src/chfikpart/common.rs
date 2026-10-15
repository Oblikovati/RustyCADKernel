// Rust translation of ChFiKPart_IndexCurveInDS and ChFiKPart_IndexSurfaceInDS
// (ChFiKPart_ComputeData_Fcts.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The geometries of the particular cases put in the data structure, and the small
//! frame helpers the `Make*` functions share.
use crate::chfi_ds::{SurfDataHandle, lock};
use rustycad_tkbool::top_ope_brep_ds::{
    TopOpeBRepDSCurve, TopOpeBRepDSDataStructure, TopOpeBRepDSSurface,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Axis3, Direction2, Direction3, Point2, Point3};

/// `ChFiKPart_IndexCurveInDS(C, DStr)`: the curve added to the structure (tolerance 0);
/// returns its index; `index_curve_in_ds(&c, &mut ds)`.
pub fn index_curve_in_ds(c: &Handle<dyn GeomCurve>, dstr: &mut TopOpeBRepDSDataStructure) -> i32 {
    dstr.add_curve(&mut TopOpeBRepDSCurve::new(c, 0., false))
}

/// `ChFiKPart_IndexSurfaceInDS(S, DStr)`: the surface added to the structure (tolerance
/// 0); returns its index; `index_surface_in_ds(&s, &mut ds)`.
pub fn index_surface_in_ds(
    s: &Handle<dyn GeomSurface>,
    dstr: &mut TopOpeBRepDSDataStructure,
) -> i32 {
    dstr.add_surface(&TopOpeBRepDSSurface::new(s, 0.))
}

/// The normal `X ^ Y` of a frame.
pub(super) fn frame_normal(ax: &Axis3) -> Result<Direction3, Failure> {
    ax.x_direction().crossed(ax.y_direction())
}

/// A direction reversed when the orientation is REVERSED.
pub(super) fn oriented(mut d: Direction3, o: Orientation) -> Direction3 {
    if o == Orientation::Reversed {
        d.reverse();
    }
    d
}

/// A 3d line as a curve handle.
pub(super) fn line3(p: Point3, d: Direction3) -> Handle<dyn GeomCurve> {
    Handle::new(GeomLine::from_point(p, d)).cast()
}

/// A 2d line as a curve handle.
pub(super) fn line2(p: Point2, d: Direction2) -> Handle<dyn Geom2dCurve> {
    Handle::new(Geom2dLine::from_point(p, d)).cast()
}

/// A surface added to the structure as the surface of the data.
pub(super) fn set_surf(
    data: &SurfDataHandle,
    s: Handle<dyn GeomSurface>,
    dstr: &mut TopOpeBRepDSDataStructure,
) {
    let i = index_surface_in_ds(&s, dstr);
    lock(data).change_surf(i);
}

/// The orientation of the data.
pub(super) fn set_orientation(data: &SurfDataHandle, o: Orientation) {
    *lock(data).change_orientation() = o;
}

/// The interference of the data on a face (1 or 2): the curve added to the structure,
/// its transition and the pcurves on the face and on the surface.
pub(super) fn set_interference(
    data: &SurfDataHandle,
    on_s: i32,
    c: &Handle<dyn GeomCurve>,
    trans: Orientation,
    (pc_face, pc_surf): (&Handle<dyn Geom2dCurve>, &Handle<dyn Geom2dCurve>),
    dstr: &mut TopOpeBRepDSDataStructure,
) {
    let i = index_curve_in_ds(c, dstr);
    lock(data)
        .change_interference(on_s)
        .set_interference(i, trans, pc_face, pc_surf);
}

/// FORWARD when the flag, REVERSED otherwise.
pub(super) fn fwd_if(b: bool) -> Orientation {
    if b {
        Orientation::Forward
    } else {
        Orientation::Reversed
    }
}
