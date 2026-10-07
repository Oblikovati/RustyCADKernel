// Rust translation of the projected edge of BRepAlgo_NormalProjection::Build
// (ModelingAlgorithms/TKBool/BRepAlgo/BRepAlgo_NormalProjection.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! One projected edge: its p-curve (exact for isoparametric projections, else
//! approximated), its 3d curve (the edge itself when elementary and close enough, else
//! approximated) and its degeneration to a point.
use super::BRepAlgoNormalProjection;
use super::build::null_adaptor;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkg3d::geom_adaptor::make_curve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::approx::{CurveOnSurface, CurveOnSurfaceOptions};
use rustycad_tkgeombase::proj_lib::CompProjectedCurve;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;

/// A projected edge with its p-curve and whether it degenerated to a point.
pub(super) struct ProjectedPiece {
    pub prj: Shape,
    pub pcur2d: Handle<dyn Geom2dCurve>,
    pub degenerated: bool,
}

/// An array error as a native exception.
fn array_failure(e: impl std::fmt::Debug) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("BRepAlgo_NormalProjection: {e:?}; expected an index in the array bounds"),
        "NCollection_Array1",
    )
}

/// The degree-1 B-spline p-curve between the projections of the bounds (an
/// isoparametric projection), trimmed.
fn iso_pcurve(
    projector: &CompProjectedCurve,
    (udeb, ufin): (f64, f64),
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let (pdeb, pfin) = (projector.d0(udeb)?, projector.d0(ufin)?);
    let mut poles: Array1<'static, Point2> = Array1::with_bounds(1, 2).map_err(array_failure)?;
    let mut knots: Array1<'static, f64> = Array1::with_bounds(1, 2).map_err(array_failure)?;
    let mut mults: Array1<'static, i32> = Array1::with_bounds(1, 2).map_err(array_failure)?;
    (poles[1], poles[2], knots[1], knots[2], mults[1], mults[2]) = (pdeb, pfin, udeb, ufin, 2, 2);
    let bs2d: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dBSplineCurve::new(&poles, &knots, &mults, 1, false)?).cast();
    Ok(Handle::new(Geom2dTrimmedCurve::new(&bs2d, udeb, ufin, true, true)?).cast())
}

/// The barycentre of `nb` points of the curve at equal steps, and whether they all
/// lie within the tolerance of it.
fn point_cloud(bs3d: &GeomBSplineCurve, nb: i32, tol: f64) -> Result<(Point3, bool), Failure> {
    let (first, last) = (bs3d.first_parameter(), bs3d.last_parameter());
    let d_par: f64 = (last - first) / f64::from(nb - 1);
    let mut p1 = Point3::new(0., 0., 0.);
    for ii in 0..nb {
        let pp = bs3d.value(first + f64::from(ii) * d_par)?;
        p1.set_xyz(p1.xyz() + pp.xyz() / f64::from(nb));
    }
    for ii in 0..nb {
        let pp = bs3d.value(first + f64::from(ii) * d_par)?;
        if p1.distance(&pp) > tol {
            return Ok((p1, false));
        }
    }
    Ok((p1, true))
}

/// Whether the approximated curve degenerates to a point (3 points, then 10, within
/// the tolerance of their barycentre), with the barycentre.
fn degenerated_point(bs3d: &GeomBSplineCurve, tol: f64) -> Result<Option<Point3>, Failure> {
    let (_, small) = point_cloud(bs3d, 3, tol)?;
    if !small {
        return Ok(None);
    }
    let (p1, small) = point_cloud(bs3d, 10, tol)?;
    Ok(small.then_some(p1))
}

/// The degenerated edge at a point.
fn degenerated_edge(p1: Point3, tol: f64) -> Result<Shape, Failure> {
    let bb = BRepBuilder::default();
    let mut vv = Vertex::default();
    bb.make_vertex_at(&mut vv, p1, tol)?;
    let mut prj = Edge::default();
    bb.make_edge(&mut prj)?;
    bb.add(&mut prj, &vv.oriented(Orientation::Forward))?;
    bb.add(&mut prj, &vv.oriented(Orientation::Reversed))?;
    bb.degenerated(&prj, true)?;
    Ok(prj.into_shape())
}

/// The edge of the 3d curve of the projected edge on the bounds.
fn edge_on_bounds(hcur: &BRepAdaptorCurve, (udeb, ufin): (f64, f64)) -> Result<Shape, Failure> {
    let curve: Handle<dyn GeomCurve> = make_curve(hcur)?;
    Ok(BRepLibMakeEdge::from_curve_params(&curve, udeb, ufin)?
        .edge()?
        .into_shape())
}

impl ProjectedPiece {
    /// The projected edge of the curve `k` of the projector: `None` when its
    /// approximation misses by more than 1000 times the 3d tolerance. `degenerated` is
    /// the native flag declared once per face: only an approximated 3d curve sets it,
    /// the other pieces keep the value of the previous one (a native quirk).
    pub(super) fn of(
        np: &BRepAlgoNormalProjection,
        (h_projector, k): (&Handle<CompProjectedCurve>, i32),
        (hcur, hsur, face): (
            &Handle<BRepAdaptorCurve>,
            &Handle<dyn Adaptor3dSurface>,
            &Face,
        ),
        (elementary, degenerated): (bool, &mut bool),
    ) -> Result<Option<Self>, Failure> {
        let projector = h_projector.get().ok_or_else(null_adaptor)?;
        let curve = hcur.get().ok_or_else(null_adaptor)?;
        let (udeb, ufin) = projector.bounds(k)?;
        let (mut pcur2d, hpcur, only3d): (
            Handle<dyn Geom2dCurve>,
            Handle<dyn Adaptor2dCurve2d>,
            bool,
        ) = if projector.is_u_iso(k)?.0 || projector.is_v_iso(k)?.0 {
            let pcur = iso_pcurve(projector, (udeb, ufin))?;
            let hp: Handle<dyn Adaptor2dCurve2d> =
                Handle::new(Geom2dAdaptorCurve::with_curve(&pcur)?).cast();
            (pcur, hp, true)
        } else {
            (Handle::default(), h_projector.clone().cast(), false)
        };
        let only2d: bool =
            (!np.my_with3d || elementary) && projector.max_distance(k)? <= np.my_tol3d;
        let bb = BRepBuilder::default();
        if only2d && only3d {
            let prj = edge_on_bounds(curve, (udeb, ufin))?;
            let e = topods::edge(&prj)?;
            bb.update_edge_pcurve_on_face(&e, &pcur2d, face, np.my_tol3d)?;
            bb.update_vertex_tolerance(&topexp::first_vertex(&e, false), np.my_tol3d)?;
            bb.update_vertex_tolerance(&topexp::last_vertex(&e, false), np.my_tol3d)?;
            return Ok(Some(Self {
                prj,
                pcur2d,
                degenerated: *degenerated,
            }));
        }
        let options = CurveOnSurfaceOptions {
            max_segments: np.my_max_seg,
            max_degree: np.my_max_degree,
            continuity: np.my_continuity,
            only_3d: only3d,
            only_2d: only2d,
        };
        let appr = CurveOnSurface::new(&hpcur, hsur, udeb, ufin, np.my_tol3d, options)?;
        if appr.max_error_3d() > 1.0e3 * np.my_tol3d {
            return Ok(None);
        }
        if !only3d {
            pcur2d = appr.curve_2d().cast();
        }
        let prj: Shape = if only2d {
            edge_on_bounds(curve, (udeb, ufin))?
        } else {
            let (prj, d) = approximated_edge(&appr, np.my_tol3d)?;
            *degenerated = d;
            prj
        };
        let e = topods::edge(&prj)?;
        let err: f64 = appr.max_error_3d();
        bb.update_edge_pcurve_on_face(&e, &pcur2d, face, err)?;
        bb.update_vertex_tolerance(&topexp::first_vertex(&e, false), err)?;
        bb.update_vertex_tolerance(&topexp::last_vertex(&e, false), err)?;
        if *degenerated {
            bb.range_on_face(&e, face, udeb, ufin)?;
        }
        Ok(Some(Self {
            prj,
            pcur2d,
            degenerated: *degenerated,
        }))
    }
}

/// The edge of the approximated 3d curve, or a degenerated edge at its point when it
/// is no longer than the tolerance (to set the flag).
fn approximated_edge(appr: &CurveOnSurface, tol: f64) -> Result<(Shape, bool), Failure> {
    let bs3d_h = appr.curve_3d();
    let bs3d = bs3d_h.get().ok_or_else(null_adaptor)?;
    if let Some(p1) = degenerated_point(bs3d, tol)? {
        return Ok((degenerated_edge(p1, tol)?, true));
    }
    Ok((
        BRepLibMakeEdge::from_curve(&bs3d_h.cast())?
            .edge()?
            .into_shape(),
        false,
    ))
}
