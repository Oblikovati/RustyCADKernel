//! The ChFi3d_Builder_0 edges, common points, stripes, tolerances and solved points
//! (`fl_compute_arete`, `fl_fil_common_point`, `fl_b0_stripes`, `fl_b0_tols`, `fl_int_cs`,
//! `fl_computes_int_pc`, `fl_b0_inter2d`); the twin of the second half of
//! tools/oracle/tkfillet_probe_builder0.cxx.
use super::blend_func::{geom_guide, geom_surface};
use super::brep_blend::geom_curve2d_kind;
use super::builder0::curve2d;
use super::chfi_ds::segment;
use super::chfi3d::solid;
use super::common::{Reader, attempt, end, flag, int, pnt, pnt2d, real, xyz};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve2d;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkfillet::brep_blend::BRepBlendExtremity;
use rustycad_tkfillet::chfi_ds::{
    CommonPoint, FaceInterference, Spine, Stripe, SurfData, SurfDataHandle, shared,
};
use rustycad_tkfillet::chfi3d as b0;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dHVertex, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomBezierCurve, GeomCurve, GeomOffsetSurface, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkgeomalgo::int_surf::{IntSurfTransition, IntSurfTypeTrans};
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorHVertex;
use std::sync::{Arc, Mutex};

/// `fl_common_point`.
pub(super) fn common_point(cp: &CommonPoint) -> Result<(), Failure> {
    pnt(cp.point());
    flag(cp.has_vector());
    if cp.has_vector() {
        xyz(*cp.vector()?.xyz());
    }
    real(cp.parameter());
    flag(cp.is_vertex());
    if cp.is_vertex() {
        pnt(BRepTool::pnt(cp.vertex()?)?);
    }
    flag(cp.is_on_arc());
    if cp.is_on_arc() {
        real(cp.parameter_on_arc()?);
        int(cp.transition_on_arc()? as i32);
    }
    real(cp.tolerance());
    Ok(())
}

/// `fl_arete_point`.
fn arete_point(mode: i32, arc: &Edge, param: f64) -> CommonPoint {
    let mut cp = CommonPoint::new();
    if mode == 1 {
        cp.set_arc(1e-7, arc, param, Orientation::Forward);
    }
    if mode == 2 {
        cp.set_vertex(&topexp::first_vertex(arc, false));
    }
    cp
}

/// `fl_compute_arete`: see tools/oracle/tkfillet_probe_builder0.cxx.
pub fn compute_arete_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let s = geom_surface(r);
    let (uv1, uv2) = (r.pnt2d(), r.pnt2d());
    let (m1, pa1, m2, pa2) = (r.integer(), r.real(), r.integer(), r.real());
    let (iflag, tol3d, tol2d) = (r.integer(), r.real(), r.real());
    let c3d = geom_guide(r);
    let (pardeb, parfin) = (r.real(), r.real());
    let arc1 = segment(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        false,
    )
    .expect("arc");
    let arc2 = segment(
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(1.0, 1.0, 1.0),
        false,
    )
    .expect("arc");
    let (p1, p2) = (arete_point(m1, &arc1, pa1), arete_point(m2, &arc2, pa2));
    let c3d = if iflag == 0 { Handle::default() } else { c3d };
    attempt(|| {
        let mut arete = b0::Arete {
            c3d,
            pcurv: Handle::default(),
            pardeb,
            parfin,
            tolreached: -1.0,
        };
        b0::compute_arete(
            (&p1, uv1),
            (&p2, uv2),
            &s,
            &mut arete,
            (tol3d, tol2d),
            iflag,
        )?;
        [arete.tolreached, arete.pardeb, arete.parfin]
            .into_iter()
            .for_each(real);
        flag(arete.c3d.is_null());
        if let Some(c) = arete.c3d.get() {
            pnt(c.value(arete.pardeb)?);
            pnt(c.value(arete.parfin)?);
        }
        curve2d(&arete.pcurv);
        Ok(())
    });
    end();
}

/// The face `fi` of the solid and its edge `ei` (explorer orders, 1-based).
fn face_and_edge(
    s: &rustycad_tkbrep::topods::Shape,
    fi: i32,
    ei: i32,
) -> Result<(Face, Edge), Failure> {
    let mut ef = Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)?;
    for _ in 1..fi {
        ef.next()?;
    }
    let f = topods::face(ef.current())?;
    let mut ee = Explorer::new(&f, ShapeEnum::Edge, ShapeEnum::Shape)?;
    for _ in 1..ei {
        ee.next()?;
    }
    let e = topods::edge(ee.current())?;
    Ok((f, e))
}

/// The transition type of its native integer value.
fn type_trans(k: i32) -> IntSurfTypeTrans {
    match k {
        0 => IntSurfTypeTrans::In,
        1 => IntSurfTypeTrans::Out,
        2 => IntSurfTypeTrans::Touch,
        _ => IntSurfTypeTrans::Undecided,
    }
}

/// `fl_fil_common_point`: see tools/oracle/tkfillet_probe_builder0.cxx.
pub fn fil_common_point_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let s = solid(r).expect("probe solid");
    let (fi, ei, p) = (r.integer(), r.integer(), r.pnt());
    let (u, v, param, tol) = (r.real(), r.real(), r.real(), r.real());
    let (isv, isa, pa, hast, t) = (
        r.flag(),
        r.flag(),
        r.real(),
        r.flag(),
        Vector3::from(r.xyz()),
    );
    let (trans, start, cptol, big_tol) = (r.integer(), r.flag(), r.real(), r.real());
    let (f, e) = face_and_edge(&s, fi, ei).expect("face and edge");
    let arc = Handle::new(BRepAdaptorCurve2d::with_edge_on_face(&e, &f).expect("arc"));
    let mut ext = BRepBlendExtremity::on_surface(p, u, v, param, tol);
    if isv {
        let hv: Handle<dyn Adaptor3dHVertex> = Handle::new(BRepTopAdaptorHVertex::new(
            &topexp::first_vertex(&e, false),
            &arc,
        ))
        .cast();
        ext.set_vertex(&hv);
    }
    if isa {
        let a: Handle<dyn Adaptor2dCurve2d> = arc.clone().cast();
        ext.add_arc(
            &a,
            pa,
            IntSurfTransition::default(),
            IntSurfTransition::default(),
        );
    }
    if hast {
        ext.set_tangent(t);
    }
    let mut cp = CommonPoint::new();
    cp.set_tolerance(cptol);
    attempt(|| {
        b0::fil_common_point(&ext, type_trans(trans), start, &mut cp, big_tol)?;
        common_point(&cp)
    });
    end();
}

/// `fl_b0_stripes`: see tools/oracle/tkfillet_probe_builder0.cxx.
pub fn stripes_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let n = r.integer();
    let pts: Vec<Point3> = (0..n).map(|_| r.pnt()).collect();
    let (rev, nsd) = (r.integer(), r.integer());
    let mut spine = Spine::new();
    let edges: Vec<Edge> = pts
        .windows(2)
        .enumerate()
        .map(|(i, w)| segment(w[0], w[1], (rev >> i) & 1 == 1).expect("segment"))
        .collect();
    edges.iter().for_each(|e| spine.set_edges(e));
    let mut stripe = Stripe::new();
    *stripe.change_spine() = Some(Arc::new(Mutex::new(spine)));
    if nsd >= 0 {
        *stripe.change_set_of_surf_data() =
            Some(shared((0..nsd).map(|_| shared(SurfData::new())).collect()));
    }
    let st = shared(stripe);
    for e in &edges {
        for v in [
            topexp::first_vertex(e, false),
            topexp::last_vertex(e, false),
        ] {
            attempt(|| {
                b0::index_of_surf_data(&v, &st)
                    .map(|(index, sens)| [index, sens].into_iter().for_each(int))
            });
            attempt(|| {
                let (x, sens) = b0::edge_from_v1(&v, &st)?;
                flag(x.is_same(&edges[0]));
                flag(x.is_same(&edges[edges.len() - 1]));
                int(sens);
                Ok(())
            });
        }
    }
    end();
}

/// The 2d line of a probe request.
fn line2d(r: &mut Reader<'_>) -> Handle<dyn Geom2dCurve> {
    let (p, d) = (r.pnt2d(), r.dir2d());
    Handle::new(Geom2dLine::from_point(p, d)).cast()
}

/// The surface of `ChFi3d_Parameters`, offset when asked.
fn offset_surface(
    gs: &Handle<dyn GeomSurface>,
    off: f64,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    if off == 0.0 {
        return Ok(gs.clone());
    }
    Ok(Handle::new(GeomOffsetSurface::new(gs, off, false)?).cast())
}

/// `fl_b0_tols`: see tools/oracle/tkfillet_probe_builder0.cxx.
pub fn tols_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let (gs, tol2d) = (geom_surface(r), r.real());
    let (l1, l2) = (line2d(r), line2d(r));
    let (c, f, l) = (geom_guide(r), r.real(), r.real());
    let s = [r.integer(), r.integer(), r.integer()];
    let u = [
        [r.real(), r.real(), r.real()],
        [r.real(), r.real(), r.real()],
        [r.real(), r.real(), r.real()],
    ];
    let (t, p3d, off, q1, q2, bez) = (r.real(), r.pnt(), r.real(), r.pnt(), r.pnt(), r.flag());
    let mut poles = [r.pnt(), r.pnt(), r.pnt()];
    let hs: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(&gs).expect("adaptor")).cast();
    attempt(|| b0::conv_tol2d_to_tol3d(&hs, tol2d).map(real));
    attempt(|| {
        let tc: Handle<dyn GeomCurve> =
            Handle::new(GeomTrimmedCurve::new(&c, f, l, true, true)?).cast();
        b0::eval_tol_reached((&hs, &l1), (&hs, &l2), &tc).map(real)
    });
    (0..4).for_each(|k| int(b0::trsf_trans(type_trans(k)) as i32));
    int(b0::search_pivot(&s, &u, t));
    attempt(|| {
        let (pu, pv) = b0::parameters(&offset_surface(&gs, off)?, p3d)?;
        real(pu);
        real(pv);
        Ok(())
    });
    attempt(|| {
        let tc: Handle<dyn GeomCurve> = if bez {
            Handle::new(GeomBezierCurve::new(&Array1::from_slice(&mut poles))?).cast()
        } else {
            c.clone()
        };
        let g = b0::trim_curve(&tc, q1, q2)?;
        let g = g.get().ok_or_else(|| Failure::new("null trimmed curve"))?;
        real(g.first_parameter());
        real(g.last_parameter());
        Ok(())
    });
    end();
}

/// `fl_int_cs`: see tools/oracle/tkfillet_probe_builder0.cxx.
pub fn int_cs_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let gs = geom_surface(r);
    let bounds = (r.real(), r.real(), r.real(), r.real());
    let (c, f, l, wc) = (geom_guide(r), r.real(), r.real(), r.real());
    attempt(|| {
        let hs: Handle<dyn Adaptor3dSurface> =
            Handle::new(GeomAdaptorSurface::with_bounds(&gs, bounds)?).cast();
        let hc: Handle<dyn Adaptor3dCurve> =
            Handle::new(GeomAdaptorCurve::with_range(&c, f, l)?).cast();
        let found = b0::int_cs(&hs, &hc, wc)?;
        flag(found.is_some());
        let (p, w) = found.unwrap_or((rustycad_tkmath::gp::Point2::new(-3.0, -3.0), wc));
        pnt2d(p);
        real(w);
        Ok(())
    });
    end();
}

/// `fl_computes_int_pc`: see tools/oracle/tkfillet_probe_builder0.cxx.
pub fn computes_int_pc_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let mut sides = Vec::new();
    for _ in 0..2 {
        let s = Handle::new(GeomAdaptorSurface::with_surface(&geom_surface(r)).expect("adaptor"));
        let pc = line2d(r);
        let (f, l) = (r.real(), r.real());
        let mut fi = FaceInterference::new();
        fi.set_interference(0, Orientation::Forward, &Handle::default(), &pc);
        fi.set_first_parameter(f);
        fi.set_last_parameter(l);
        sides.push((fi, s));
    }
    let (u1, u2) = (r.real(), r.real());
    attempt(|| {
        let (a, b, p) = b0::computes_int_pc(
            (&sides[0].0, &sides[0].1),
            (&sides[1].0, &sides[1].1),
            (u1, u2),
        )?;
        real(a);
        real(b);
        pnt(p);
        Ok(())
    });
    attempt(|| {
        let (a, b, _) = b0::computes_int_pc(
            (&sides[0].0, &sides[0].1),
            (&sides[1].0, &sides[1].1),
            (u1, u2),
        )?;
        real(a);
        real(b);
        Ok(())
    });
    end();
}

/// `fl_b0_inter2d`: see tools/oracle/tkfillet_probe_builder0.cxx.
pub fn inter2d_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let (c1, f1, l1) = (geom_curve2d_kind(r), r.real(), r.real());
    let (c2, f2, l2) = (geom_curve2d_kind(r), r.real(), r.real());
    let tol = r.real();
    attempt(|| {
        let (a1, a2) = (
            Geom2dAdaptorCurve::with_range(&c1, f1, l1)?,
            Geom2dAdaptorCurve::with_range(&c2, f2, l2)?,
        );
        let inter = GInter::between(&a1, &a2, tol, tol)?;
        flag(inter.is_done());
        int(inter.nb_points()?);
        int(inter.nb_segments()?);
        flag(b0::has_transversal_intersection(&inter)?);
        let (mut s1, mut s2) = (false, true);
        flag(b0::has_common_endpoint(
            &inter,
            (&a1, &a2),
            tol,
            (&mut s1, &mut s2),
        )?);
        flag(s1);
        flag(s2);
        Ok(())
    });
    end();
}

/// `fl_trace_curve`.
fn trace_curve(r: &mut Reader<'_>) -> Handle<dyn Geom2dCurve> {
    let c = geom_curve2d_kind(r);
    let (trim, a, b) = (r.flag(), r.real(), r.real());
    if !trim {
        return c;
    }
    Handle::new(Geom2dTrimmedCurve::new(&c, a, b, true, true).expect("trimmed trace")).cast()
}

/// `fl_trace_data`.
fn trace_data(c: &Handle<dyn Geom2dCurve>, jf: i32, f: f64, l: f64) -> SurfDataHandle {
    let mut fd = SurfData::new();
    let fi = fd.change_interference(jf);
    fi.set_interference(0, Orientation::Forward, c, &Handle::default());
    fi.set_first_parameter(f);
    fi.set_last_parameter(l);
    shared(fd)
}

/// `fl_int_traces`: see tools/oracle/tkfillet_probe_builder0.cxx.
pub fn int_traces_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let c1 = trace_curve(r);
    let (f1, l1, jf1, pref1, sens1) = (r.real(), r.real(), r.integer(), r.real(), r.integer());
    let same = r.flag();
    let c2 = trace_curve(r);
    let c2 = if same { c1.clone() } else { c2 };
    let (f2, l2, jf2, pref2, sens2) = (r.real(), r.real(), r.integer(), r.real(), r.integer());
    let (reference, check, enlarge) = (r.pnt2d(), r.flag(), r.flag());
    let (fd1, fd2) = (trace_data(&c1, jf1, f1, l1), trace_data(&c2, jf2, f2, l2));
    attempt(|| {
        let found = b0::int_traces(
            (&fd1, jf1, pref1, sens1),
            (&fd2, jf2, pref2, sens2),
            (reference, check),
            enlarge,
        )?;
        flag(found.is_some());
        let (p1, p2) = found.unwrap_or((-7.0, -7.0));
        real(p1);
        real(p2);
        Ok(())
    });
    end();
}
