// Rust translation of the static functions of
// ModelingAlgorithms/TKOffset/BiTgte/BiTgte_Blend.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The helpers of the rolling-ball blend: restriction tests, the 3d curves of the
//! iso p-curves, the curves on the edges, the vertex lookups and the degenerated edges.
use crate::bi_tgte::BiTgteCurveOnEdge;
use crate::brep_offset::OffsetFaces;
use crate::brep_offset::{BRepOffsetAnalyse, BRepOffsetOffset};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomCircle, GeomCurve, GeomLine, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom_api::to_3d;
use rustycad_tkgeomalgo::geom2d_api::ProjectPointOnCurve as ProjectPointOnCurve2d;
use rustycad_tkgeombase::app_cont::{Function, FunctionBase};
use rustycad_tkgeombase::app_par_curves::MultiCurveEval;
use rustycad_tkgeombase::approx::{ComputeCLineOptions, FitAndDivide};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::convert::CompBezierCurvesToBSplineCurve;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Axis1, Direction2, Point2, Point3, Vector2, Vector3};
use std::cell::RefCell;
use std::f64::consts::PI;

/// The object of a live handle; Standard_NullObject otherwise (a null dereference
/// natively).
pub(super) fn live<'a, T: ?Sized + rustycad_tkernel::standard::RuntimeTyped>(
    handle: &'a Handle<T>,
    what: &str,
) -> Result<&'a T, Failure> {
    handle.get().ok_or_else(|| {
        Failure::with_message("Standard_NullObject", &format!("BiTgte_Blend: null {what}"))
    })
}

/// `IsOnRestriction(V, CurE, F, E)`: the edge of the face whose p-curve passes within
/// the vertex tolerance of the vertex of `CurE` (the last tested edge otherwise).
pub(super) fn is_on_restriction(
    v: &Vertex,
    cur_e: &Edge,
    f: &Face,
) -> Result<(bool, Edge), Failure> {
    // find if Vertex V of CurE is on a restriction of F.
    // if yes, store this restriction in E.
    let cur_c = BRepTool::curve_on_face(cur_e, f)?;
    let u = BRepTool::parameter_on_face(v, cur_e, f)?;
    let p = live(&cur_c.pcurve, "pcurve")?.value(u)?;

    let mut proj = ProjectPointOnCurve2d::new();

    // The tolerance is exaggerated : it is better to construct too many
    // tubes than to miss intersections.
    let tol = BRepTool::vertex_tolerance(v)?;
    let mut e = Edge::default();
    for s in Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)? {
        e = topods::edge(&s)?;
        let pc = BRepTool::curve_on_face(&e, f)?;
        proj.init_range(p, &pc.pcurve, pc.first, pc.last)?;
        if proj.nb_points() > 0 && proj.lower_distance()? < tol {
            return Ok((true, e));
        }
    }
    Ok((false, e))
}

/// `Add(E, Map, S, OF, Analyse, WarningSurBordLibre)`: the edge of the face generating
/// `E` (not on a free border when warned), or the ancestors of the vertex of the edge
/// generating `E`.
pub(super) fn add(
    e: &Edge,
    map: &mut IndexedMapOfShape,
    s: &Shape,
    of: &BRepOffsetOffset,
    analyse: &BRepOffsetAnalyse,
    warning_sur_bord_libre: bool,
) -> Result<(), Failure> {
    match s.shape_type()? {
        ShapeEnum::Face => {
            for ori_e in Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)? {
                let ie = topods::edge(&of.generated(&ori_e))?;
                if e.is_equal(&ie) {
                    // It is checked that the border is not free.
                    if warning_sur_bord_libre && analyse.ancestors(&ori_e)?.len() == 1 {
                        break; // Nothing is done.
                    }
                    map.add(ori_e);
                    break;
                }
            }
        }
        ShapeEnum::Edge => {
            for v in Explorer::new(s, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                let ie = topods::edge(&of.generated(&v))?;
                if e.is_equal(&ie) {
                    for a in analyse.ancestors(&v)? {
                        map.add(a.clone());
                    }
                    break;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// `IsInFace(E, F)`.
pub(super) fn is_in_face(e: &Edge, f: &Face) -> Result<bool, Failure> {
    Ok(Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)?.any(|s| e.is_same(&s)))
}

/// The circle rotated about the axis of the frame by `angle`.
fn rotated_circle(
    ci: rustycad_tkmath::gp::Circle3,
    axis: rustycad_tkmath::gp::Axis3,
    angle: f64,
) -> Result<rustycad_tkmath::gp::Circle3, Failure> {
    let d_rev = axis.x_direction().crossed(axis.y_direction())?;
    let axe_rev = Axis1::new(axis.location(), d_rev);
    ci.rotated(axe_rev, angle)
}

/// `KPartCurve3d(Edge, Curve, Surf)`: the 3d curve of an iso p-curve on an analytic
/// surface (a circle or a line), of a p-curve on a plane; nothing otherwise.
pub(super) fn k_part_curve3d(
    edge: &Edge,
    curve: &Handle<dyn Geom2dCurve>,
    surf: &Handle<dyn GeomSurface>,
) -> Result<(), Failure> {
    // try to find the particular case
    // if not found call BRepLib::BuildCurve3d
    let loc = Default::default();
    let tol = precision::confusion();

    // Search only isos on analytical surfaces.
    let c = Geom2dAdaptorCurve::with_curve(curve)?;
    let s = GeomAdaptorSurface::with_surface(surf)?;
    let c_ty = c.get_type()?;
    let s_ty = s.get_type()?;
    let the_builder = BRepBuilder::default();
    let dx2d = Direction2::new(1., 0.)?;
    let dy2d = Direction2::new(0., 1.)?;

    let update =
        |c3d: Handle<dyn GeomCurve>| the_builder.update_edge_located_curve(edge, &c3d, &loc, tol);

    if s_ty != SurfaceType::Plane {
        // if plane buildcurve3d manage KPart
        if c_ty == CurveType::Line {
            let d = c.line()?.direction();
            if d.is_parallel(dx2d, precision::angular()) {
                // Iso V.
                let p = c.line()?.location();
                let ci = match s_ty {
                    SurfaceType::Sphere => {
                        if (p.y().abs() - PI / 2.).abs() < precision::p_confusion() {
                            the_builder.degenerated(edge, true)?;
                            None
                        } else {
                            let sph = s.sphere()?;
                            let axis = sph.position();
                            let ci = elslib::sphere_v_iso(axis, sph.radius(), p.y());
                            Some(rotated_circle(ci, axis, p.x())?)
                        }
                    }
                    SurfaceType::Cylinder => {
                        let cyl = s.cylinder()?;
                        let axis = cyl.position();
                        let ci = elslib::cylinder_v_iso(axis, cyl.radius(), p.y());
                        Some(rotated_circle(ci, axis, p.x())?)
                    }
                    SurfaceType::Cone => {
                        let cone = s.cone()?;
                        let axis = cone.position();
                        let ci =
                            elslib::cone_v_iso(axis, cone.ref_radius(), cone.semi_angle(), p.y());
                        Some(rotated_circle(ci, axis, p.x())?)
                    }
                    SurfaceType::Torus => {
                        let tore = s.torus()?;
                        let axis = tore.position();
                        let ci = elslib::torus_v_iso(
                            axis,
                            tore.major_radius(),
                            tore.minor_radius(),
                            p.y(),
                        );
                        Some(rotated_circle(ci, axis, p.x())?)
                    }
                    _ => None,
                };
                if let Some(ci) = ci {
                    let circle = GeomCircle::from_circ(ci);
                    if d.is_opposite(dx2d, precision::angular()) {
                        circle.reverse()?;
                    }
                    update(Handle::new(circle).cast())?;
                }
            } else if d.is_parallel(dy2d, precision::angular()) {
                // Iso U.
                let p = c.line()?.location();
                match s_ty {
                    SurfaceType::Sphere => {
                        let sph = s.sphere()?;
                        let axis = sph.position();
                        // calculate iso 0.
                        let mut ci = elslib::sphere_u_iso(axis, sph.radius(), 0.);

                        // set to sameparameter (rotation of the circle - offset from Y)
                        let d_rev = axis.x_direction().crossed(axis.direction())?;
                        let axe_rev = Axis1::new(axis.location(), d_rev);
                        ci = ci.rotated(axe_rev, p.y())?;

                        // transformation by iso U ( = P.X())
                        ci = rotated_circle(ci, axis, p.x())?;
                        let circle = GeomCircle::from_circ(ci);

                        if d.is_opposite(dy2d, precision::angular()) {
                            circle.reverse()?;
                        }
                        update(Handle::new(circle).cast())?;
                    }
                    SurfaceType::Cylinder => {
                        let cyl = s.cylinder()?;
                        let mut l = elslib::cylinder_u_iso(cyl.position(), cyl.radius(), p.x());
                        let tr = Vector3::from(l.direction()) * p.y();
                        l.translate(tr);
                        let line = GeomLine::from_lin(l);
                        if d.is_opposite(dy2d, precision::angular()) {
                            line.reverse()?;
                        }
                        update(Handle::new(line).cast())?;
                    }
                    SurfaceType::Cone => {
                        let cone = s.cone()?;
                        let mut l = elslib::cone_u_iso(
                            cone.position(),
                            cone.ref_radius(),
                            cone.semi_angle(),
                            p.x(),
                        );
                        let tr = Vector3::from(l.direction()) * p.y();
                        l.translate(tr);
                        let line = GeomLine::from_lin(l);
                        if d.is_opposite(dy2d, precision::angular()) {
                            line.reverse()?;
                        }
                        update(Handle::new(line).cast())?;
                    }
                    _ => {}
                }
            }
        }
    } else {
        // Case Plane
        let c3d = to_3d(curve, &s.plane()?)?;
        update(c3d)?;
    }
    Ok(())
}

/// `MakeCurve_Function`: the points of a curve on edge for the approximation (the
/// derivatives are not given). A failure of the projection is recorded and returned
/// after the fit (natively it leaves the fit as an exception).
struct MakeCurveFunction<'a> {
    base: FunctionBase,
    my_curve: &'a BiTgteCurveOnEdge,
    error: RefCell<Option<Failure>>,
}

impl Function for MakeCurveFunction<'_> {
    fn base(&self) -> &FunctionBase {
        &self.base
    }

    fn first_parameter(&self) -> f64 {
        self.my_curve.first_parameter().unwrap_or_else(|e| {
            self.record(e);
            0.
        })
    }

    fn last_parameter(&self) -> f64 {
        self.my_curve.last_parameter().unwrap_or_else(|e| {
            self.record(e);
            0.
        })
    }

    fn value(&self, t: f64, _pnt2d: &mut Array1<'_, Point2>, pnt: &mut Array1<'_, Point3>) -> bool {
        match self.my_curve.value(t) {
            Ok(p) => {
                pnt[1] = p;
                true
            }
            Err(e) => {
                self.record(e);
                false
            }
        }
    }

    fn d1(
        &self,
        _t: f64,
        _vec2d: &mut Array1<'_, Vector2>,
        _vec: &mut Array1<'_, Vector3>,
    ) -> bool {
        false
    }
}

impl MakeCurveFunction<'_> {
    fn record(&self, e: Failure) {
        self.error.borrow_mut().get_or_insert(e);
    }
}

/// `MakeCurve(HC)`: the circle of a degenerated tube, else the approximation of the
/// curve on edge by C1 B-splines of degree 8 on its parameter range.
pub(super) fn make_curve(hc: &BiTgteCurveOnEdge) -> Result<Handle<dyn GeomCurve>, Failure> {
    let c: Handle<dyn GeomCurve> = if hc.get_type()? == CurveType::Circle {
        let circle: Handle<dyn GeomCurve> = Handle::new(GeomCircle::from_circ(hc.circle()?)).cast();
        Handle::new(GeomTrimmedCurve::new(
            &circle,
            hc.first_parameter()?,
            hc.last_parameter()?,
            true,
            true,
        )?)
        .cast()
    } else {
        // the approximation is done
        let mut base = FunctionBase::new();
        base.set_number_of_points(1, 0);
        let f = MakeCurveFunction {
            base,
            my_curve: hc,
            error: RefCell::new(None),
        };
        let (deg1, deg2) = (8, 8);
        let tol = precision::approximation();
        let options = ComputeCLineOptions {
            degree_min: deg1,
            degree_max: deg2,
            tolerance_3d: tol,
            tolerance_2d: tol,
            cutting: true,
            ..ComputeCLineOptions::default()
        };
        let fit = FitAndDivide::with_line(&f, options);
        if let Some(e) = f.error.borrow_mut().take() {
            return Err(e);
        }
        let fit = fit?;
        let nb_curves = fit.nb_multi_curves();
        // it is attempted to make the curve at least C1
        let mut conv = CompBezierCurvesToBSplineCurve::new(precision::angular());

        for i in 1..=nb_curves {
            let mc = fit.value(i)?; // Load the Ith Curve
            let mut poles: Array1<'static, Point3> = array(1, mc.degree()? + 1)?; // Return poles
            mc.curve_3d(1, &mut poles)?;
            conv.add_curve(&poles);
        }

        conv.perform()?;

        let nb_poles = conv.nb_poles();
        let nb_knots = conv.nb_knots();
        let mut new_poles: Array1<'static, Point3> = array(1, nb_poles)?;
        let mut new_knots: Array1<'static, f64> = array(1, nb_knots)?;
        let mut new_mults: Array1<'static, i32> = array(1, nb_knots)?;

        conv.knots_and_mults(&mut new_knots, &mut new_mults)?;
        conv.poles(&mut new_poles)?;

        bsplclib::reparametrize(hc.first_parameter()?, hc.last_parameter()?, &mut new_knots)?;

        Handle::new(GeomBSplineCurve::new(
            &new_poles,
            &new_knots,
            &new_mults,
            conv.degree(),
            false,
        )?)
        .cast()
    };
    Ok(c)
}

/// An array with the bounds (Standard_RangeError on bad bounds).
fn array<T: Clone + Default>(lower: i32, upper: i32) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(lower, upper).map_err(|e| {
        Failure::with_message(
            "Standard_RangeError",
            &format!("BiTgte_Blend: NCollection_Array1 {e:?}"),
        )
    })
}

/// `FindVertex(P, Map, Tol)`: a vertex of the edges of the map at the point (within its
/// tolerance, else within `Tol`, raising its tolerance); null when there is none.
pub(super) fn find_vertex(p: Point3, map: &MapOfShape, tol: f64) -> Result<Vertex, Failure> {
    let b = BRepBuilder::default();
    // Find in <Map> a vertex which represent the point <P>.
    let tol_carre = tol * tol;
    for e in map.iter() {
        let e = topods::edge(e)?;
        if !e.is_null() {
            let (v0, v1) = topexp::vertices(&e, false);
            for vv in [v0, v1] {
                // if OK la Tolerance du Vertex
                let mut tol2 = BRepTool::vertex_tolerance(&vv)?;
                tol2 *= tol2;
                let p1 = BRepTool::pnt(&vv)?;
                let dist = p.square_distance(&p1);
                if dist <= tol2 {
                    return Ok(vv);
                }
                // otherwise with the required tolerance.
                if tol_carre > tol2 && dist <= tol_carre {
                    // so it is necessary to update the tolerance of Vertex.
                    b.update_vertex_tolerance(&vv, tol)?;
                    return Ok(vv);
                }
            }
        }
    }
    Ok(Vertex::default())
}

/// `MakeDegeneratedEdge(CC, VfOnE)`: the edge of the untrimmed curve closed on the
/// vertex (a new one at the start of the curve when null), on the range of `CC`.
pub(super) fn make_degenerated_edge(
    cc: &Handle<dyn GeomCurve>,
    vf_on_e: &Vertex,
) -> Result<Edge, Failure> {
    let b = BRepBuilder::default();
    let tol = precision::confusion();
    // kill trimmed curves
    let mut c = cc.clone();
    while let Some(ct) = c.clone().cast::<GeomTrimmedCurve>().get() {
        c = ct.basis_curve();
    }

    let (mut v1, mut v2);
    if vf_on_e.is_null() {
        let curve = live(&c, "curve")?;
        let p = curve.value(curve.first_parameter())?;
        v1 = Vertex::default();
        b.make_vertex_at(&mut v1, p, tol)?;
        v2 = v1.clone();
    } else {
        v1 = vf_on_e.clone();
        v2 = vf_on_e.clone();
    }
    v1.set_orientation(Orientation::Forward);
    v2.set_orientation(Orientation::Reversed);

    let mut e = Edge::default();
    b.make_edge_with_curve(&mut e, &c, tol)?;
    b.add(&mut e, &v1)?;
    b.add(&mut e, &v2)?;
    let curve = live(cc, "curve")?;
    b.range(&e, curve.first_parameter(), curve.last_parameter(), false)?;
    Ok(e)
}

/// `Orientation(E, F, L)`: the orientation of the edge in the list, reversed with a
/// REVERSED face.
pub(super) fn orientation(e: &Edge, f: &Face, l: &[Shape]) -> Orientation {
    let mut orien = Orientation::Forward;
    if let Some(s) = l.iter().find(|s| s.is_same(e)) {
        orien = s.orientation();
    }
    if f.orientation() == Orientation::Reversed {
        orien = rustycad_tkg3d::top_abs::reverse(orien);
    }
    orien
}

/// `FindCreatedEdge(V1, E, MapSF, MapOnV, CenterAnalyse, Radius, Tol)`: the edge
/// generated at the vertex by a tube on a tangent edge; the edges generated at the
/// vertex are collected in `MapOnV`.
pub(super) fn find_created_edge(
    v1: &Vertex,
    e: &Edge,
    map_sf: &OffsetFaces,
    map_on_v: &mut MapOfShape,
    center_analyse: &BRepOffsetAnalyse,
    radius: f64,
    tol: f64,
) -> Result<Edge, Failure> {
    let mut e1 = Edge::default();
    if !center_analyse.has_ancestor(v1) {
        return Ok(e1); // return a Null Shape.
    }

    let mut tang_e = Vec::new();
    center_analyse.tangent_edges(e, v1, &mut tang_e)?;

    let mut find = false;
    for et in &tang_e {
        if find {
            break;
        }
        if let Some(of) = map_sf.seek(et) {
            e1 = topods::edge(&of.generated(v1))?;
            map_on_v.add(e1.clone().into_shape());
            find = true;
        } else {
            // Find the sharing of vertices in case of tangent consecutive 3 edges
            // the second of which is the edge that degenerates the tube.
            let et = topods::edge(et)?;
            let mut cet = BRepTool::curve_with_location(&et)?.curve;
            if live(&cet, "curve")?.is_instance_name("Geom_TrimmedCurve") {
                cet = live(&cet.clone().cast::<GeomTrimmedCurve>(), "curve")?.basis_curve();
            }
            let circ = cet.cast::<GeomCircle>();
            let Some(circ) = circ.get() else {
                continue;
            };
            if (circ.radius() - radius.abs()).abs() > tol {
                continue;
            }

            let (mut u1, u2) = topexp::vertices(&et, false);
            if u1.is_same(v1) {
                u1 = u2;
            }
            let mut tang2 = Vec::new();
            center_analyse.tangent_edges(&et, &u1, &mut tang2)?;
            for et2 in &tang2 {
                if let Some(of) = map_sf.seek(et2) {
                    map_on_v.add(topods::edge(&of.generated(&u1))?.into_shape());
                }
            }
        }
    }
    if !find && center_analyse.has_ancestor(v1) {
        for a in center_analyse.ancestors(v1)? {
            if let Some(of) = map_sf.seek(a) {
                map_on_v.add(of.generated(v1));
            }
        }
    }
    Ok(e1)
}
