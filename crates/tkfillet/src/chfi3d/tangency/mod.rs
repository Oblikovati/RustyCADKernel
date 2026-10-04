//! ChFi3d.cxx::IsTangentFaces at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
mod geometry;
use geometry::TangencyGeometry;
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_adaptor::BRepAdaptorSurface,
    topods::{Edge, Face},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkgeomalgo::local_analysis::{
    LocalAnalysisStatusErrorType, LocalAnalysisSurfaceContinuity, SurfaceContinuityTolerances,
};
use rustycad_tkmath::{geom_abs::Shape, gp::Direction3};
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorTopolTool;

/// Check sampled surface continuity and oriented middle normals; `is_tangent_faces(&edge,&first,&second,Shape::G1)?`.
pub fn is_tangent_faces(
    edge: &Edge,
    first: &Face,
    second: &Face,
    order: Shape,
) -> Result<bool, Failure> {
    if order == Shape::G1 && BRepTool::continuity_on_faces(edge, first, second)? != Shape::C0 {
        return Ok(true);
    }
    let tolerance = (1.5 * BRepTool::edge_tolerance(edge)?).max(0.001);
    let Some(geometry) = TangencyGeometry::load(edge, first, second)? else {
        return Ok(false);
    };
    let samples = sample_count(first)?.max(sample_count(second)?);
    if !sample_continuity(&geometry, order, tolerance, samples)? {
        return Ok(false);
    }
    compatible_normals(&geometry, first, second)
}
fn sample_count(face: &Face) -> Result<i32, Failure> {
    let surface: Handle<dyn Adaptor3dSurface> =
        Handle::new(BRepAdaptorSurface::with_face(face, true)?).cast();
    BRepTopAdaptorTopolTool::with_surface(&surface)?.nb_samples()
}
fn sample_continuity(
    geometry: &TangencyGeometry,
    order: Shape,
    tolerance: f64,
    samples: i32,
) -> Result<bool, Failure> {
    let delta = (geometry.last - geometry.first) / f64::from(samples - 1);
    let mut parameter = geometry.first;
    let mut not_done = 0;
    for index in 1..=samples {
        if index == samples {
            parameter = geometry.last;
        }
        match continuity_sample(geometry, parameter, order, tolerance)? {
            None => not_done += 1,
            Some(false) => return Ok(false),
            Some(true) => (),
        }
        parameter += delta;
    }
    Ok(not_done != samples)
}
fn continuity_sample(
    geometry: &TangencyGeometry,
    parameter: f64,
    order: Shape,
    tolerance: f64,
) -> Result<Option<bool>, Failure> {
    let tolerances = continuity_tolerances(tolerance);
    let analysis = LocalAnalysisSurfaceContinuity::on_curves(
        (&geometry.first_curve, &geometry.second_curve),
        parameter,
        &geometry.first_surface,
        &geometry.second_surface,
        order,
        &tolerances,
    )?;
    if analysis.is_done() {
        return sample_satisfies(&analysis, order).map(Some);
    }
    // OCCT accepts this C2 failure without counting it toward the all-failed case.
    Ok((order == Shape::C2
        && analysis.status_error() == Some(LocalAnalysisStatusErrorType::NullSecondDerivative))
    .then_some(true))
}
fn continuity_tolerances(tolerance: f64) -> SurfaceContinuityTolerances {
    SurfaceContinuityTolerances {
        eps_c0: tolerance,
        eps_c1: 0.1,
        eps_c2: 0.1,
        eps_g1: 0.1,
        ..Default::default()
    }
}
fn sample_satisfies(
    analysis: &LocalAnalysisSurfaceContinuity,
    order: Shape,
) -> Result<bool, Failure> {
    if order == Shape::G1 {
        analysis.is_g1()
    } else {
        analysis.is_g2()
    }
}
fn compatible_normals(
    geometry: &TangencyGeometry,
    first: &Face,
    second: &Face,
) -> Result<bool, Failure> {
    let parameter = (geometry.first + geometry.last) / 2.;
    let mut first_normal = Direction3::default();
    let mut second_normal = Direction3::default();
    // OCCT deliberately ignores Nt's bool result and retains a default direction on false.
    rustycad_tkbool::top_ope_brep_tool::nt(
        geometry.first_curve.value(parameter)?,
        first,
        &mut first_normal,
    )?;
    rustycad_tkbool::top_ope_brep_tool::nt(
        geometry.second_curve.value(parameter)?,
        second,
        &mut second_normal,
    )?;
    Ok(first_normal.dot(second_normal) >= 0.)
}
