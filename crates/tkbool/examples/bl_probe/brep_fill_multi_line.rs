//! BRepFill_MultiLine, BRepFill_ApproxSeewing and BRepFill_TrimSurfaceTool probe
//! operations, the Rust twin of tools/oracle/tkbool_probe_brepfill_multi_line.cxx (the
//! records are documented there).
use super::brep_fill_generator::failed;
use super::brep_fill_location_law::at;
use super::common::{Reader, attempt, end, flag, int, pnt, pnt2d, real};
use super::shapes::{TpShapes, tp_shapes};
use rustycad_tkbool::brep_fill::{
    BRepFillApproxSeewing, BRepFillMultiLine, BRepFillTrimSurfaceTool, MultiLineCurves,
};
use rustycad_tkbrep::topods::{self, Edge, Face};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkgeombase::app_cont::Function;
use rustycad_tkmath::gp::Point3;

/// `bl_curve3d`.
fn curve3d(c: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
    let Some(curve) = c.get() else {
        int(-1);
        return Ok(());
    };
    let (f, l) = (curve.first_parameter(), curve.last_parameter());
    real(f);
    real(l);
    for u in [f, 0.5 * (f + l), l] {
        pnt(curve.value(u)?);
    }
    Ok(())
}

/// `bl_curve2d`.
fn curve2d(c: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
    let Some(curve) = c.get() else {
        int(-1);
        return Ok(());
    };
    let (f, l) = (curve.first_parameter(), curve.last_parameter());
    real(f);
    real(l);
    for u in [f, 0.5 * (f + l), l] {
        pnt2d(curve.value(u)?);
    }
    Ok(())
}

/// The three curves of a multi-line.
fn curves(c: &MultiLineCurves) -> Result<(), Failure> {
    curve3d(&c.0)?;
    curve2d(&c.1)?;
    curve2d(&c.2)
}

/// The faces and edges of a request.
fn faces_and_edges(r: &mut Reader<'_>, t: &TpShapes) -> Result<(Face, Face, Edge, Edge), Failure> {
    let f1 = topods::face(&at(t, r.integer())?)?;
    let f2 = topods::face(&at(t, r.integer())?)?;
    let e1 = topods::edge(&at(t, r.integer())?)?;
    let e2 = topods::edge(&at(t, r.integer())?)?;
    Ok((f1, f2, e1, e2))
}

/// The p-curve of a program index.
fn pcurve_at(t: &TpShapes, index: i32) -> Handle<dyn Geom2dCurve> {
    t.pcurves.get(index as usize).cloned().unwrap_or_default()
}

/// The points, curves and approximation of a multi-line.
fn print_multi_line(ml: &BRepFillMultiLine, n: i32) {
    flag(ml.is_particular_case());
    int(ml.continuity() as i32);
    let (u1, u2) = (ml.first_parameter(), ml.last_parameter());
    real(u1);
    real(u2);
    for i in 0..n {
        let u = u1 + f64::from(i) * (u2 - u1) / f64::from(n - 1);
        attempt(|| {
            pnt2d(ml.value_on_f1(u)?);
            pnt2d(ml.value_on_f2(u)?);
            pnt(ml.value_3d(u)?);
            Ok(())
        });
    }
    attempt(|| curves(&ml.curves()?));
    attempt(|| {
        let app = BRepFillApproxSeewing::with_multi_line(ml)?;
        flag(app.is_done());
        curve3d(app.curve()?)?;
        curve2d(app.curve_on_f1()?)?;
        curve2d(app.curve_on_f2()?)
    });
}

/// `bf_multi_line program f1 f2 e1 e2 inv1 inv2 bis n`.
pub fn multi_line_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = match tp_shapes(&mut r) {
        Ok(t) => t,
        Err(error) => return failed(error),
    };
    let shapes = faces_and_edges(&mut r, &t);
    let (inv1, inv2) = (r.flag(), r.flag());
    let bis = pcurve_at(&t, r.integer());
    let n = r.integer();
    let ml = shapes
        .and_then(|(f1, f2, e1, e2)| BRepFillMultiLine::new(&f1, &f2, &e1, &e2, inv1, inv2, &bis));
    match ml {
        Ok(ml) => print_multi_line(&ml, n),
        Err(error) => return failed(error),
    }
    end();
}

/// `bf_trim_surface program bis f1 f2 e1 e2 inv1 inv2 ef1 ef2 x y pe u1 u2`.
pub fn trim_surface_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = match tp_shapes(&mut r) {
        Ok(t) => t,
        Err(error) => return failed(error),
    };
    let bis = pcurve_at(&t, r.integer());
    let shapes = faces_and_edges(&mut r, &t);
    let (inv1, inv2) = (r.flag(), r.flag());
    let ef = (at(&t, r.integer()), at(&t, r.integer()));
    let p = r.pnt2d();
    let pe = at(&t, r.integer());
    let (u1, u2) = (r.real(), r.real());
    let Ok((f1, f2, e1, e2)) = shapes else {
        return end();
    };
    let tool = BRepFillTrimSurfaceTool::new(&bis, &f1, &f2, &e1, &e2, inv1, inv2);
    attempt(|| {
        let mut pts: Sequence<Point3> = Sequence::default();
        tool.intersect_with(
            &topods::edge(&ef.0.clone()?)?,
            &topods::edge(&ef.1.clone()?)?,
            &mut pts,
        )?;
        int(pts.len() as i32);
        for q in pts.iter() {
            pnt(*q);
        }
        Ok(())
    });
    attempt(|| tool.is_on_face(p).map(flag));
    attempt(|| tool.proj_on(p, &topods::edge(&pe.clone()?)?).map(real));
    attempt(|| {
        let (c, cont) = tool.project(u1, u2)?;
        int(cont as i32);
        curves(&c)
    });
    end();
}
