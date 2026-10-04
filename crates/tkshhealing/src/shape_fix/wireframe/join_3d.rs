use crate::{shape_analysis::ShapeAnalysisEdge, shape_construct::ShapeConstructCurve};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Builder, Edge, Face},
};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBoundedCurve, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBoundedCurve, GeomCurve};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::geom_convert::CompCurveToBSplineCurve;
use rustycad_tkgeombase::geom2d_convert::CompCurveToBSplineCurve as CompCurve2d;
use rustycad_tkmath::{convert::ParameterisationType, gp::Point3};

pub(super) struct JoinedEdge3d {
    pub edge: Edge,
    pub replace_first: bool,
}

type JoinedCurve3d = (Handle<GeomBSplineCurve>, (f64, f64), bool);
type JoinedCurve2d = (Handle<Geom2dBSplineCurve>, (f64, f64));

pub(super) fn join_edges_3d(
    first: &Edge,
    second: &Edge,
    faces: &[Face],
) -> Result<Option<JoinedEdge3d>, Failure> {
    if has_seam(first, second, faces)? {
        return Ok(None);
    }
    let Some((curve, range, replace_first)) = joined_curve(first, second)? else {
        return Ok(None);
    };
    let mut edge = make_joined_edge(first, second, &curve, range)?;
    if !append_face_pcurves(&mut edge, first, second, faces)? {
        return Ok(None);
    }
    Ok(Some(JoinedEdge3d {
        edge,
        replace_first,
    }))
}

fn has_seam(first: &Edge, second: &Edge, faces: &[Face]) -> Result<bool, Failure> {
    let analysis = ShapeAnalysisEdge::new();
    for face in faces {
        if analysis.is_seam_on_face(first, face)? || analysis.is_seam_on_face(second, face)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn append_face_pcurves(
    joined: &mut Edge,
    first: &Edge,
    second: &Edge,
    faces: &[Face],
) -> Result<bool, Failure> {
    for face in faces {
        if !append_face_pcurve(joined, first, second, face)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn append_face_pcurve(
    joined: &Edge,
    first: &Edge,
    second: &Edge,
    face: &Face,
) -> Result<bool, Failure> {
    let Some((curve, range)) = joined_pcurve(first, second, face)? else {
        return Ok(false);
    };
    install_face_pcurve(joined, face, &curve, range)?;
    Ok(true)
}

fn joined_pcurve(
    first: &Edge,
    second: &Edge,
    face: &Face,
) -> Result<Option<JoinedCurve2d>, Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let first_curve = analysis.pcurve_on_face(first, face, false)?;
    let second_curve = analysis.pcurve_on_face(second, face, false)?;
    if first_curve.0.is_null() || second_curve.0.is_null() {
        return Ok(None);
    }
    let first_curve = oriented_pcurve(first, first_curve)?;
    let second_curve = oriented_pcurve(second, second_curve)?;
    compose_pcurves(first_curve, second_curve)
}

fn oriented_pcurve(
    edge: &Edge,
    (curve, mut first, mut last): (Handle<dyn Geom2dCurve>, f64, f64),
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let copy: Handle<dyn Geom2dCurve> = curve.copy()?.cast();
    if edge.orientation() == Orientation::Reversed {
        (first, last) = reversed_pcurve_range(&curve, first, last)?;
        copy.reverse()?;
    }
    ShapeConstructCurve.convert_to_bspline_2d(&copy, first, last, precision::confusion())
}

fn reversed_pcurve_range(
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<(f64, f64), Failure> {
    Ok((
        curve.reversed_parameter(last)?,
        curve.reversed_parameter(first)?,
    ))
}

fn compose_pcurves(
    first: Handle<Geom2dBSplineCurve>,
    second: Handle<Geom2dBSplineCurve>,
) -> Result<Option<JoinedCurve2d>, Failure> {
    let ranges = (
        (first.first_parameter(), first.last_parameter()),
        (second.first_parameter(), second.last_parameter()),
    );
    let curve = append_pcurve_bspline(&first, &second)?;
    Ok(curve.map(|curve| (curve, composed_pcurve_range(ranges.0, ranges.1))))
}

fn append_pcurve_bspline(
    first: &Handle<Geom2dBSplineCurve>,
    second: &Handle<Geom2dBSplineCurve>,
) -> Result<Option<Handle<Geom2dBSplineCurve>>, Failure> {
    let bounded = first.cast::<dyn Geom2dBoundedCurve>();
    let mut builder = CompCurve2d::with_curve(&bounded, ParameterisationType::TgtThetaOver2)?;
    let added = builder.add(
        &second.cast::<dyn Geom2dBoundedCurve>(),
        precision::confusion(),
        true,
    )?;
    if !added {
        return Ok(None);
    }
    Ok(Some(builder.bspline_curve()))
}

fn composed_pcurve_range(first: (f64, f64), second: (f64, f64)) -> (f64, f64) {
    (first.0, first.1 + second.1 - second.0)
}

fn install_face_pcurve(
    edge: &Edge,
    face: &Face,
    curve: &Handle<Geom2dBSplineCurve>,
    range: (f64, f64),
) -> Result<(), Failure> {
    let curve: Handle<dyn Geom2dCurve> = curve.clone().cast();
    let builder = BRepBuilder::default();
    builder.update_edge_pcurve_on_face(edge, &curve, face, 0.)?;
    builder.range_on_face(edge, face, range.0, range.1)?;
    builder.same_range(edge, false)
}

fn joined_curve(first: &Edge, second: &Edge) -> Result<Option<JoinedCurve3d>, Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let first_curve = analysis.curve_3d(first, false)?;
    let second_curve = analysis.curve_3d(second, false)?;
    if first_curve.0.is_null() || second_curve.0.is_null() {
        return Ok(None);
    }
    let first_curve = oriented_bspline(first, first_curve)?;
    let second_curve = oriented_bspline(second, second_curve)?;
    compose_curves(first_curve, second_curve)
}

fn oriented_bspline(
    edge: &Edge,
    (curve, mut first, mut last): (Handle<dyn GeomCurve>, f64, f64),
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let copy: Handle<dyn GeomCurve> = curve.copy()?.cast();
    if edge.orientation() == Orientation::Reversed {
        (first, last) = reversed_range(&curve, first, last)?;
        copy.reverse()?;
    }
    ShapeConstructCurve.convert_to_bspline(&copy, first, last, precision::confusion())
}

fn reversed_range(
    curve: &Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
) -> Result<(f64, f64), Failure> {
    Ok((
        curve.reversed_parameter(last)?,
        curve.reversed_parameter(first)?,
    ))
}

fn compose_curves(
    first: Handle<GeomBSplineCurve>,
    second: Handle<GeomBSplineCurve>,
) -> Result<Option<JoinedCurve3d>, Failure> {
    let reversals = endpoint_reversals(&first, &second)?;
    reverse_bspline_if(&first, reversals.0)?;
    reverse_bspline_if(&second, reversals.1)?;
    join_bspline_pair(first, second, reversals.0)
}

fn endpoint_reversals(
    first: &GeomBSplineCurve,
    second: &GeomBSplineCurve,
) -> Result<(bool, bool), Failure> {
    let p11 = first.pole(1)?;
    let p12 = first.pole(first.nb_poles())?;
    let p21 = second.pole(1)?;
    let p22 = second.pole(second.nb_poles())?;
    nearest_endpoint_pair(p11, p12, p21, p22)
}

fn nearest_endpoint_pair(
    p11: Point3,
    p12: Point3,
    p21: Point3,
    p22: Point3,
) -> Result<(bool, bool), Failure> {
    let d11 = p11.distance(&p21);
    let d21 = p12.distance(&p21);
    let d12 = p11.distance(&p22);
    let d22 = p22.distance(&p12);
    let min1 = d11.min(d21);
    let min2 = d12.min(d22);
    if (min1 - min2).abs() <= precision::confusion() || min2 > min1 {
        return Ok((d11 < d21, false));
    }
    Ok((d12 < d22, true))
}

fn reverse_bspline_if(curve: &GeomBSplineCurve, reverse: bool) -> Result<(), Failure> {
    if reverse {
        curve.reverse()?;
    }
    Ok(())
}

fn join_bspline_pair(
    first: Handle<GeomBSplineCurve>,
    second: Handle<GeomBSplineCurve>,
    reverse_first: bool,
) -> Result<Option<JoinedCurve3d>, Failure> {
    let first_last = first.nb_poles();
    let midpoint = pole_midpoint(first.pole(first_last)?, second.pole(1)?);
    first.set_pole(first_last, midpoint)?;
    second.set_pole(1, midpoint)?;
    let first_range = (first.first_parameter(), first.last_parameter());
    let second_range = (second.first_parameter(), second.last_parameter());
    let Some(curve) = append_bspline(&first, &second)? else {
        return Ok(None);
    };
    let last = first_range.1 + second_range.1 - second_range.0;
    Ok(Some((curve, (first_range.0, last), !reverse_first)))
}

fn append_bspline(
    first: &Handle<GeomBSplineCurve>,
    second: &Handle<GeomBSplineCurve>,
) -> Result<Option<Handle<GeomBSplineCurve>>, Failure> {
    let mut builder = CompCurveToBSplineCurve::with_curve(
        &first.cast::<dyn GeomBoundedCurve>(),
        ParameterisationType::TgtThetaOver2,
    )?;
    let added = builder.add(
        &second.cast::<dyn GeomBoundedCurve>(),
        precision::confusion(),
        true,
        false,
        0,
    )?;
    if !added {
        return Ok(None);
    }
    Ok(Some(builder.bspline_curve()))
}

fn pole_midpoint(first: Point3, second: Point3) -> Point3 {
    Point3::new(
        0.5 * (first.x() + second.x()),
        0.5 * (first.y() + second.y()),
        0.5 * (first.z() + second.z()),
    )
}

fn make_joined_edge(
    first: &Edge,
    second: &Edge,
    curve: &Handle<GeomBSplineCurve>,
    range: (f64, f64),
) -> Result<Edge, Failure> {
    let curve: Handle<dyn GeomCurve> = curve.clone().cast();
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge_with_curve(&mut edge, &curve, tolerance(first, second)?)?;
    let bounded_range = (
        range.0.max(curve.first_parameter()),
        range.1.min(curve.last_parameter()),
    );
    builder.range(&edge, bounded_range.0, bounded_range.1, true)?;
    add_joined_endpoints(&mut edge, first, second)?;
    edge.set_orientation(Orientation::Forward);
    builder.same_range(&edge, false)?;
    Ok(edge)
}

fn tolerance(first: &Edge, second: &Edge) -> Result<f64, Failure> {
    Ok(BRepTool::edge_tolerance(first)?.max(BRepTool::edge_tolerance(second)?))
}

fn add_joined_endpoints(edge: &mut Edge, first: &Edge, second: &Edge) -> Result<(), Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let endpoints = (analysis.first_vertex(first), analysis.last_vertex(second));
    let mut shape = edge.clone().into_shape();
    Builder.add(&mut shape, &endpoints.0.oriented(Orientation::Forward))?;
    Builder.add(&mut shape, &endpoints.1.oriented(Orientation::Reversed))?;
    *edge = rustycad_tkbrep::topods::edge(&shape)?;
    Ok(())
}
