//! The `tt_topology` operation of tools/oracle/tkbool_probe_topopebreptool_tool.cxx.
use super::brep_fill_generator::failed;
use super::common::{Reader, attempt, dir, end, flag, int, pnt, pnt2d, real, xy, xyz};
use super::top_ope_brep_tool_tool::{EdgeFace, edge_face, normal, other_edge};
use rustycad_tkbool::top_ope_brep_tool::*;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::{Direction3, Point2, Point3, Vector2, Vector3};

/// Four reals.
fn reals4(b: (f64, f64, f64, f64)) {
    [b.0, b.1, b.2, b.3].into_iter().for_each(real);
}

/// The bounds members of `tt_topology`.
fn topology_bounds(x: &EdgeFace) {
    let f = &x.f;
    attempt(|| {
        let (a, b) = fun_tool_tol_uv(f)?;
        real(a);
        real(b);
        Ok(())
    });
    attempt(|| {
        let mut d = false;
        flag(fun_tool_direct(f, &mut d)?);
        flag(d);
        Ok(())
    });
    attempt(|| {
        let mut b = (-1.0, -1.0, -1.0, -1.0);
        flag(fun_tool_bounds_face(
            f, &mut b.0, &mut b.1, &mut b.2, &mut b.3,
        )?);
        reals4(b);
        Ok(())
    });
    attempt(|| {
        let b = fun_tool_geombounds(f)?;
        flag(b.is_some());
        reals4(b.unwrap_or((-1.0, -1.0, -1.0, -1.0)));
        Ok(())
    });
    attempt(|| {
        let mut b = (-1.0, -1.0, -1.0, -1.0);
        flag(fun_tool_isobounds(f, &mut b)?);
        reals4(b);
        Ok(())
    });
    attempt(|| {
        let (mut b, mut o) = ((-1.0, -1.0, -1.0, -1.0), false);
        flag(fun_tool_outbounds(f, &mut b, &mut o)?);
        reals4(b);
        flag(o);
        Ok(())
    });
}

/// The point members of `tt_topology`.
fn topology_points(x: &EdgeFace, e2: &Edge) {
    let (e, f, par, uv) = (&x.e, &x.f, x.par, x.uv);
    attempt(|| {
        let bc = BRepAdaptorCurve::with_edge(e)?;
        let p = bc.value(par)?;
        flag(fun_tool_pin_c(p, &bc, 1.0e-7)?);
        let (a, b) = (bc.first_parameter()?, bc.last_parameter()?);
        flag(fun_tool_pin_c_range(
            p.translated(Vector3::new(0.0, 0.0, 1.0e-3)),
            &bc,
            a,
            b,
            1.0e-2,
        )?);
        Ok(())
    });
    attempt(|| {
        let p = fun_tool_value_on_edge(par, e)?;
        flag(p.is_some());
        pnt(p.unwrap_or(Point3::new(-1.0, -1.0, -1.0)));
        pnt(fun_tool_value_on_face(uv, f)?);
        Ok(())
    });
    attempt(|| {
        let p = BRepAdaptorCurve::with_edge(e)?.value(par)?;
        int(fun_tool_sta_pin_e(p, e)? as i32);
        int(fun_tool_sta_pin_e_tol(p.translated(Vector3::new(0.0, 0.0, 0.5)), e, 1.0e-3)? as i32);
        Ok(())
    });
    attempt(|| {
        int(fun_tool_orient_v_in_e(&topexp::first_vertex(e, false), e));
        Ok(())
    });
    attempt(|| {
        let (ok, o) = fun_tool_orient_e_in_f(e, f)?;
        flag(ok);
        int(o as i32);
        let (ok, o) = fun_tool_orient_e_in_f_forward(e, &topods::face(&f.reversed())?)?;
        flag(ok);
        int(o as i32);
        flag(fun_tool_ebound_f(e, f)?);
        Ok(())
    });
    attempt(|| fun_tool_nggeom_f(uv, f).map(|v| xyz(*v.xyz())));
    attempt(|| {
        let n = fun_tool_nggeom_f_on_edge(par, e, f)?;
        flag(n.is_some());
        xyz(*n.unwrap_or_default().xyz());
        let n2 = fun_tool_nggeom_f_on_edge_tol(par, e2, f, 1.0e-9)?;
        flag(n2.is_some());
        xyz(*n2.or(n).unwrap_or_default().xyz());
        Ok(())
    });
    attempt(|| {
        flag(fun_tool_etg_f(par, e, uv, f, 1.0e-6)?);
        flag(fun_tool_etg_ooe(par, e, par, e2, 1.0e-6)?);
        Ok(())
    });
    attempt(|| {
        let (mut a, mut b) = (-10.0, 10.0);
        flag(fun_nearest_iso(f, uv.x(), true, &mut a, &mut b)?);
        real(a);
        real(b);
        Ok(())
    });
    attempt(|| {
        let (_, n) = normal(uv, f)?;
        flag(fun_tool_eitangentto_fe(n, e, par)?);
        Ok(())
    });
}

/// The type and closure members of `tt_topology`.
fn topology_types(x: &EdgeFace) {
    let (e, f) = (&x.e, &x.f);
    attempt(|| {
        int(fun_tool_typ(e)? as i32);
        flag(fun_tool_line_edge(e)?);
        flag(fun_tool_plane_face(f)?);
        flag(fun_tool_cylinder_face(f)?);
        Ok(())
    });
    attempt(|| {
        flag(fun_tool_closed_s_flag(f)?);
        let c = fun_tool_closed_s(f)?;
        let (uc, up, vc, vp) = c.unwrap_or((false, -1.0, false, -1.0));
        flag(c.is_some_and(|c| c.0 || c.2));
        flag(uc);
        real(up);
        flag(vc);
        real(vp);
        let (mut inu, mut xm, mut xp) = (false, -1.0, -1.0);
        flag(fun_tool_closed_s_in_u(f, &mut inu, &mut xm, &mut xp)?);
        flag(inu);
        real(xm);
        real(xp);
        Ok(())
    });
    attempt(|| {
        let ex = Explorer::new(f, ShapeEnum::Wire, ShapeEnum::Shape)?;
        let w = ex.current().clone();
        let mut b = BoundingBox2::default();
        fun_tool_mk_bnd2d(&w, f, &mut b)?;
        b.get()?.into_iter().for_each(real);
        flag(fun_tool_closing_e(e, &topods::wire(&w)?, f)?);
        flag(fun_tool_closed_w(&topods::wire(&w)?)?);
        Ok(())
    });
    attempt(|| {
        flag(fun_tool_is_closing_e(e, f, f)?);
        flag(fun_tool_in_s(e, f)?);
        flag(fun_tool_in_s(e, &x.f2)?);
        Ok(())
    });
}

/// The parameter members of `tt_topology`.
fn topology_params(x: &EdgeFace, e2: &Edge) {
    let (e, f, par) = (&x.e, &x.f, x.par);
    let v1 = topexp::first_vertex(e, false);
    attempt(|| {
        let es = fun_tool_eshared(&v1, f, &x.f2)?;
        flag(es.is_some());
        flag(es.is_some_and(|s| s.is_same(e)));
        Ok(())
    });
    attempt(|| {
        let p = fun_tool_par_v_on_e(&v1, e)?;
        flag(p.is_some());
        real(p.unwrap_or(-1.0));
        let mut q = -1.0;
        flag(fun_tool_par_e(e, par, e2, &mut q)?);
        real(q);
        Ok(())
    });
    attempt(|| {
        let mut a = Point2::new(-1.0, -1.0);
        flag(fun_tool_par_f(e, par, f, &mut a)?);
        pnt2d(a);
        flag(fun_tool_paron_ef(e, par, &x.f2, &mut a)?);
        pnt2d(a);
        Ok(())
    });
    attempt(|| {
        let bc = BRepAdaptorCurve::with_edge(e)?;
        dir(fun_tool_dir_c_on_curve(par, &bc)?);
        xyz(*fun_tool_tggeom_e_vec(par, e)?.xyz());
        flag(fun_tool_line_bac(&bc)?);
        flag(fun_tool_quad_edge(e)?);
        flag(fun_tool_quad_bac(&bc)?);
        flag(fun_tool_quad_face(f)?);
        Ok(())
    });
    attempt(|| {
        let (p, q) = fun_tool_find_pin_e(e)?;
        pnt(p);
        real(q);
        Ok(())
    });
}

/// The shape members of `tt_topology`.
fn topology_shapes(x: &EdgeFace, e2: &Edge) {
    let (e, f, par) = (&x.e, &x.f, x.par);
    attempt(|| {
        real(fun_tool_maxtol(&x.s)?);
        let mut t = 0.0;
        flag(fun_tool_maxtol_of_type(f, ShapeEnum::Edge, &mut t)?);
        real(t);
        Ok(())
    });
    attempt(|| {
        int(fun_tool_nbshapes(&x.s, ShapeEnum::Edge)?);
        let mut l: List<Shape> = List::default();
        fun_tool_shapes(f, ShapeEnum::Edge, &mut l)?;
        int(l.extent());
        Ok(())
    });
    attempt(|| {
        int(fun_tool_comparebndkole(f, &x.s)?);
        int(fun_tool_comparebndkole(e, f)?);
        Ok(())
    });
    attempt(|| {
        flag(fun_tool_same_ori(e, e)?);
        flag(fun_tool_same_ori(e, &topods::edge(&e.reversed())?)?);
        flag(fun_tool_haspc(e, f)?);
        Ok(())
    });
    attempt(|| {
        let mut so = false;
        flag(fun_tool_curves_so(e, e2, &mut so)?);
        flag(so);
        so = false;
        flag(fun_tool_curves_so_at(e, par, e, &mut so)?);
        flag(so);
        Ok(())
    });
    attempt(|| {
        let mut l: List<Shape> = List::default();
        l.append(x.f2.clone().into_shape());
        l.append(f.clone().into_shape());
        let fa = fun_tool_find_ancestor(&l, e)?;
        flag(fa.is_some());
        flag(fa.is_some_and(|fa| fa.is_same(&x.f2)));
        Ok(())
    });
    attempt(|| {
        xyz(*fun_tool_getgeomxx(f, e, par)?.xyz());
        let d = fun_tool_getxx(f, e, par)?;
        flag(d.is_some());
        dir(d.unwrap_or(Direction3::Z));
        Ok(())
    });
    attempt(|| {
        let d = fun_tool_getdxx(f, e, par)?;
        flag(d.is_some());
        // Natively the step is reset to (0, 0) first.
        xy(d.unwrap_or(Vector2::new(0.0, 0.0)).xy());
        Ok(())
    });
}

/// `tt_topology solid ie k t`.
pub fn topology_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let x = match edge_face(&mut r) {
        Ok(x) => x,
        Err(error) => return failed(error),
    };
    let e2 = other_edge(&x.f, &x.e).expect("other edge");
    topology_bounds(&x);
    topology_points(&x, &e2);
    topology_types(&x);
    topology_params(&x, &e2);
    topology_shapes(&x, &e2);
    end();
}
