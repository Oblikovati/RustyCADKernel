// Rust translation of BRepSweep_Translation.cxx (MakeEmptyFace, SetPCurve,
// SetGeneratingPCurve, SetDirectingPCurve, DirectSolid) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The faces and p-curves of the translation sweep.
pub(crate) use super::sweep_geometry::{
    copied_generating_edge, set_cap_parameters, set_generating_vertex_parameter,
};
use super::sweep_geometry::{null_curve, set_the_pcurve};
use super::{BRepSweepNumLinearRegularSweep, BRepSweepTranslation};
use crate::sweep::SweepNumShape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomCurve, GeomCylindricalSurface, GeomPlane, GeomSurface, GeomSurfaceOfLinearExtrusion,
};
use rustycad_tkg3d::geom_adaptor::{
    GeomAdaptorCurve, GeomAdaptorSurface, GeomAdaptorSurfaceOfLinearExtrusion,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Direction2, Direction3, Line2, Point2, StandardDirection2, Vector3};

/// The translation's `SetThePCurve`: at Precision::Confusion().
fn set_pcurve_at_confusion(
    builder: BRepBuilder,
    edge: &Shape,
    face: &Shape,
    orientation: rustycad_tkg3d::top_abs::Orientation,
    pcurve: &Handle<dyn Geom2dCurve>,
) -> Result<(), Failure> {
    set_the_pcurve(builder, edge, face, orientation, pcurve, |_, _, _| {
        Ok(precision::confusion())
    })
}

/// The kind of the raw surface of a new face (`GeomAdaptor_Surface(BRep_Tool::Surface(F,
/// L)).GetType()`).
pub(crate) fn raw_surface_type(face: &Shape) -> Result<SurfaceType, Failure> {
    let (surface, _) = BRepTool::surface_with_location(&topods::face(face)?)?;
    GeomAdaptorSurface::with_surface(&surface)?.get_type()
}

/// `MakeEmptyFace`: along the edge, the extrusion of a copy of the located curve of the
/// generating edge by the reversed vector (canonized into a plane or a cylinder on
/// request); at a vertex, a copy of the located surface of the generating face,
/// translated at the last vertex; the tolerance of the generating shape.
pub(crate) fn make_empty_face(
    t: &BRepSweepTranslation,
    gen_s: &Shape,
    dir_s: &SweepNumShape,
) -> Result<Shape, Failure> {
    let along_edge = t.sweep_base().my_dir_shape_tool.kind(dir_s) == ShapeEnum::Edge;
    let (surface, toler) = if along_edge {
        side_surface(t, &topods::edge(gen_s)?)?
    } else {
        cap_surface(t, &topods::face(gen_s)?, dir_s)?
    };
    let mut face = Face::default();
    t.brep_builder()
        .make_face_on_surface(&mut face, &surface, toler)?;
    Ok(face.into())
}

/// The extrusion surface of a generating edge and its tolerance.
fn side_surface(
    t: &BRepSweepTranslation,
    edge: &Edge,
) -> Result<(Handle<dyn GeomSurface>, f64), Failure> {
    let found = BRepTool::curve_with_location(edge)?;
    let toler = BRepTool::edge_tolerance(edge)?;
    let source = found
        .curve
        .get()
        .ok_or_else(|| null_curve("MakeEmptyFace"))?;
    let curve: Handle<dyn GeomCurve> = source.copy()?.cast();
    curve
        .get()
        .ok_or_else(|| null_curve("MakeEmptyFace"))?
        .transform(found.location.transformation())?;
    // Extruded surfaces are inverted correspondingly to the topology, so reverse.
    let direction = Direction3::from_vector(t.vec())?.reversed();
    if !t.canonize() {
        return Ok((
            Handle::new(GeomSurfaceOfLinearExtrusion::new(&curve, direction)?).cast(),
            toler,
        ));
    }
    let adapted: Handle<dyn Adaptor3dCurve> = Handle::new(GeomAdaptorCurve::with_range(
        &curve,
        found.first,
        found.last,
    )?)
    .cast();
    let extrusion =
        GeomAdaptorSurfaceOfLinearExtrusion::with_curve_and_direction(&adapted, direction);
    let surface: Handle<dyn GeomSurface> = match extrusion.get_type()? {
        SurfaceType::Plane => Handle::new(GeomPlane::from_pln(extrusion.plane()?)).cast(),
        SurfaceType::Cylinder => {
            Handle::new(GeomCylindricalSurface::from_cylinder(extrusion.cylinder()?)).cast()
        }
        _ => Handle::new(GeomSurfaceOfLinearExtrusion::new(&curve, direction)?).cast(),
    };
    Ok((surface, toler))
}

/// The copied cap surface of a generating face and its tolerance.
fn cap_surface(
    t: &BRepSweepTranslation,
    face: &Face,
    dir_s: &SweepNumShape,
) -> Result<(Handle<dyn GeomSurface>, f64), Failure> {
    let (raw, location) = BRepTool::surface_with_location(face)?;
    let toler = BRepTool::face_tolerance(face)?;
    let source = raw
        .get()
        .ok_or_else(|| null_curve("MakeEmptyFace (surface)"))?;
    let surface: Handle<dyn GeomSurface> = source.copy()?.cast();
    let copy = surface
        .get()
        .ok_or_else(|| null_curve("MakeEmptyFace (surface)"))?;
    copy.transform(location.transformation())?;
    if dir_s.index() == 2 {
        copy.translate(t.vec())?;
    }
    Ok((surface, toler))
}

/// `SetPCurve`: the p-curves of the generating edge on the generating face (both of a
/// seam) on the new edge of the new cap face, at Precision::PConfusion().
pub(crate) fn set_cap_pcurve(
    builder: BRepBuilder,
    new_face: &Shape,
    new_edge: &Shape,
    gen_f: &Shape,
    gen_e: &Shape,
) -> Result<(), Failure> {
    let (gen_f, gen_e) = (topods::face(gen_f)?, topods::edge(gen_e)?);
    let (new_face, new_edge) = (topods::face(new_face)?, topods::edge(new_edge)?);
    if !BRepTool::is_closed_on_face(&gen_e, &gen_f)? {
        let pcurve = BRepTool::curve_on_face(&gen_e, &gen_f)?.pcurve;
        return builder.update_edge_pcurve_on_face(
            &new_edge,
            &pcurve,
            &new_face,
            precision::p_confusion(),
        );
    }
    let mut forward = topods::edge(&gen_e.oriented(rustycad_tkg3d::top_abs::Orientation::Forward))?;
    let c1 = BRepTool::curve_on_face(&forward, &gen_f)?.pcurve;
    forward.reverse();
    let c2 = BRepTool::curve_on_face(&forward, &gen_f)?.pcurve;
    builder.update_edge_pcurves_on_face(&new_edge, (&c1, &c2), &new_face, precision::p_confusion())
}

/// `SetGeneratingPCurve`: nothing on a planar side face (JAG), else the line
/// `v = 0` (or `-|V|` at the last vertex) along U.
pub(crate) fn set_generating_pcurve(
    t: &BRepSweepTranslation,
    new_face: &Shape,
    new_edge: &Shape,
    dir_v: &SweepNumShape,
    orien: rustycad_tkg3d::top_abs::Orientation,
) -> Result<(), Failure> {
    if raw_surface_type(new_face)? == SurfaceType::Plane {
        return Ok(());
    }
    let v = if dir_v.index() == 2 {
        -t.vec().magnitude()
    } else {
        0.0
    };
    let line = Line2::new(
        Point2::new(0.0, v),
        Direction2::from_standard(StandardDirection2::X),
    );
    let pcurve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_lin2d(line)).cast();
    set_pcurve_at_confusion(t.brep_builder(), new_edge, new_face, orien, &pcurve)
}

/// `SetDirectingPCurve`: nothing on a planar side face (JAG), else the line at the
/// parameter of the generating vertex along -V.
pub(crate) fn set_directing_pcurve(
    builder: BRepBuilder,
    new_face: &Shape,
    new_edge: &Shape,
    (gen_e, gen_v): (&Shape, &Shape),
    orien: rustycad_tkg3d::top_abs::Orientation,
) -> Result<(), Failure> {
    if raw_surface_type(new_face)? == SurfaceType::Plane {
        return Ok(());
    }
    let parameter = BRepTool::parameter(&topods::vertex(gen_v)?, &topods::edge(gen_e)?)?;
    let line = Line2::new(
        Point2::new(parameter, 0.0),
        Direction2::from_standard(StandardDirection2::NegY),
    );
    let pcurve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_lin2d(line)).cast();
    set_pcurve_at_confusion(builder, new_edge, new_face, orien, &pcurve)
}

/// `DirectSolid`: REVERSED when the vector is on the side of the normal of the
/// generating face at the middle of its parametric bounds.
pub(crate) fn direct_solid(
    vec: Vector3,
    gen_s: &Shape,
) -> Result<rustycad_tkg3d::top_abs::Orientation, Failure> {
    let surf = BRepAdaptorSurface::with_face(&topods::face(gen_s)?, true)?;
    let u = (surf.first_u_parameter()? + surf.last_u_parameter()?) / 2.0;
    let v = (surf.first_v_parameter()? + surf.last_v_parameter()?) / 2.0;
    let d1 = surf.d1(u, v)?;
    let x = vec.dot_cross(d1.d1u, d1.d1v);
    Ok(if x > 0.0 {
        rustycad_tkg3d::top_abs::Orientation::Reversed
    } else {
        rustycad_tkg3d::top_abs::Orientation::Forward
    })
}
