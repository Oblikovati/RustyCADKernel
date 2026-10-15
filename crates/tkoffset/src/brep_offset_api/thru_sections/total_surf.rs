// Rust translation of the static EdgeToBSpline and of BRepOffsetAPI_ThruSections::TotalSurf
// (ModelingAlgorithms/TKOffset/BRepOffsetAPI/BRepOffsetAPI_ThruSections.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The surface of the smoothed loft: each section concatenated into one B-spline and
//! the sections skinned by `GeomFill_AppSurf`.
use super::BRepOffsetAPIThruSections;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::{one_based, put};
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

/// The edges of the sections, `nb_edges` per section, and the shape of the loft.
pub(super) struct SectionEdges<'a> {
    pub(super) shapes: &'a [Shape],
    pub(super) nb_sects: usize,
    pub(super) nb_edges: usize,
    pub(super) w1_point: bool,
    pub(super) w2_point: bool,
    pub(super) v_closed: bool,
}

impl SectionEdges<'_> {
    /// The edge at the 1-based index of the native `shapes` array.
    pub(super) fn edge(&self, index: usize) -> Result<Edge, Failure> {
        let shape = self.shapes.get(index.wrapping_sub(1)).ok_or_else(|| {
            Failure::with_message(
                "Standard_OutOfRange",
                &format!(
                    "ThruSections: section edge {index}; expected 1..={}",
                    self.shapes.len()
                ),
            )
        })?;
        topods::edge(shape)
    }
}

/// The degree 1 B-spline from `p1` to `p2` on `[0, 1]`.
fn point_curve(p1: Point3, p2: Point3) -> Result<Handle<GeomBSplineCurve>, Failure> {
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

/// The point curve of a degenerated edge, from its last vertex to its first
/// (`TopExp::Vertices(edge, vl, vf)` natively).
fn degenerated_curve(edge: &Edge) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let (vl, vf) = topexp::vertices(edge, false);
    point_curve(BRepTool::pnt(&vf)?, BRepTool::pnt(&vl)?)
}

/// `EdgeToBSpline(theEdge)`: the curve of the edge on its range as a B-spline
/// parameterized on `[0, 1]` (a conic approximated C1 within Precision::Confusion(),
/// any curve converted otherwise), located and reversed for a REVERSED edge; a null
/// handle for an edge without 3d curve.
fn edge_to_bspline(edge: &Edge) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let bs = if BRepTool::degenerated(edge)? {
        degenerated_curve(edge)?
    } else {
        located_bspline(edge)?
    };
    if bs.is_null() {
        return Ok(bs);
    }
    if edge.orientation() == Orientation::Reversed {
        live(&bs)?.reverse()?;
    }
    Ok(bs)
}

/// The non-degenerated branch of `EdgeToBSpline`.
fn located_bspline(edge: &Edge) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let found = BRepTool::curve_with_location(edge)?;
    if found.curve.is_null() {
        return Ok(Handle::default());
    }
    let trim = Handle::new(GeomTrimmedCurve::new(
        &found.curve,
        found.first,
        found.last,
        true,
        true,
    )?);
    let is_conic = live(&trim)?.basis_curve().is_kind_name("Geom_Conic");
    let trim: Handle<dyn GeomCurve> = trim.cast();
    let mut bs: Handle<GeomBSplineCurve> = Handle::default();
    if is_conic {
        let approx =
            ApproxCurve::from_curve(&trim, precision::confusion(), Continuity::C1, 16, 14)?;
        if approx.has_result() {
            bs = approx.curve();
        }
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

/// The object of a live handle.
fn live<T: ?Sized + rustycad_tkernel::standard::RuntimeTyped>(
    handle: &Handle<T>,
) -> Result<&T, Failure> {
    handle.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "BRepOffsetAPI_ThruSections: null curve; expected a curve",
        )
    })
}

/// The concatenated B-spline of section `j` (1-based), or a null handle when an edge
/// has no 3d curve.
fn section_bspline(edges: &SectionEdges, j: usize) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let base = (j - 1) * edges.nb_edges;
    let first = edge_to_bspline(&edges.edge(base + 1)?)?;
    if first.is_null() {
        return Ok(first);
    }
    let mut comp = CompCurveToBSplineCurve::with_curve(
        &first.cast::<dyn GeomBoundedCurve>(),
        ParameterisationType::TgtThetaOver2,
    )?;
    for i in 2..=edges.nb_edges {
        let next_edge = edges.edge(base + i)?;
        let tol_v = vertex_gap_tolerance(&next_edge)?;
        let curv_bs = edge_to_bspline(&next_edge)?;
        if curv_bs.is_null() {
            return Ok(curv_bs);
        }
        comp.add(
            &curv_bs.cast::<dyn GeomBoundedCurve>(),
            tol_v,
            true,
            false,
            1,
        )?;
    }
    Ok(comp.bspline_curve())
}

/// The concatenation tolerance at an edge: its vertex tolerances, at least
/// Precision::Confusion() and at most 1e-3.
fn vertex_gap_tolerance(edge: &Edge) -> Result<f64, Failure> {
    let (vf, vl) = topexp::vertices(edge, false);
    let tol = precision::confusion()
        .max(BRepTool::vertex_tolerance(&vf)?)
        .max(BRepTool::vertex_tolerance(&vl)?);
    Ok(tol.min(1.0e-3))
}

/// The section generator of `TotalSurf`, or `None` when an edge has no 3d curve.
fn sections_generator(edges: &SectionEdges) -> Result<Option<GeomFillSectionGenerator>, Failure> {
    let mut section = GeomFillSectionGenerator::new();
    let (mut jdeb, mut jfin) = (1, edges.nb_sects);
    if edges.w1_point {
        jdeb += 1;
        section.add_curve(&degenerated_curve(&edges.edge(1)?)?.cast())?;
    }
    if edges.w2_point {
        jfin -= 1;
    }
    let mut bs1: Handle<GeomBSplineCurve> = Handle::default();
    for j in jdeb..=jfin {
        if j == jfin && edges.v_closed {
            section.add_curve(&bs1.clone().cast())?;
            continue;
        }
        let bs = section_bspline(edges, j)?;
        if bs.is_null() {
            return Ok(None);
        }
        section.add_curve(&bs.clone().cast())?;
        if j == jdeb && edges.v_closed {
            bs1 = bs;
        }
    }
    if edges.w2_point {
        let last = edges.edge(edges.nb_sects * edges.nb_edges)?;
        section.add_curve(&degenerated_curve(&last)?.cast())?;
    }
    section.perform(precision::p_confusion())?;
    Ok(Some(section))
}

impl BRepOffsetAPIThruSections {
    /// `TotalSurf(shapes, NbSects, NbEdges, w1Point, w2Point, vClosed)`: the skinning
    /// surface of the sections (degrees 2 to the maximum degree, three iterations
    /// above a 1e-3 precision), smoothed or with the parametrization type; a null
    /// handle when the approximation is not done or an edge has no 3d curve.
    pub(super) fn total_surf(
        &self,
        edges: &SectionEdges,
    ) -> Result<Handle<GeomBSplineSurface>, Failure> {
        let Some(mut section) = sections_generator(edges)? else {
            return Ok(Handle::default());
        };
        let line = GeomFillLine::with_nb_points(edges.nb_sects as i32);
        let nb_it = if self.my_pres3d <= 1.0e-3 { 0 } else { 3 };
        let degmax = self.my_deg_max.max(2);
        let mut approx =
            GeomFillAppSurf::new(2, degmax, self.my_pres3d, self.my_pres3d, nb_it, false);
        approx.set_continuity(self.my_continuity);
        if self.my_use_smoothing {
            let [w1, w2, w3] = self.my_crit_weights;
            approx.set_criterium_weight(w1, w2, w3)?;
            approx.perform_smoothing(Some(&line), &mut section)?;
        } else {
            approx.set_par_type(self.my_param_type);
            approx.perform(Some(&line), &mut section, true)?;
        }
        if !approx.is_done() {
            return Ok(Handle::default());
        }
        let surface = GeomBSplineSurface::with_weights(
            approx.surf_poles()?,
            approx.surf_weights()?,
            (approx.surf_u_knots()?, approx.surf_u_mults()?),
            (approx.surf_v_knots()?, approx.surf_v_mults()?),
            (approx.u_degree()?, approx.v_degree()?),
            (false, false),
        )?;
        Ok(Handle::new(surface))
    }
}
