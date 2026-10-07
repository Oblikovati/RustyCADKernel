//! TopOpeBRepTool_GeomTool and TopOpeBRepTool_CurveTool probe operations, the Rust twin
//! of tools/oracle/tkbool_probe_topopebreptool_curve_tool.cxx (the records are
//! documented there).
use super::brep_fill_generator::failed;
use super::brep_fill_location_law::at;
use super::brep_fill_multi_line::{curve2d, curve3d};
use super::common::{Reader, attempt, end, flag, int, real};
use super::shapes::{TpShapes, tp_shapes};
use rustycad_tkbool::top_ope_brep_tool::{
    SectionCurves, TopOpeBRepToolCurveTool, TopOpeBRepToolOutCurveType,
};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkmath::gp::{Point2, Point3};

/// `bl_poles3d`.
fn poles3d(c: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
    let b: Handle<GeomBSplineCurve> = c.clone().cast();
    let Some(curve) = b.get() else {
        int(-1);
        return Ok(());
    };
    int(curve.nb_poles());
    int(curve.degree());
    curve3d(c)
}

/// `bl_poles2d`.
fn poles2d(c: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
    let b: Handle<Geom2dBSplineCurve> = c.clone().cast();
    let Some(curve) = b.get() else {
        int(-1);
        return Ok(());
    };
    int(curve.nb_poles());
    int(curve.degree());
    curve2d(c)
}

/// The curve tool of a request: APPROX, the asked curves, tolerances and point count.
fn tool_of(r: &mut Reader<'_>) -> TopOpeBRepToolCurveTool {
    let mut ct = TopOpeBRepToolCurveTool::with_type(TopOpeBRepToolOutCurveType::Approx);
    let (c3d, pc1, pc2) = (r.flag(), r.flag(), r.flag());
    let gt = ct.change_geom_tool();
    gt.define(TopOpeBRepToolOutCurveType::Approx, c3d, pc1, pc2);
    let (tol3d, tol2d) = (r.real(), r.real());
    gt.set_tolerances(tol3d, tol2d);
    gt.set_nb_pnt_max(r.integer());
    ct
}

/// The B-splines of degree 1 through the points of a request.
fn input_curves(r: &mut Reader<'_>) -> Result<SectionCurves, Failure> {
    let n = r.integer();
    let mut p: Array1<'static, Point3> =
        Array1::with_bounds(1, n).map_err(|_| Failure::out_of_range("bl_probe: points"))?;
    let mut p1: Array1<'static, Point2> =
        Array1::with_bounds(1, n).map_err(|_| Failure::out_of_range("bl_probe: points"))?;
    let mut p2: Array1<'static, Point2> =
        Array1::with_bounds(1, n).map_err(|_| Failure::out_of_range("bl_probe: points"))?;
    for i in 1..=n {
        (p[i], p1[i], p2[i]) = (r.pnt(), r.pnt2d(), r.pnt2d());
    }
    Ok(SectionCurves {
        c3d: TopOpeBRepToolCurveTool::make_bspline1_from_pnt(&p)?,
        pc1: TopOpeBRepToolCurveTool::make_bspline1_from_pnt2d(&p1)?,
        pc2: TopOpeBRepToolCurveTool::make_bspline1_from_pnt2d(&p2)?,
    })
}

/// The three curves of a section.
fn section(c: &SectionCurves) -> Result<(), Failure> {
    poles3d(&c.c3d)?;
    poles2d(&c.pc1)?;
    poles2d(&c.pc2)
}

/// `tt_make_curves program f1 f2 c3d pc1 pc2 tol3d tol2d nbpntmax parmin parmax n
/// (x y z u1 v1 u2 v2)...`.
pub fn make_curves_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = match tp_shapes(&mut r) {
        Ok(t) => t,
        Err(error) => return failed(error),
    };
    let (i1, i2) = (r.integer(), r.integer());
    let ct = tool_of(&mut r);
    let (parmin, parmax) = (r.real(), r.real());
    let made = (|| {
        let (f1, f2) = (at(&t, i1)?, at(&t, i2)?);
        let mut input = input_curves(&mut r)?;
        let made = ct.make_curves(parmin, parmax, &mut input, &f1, &f2)?;
        Ok((made, input))
    })();
    let (made, input) = match made {
        Ok(v) => v,
        Err(error) => return failed(error),
    };
    flag(made.is_some());
    // The native reached tolerances stay at their initial -1 when MakeCurves fails.
    let tols = made
        .as_ref()
        .map_or((-1., -1.), |m| (m.tol_reached3d, m.tol_reached2d));
    let curves = made.map(|m| m.curves).unwrap_or_default();
    attempt(|| section(&curves));
    real(tols.0);
    real(tols.1);
    attempt(|| section(&input));
    end();
}

/// The program curve at `index`.
fn curve_at(t: &TpShapes, index: i32) -> Result<Handle<dyn GeomCurve>, Failure> {
    t.curves
        .get(index as usize)
        .cloned()
        .ok_or_else(|| Failure::out_of_range("bl_probe: curve index"))
}

/// `tt_pcurve_on_face program face curve first last`.
pub fn pcurve_on_face_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = match tp_shapes(&mut r) {
        Ok(t) => t,
        Err(error) => return failed(error),
    };
    let (fi, ci) = (r.integer(), r.integer());
    let (first, last) = (r.real(), r.real());
    let (f, c) = match at(&t, fi).and_then(|f| curve_at(&t, ci).map(|c| (f, c))) {
        Ok(v) => v,
        Err(error) => return failed(error),
    };
    attempt(|| TopOpeBRepToolCurveTool::is_projectable(&f, &c).map(flag));
    attempt(|| {
        let (pc, tol) = TopOpeBRepToolCurveTool::make_pcurve_on_face(&f, &c, first, last)?;
        curve2d(&pc)?;
        real(tol);
        Ok(())
    });
    end();
}
