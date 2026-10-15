//! TopOpeBRepTool_TOOL, TOPOLOGY, makeTransition and mkTondgE probe operations, the Rust
//! twin of tools/oracle/tkbool_probe_topopebreptool_tool.cxx (the records are documented
//! there).
use super::brep_fill_generator::failed;
use super::common::{Reader, attempt, dir, end, flag, int, pnt2d, real, xy, xyz};
use rustycad_tkbool::top_ope_brep_tool::*;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag};
use rustycad_tkmath::gp::{Direction3, Point2, Point3, Transform3, Vector2, Vector3};
use rustycad_tkmath::toploc::Location;
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCone, BRepPrimAPIMakeCylinder, BRepPrimAPIMakePrism,
    BRepPrimAPIMakeSphere, BRepPrimAPIMakeTorus,
};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeShape,
};
use std::f64::consts::PI;

/// The shape of a prim maker.
fn built(mut maker: impl BRepBuilderAPIMakeShape) -> Result<Shape, Failure> {
    Ok(maker.shape()?.clone())
}

/// The L-shaped prism of `bl_tt_solid`.
fn l_prism(a: f64, b: f64, c: f64, d: f64) -> Result<Shape, Failure> {
    let mut poly = BRepBuilderAPIMakePolygon::new();
    for (x, y) in [(0.0, 0.0), (a, 0.0), (a, d), (d, d), (d, b), (0.0, b)] {
        poly.add_point(Point3::new(x, y, 0.0))?;
    }
    poly.close()?;
    let f = BRepBuilderAPIMakeFace::from_wire(&poly.wire()?, false)?.face()?;
    built(BRepPrimAPIMakePrism::new(
        &f.into_shape(),
        Vector3::new(0.0, 0.0, c),
        false,
        true,
    )?)
}

/// `bl_tt_solid`.
pub(crate) fn solid(r: &mut Reader<'_>) -> Result<Shape, Failure> {
    let kind = r.integer();
    let (a, b, c, d) = (r.real(), r.real(), r.real(), r.real());
    let t = Vector3::from(r.xyz());
    let s = match kind {
        0 => built(BRepPrimAPIMakeBox::from_sizes(a, b, c)?)?,
        1 => built(BRepPrimAPIMakeCylinder::new(a, b))?,
        2 => built(BRepPrimAPIMakeCone::new(a, b, c)?)?,
        3 => built(BRepPrimAPIMakeSphere::new(a)?)?,
        4 => built(BRepPrimAPIMakeTorus::new(a, b)?)?,
        _ => l_prism(a, b, c, d)?,
    };
    let mut tr = Transform3::default();
    tr.set_translation(t);
    s.moved(&Location::new(tr), false)
}

/// `BlTtEdgeFace`.
pub(crate) struct EdgeFace {
    pub(crate) s: Shape,
    pub(crate) e: Edge,
    pub(crate) f: Face,
    pub(crate) f2: Face,
    pub(crate) par: f64,
    pub(crate) uv: Point2,
}

/// `bl_tt_edge_face`.
pub(crate) fn edge_face(r: &mut Reader<'_>) -> Result<EdgeFace, Failure> {
    let s = solid(r)?;
    let (ie, k, t) = (r.integer(), r.integer(), r.real());
    let mut m = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(&s, ShapeEnum::Edge, ShapeEnum::Face, &mut m)?;
    let e = topods::edge(m.find_key(ie as usize)?)?;
    let l = m.find_from_index(ie as usize)?;
    let (first, last) = (
        l.first().cloned().unwrap_or_default(),
        l.last().cloned().unwrap_or_default(),
    );
    let (f, f2) = if k == 1 { (first, last) } else { (last, first) };
    let (f, f2) = (topods::face(&f)?, topods::face(&f2)?);
    let (a, b) = BRepTool::range(&e)?;
    let par = a + t * (b - a);
    let pc = BRepTool::curve_on_face(&e, &f)?;
    let uv = match pc.pcurve.get() {
        Some(c) => c.value(par)?,
        None => Point2::new(0.0, 0.0),
    };
    Ok(EdgeFace {
        s,
        e,
        f,
        f2,
        par,
        uv,
    })
}

/// `bl_tt_other_edge`.
pub(crate) fn other_edge(f: &Face, e: &Edge) -> Result<Edge, Failure> {
    for c in Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if !c.is_same(e) {
            return topods::edge(&c);
        }
    }
    Ok(e.clone())
}

/// The geometric tangent of the edge (zero when there is none).
pub(crate) fn tangent(par: f64, e: &Edge) -> Result<Vector3, Failure> {
    Ok(tggeom_e(par, e)?.unwrap_or_default())
}

/// The oriented normal of the face (Z when it fails).
pub(crate) fn normal(uv: Point2, f: &Face) -> Result<(bool, Direction3), Failure> {
    let mut n = Direction3::Z;
    let ok = nt(uv, f, &mut n)?;
    Ok((ok, n))
}

/// The orientation, closure and vertex members of `tt_tool`.
fn tool_shapes(x: &EdgeFace) {
    let (e, f, par) = (&x.e, &x.f, x.par);
    attempt(|| {
        int(TopOpeBRepToolTOOL::oriin_sor(e, f, true)?);
        int(TopOpeBRepToolTOOL::oriin_sor(e, f, false)?);
        int(TopOpeBRepToolTOOL::oriin_sorclosed(e, f)?);
        Ok(())
    });
    attempt(|| {
        flag(TopOpeBRepToolTOOL::closed_e(e)?.is_some());
        flag(TopOpeBRepToolTOOL::closed_s(f)?);
        flag(TopOpeBRepToolTOOL::is_closing_e(e, f)?);
        flag(TopOpeBRepToolTOOL::is_closing_e_in(e, f, f)?);
        Ok(())
    });
    attempt(|| {
        real(TopOpeBRepToolTOOL::par_e(1, e)?);
        real(TopOpeBRepToolTOOL::par_e(2, e)?);
        int(TopOpeBRepToolTOOL::oriin_sor(
            &TopOpeBRepToolTOOL::vertex(1, e)?.into_shape(),
            e,
            true,
        )?);
        Ok(())
    });
    attempt(|| {
        int(TopOpeBRepToolTOOL::on_boundary(par, e)?);
        int(TopOpeBRepToolTOOL::on_boundary(
            TopOpeBRepToolTOOL::par_e(1, e)?,
            e,
        )?);
        Ok(())
    });
    attempt(|| {
        let mut ov = -1;
        let found = TopOpeBRepToolTOOL::tg_inside(&TopOpeBRepToolTOOL::vertex(1, e)?, e, &mut ov)?;
        flag(found.is_some());
        xyz(*found.unwrap_or_default().xyz());
        int(ov);
        Ok(())
    });
    attempt(|| TopOpeBRepToolTOOL::try_ori_e_in_f(par, e, f).map(int));
}

/// The normal and curvature members of `tt_tool`.
fn tool_normals(x: &EdgeFace) {
    let (e, f, par, uv) = (&x.e, &x.f, x.par, x.uv);
    attempt(|| {
        let n = TopOpeBRepToolTOOL::ng_app(par, e, f, 1.0e-6)?;
        flag(n.is_some());
        dir(n.unwrap_or(Direction3::Z));
        Ok(())
    });
    attempt(|| {
        let n = TopOpeBRepToolTOOL::try_ng_app(par, e, f, 1.0e-6)?;
        flag(n.is_some());
        dir(n.unwrap_or(Direction3::Z));
        Ok(())
    });
    attempt(|| {
        flag(TopOpeBRepToolTOOL::is_quad_edge(e)?);
        flag(TopOpeBRepToolTOOL::is_quad_face(f)?);
        Ok(())
    });
    attempt(|| {
        let (ok, n) = normal(uv, f)?;
        flag(ok);
        dir(n);
        let (okc, cv) = TopOpeBRepToolTOOL::curv_e(e, par, n)?;
        flag(okc);
        real(cv);
        Ok(())
    });
    attempt(|| {
        let tg = tggeom_e(par, e)?;
        flag(tg.is_some());
        let d = Direction3::from_xyz(*tg.unwrap_or_default().xyz())?;
        let (ok, cv, direct) = TopOpeBRepToolTOOL::curv_f(f, uv, d, false)?;
        flag(ok);
        real(cv);
        flag(direct);
        Ok(())
    });
    attempt(|| {
        let xx = TopOpeBRepToolTOOL::xx(uv, f, par, e)?;
        flag(xx.is_some());
        dir(xx.unwrap_or(Direction3::Z));
        Ok(())
    });
}

/// The matter and state members of `tt_tool`.
fn tool_matter(x: &EdgeFace) {
    let (e, f, par, uv) = (&x.e, &x.f, x.par, x.uv);
    attempt(|| {
        let ang = TopOpeBRepToolTOOL::matter_faces(f, &x.f2, e, par, 1.0e-6)?;
        flag(ang.is_some());
        real(ang.unwrap_or(-1.0));
        Ok(())
    });
    attempt(|| {
        let ang = TopOpeBRepToolTOOL::matter_k_ptg(f, &x.f2, e)?;
        flag(ang.is_some());
        real(ang.unwrap_or(-1.0));
        Ok(())
    });
    attempt(|| {
        let p = BRepAdaptorCurve::with_edge(e)?
            .value(par)?
            .translated(Vector3::new(0.1, 0.2, 0.3));
        let mut puv = Point2::new(-1.0, -1.0);
        let (ok, st) = TopOpeBRepToolTOOL::getstp3d_f(p, f, &mut puv)?;
        flag(ok);
        pnt2d(puv);
        int(st as i32);
        Ok(())
    });
    attempt(|| {
        let (ou, ov) = TopOpeBRepToolTOOL::stuv_f(uv, f)?;
        int(ou);
        int(ov);
        flag(TopOpeBRepToolTOOL::out_uv_bounds(uv, f)?);
        real(TopOpeBRepToolTOOL::tol_uv(f, 1.0e-7)?);
        real(TopOpeBRepToolTOOL::tol_p(e, f)?);
        Ok(())
    });
    attempt(|| {
        let on = TopOpeBRepToolTOOL::edge_on_face(par, e, uv, f)?;
        flag(on.is_some());
        flag(on.unwrap_or(false));
        Ok(())
    });
    attempt(|| {
        let mut a = Point2::new(-1.0, -1.0);
        flag(TopOpeBRepToolTOOL::uv_app(f, e, par, 0.1, &mut a)?);
        pnt2d(a);
        Ok(())
    });
    attempt(|| {
        let duv = TopOpeBRepToolTOOL::getduv(f, uv, tangent(par, e)?, 0.1)?;
        flag(duv.is_some());
        xy(duv.unwrap_or(rustycad_tkmath::gp::Direction2::X).xy());
        Ok(())
    });
}

/// The UV members of `tt_tool`.
fn tool_uv(x: &EdgeFace) {
    let (e, f, par, uv) = (&x.e, &x.f, x.par, x.uv);
    attempt(|| {
        let mut p = -1.0;
        flag(TopOpeBRepToolTOOL::par_iso(uv, e, f, &mut p)?);
        real(p);
        Ok(())
    });
    attempt(|| {
        let (p, d) = TopOpeBRepToolTOOL::par_e2d(uv, e, f)?;
        flag(true);
        real(p);
        real(d);
        Ok(())
    });
    attempt(|| {
        let pc = BRepTool::curve_on_face(e, f)?;
        let c = TopOpeBRepToolC2DF::new(&pc.pcurve, pc.first, pc.last, 1.0e-7, f);
        xy(TopOpeBRepToolTOOL::tg2d(1, e, &c)?.xy());
        xy(TopOpeBRepToolTOOL::tg2d_app(1, e, &c, 0.2)?.xy());
        xy(TopOpeBRepToolTOOL::try_tg2d_app(2, e, &c, 0.2)?.xy());
        flag(TopOpeBRepToolTOOL::ison_clo_c2df(
            &c,
            true,
            0.0,
            2.0 * PI,
            1.0e-7,
        )?);
        pnt2d(TopOpeBRepToolTOOL::uvf(par, &c)?);
        Ok(())
    });
    attempt(|| {
        let mut l: List<Shape> = List::default();
        flag(TopOpeBRepToolTOOL::split_e(e, &mut l)?);
        int(l.extent());
        Ok(())
    });
    attempt(|| {
        let tg = tangent(par, e)?;
        let (_, n) = normal(uv, f)?;
        let nv = Vector3::from(n);
        real(TopOpeBRepToolTOOL::matter_vec(tg, nv, nv.crossed(tg))?);
        real(TopOpeBRepToolTOOL::matter_2d(
            Vector2::new(1.0, 0.0),
            Vector2::new(par.cos(), par.sin()),
        )?);
        Ok(())
    });
}

/// `tt_tool solid ie k t`.
pub fn tool_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let x = match edge_face(&mut r) {
        Ok(x) => x,
        Err(error) => return failed(error),
    };
    tool_shapes(&x);
    tool_normals(&x);
    tool_matter(&x);
    tool_uv(&x);
    end();
}

/// Two solids of a request.
fn two_solids(r: &mut Reader<'_>) -> Result<(Shape, Shape), Failure> {
    let s1 = solid(r)?;
    Ok((s1, solid(r)?))
}

/// The shapes of a type in MapShapes order.
fn mapped(s: &Shape, t: ShapeEnum) -> Result<IndexedMapOfShape, Failure> {
    let mut m = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(s, t, &mut m)?;
    Ok(m)
}

/// The projection UV of a point on a face ((0, 0) when it fails).
fn proj_uv(p: Point3, f: &Face) -> Result<Point2, Failure> {
    let (mut uv, mut d) = (Point2::new(0.0, 0.0), 0.0);
    fun_tool_proj_ponf(p, f, &mut uv, &mut d, ExtFlag::MinMax, ExtAlgo::Grad)?;
    Ok(uv)
}

/// A state pair.
fn states(found: Option<(State, State)>) {
    flag(found.is_some());
    let (b, a) = found.unwrap_or((State::Unknown, State::Unknown));
    int(b as i32);
    int(a as i32);
}

/// `tt_make_transition solid1 solid2 ie fi parE factor ies parES`.
pub fn make_transition_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (s1, s2) = match two_solids(&mut r) {
        Ok(s) => s,
        Err(error) => return failed(error),
    };
    let (ie, fi, pe, fac, ies, pes) = (
        r.integer(),
        r.integer(),
        r.real(),
        r.real(),
        r.integer(),
        r.real(),
    );
    let (me1, mf2, me2) = (
        mapped(&s1, ShapeEnum::Edge).expect("edges"),
        mapped(&s2, ShapeEnum::Face).expect("faces"),
        mapped(&s2, ShapeEnum::Edge).expect("edges"),
    );
    let e = topods::edge(me1.find_key(ie as usize).expect("edge")).expect("edge");
    let fs = topods::face(mf2.find_key(fi as usize).expect("face")).expect("face");
    let mut mt = TopOpeBRepToolMakeTransition::new();
    attempt(|| {
        let uv = proj_uv(BRepAdaptorCurve::with_edge(&e)?.value(pe)?, &fs)?;
        let (a, b) = BRepTool::range(&e)?;
        flag(mt.initialize(&e, (a, b, pe), &fs, uv, fac)?);
        flag(mt.is_t2d());
        Ok(())
    });
    if ies > 0 {
        attempt(|| {
            flag(mt.set_rest(&topods::edge(me2.find_key(ies as usize)?)?, pes)?);
            flag(mt.has_rest());
            Ok(())
        });
    }
    attempt(|| mt.mk_t2don_e().map(states));
    attempt(|| mt.mk_t3on_e().map(states));
    attempt(|| {
        let (ok, b, a) = mt.mk_t3dproj()?;
        flag(ok);
        int(b as i32);
        int(a as i32);
        Ok(())
    });
    attempt(|| {
        let (ok, b, a) = mt.mk_ton_e()?;
        flag(ok);
        int(b as i32);
        int(a as i32);
        real(mt.getfactor());
        Ok(())
    });
    end();
}

/// A degenerated-edge transition.
fn dg_transition(ok: bool, (k, p1, p2): DgTransition) {
    flag(ok);
    int(k);
    real(p1);
    real(p2);
}

/// `tt_mktondge solid1 solid2 ie fi`.
pub fn mktondge_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (s1, s2) = match two_solids(&mut r) {
        Ok(s) => s,
        Err(error) => return failed(error),
    };
    let (ie, fi) = (r.integer(), r.integer());
    let mf2 = mapped(&s2, ShapeEnum::Face).expect("faces");
    let mut m = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(&s1, ShapeEnum::Edge, ShapeEnum::Face, &mut m)
        .expect("ancestors");
    let dge = topods::edge(m.find_key(ie as usize).expect("edge")).expect("edge");
    let f = topods::face(
        m.find_from_index(ie as usize)
            .expect("faces")
            .first()
            .expect("face"),
    )
    .expect("face");
    let fi_ = topods::face(mf2.find_key(fi as usize).expect("face")).expect("face");
    let mut mk = TopOpeBRepToolMkTondgE::new();
    attempt(|| {
        let uv = proj_uv(BRepTool::pnt(&topexp::first_vertex(&dge, false))?, &fi_)?;
        flag(mk.initialize(&dge, &f, uv, &fi_)?);
        flag(mk.is_t2d());
        Ok(())
    });
    attempt(|| {
        let mut out = (-1, -1.0, -1.0);
        let ok = mk.mk_ton_e(&mut out)?;
        dg_transition(ok, out);
        Ok(())
    });
    attempt(|| {
        let (n, l) = mk.get_all_rest()?;
        int(n);
        for ei in l.iter() {
            attempt(|| {
                let mut out = (-1, -1.0, -1.0);
                let ok = mk.mk_ton_e_on(&topods::edge(ei)?, &mut out)?;
                dg_transition(ok, out);
                Ok(())
            });
        }
        Ok(())
    });
    end();
}
