//! TopOpeBRepTool_ShapeTool, PROJECT, GEOMETRY, FC2D, FTOL, HBoxTool and TOOL::UVISO
//! probe operations, the Rust twin of tools/oracle/tkbool_probe_topopebreptool_shapes.cxx
//! (the records are documented there).
use super::brep_fill_generator::failed;
use super::brep_fill_location_law::at;
use super::brep_fill_multi_line::curve2d;
use super::common::{Reader, attempt, dir, end, flag, int, pnt, pnt2d, real, text, xy};
use super::shapes::tp_shapes;
use rustycad_tkbool::top_ope_brep_tool::{
    PCurveOnFace, TopOpeBRepToolHBoxTool, TopOpeBRepToolShapeTool, fc2d_curve_on_surface,
    fc2d_curve_on_surface_on_edge, fc2d_editable_curve_on_surface, fc2d_has_c3d,
    fc2d_has_new_curve_on_surface, fc2d_has_old_curve_on_surface, fc2d_prepare,
    ftol_face_tolerances3d, fun_tool_bounds, fun_tool_closed, fun_tool_dir_c, fun_tool_line,
    fun_tool_ng_s, fun_tool_onapex, fun_tool_proj_ponbounded_f, fun_tool_proj_ponc2d,
    fun_tool_proj_pone, fun_tool_proj_ponf, fun_tool_quad, fun_tool_quad_surface, uviso_edge,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve2d;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Direction2, Direction3, Point2};

/// The shapes of a request.
fn shapes_of(r: &mut Reader<'_>, n: usize) -> Result<Vec<Shape>, Failure> {
    let t = tp_shapes(r)?;
    (0..n).map(|_| at(&t, r.integer())).collect()
}

/// The six coordinates of a box.
fn bounds(b: &BoundingBox3) -> Result<(), Failure> {
    b.get()?.into_iter().for_each(real);
    Ok(())
}

/// A p-curve with its range and tolerance.
fn pcurve_on_face(pc: &PCurveOnFace) -> Result<(), Failure> {
    curve2d(&pc.0)?;
    real(pc.1);
    real(pc.2);
    real(pc.3);
    Ok(())
}

/// The UV bounds and periodic adjustment of `tt_shape_tool`.
fn uv_members(f: &Shape, u: f64, v: f64) {
    attempt(|| {
        let mut out = (false, false, -1., -1., -1., -1.);
        TopOpeBRepToolShapeTool::uv_bounds_of_face(&topods::face(f)?, &mut out)?;
        flag(out.0);
        flag(out.1);
        [out.2, out.3, out.4, out.5].into_iter().for_each(real);
        Ok(())
    });
    attempt(|| {
        let (mut uu, mut vv) = (u, v);
        TopOpeBRepToolShapeTool::adjust_on_periodic(f, &mut uu, &mut vv)?;
        real(uu);
        real(vv);
        Ok(())
    });
}

/// `tt_shape_tool program s1 s2 e f par u v`.
pub fn shape_tool_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = match shapes_of(&mut r, 4) {
        Ok(s) => s,
        Err(error) => return failed(error),
    };
    let (s1, s2, e, f) = (&s[0], &s[1], &s[2], &s[3]);
    let (par, u, v) = (r.real(), r.real(), r.real());
    attempt(|| TopOpeBRepToolShapeTool::tolerance(s1).map(real));
    attempt(|| TopOpeBRepToolShapeTool::pnt(s1).map(pnt));
    attempt(|| TopOpeBRepToolShapeTool::shapes_same_oriented(s1, s2).map(flag));
    attempt(|| {
        let bs = TopOpeBRepToolShapeTool::basis_surface_of_face(&topods::face(f)?)?;
        text(bs.dynamic_type().map_or("", |t| t.name()).as_bytes());
        Ok(())
    });
    uv_members(f, u, v);
    attempt(|| TopOpeBRepToolShapeTool::closed(e, f).map(flag));
    attempt(|| TopOpeBRepToolShapeTool::periodize_parameter(par, e, f).map(real));
    attempt(|| {
        let (mut t, mut n, mut c) = (Direction3::Z, Direction3::Z, -1.);
        real(TopOpeBRepToolShapeTool::edge_data_of_edge(
            e, par, &mut t, &mut n, &mut c,
        )?);
        dir(t);
        dir(n);
        real(c);
        Ok(())
    });
    attempt(|| TopOpeBRepToolShapeTool::resolution3d_of_face(&topods::face(f)?, 1.0e-9).map(real));
    end();
}

/// `tt_project program e f x y z`.
pub fn project_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = match shapes_of(&mut r, 2) {
        Ok(s) => s,
        Err(error) => return failed(error),
    };
    let p = r.pnt();
    let ef = (|| Ok::<_, Failure>((topods::edge(&s[0])?, topods::face(&s[1])?)))();
    let (e, f) = match ef {
        Ok(v) => v,
        Err(error) => return failed(error),
    };
    attempt(|| {
        let (a, b) = fun_tool_bounds(&e)?;
        real(a);
        real(b);
        Ok(())
    });
    attempt(|| {
        let (mut pa, mut d) = (-1., -1.);
        flag(fun_tool_proj_pone(p, &e, &mut pa, &mut d)?);
        real(pa);
        real(d);
        Ok(())
    });
    attempt(|| {
        let (mut uv, mut d) = (Point2::new(-1., -1.), -1.);
        flag(fun_tool_proj_ponf(
            p,
            &f,
            &mut uv,
            &mut d,
            ExtFlag::MinMax,
            ExtAlgo::Grad,
        )?);
        pnt2d(uv);
        real(d);
        Ok(())
    });
    attempt(|| {
        let (mut uv, mut d) = (Point2::new(-1., -1.), -1.);
        flag(fun_tool_proj_ponbounded_f(p, &f, &mut uv, &mut d)?);
        pnt2d(uv);
        real(d);
        Ok(())
    });
    attempt(|| {
        let bc = BRepAdaptorCurve2d::with_edge_on_face(&e, &f)?;
        let (mut pa, mut d) = (-1., -1.);
        flag(fun_tool_proj_ponc2d(p, &bc, &mut pa, &mut d)?);
        real(pa);
        real(d);
        Ok(())
    });
    end();
}

/// `tt_geometry program e f u v`.
pub fn geometry_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = match shapes_of(&mut r, 2) {
        Ok(s) => s,
        Err(error) => return failed(error),
    };
    let uv = r.pnt2d();
    let geometry = (|| {
        let (c, a, b) = BRepTool::curve(&topods::edge(&s[0])?)?;
        Ok::<_, Failure>((c, 0.5 * (a + b), BRepTool::surface(&topods::face(&s[1])?)?))
    })();
    let (c, mid, surface) = match geometry {
        Ok(v) => v,
        Err(error) => return failed(error),
    };
    attempt(|| fun_tool_dir_c(mid, &c).map(dir));
    attempt(|| fun_tool_line(&c).map(flag));
    attempt(|| fun_tool_quad(&c).map(flag));
    attempt(|| fun_tool_ng_s(uv, &surface).map(dir));
    attempt(|| fun_tool_onapex(uv, &surface).map(flag));
    attempt(|| fun_tool_quad_surface(&surface).map(flag));
    attempt(|| {
        let (uc, up, vc, vp) = fun_tool_closed(&surface)?.unwrap_or((false, 0., false, 0.));
        flag(uc || vc);
        flag(uc);
        real(up);
        flag(vc);
        real(vp);
        Ok(())
    });
    end();
}

/// `tt_fc2d program s1 s2 e f ef`.
pub fn fc2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = match shapes_of(&mut r, 5) {
        Ok(s) => s,
        Err(error) => return failed(error),
    };
    let shapes = (|| {
        Ok::<_, Failure>((
            topods::edge(&s[2])?,
            topods::face(&s[3])?,
            topods::edge(&s[4])?,
        ))
    })();
    let (e, f, ef) = match shapes {
        Ok(v) => v,
        Err(error) => return failed(error),
    };
    fc2d_prepare(&s[0], &s[1]);
    attempt(|| fc2d_has_c3d(&e).map(flag));
    attempt(|| fc2d_has_old_curve_on_surface(&e, &f).map(|o| flag(o.is_some())));
    attempt(|| pcurve_on_face(&fc2d_curve_on_surface(&e, &f, false)?));
    flag(fc2d_has_new_curve_on_surface(&e, &f).is_some());
    attempt(|| pcurve_on_face(&fc2d_editable_curve_on_surface(&e, &f, false)?));
    attempt(|| {
        let (mut d, mut o) = (Direction2::X, Point2::new(0., 0.));
        let (iso, iu, iv) = uviso_edge(&e, &f, &mut d, &mut o)?;
        [iso, iu, iv].into_iter().for_each(flag);
        xy(d.xy());
        pnt2d(o);
        Ok(())
    });
    attempt(|| pcurve_on_face(&fc2d_curve_on_surface_on_edge(&e, &f, &ef, false)?));
    attempt(|| ftol_face_tolerances3d(&f, &f).map(real));
    end();
}

/// `tt_hbox program s ts`.
pub fn hbox_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = match shapes_of(&mut r, 1) {
        Ok(s) => s,
        Err(error) => return failed(error),
    };
    let ts = ShapeEnum::ALL[r.integer() as usize];
    let hbt = TopOpeBRepToolHBoxTool::new();
    attempt(|| hbt.add_boxes(&s[0], ts, ShapeEnum::Shape));
    int(hbt.extent());
    for i in 1..=hbt.extent() {
        attempt(|| bounds(&hbt.box_of_index(i)?));
        int(hbt.shape(i).map_or(0, |sh| hbt.index(&sh)));
    }
    attempt(|| hbt.box_of_index(hbt.extent() + 1).map(|_| ()));
    attempt(|| {
        let mut b = BoundingBox3::default();
        TopOpeBRepToolHBoxTool::compute_box_on_vertices(&s[0], &mut b)?;
        bounds(&b)
    });
    end();
}
