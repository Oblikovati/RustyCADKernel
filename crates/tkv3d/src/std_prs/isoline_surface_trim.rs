//! Sample real oriented p-curves before hatch trimming; preserve OCC13144 finite-window origin policy.
use super::StdPrsIsolineSurfaceSource;
use super::StdPrsToolRFace;
use super::isoline_surface_reference::IsolineSurfaceReference;
use rustycad_tkernel::{precision, standard::Failure, standard::real::real_last};
use rustycad_tkg2d::{
    adaptor2d::Adaptor2dCurve2d,
    geom2d::native::{std_max, std_min},
};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::gcpnts::QuasiUniformDeflection;
use rustycad_tkmath::{
    bnd::Range1,
    geom_abs::{CurveType, Shape},
    gp::Point2,
};
pub(super) struct IsolineTrimRegion {
    pub points: Vec<Point2>,
    pub ranges: [Range1; 2],
    pub tolerance: f64,
}
pub(super) fn surface_isoline_trimming<
    S: StdPrsIsolineSurfaceSource + rustycad_tkernel::standard::RuntimeTyped + ?Sized,
>(
    surface: &IsolineSurfaceReference<'_, S>,
    limit: f64,
    coefficient: f64,
) -> Result<IsolineTrimRegion, Failure> {
    let deflection = pcurve_sampler_deflection(surface, limit, coefficient)?;
    let tool = StdPrsToolRFace::with_surface(&surface.snapshot())?;
    let (points, mut tolerance) = sampled_isoline_trim_edges(&tool, limit, deflection)?;
    let ranges = isoline_trim_ranges(&points, deflection);
    tolerance = std_min(1e-5, std_max(precision::confusion(), tolerance * 0.1));
    Ok(IsolineTrimRegion {
        points,
        ranges,
        tolerance,
    })
}
fn pcurve_sampler_deflection<
    S: StdPrsIsolineSurfaceSource + rustycad_tkernel::standard::RuntimeTyped + ?Sized,
>(
    surface: &IsolineSurfaceReference<'_, S>,
    limit: f64,
    coefficient: f64,
) -> Result<f64, Failure> {
    let u0 = std_max(surface.snapshot().first_u_parameter()?, -limit);
    let u1 = std_min(surface.snapshot().last_u_parameter()?, limit);
    let v0 = std_max(surface.snapshot().first_v_parameter()?, -limit);
    let v1 = std_min(surface.snapshot().last_v_parameter()?, limit);
    Ok(std_max(u1 - u0, v1 - v0) * coefficient)
}
fn append_isoline_trim_edge(
    curve: &dyn Adaptor2dCurve2d,
    orientation: Orientation,
    limit: f64,
    deflection: f64,
    points: &mut Vec<Point2>,
    tolerance: &mut f64,
) -> Result<(), Failure> {
    if curve.get_type()? == CurveType::Line {
        let bounds = linear_isoline_trim_bounds(curve, limit)?;
        append_oriented_trim_pair(
            [curve.value(bounds[0])?, curve.value(bounds[1])?],
            orientation,
            points,
            tolerance,
        );
        return Ok(());
    }
    append_curved_isoline_trim_edge(curve, orientation, deflection, points, tolerance)
}
fn append_curved_isoline_trim_edge(
    curve: &dyn Adaptor2dCurve2d,
    orientation: Orientation,
    deflection: f64,
    points: &mut Vec<Point2>,
    tolerance: &mut f64,
) -> Result<(), Failure> {
    let sampler = QuasiUniformDeflection::with_curve(curve, deflection, Shape::C1)?;
    if !sampler.is_done() {
        return Ok(());
    }
    for index in 1..sampler.nb_points()? {
        append_sampled_isoline_trim_pair(&sampler, index, orientation, points, tolerance)?;
    }
    Ok(())
}
fn linear_isoline_trim_bounds(
    curve: &dyn Adaptor2dCurve2d,
    limit: f64,
) -> Result<[f64; 2], Failure> {
    let first = curve.first_parameter()?;
    let last = curve.last_parameter()?;
    let origin = match (
        precision::is_negative_infinite(first),
        precision::is_positive_infinite(last),
    ) {
        (true, true) => 0.,
        (true, false) => last - limit,
        (false, true) => first + limit,
        (false, false) => (first + last) * 0.5,
    };
    Ok([
        std_max(origin - limit, first),
        std_min(origin + limit, last),
    ])
}
fn append_oriented_trim_pair(
    mut pair: [Point2; 2],
    orientation: Orientation,
    points: &mut Vec<Point2>,
    tolerance: &mut f64,
) {
    *tolerance = std_min(pair[0].square_distance(pair[1]), *tolerance);
    if orientation != Orientation::Forward {
        pair.swap(0, 1);
    }
    points.extend_from_slice(&pair);
}
fn isoline_trim_ranges(points: &[Point2], deflection: f64) -> [Range1; 2] {
    let mut ranges = [Range1::default(), Range1::default()];
    for point in points {
        ranges[0].add(point.x());
        ranges[1].add(point.y());
    }
    for range in &mut ranges {
        if !range.is_void() && range.delta() <= deflection {
            range.set_void();
        }
    }
    ranges
}

fn sampled_isoline_trim_edges(
    tool: &StdPrsToolRFace,
    limit: f64,
    deflection: f64,
) -> Result<(Vec<Point2>, f64), Failure> {
    let (mut points, mut tolerance) = (Vec::new(), real_last());
    tool.init()?;
    while tool.more() {
        append_isoline_trim_edge(
            tool.value(),
            tool.orientation(),
            limit,
            deflection,
            &mut points,
            &mut tolerance,
        )?;
        tool.next()?;
    }
    Ok((points, tolerance))
}

fn append_sampled_isoline_trim_pair(
    sampler: &QuasiUniformDeflection,
    index: i32,
    orientation: Orientation,
    points: &mut Vec<Point2>,
    tolerance: &mut f64,
) -> Result<(), Failure> {
    let (a, b) = (sampler.value(index)?, sampler.value(index + 1)?);
    let pair = [Point2::new(a.x(), a.y()), Point2::new(b.x(), b.y())];
    append_oriented_trim_pair(pair, orientation, points, tolerance);
    Ok(())
}
