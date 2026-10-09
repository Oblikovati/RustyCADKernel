// Rust translation of the static EdgeToBSpline and totalsurf
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_NSections.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The surface skinned through the sections of BRepFill_NSections.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkernel::ncollection::{Array1, Array2, HArray1};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg2d::geom2d::arrays::{at, one_based, put};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBSplineSurface, GeomBoundedCurve, GeomCurve, GeomGeometry,
    GeomTrimmedCurve,
};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::app_blend::AppBlendApprox;
use rustycad_tkgeomalgo::geom_fill::{GeomFillAppSurf, GeomFillLine, GeomFillSectionGenerator};
use rustycad_tkgeombase::geom_convert::{
    ApproxCurve, CompCurveToBSplineCurve, curve_to_bspline_curve,
};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point3;

/// The degree 1 B-spline from `p1` to `p2` on `[0, 1]` (the curve of a point section).
pub(super) fn point_curve(p1: Point3, p2: Point3) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let mut poles = one_based::<Point3>(2)?;
    put(&mut poles, 1, p1)?;
    put(&mut poles, 2, p2)?;
    let mut knots = one_based::<f64>(2)?;
    put(&mut knots, 1, 0.)?;
    put(&mut knots, 2, 1.)?;
    let mut mults = one_based::<i32>(2)?;
    mults.init(2);
    Ok(Handle::new(GeomBSplineCurve::new(
        &poles, &knots, &mults, 1, false,
    )?))
}

/// The degree 1 curve between the vertices of a degenerated edge, from its last
/// vertex to its first (`TopExp::Vertices(edge, vl, vf)` natively).
pub(super) fn degenerated_curve(edge: &Edge) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let (vl, vf) = topexp::vertices(edge, false);
    point_curve(BRepTool::pnt(&vf)?, BRepTool::pnt(&vl)?)
}

/// `EdgeToBSpline`: the curve of the edge on its range as a B-spline (approximated C1
/// within Precision::Confusion(), converted otherwise), located, reparametrized on
/// `[0, 1]` and reversed for a REVERSED edge.
pub(super) fn edge_to_bspline(edge: &Edge) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let bs: Handle<GeomBSplineCurve> = if BRepTool::degenerated(edge)? {
        degenerated_curve(edge)?
    } else {
        located_bspline(edge)?
    };
    if edge.orientation() == Orientation::Reversed {
        live(&bs)?.reverse()?;
    }
    Ok(bs)
}

/// The non-degenerated branch of `EdgeToBSpline`.
fn located_bspline(edge: &Edge) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let found = BRepTool::curve_with_location(edge)?;
    let trim: Handle<dyn GeomCurve> = Handle::new(GeomTrimmedCurve::new(
        &found.curve,
        found.first,
        found.last,
        true,
        true,
    )?)
    .cast();
    let approx = ApproxCurve::from_curve(&trim, precision::confusion(), Continuity::C1, 16, 14)?;
    let mut bs: Handle<GeomBSplineCurve> = Handle::default();
    if approx.has_result() {
        bs = approx.curve();
    }
    if bs.is_null() {
        bs = curve_to_bspline_curve(&trim, ParameterisationType::TgtThetaOver2)?;
    }
    let curve = live(&bs)?;
    if !found.location.is_identity() {
        curve.transform(found.location.transformation())?;
    }
    let mut knots: Array1<'static, f64> = curve.knots();
    bsplclib::reparametrize(0., 1., &mut knots)?;
    curve.set_knots(&knots)?;
    Ok(bs)
}

/// The curve of a live handle.
fn live<T: ?Sized + RuntimeTyped>(handle: &Handle<T>) -> Result<&T, Failure> {
    handle.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "BRepFill_NSections: null curve; expected a curve",
            "BRepFill_NSections",
        )
    })
}

/// The sections and flags of `totalsurf`.
pub(super) struct SkinInput<'a> {
    pub(super) shapes: &'a Array2<'static, Shape>,
    pub(super) nb_sects: i32,
    pub(super) nb_edges: i32,
    pub(super) params: &'a [f64],
    pub(super) w1_point: bool,
    pub(super) w2_point: bool,
    pub(super) u_closed: bool,
    pub(super) v_closed: bool,
}

/// The edge of the table at `(i, j)`.
fn edge_at(shapes: &Array2<'static, Shape>, i: i32, j: i32) -> Result<Edge, Failure> {
    let shape = shapes.value(i, j).map_err(|e| {
        Failure::with_type(
            "Standard_OutOfRange",
            &format!("BRepFill_NSections: edge ({i}, {j}): {e}; expected a table index"),
            "BRepFill_NSections",
        )
    })?;
    topods::edge(shape)
}

/// The B-spline of section `j`: its edges concatenated (with the tolerance of the
/// common vertex, then 200 times it).
fn section_bspline(
    shapes: &Array2<'static, Shape>,
    nb_edges: i32,
    j: i32,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let mut prev_edge: Edge = edge_at(shapes, 1, j)?;
    let first = edge_to_bspline(&prev_edge)?.cast::<dyn GeomBoundedCurve>();
    let mut comp =
        CompCurveToBSplineCurve::with_curve(&first, ParameterisationType::TgtThetaOver2)?;
    for i in 2..=nb_edges {
        let next_edge: Edge = edge_at(shapes, i, j)?;
        let curv_bs = edge_to_bspline(&next_edge)?.cast::<dyn GeomBoundedCurve>();
        let eps_v: f64 = match topexp::common_vertex(&prev_edge, &next_edge) {
            Some(v) => BRepTool::vertex_tolerance(&v)?,
            None => precision::confusion(),
        };
        if !comp.add(&curv_bs, eps_v, true, false, 1)? {
            comp.add(&curv_bs, 200. * eps_v, true, false, 1)?;
        }
        prev_edge = next_edge;
    }
    Ok(comp.bspline_curve())
}

/// The section generator of `totalsurf`: the point sections, then the concatenated
/// sections (the first one again for looping sections).
fn sections_generator(input: &SkinInput) -> Result<GeomFillSectionGenerator, Failure> {
    let mut section = GeomFillSectionGenerator::new();
    let (mut jdeb, mut jfin) = (1, input.nb_sects);
    if input.w1_point {
        jdeb += 1;
        section.add_curve(&degenerated_curve(&edge_at(input.shapes, 1, 1)?)?.cast())?;
    }
    if input.w2_point {
        jfin -= 1;
    }
    let mut bs1: Handle<GeomBSplineCurve> = Handle::default();
    for j in jdeb..=jfin {
        if j == jfin && input.v_closed {
            section.add_curve(&bs1.cast())?;
            continue;
        }
        let bs = section_bspline(input.shapes, input.nb_edges, j)?;
        section.add_curve(&bs.cast())?;
        if j == jdeb && input.v_closed {
            bs1 = bs;
        }
    }
    if input.w2_point {
        section.add_curve(
            &degenerated_curve(&edge_at(input.shapes, input.nb_edges, input.nb_sects)?)?.cast(),
        )?;
    }
    let mut hpar = one_based::<f64>(input.params.len() as i32)?;
    hpar.as_mut_slice().copy_from_slice(input.params);
    section.set_param(&Handle::new(HArray1::from_owned(hpar)))?;
    section.perform(precision::p_confusion())?;
    Ok(section)
}

/// `totalsurf`: the surface skinned through the sections by GeomFill_AppSurf
/// (degrees 2 to 6, known parameters), made periodic in U or V for closed sections or
/// looping sections, and non-rational when its weights are equal within 1e-13.
pub(super) fn totalsurf(
    input: &SkinInput,
    pres3d: f64,
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    let mut section = sections_generator(input)?;
    let line = GeomFillLine::with_nb_points(input.nb_sects);
    let mut approx = GeomFillAppSurf::new(2, 6, pres3d, pres3d, 0, true);
    approx.perform(Some(&line), &mut section, true)?;
    let mut nup: i32 = approx.surf_poles()?.upper_row();
    let mut nvp: i32 = approx.surf_poles()?.upper_col();
    let mut umults: Array1<'static, i32> = approx.surf_u_mults()?.clone();
    let mut vmults: Array1<'static, i32> = approx.surf_v_mults()?.clone();
    if input.u_closed {
        lower_end_mults(&mut umults)?;
        nup -= 1;
    }
    if input.v_closed {
        lower_end_mults(&mut vmults)?;
        nvp -= 1;
    }
    let (poles, mut weights) = trimmed_grids(&approx, nup, nvp)?;
    uniform_weights_when_equal(&mut weights)?;
    let surface = GeomBSplineSurface::with_weights(
        &poles,
        &weights,
        (approx.surf_u_knots()?, &umults),
        (approx.surf_v_knots()?, &vmults),
        (approx.u_degree()?, approx.v_degree()?),
        (input.u_closed, input.v_closed),
    )?;
    Ok(Handle::new(surface))
}

/// The first and last multiplicities lowered by one (a periodic surface).
fn lower_end_mults(mults: &mut Array1<'static, i32>) -> Result<(), Failure> {
    let last: i32 = mults.upper();
    put(mults, 1, at(mults, 1)? - 1)?;
    put(mults, last, at(mults, last)? - 1)
}

/// The poles and weights of the approximation in `1..=nup x 1..=nvp`.
fn trimmed_grids(
    approx: &GeomFillAppSurf,
    nup: i32,
    nvp: i32,
) -> Result<(Array2<'static, Point3>, Array2<'static, f64>), Failure> {
    let grid_error = |e| {
        Failure::with_type(
            "Standard_OutOfRange",
            &format!("BRepFill_NSections totalsurf: {e}; expected pole grids"),
            "BRepFill_NSections",
        )
    };
    let mut poles: Array2<'static, Point3> =
        Array2::with_bounds(1, nup, 1, nvp).map_err(grid_error)?;
    let mut weights: Array2<'static, f64> =
        Array2::with_bounds(1, nup, 1, nvp).map_err(grid_error)?;
    for j in 1..=nvp {
        for i in 1..=nup {
            *poles.change_value(i, j).map_err(grid_error)? =
                *approx.surf_poles()?.value(i, j).map_err(grid_error)?;
            *weights.change_value(i, j).map_err(grid_error)? =
                *approx.surf_weights()?.value(i, j).map_err(grid_error)?;
        }
    }
    Ok((poles, weights))
}

/// The weights set to the first one when they are equal within 1e-13 along both
/// directions (to create a non-rational surface if possible).
fn uniform_weights_when_equal(weights: &mut Array2<'static, f64>) -> Result<(), Failure> {
    const TOL_EPS: f64 = 1.0e-13;
    let w = |weights: &Array2<'static, f64>, i: i32, j: i32| -> f64 {
        weights.value(i, j).copied().unwrap_or(0.)
    };
    let (rows, cols) = (weights.upper_row(), weights.upper_col());
    let v_rational: bool = (1..=cols)
        .any(|j| (1..rows).any(|i| (w(weights, i, j) - w(weights, i + 1, j)).abs() > TOL_EPS));
    let u_rational: bool = (1..=rows)
        .any(|i| (1..cols).any(|j| (w(weights, i, j) - w(weights, i, j + 1)).abs() > TOL_EPS));
    if v_rational || u_rational {
        return Ok(());
    }
    let the_weight: f64 = w(weights, 1, 1);
    for i in 1..=rows {
        for j in 1..=cols {
            if let Ok(value) = weights.change_value(i, j) {
                *value = the_weight;
            }
        }
    }
    Ok(())
}
