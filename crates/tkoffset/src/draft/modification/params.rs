// Rust translation of the statics Choose, Parameter and SmartParameter of
// ModelingAlgorithms/TKOffset/Draft/Draft_Modification_1.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parameters of the new vertices on the new edge curves.
use super::{EdgeMap, FaceMap, live};
use super::add::located_curve;
use crate::draft::info::{DraftEdgeInfo, DraftVertexInfo};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dBoundedCurve, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomEllipse, GeomHyperbola, GeomLine, GeomParabola, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeomalgo::geom2d_api::ProjectPointOnCurve;
use rustycad_tkgeomalgo::geom_api::ProjectPointOnSurf;
use rustycad_tkgeombase::approx::{CurveOnSurface, CurveOnSurfaceOptions};
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtPC};
use rustycad_tkgeombase::geom_proj_lib;
use rustycad_tkgeombase::geom2d_convert::{self, CompCurveToBSplineCurve};
use rustycad_tkgeombase::proj_lib::HCompProjectedCurve;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Point2, Point3, Vector3};
use std::f64::consts::PI;

/// `Epsilon(2 * PI)`: the gap to the next representable value of 2 PI.
fn epsilon_two_pi() -> f64 {
    let x = 2. * PI;
    f64::from_bits(x.to_bits() + 1) - x
}

/// `Parameter(C, P, done)`: the parameter of the point on the curve (the analytic
/// projection on a conic, else the nearest extremum; the nearer end with `done` -1 or
/// 1 when there is none).
pub(super) fn parameter(c: &Handle<dyn GeomCurve>, p: Point3) -> Result<(f64, i32), Failure> {
    let mut done = 0;
    let mut cbase = c.clone();
    if live(c, "curve")?.is_instance_name("Geom_TrimmedCurve") {
        cbase = super::untrimmed_curve(c)?;
    }
    let ctyp = live(&cbase, "curve")?;
    let mut param;
    if ctyp.is_instance_name("Geom_Line") {
        param = elclib::parameter(&live(&cbase.clone().cast::<GeomLine>(), "line")?.lin(), p)?;
    } else if ctyp.is_instance_name("Geom_Circle") {
        param = elclib::parameter(&live(&cbase.clone().cast::<GeomCircle>(), "circle")?.circ()?, p)?;
        if (2. * PI - param).abs() <= epsilon_two_pi() {
            param = 0.;
        }
    } else if ctyp.is_instance_name("Geom_Ellipse") {
        param = elclib::parameter(&live(&cbase.clone().cast::<GeomEllipse>(), "ellipse")?.elips()?, p)?;
        if (2. * PI - param).abs() <= epsilon_two_pi() {
            param = 0.;
        }
    } else if ctyp.is_instance_name("Geom_Parabola") {
        param = elclib::parameter(&live(&cbase.clone().cast::<GeomParabola>(), "parabola")?.parab()?, p)?;
    } else if ctyp.is_instance_name("Geom_Hyperbola") {
        param = elclib::parameter(&live(&cbase.clone().cast::<GeomHyperbola>(), "hyperbola")?.hypr()?, p)?;
    } else {
        let the_curve = GeomAdaptorCurve::with_curve(c)?;
        let my_ext_pc = ExtPC::with_curve(p, &the_curve, 1.0e-10)?;
        if !my_ext_pc.is_done() {
            return Err(Failure::with_message(
                "Standard_Failure",
                "Draft_Modification_1::Parameter: ExtremaPC not done.",
            ));
        }
        if my_ext_pc.nb_ext()? >= 1 {
            let mut dist2_min = my_ext_pc.square_distance(1)?;
            let mut jmin = 1;
            for j in 2..=my_ext_pc.nb_ext()? {
                let dist2 = my_ext_pc.square_distance(j)?;
                if dist2 < dist2_min {
                    dist2_min = dist2;
                    jmin = j;
                }
            }
            param = my_ext_pc.point(jmin)?.parameter();
        } else {
            let (dist1_2, dist2_2, _, _) = my_ext_pc.trimmed_square_distances();
            if dist1_2 < dist2_2 {
                done = -1;
                param = the_curve.first_parameter()?;
            } else {
                done = 1;
                param = the_curve.last_parameter()?;
            }
        }

        let base = live(&cbase, "curve")?;
        if base.is_periodic() {
            let per = base.period()?;
            let tolp = precision::parametric(precision::confusion());
            if (per - param).abs() <= tolp {
                param = 0.;
            }
        }
    }
    Ok((param, done))
}

/// The bounded 2d curve of a B-spline handle.
fn bounded_2d(c: &Handle<Geom2dBSplineCurve>) -> Handle<dyn Geom2dBoundedCurve> {
    c.clone().cast()
}

/// `SmartParameter(Einf, EdgeTol, Pnt, sign, S1, S2)`: the edge curve extended to the
/// point (its p-curve on the first surface extended by a segment at the start or the
/// end, then projected on the second surface and approximated); the first or last
/// parameter of the new curve.
pub(super) fn smart_parameter(
    einf: &mut DraftEdgeInfo,
    edge_tol: f64,
    pnt: Point3,
    sign: i32,
    s1: &Handle<dyn GeomSurface>,
    s2: &Handle<dyn GeomSurface>,
) -> Result<f64, Failure> {
    let tol = precision::confusion();
    let etol = edge_tol;

    let mut pcu1 = einf.first_pc().clone();
    let mut pcu2 = einf.second_pc().clone();

    if pcu1.is_null() {
        let the_curve = einf.geometry().clone();
        let curve = live(&the_curve, "edge geometry")?;
        pcu1 = geom_proj_lib::curve2d_range_tolerance(
            &the_curve,
            (curve.first_parameter(), curve.last_parameter()),
            s1,
            etol,
        )?
        .0;
        *einf.change_first_pc() = pcu1.clone();
    }
    if pcu2.is_null() {
        let the_curve = einf.geometry().clone();
        let curve = live(&the_curve, "edge geometry")?;
        pcu2 = geom_proj_lib::curve2d_range_tolerance(
            &the_curve,
            (curve.first_parameter(), curve.last_parameter()),
            s2,
            etol,
        )?
        .0;
        *einf.change_second_pc() = pcu2.clone();
    }
    let _ = (pcu1, pcu2);

    let projector = ProjectPointOnSurf::with_surface(pnt, s1, ExtAlgo::Grad)?;
    let (u, v) = projector.lower_distance_parameters()?;

    let mut new_c2d = einf.first_pc().clone();
    if live(&new_c2d, "pcurve")?.is_instance_name("Geom2d_TrimmedCurve") {
        new_c2d = live(&new_c2d.clone().cast::<Geom2dTrimmedCurve>(), "pcurve")?.basis_curve();
    }

    let p2d = Point2::new(u, v);
    let projector2d = ProjectPointOnCurve::with_curve(p2d, &new_c2d)?;
    if projector2d.nb_points() == 0 || projector2d.lower_distance()? > tol {
        let mut bcurve: Handle<Geom2dBSplineCurve> = if !live(&new_c2d, "pcurve")?.is_instance_name("Geom2d_BSplineCurve")
        {
            geom2d_convert::curve_to_bspline_curve(&new_c2d, ParameterisationType::TgtThetaOver2)?
        } else {
            new_c2d.clone().cast::<Geom2dBSplineCurve>()
        };
        let curve = live(&bcurve, "B-spline pcurve")?;
        let mut pnt_array: Array1<'static, Point2> = Array1::with_bounds(1, 2)
            .map_err(|e| Failure::with_message("Standard_RangeError", &format!("{e:?}")))?;
        let after = sign != -1;
        if sign == -1 {
            pnt_array[1] = p2d;
            pnt_array[2] = curve.pole(1)?;
        } else {
            pnt_array[1] = curve.pole(curve.nb_poles())?;
            pnt_array[2] = p2d;
        }
        let patch: Handle<dyn Geom2dBoundedCurve> = Handle::new(Geom2dBezierCurve::new(&pnt_array)?).cast();
        let mut concat = CompCurveToBSplineCurve::with_curve(&bounded_2d(&bcurve), ParameterisationType::QuasiAngular)?;
        concat.add(&patch, tol, after)?;
        bcurve = concat.bspline_curve();
        new_c2d = bcurve.cast();
    }
    *einf.change_first_pc() = new_c2d.clone();
    let hcur: Handle<dyn Adaptor2dCurve2d> = Handle::new(Geom2dAdaptorCurve::with_curve(&new_c2d)?).cast();
    let hsur: Handle<dyn Adaptor3dSurface> = Handle::new(GeomAdaptorSurface::with_surface(s1)?).cast();
    let cons = Adaptor3dCurveOnSurface::with_curve(&hcur, &hsur)?;
    let hcons: Handle<dyn Adaptor3dCurve> = Handle::new(cons).cast();
    let hsur2: Handle<dyn Adaptor3dSurface> = Handle::new(GeomAdaptorSurface::with_surface(s2)?).cast();
    let h_projector = HCompProjectedCurve::with_tolerances(&hsur2, &hcons, tol, tol)?;
    let (udeb, ufin) = h_projector.bounds(1)?;
    let max_seg = 20 + h_projector.nb_intervals(Continuity::C3)?;
    let h_projector: Handle<dyn Adaptor2dCurve2d> = Handle::new(h_projector).cast();
    let appr = CurveOnSurface::new(
        &h_projector,
        &hsur2,
        udeb,
        ufin,
        tol,
        CurveOnSurfaceOptions {
            max_segments: max_seg,
            max_degree: 10,
            continuity: Continuity::C1,
            only_3d: false,
            only_2d: false,
        },
    )?;
    *einf.change_second_pc() = appr.curve_2d().cast();
    *einf.change_geometry() = appr.curve_3d().cast();
    einf.set_new_geometry(true);

    let geometry = live(einf.geometry(), "edge geometry")?;
    if sign == -1 {
        Ok(geometry.first_parameter())
    } else {
        Ok(geometry.last_parameter())
    }
}

/// The parameter of the point on a curve, smart (extended) when the point falls
/// outside the curve: the `Parameter` then `SmartParameter` pattern of the draft.
pub(super) fn smart_or_plain(
    f_map: &FaceMap,
    einf: &mut DraftEdgeInfo,
    curve: &Handle<dyn GeomCurve>,
    edge_tol: f64,
    p: Point3,
) -> Result<f64, Failure> {
    let (param, done) = parameter(curve, p)?;
    if done != 0 {
        let s1 = f_map.find_from_key(einf.first_face())?.geometry().clone();
        let s2 = f_map.find_from_key(einf.second_face())?.geometry().clone();
        return smart_parameter(einf, edge_tol, p, done, &s1, &s2);
    }
    Ok(param)
}

/// The curve adaptor (`AC`) and the surface (`AS`) chosen for a vertex.
pub(super) type ChosenCurveSurface = (GeomAdaptorCurve, Handle<dyn GeomSurface>);

/// `Choose(theFMap, theEMap, Vtx, Vinf, AC, AS)`: the curve of a reference edge of the
/// vertex (an edge with one face or a tangent one, else the first) and the surface of
/// a face of a non tangent edge through the vertex to intersect it with; `None` when
/// every edge is tangent to the reference.
pub(super) fn choose(
    the_f_map: &FaceMap,
    the_e_map: &mut EdgeMap,
    vtx: &Vertex,
    vinf: &mut DraftVertexInfo,
) -> Result<Option<ChosenCurveSurface>, Failure> {
    vinf.init_edge_iterator();

    // Find a regular edge with null SecondFace
    while vinf.more_edge() {
        let e1 = vinf.edge()?;
        let einf1 = the_e_map.find_from_key(&e1)?;
        if einf1.second_face().is_null() {
            break;
        } else {
            let te = BRepTool::continuity_on_faces(&e1, einf1.first_face(), einf1.second_face())?;
            if te >= Continuity::G1 {
                break;
            }
        }
        vinf.next_edge();
    }
    if !vinf.more_edge() {
        // take the first edge
        vinf.init_edge_iterator();
    }

    let eref = vinf.edge()?;
    let ac = {
        let einf = the_e_map.change_from_key(&eref)?;
        GeomAdaptorCurve::with_curve(einf.geometry())?
    };

    let mut c = located_curve(&eref)?;
    let vtx_pnt = BRepTool::pnt(vtx)?;
    let prm = {
        let einf = the_e_map.change_from_key(&eref)?;
        smart_or_plain(the_f_map, einf, &c, BRepTool::edge_tolerance(&eref)?, vtx_pnt)?
    };
    let (_, tgref): (Point3, Vector3) = live(&c, "curve")?.d1(prm)?;

    vinf.init_edge_iterator();
    while vinf.more_edge() {
        // Find a non tangent edge
        let edg = vinf.edge()?;
        if !edg.is_same(&eref) {
            let einfo = the_e_map.change_from_key(&edg)?;
            if !einfo.second_face().is_null()
                && BRepTool::continuity_on_faces(&edg, einfo.first_face(), einfo.second_face())? <= Continuity::C0
            {
                c = located_curve(&edg)?;
                let prm = smart_or_plain(the_f_map, einfo, &c, BRepTool::edge_tolerance(&edg)?, vtx_pnt)?;
                let (_, tg): (Point3, Vector3) = live(&c, "curve")?.d1(prm)?;
                if tg.cross_magnitude(tgref) > precision::confusion() {
                    break;
                }
            }
        }
        vinf.next_edge();
    }
    if !vinf.more_edge() {
        return Ok(None);
    }

    let e2 = vinf.edge()?;
    let einf2 = the_e_map.find_from_key(&e2)?;
    let einf = the_e_map.find_from_key(&eref)?;
    let other = if !einf.second_face().is_null() {
        if einf2.first_face().is_same(einf.first_face()) || einf2.first_face().is_same(einf.second_face()) {
            einf2.second_face()
        } else {
            einf2.first_face()
        }
    } else if einf2.first_face().is_same(einf.first_face()) {
        einf2.second_face()
    } else {
        einf2.first_face()
    };
    let surface = the_f_map.find_from_key(other)?.geometry().clone();
    Ok(Some((ac, surface)))
}
