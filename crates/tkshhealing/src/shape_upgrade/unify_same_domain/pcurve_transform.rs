//! `TransformPCurves` and `AddPCurves` of ShapeUpgrade_UnifySameDomain.cxx: the p-curves
//! of the edges of the faces to merge, expressed on the surface of the reference face.
use super::geometry::null_pcurve;
use super::same_domain::same_surf;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{
    GeomCurve, GeomElementarySurface, GeomRectangularTrimmedSurface, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::geom_proj_lib;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{self, Axis3, Vector2};
use std::f64::consts::PI;

/// How the p-curves of a face are moved onto the reference surface.
#[derive(Clone, Copy, Default)]
struct PCurveTransform {
    translation: f64,
    angle: f64,
    to_translate: bool,
    to_rotate: bool,
    x_reverse: bool,
    y_reverse: bool,
    to_modify: bool,
    to_project: bool,
}

/// The surface of the face without its rectangular trimming.
fn basis_of(face: &Face) -> Result<Handle<dyn GeomSurface>, Failure> {
    let surface = BRepTool::surface(face)?;
    Ok(
        match surface.cast::<GeomRectangularTrimmedSurface>().get() {
            Some(trimmed) => trimmed.basis_surface(),
            None => surface,
        },
    )
}

/// The transformation between two elementary surfaces from their frames.
fn elementary_transform(face_axis: Axis3, ref_axis: Axis3) -> Result<PCurveTransform, Failure> {
    let mut t = PCurveTransform::default();
    let parameter = elclib::line_parameter(face_axis.axis(), ref_axis.location());
    if parameter.abs() > precision::p_confusion() {
        t.translation = -parameter;
    }
    let cross1 = ref_axis.x_direction().crossed(ref_axis.y_direction())?;
    let cross2 = face_axis.x_direction().crossed(face_axis.y_direction())?;
    t.x_reverse = cross1.dot(cross2) < 0.;
    t.y_reverse = face_axis.direction().dot(ref_axis.direction()) < 0.;
    t.angle = if !t.x_reverse && !t.y_reverse {
        let mut reference = ref_axis.direction();
        if !ref_axis.direct()? {
            reference.reverse();
        }
        ref_axis
            .x_direction()
            .angle_with_ref(face_axis.x_direction(), reference)
    } else {
        ref_axis.x_direction().angle(&face_axis.x_direction())
    };
    t.to_rotate = t.angle.abs() > precision::p_confusion();
    t.to_translate = t.translation.abs() > precision::p_confusion();
    t.to_modify = t.to_translate || t.to_rotate || t.x_reverse || t.y_reverse;
    Ok(t)
}

fn transform_of(
    ref_surf: &Handle<dyn GeomSurface>,
    face_surf: &Handle<dyn GeomSurface>,
) -> Result<PCurveTransform, Failure> {
    let face_elementary = face_surf.cast::<dyn GeomElementarySurface>();
    let ref_elementary = ref_surf.cast::<dyn GeomElementarySurface>();
    if let (Some(face_e), Some(ref_e)) = (face_elementary.get(), ref_elementary.get()) {
        return elementary_transform(face_e.position(), ref_e.position());
    }
    let (r, f) = (
        ref_surf.get().ok_or_else(null_pcurve)?,
        face_surf.get().ok_or_else(null_pcurve)?,
    );
    Ok(PCurveTransform {
        to_project: !same_surf(r, f)?,
        ..Default::default()
    })
}

/// The p-curve of `edge` (as oriented) on `face`; `range` receives its parameters, as
/// the native output arguments are overwritten by every call.
fn pcurve_on(
    edge: &Edge,
    face: &Face,
    range: &mut (f64, f64),
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let found = BRepTool::curve_on_face(edge, face)?;
    *range = (found.first, found.last);
    Ok(found.pcurve)
}

/// The new p-curve `ii`: the projection of the 3D curve, or a copy of the p-curve on the
/// face, then translated, mirrored and rotated as `t` says.
fn new_pcurve(
    t: &PCurveTransform,
    edge: &Edge,
    pcurve: &Handle<dyn Geom2dCurve>,
    ref_surf: &Handle<dyn GeomSurface>,
    range: &mut (f64, f64),
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let curve = if t.to_project {
        let (c3d, first, last) = BRepTool::curve(edge)?;
        *range = (first, last);
        let trimmed: Handle<dyn GeomCurve> =
            Handle::new(GeomTrimmedCurve::new(&c3d, first, last, true, true)?).cast();
        geom_proj_lib::curve2d(&trimmed, ref_surf)?
    } else {
        pcurve
            .get()
            .ok_or_else(null_pcurve)?
            .copy()?
            .cast::<dyn Geom2dCurve>()
    };
    apply_transform(t, &curve)?;
    Ok(curve)
}

fn apply_transform(t: &PCurveTransform, curve: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
    if !(t.to_translate || t.y_reverse || t.x_reverse || t.to_rotate) {
        return Ok(());
    }
    let c = curve.get().ok_or_else(null_pcurve)?;
    if t.to_translate {
        c.translate(Vector2::new(0., t.translation))?;
    }
    if t.y_reverse {
        c.mirror_axis(gp::ox2d())?;
    }
    if t.x_reverse {
        c.mirror_axis(gp::oy2d())?;
        c.translate(Vector2::new(2. * PI, 0.))?;
    }
    if t.to_rotate {
        c.translate(Vector2::new(t.angle, 0.))?;
    }
    Ok(())
}

/// The p-curves of one edge of `face` and its unused second reference p-curve.
struct EdgePCurves {
    on_ref: Handle<dyn Geom2dCurve>,
    second_on_ref: Handle<dyn Geom2dCurve>,
    on_face: [Handle<dyn Geom2dCurve>; 2],
}

/// The p-curves of the edge on the reference face (None when it has two) and on the face
/// (None when one is missing).
fn edge_pcurves(
    edge: &Edge,
    face: &Face,
    ref_face: &Face,
    range: &mut (f64, f64),
) -> Result<Option<EdgePCurves>, Failure> {
    let mut edge = edge.clone();
    let on_ref = pcurve_on(&edge, ref_face, range)?;
    let mut second_on_ref = Handle::default();
    if !on_ref.is_null() {
        edge.reverse();
        second_on_ref = pcurve_on(&edge, ref_face, range)?;
        if second_on_ref != on_ref {
            // "two pcurves".
            return Ok(None);
        }
    }
    edge.set_orientation(Orientation::Forward);
    let forward = pcurve_on(&edge, face, range)?;
    edge.reverse();
    let reversed = pcurve_on(&edge, face, range)?;
    if forward.is_null() || reversed.is_null() {
        // "The source face has no pcurve for this edge, so it cannot be transformed."
        return Ok(None);
    }
    Ok(Some(EdgePCurves {
        on_ref,
        second_on_ref,
        on_face: [forward, reversed],
    }))
}

/// The periods of the closed directions of the surface (0 for the others).
fn closed_periods(surface: &dyn GeomSurface) -> Result<(f64, f64), Failure> {
    let (umin, umax, vmin, vmax) = surface.bounds();
    let u_period = if surface.is_u_closed()? {
        umax - umin
    } else {
        0.
    };
    let v_period = if surface.is_v_closed()? {
        vmax - vmin
    } else {
        0.
    };
    Ok((u_period, v_period))
}

/// The single new p-curve stored on the reference face: alone, or as the other p-curve
/// of a seam when it is a period away from the existing one.
fn store_single(
    edge: &Edge,
    (orientation, range): (Orientation, (f64, f64)),
    pcurves: &EdgePCurves,
    new: &Handle<dyn Geom2dCurve>,
    (ref_face, ref_surf): (&Face, &Handle<dyn GeomSurface>),
    temporary: &mut MapOfShape,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let surface = ref_surf.get().ok_or_else(null_pcurve)?;
    if pcurves.second_on_ref.is_null() || !(surface.is_u_closed()? || surface.is_v_closed()?) {
        builder.update_edge_pcurve_on_face(edge, new, ref_face, 0.)?;
        temporary.add(edge.clone().into_shape());
        return Ok(());
    }
    // "check: may be it is the same pcurve".
    let (u_period, v_period) = closed_periods(surface)?;
    let p1 = pcurves
        .on_ref
        .get()
        .ok_or_else(null_pcurve)?
        .value(range.0)?;
    let p2 = new.get().ok_or_else(null_pcurve)?.value(range.0)?;
    let u_far = u_period != 0. && (p1.x() - p2.x()).abs() > u_period / 2.;
    let v_far = v_period != 0. && (p1.y() - p2.y()).abs() > v_period / 2.;
    if !(u_far || v_far) {
        return Ok(());
    }
    builder.update_edge_pcurve_on_face(edge, &Handle::default(), ref_face, 0.)?;
    let pair = if orientation == Orientation::Forward {
        (new, &pcurves.on_ref)
    } else {
        (&pcurves.on_ref, new)
    };
    builder.update_edge_pcurves_on_face(edge, pair, ref_face, 0.)?;
    temporary.add(edge.clone().into_shape());
    Ok(())
}

/// `TransformPCurves(theRefFace, theFace, theMapEdgesWithTemporaryPCurves)`: each edge of
/// the face without a p-curve pair on the reference face gets the p-curves of the face
/// moved onto the reference surface (or the projection of its 3D curve when the surfaces
/// differ); the edges updated are recorded as having temporary p-curves.
pub(super) fn transform_pcurves(
    ref_face: &Face,
    face: &Face,
    temporary: &mut MapOfShape,
) -> Result<(), Failure> {
    let ref_surf = basis_of(ref_face)?;
    let t = transform_of(&ref_surf, &basis_of(face)?)?;
    let mut visited = MapOfShape::default();
    for shape in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edge = topods::edge(&shape)?;
        if BRepTool::degenerated(&edge)? && t.to_modify {
            continue;
        }
        if t.to_project && BRepTool::is_closed_on_face(&edge, face)? {
            continue;
        }
        if !visited.add(shape.clone()) {
            continue;
        }
        transform_edge(&t, edge, (face, ref_face, &ref_surf), temporary)?;
    }
    Ok(())
}

fn transform_edge(
    t: &PCurveTransform,
    edge: Edge,
    (face, ref_face, ref_surf): (&Face, &Face, &Handle<dyn GeomSurface>),
    temporary: &mut MapOfShape,
) -> Result<(), Failure> {
    let orientation = edge.orientation();
    let mut range = (0., 0.);
    let Some(pcurves) = edge_pcurves(&edge, face, ref_face, &mut range)? else {
        return Ok(());
    };
    let mut forward = edge.clone();
    forward.set_orientation(Orientation::Forward);
    let count = if pcurves.on_face[0] == pcurves.on_face[1] {
        1
    } else {
        2
    };
    let mut new = Vec::with_capacity(count);
    for pcurve in &pcurves.on_face[..count] {
        new.push(new_pcurve(t, &forward, pcurve, ref_surf, &mut range)?);
    }
    if count == 1 {
        store_single(
            &forward,
            (orientation, range),
            &pcurves,
            &new[0],
            (ref_face, ref_surf),
            temporary,
        )?;
    } else {
        BRepBuilder::default().update_edge_pcurves_on_face(
            &forward,
            (&new[0], &new[1]),
            ref_face,
            0.,
        )?;
        temporary.add(forward.clone().into_shape());
    }
    BRepBuilder::default().range(&forward, range.0, range.1, false)
}

/// `AddPCurves(theFaces, theRefFace, theMapEdgesWithTemporaryPCurves)`: on a non-planar
/// reference face, `TransformPCurves` for every other face (taken forward).
pub(super) fn add_pcurves(
    faces: &[Shape],
    ref_face: &Face,
    temporary: &mut MapOfShape,
) -> Result<(), Failure> {
    let adaptor = BRepAdaptorSurface::with_face(ref_face, false)?;
    if adaptor.get_type()? == SurfaceType::Plane {
        return Ok(());
    }
    for shape in faces {
        let face = topods::face(&shape.oriented(Orientation::Forward))?;
        if face.is_same(ref_face) {
            continue;
        }
        transform_pcurves(ref_face, &face, temporary)?;
    }
    Ok(())
}
