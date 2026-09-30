//! ShapeUpgrade probe operations, the Rust twin of
//! tools/oracle/tkshhealing_probe_shapeupgrade.cxx (which documents the requests).
use super::brep_lib_common::shape;
use super::common::{Reader, attempt, end, flag, int, pnt, pnt2d, real, show, text};
use super::sh_common::{curve, curve2d, shapes};
use super::shapes::tp_same_index;
use super::wire_data::sh_shape;
use rustycad_tkernel::ncollection::{Array1, HSequence};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkshhealing::shape_build::ShapeBuildReShape;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tkshhealing::shape_upgrade::{
    ShapeUpgrade, ShapeUpgradeRemoveLocations, ShapeUpgradeSplitCurve, ShapeUpgradeTool, SplitCurve,
};

/// `nk (knot mult)...`.
fn knots(r: &mut Reader<'_>) -> (Vec<f64>, Vec<i32>) {
    let n = r.integer();
    (0..n).map(|_| (r.real(), r.integer())).unzip()
}

/// `su_bspline` in 3d: `degree periodic n (x y z w)... nk (knot mult)...`.
fn bspline3d(r: &mut Reader<'_>) -> Result<GeomBSplineCurve, Failure> {
    let (degree, periodic, n) = (r.integer(), r.flag(), r.integer());
    let (poles, weights): (Vec<Point3>, Vec<f64>) = (0..n).map(|_| (r.pnt(), r.real())).unzip();
    let (k, m) = knots(r);
    GeomBSplineCurve::with_weights(
        &Array1::from_shared(&poles, 1),
        &Array1::from_shared(&weights, 1),
        &Array1::from_shared(&k, 1),
        &Array1::from_shared(&m, 1),
        degree,
        periodic,
    )
}

/// `su_bspline` in 2d: `degree periodic n (u v w)... nk (knot mult)...`.
fn bspline2d(r: &mut Reader<'_>) -> Result<Geom2dBSplineCurve, Failure> {
    let (degree, periodic, n) = (r.integer(), r.flag(), r.integer());
    let (poles, weights): (Vec<Point2>, Vec<f64>) = (0..n).map(|_| (r.pnt2d(), r.real())).unzip();
    let (k, m) = knots(r);
    Geom2dBSplineCurve::with_weights(
        &Array1::from_shared(&poles, 1),
        &Array1::from_shared(&weights, 1),
        &Array1::from_shared(&k, 1),
        &Array1::from_shared(&m, 1),
        degree,
        periodic,
    )
}

/// `su_piece` of a 3d piece.
fn piece3d(bs: &GeomBSplineCurve) -> Result<(), Failure> {
    int(bs.degree());
    flag(bs.is_rational());
    int(bs.nb_knots());
    for i in 1..=bs.nb_knots() {
        real(bs.knot(i)?);
        int(bs.multiplicity(i)?);
    }
    int(bs.nb_poles());
    for i in 1..=bs.nb_poles() {
        pnt(bs.pole(i)?);
        real(bs.weight(i)?);
    }
    Ok(())
}

/// `su_piece` of a 2d piece.
fn piece2d(bs: &Geom2dBSplineCurve) -> Result<(), Failure> {
    int(bs.degree());
    flag(bs.is_rational());
    int(bs.nb_knots());
    for i in 1..=bs.nb_knots() {
        real(bs.knot(i)?);
        int(bs.multiplicity(i)?);
    }
    int(bs.nb_poles());
    for i in 1..=bs.nb_poles() {
        pnt2d(bs.pole(i)?);
        real(bs.weight(i)?);
    }
    Ok(())
}

fn split3d(r: &mut Reader<'_>, null: bool) -> Result<(), Failure> {
    let bs = if null {
        Handle::default()
    } else {
        Handle::new(bspline3d(r)?)
    };
    attempt(|| {
        let result = ShapeUpgrade::c0_bspline_to_sequence_of_c1_bspline_curve(&bs)?;
        flag(result.is_some());
        flag(result.is_none());
        let Some(sequence) = result else {
            return Ok(());
        };
        sequence.sequence(|items| {
            int(items.length());
            items.iter().try_for_each(|item| {
                let piece = item.cast::<GeomBSplineCurve>();
                piece3d(piece.get().expect("B-spline piece"))?;
                curve(&item.clone().cast::<dyn GeomCurve>());
                Ok(())
            })
        })
    });
    if let Some(input) = bs.get() {
        flag(input.is_periodic());
        int(input.nb_poles());
    }
    Ok(())
}

fn split2d(r: &mut Reader<'_>, null: bool) -> Result<(), Failure> {
    let bs = if null {
        Handle::default()
    } else {
        Handle::new(bspline2d(r)?)
    };
    attempt(|| {
        let result = ShapeUpgrade::c0_bspline2d_to_sequence_of_c1_bspline_curve(&bs)?;
        flag(result.is_some());
        flag(result.is_none());
        let Some(sequence) = result else {
            return Ok(());
        };
        sequence.sequence(|items| {
            int(items.length());
            items.iter().try_for_each(|item| {
                let piece = item.cast::<Geom2dBSplineCurve>();
                piece2d(piece.get().expect("B-spline piece"))?;
                curve2d(&item.clone().cast::<dyn Geom2dCurve>());
                Ok(())
            })
        })
    });
    if let Some(input) = bs.get() {
        flag(input.is_periodic());
        int(input.nb_poles());
    }
    Ok(())
}

/// `su_c0split dim null (B-spline)`.
pub fn c0split_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let dim = r.integer();
    let null = r.flag();
    let done = if dim == 3 {
        split3d(&mut r, null)
    } else {
        split2d(&mut r, null)
    };
    done.expect("probe B-spline record");
    end();
}

fn tool_op(r: &mut Reader<'_>, tool: &ShapeUpgradeTool, op: i32) {
    match op {
        0 => tool.set_precision(r.real()),
        1 => tool.set_min_tolerance(r.real()),
        2 => tool.set_max_tolerance(r.real()),
        3 => real(tool.limit_tolerance(r.real())),
        4 => {
            let other = Handle::new(ShapeUpgradeTool::new());
            other.set_precision(r.real());
            other.set_min_tolerance(r.real());
            other.set_max_tolerance(r.real());
            if r.flag() {
                other.set_context(&Handle::new(ShapeBuildReShape::new()));
            }
            tool.set(&other).expect("probe tool");
            flag(tool.context() == other.context());
        }
        _ => tool.set_context(&Handle::new(ShapeBuildReShape::new())),
    }
}

/// `su_tool nops (op)...`.
pub fn tool_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let tool = Handle::new(ShapeUpgradeTool::new());
    for _ in 0..r.integer() {
        let op = r.integer();
        tool_op(&mut r, &tool, op);
    }
    real(tool.precision());
    real(tool.min_tolerance());
    real(tool.max_tolerance());
    flag(tool.context().is_null());
    text(tool.dynamic_type().map_or("null", |t| t.name()).as_bytes());
    end();
}

/// `su_removeloc (prelude) program nops (op)...`.
pub fn removeloc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = shapes(&mut r).expect("probe shape program");
    let rl = Handle::new(ShapeUpgradeRemoveLocations::new());
    for _ in 0..r.integer() {
        match r.integer() {
            0 => {
                rl.set_remove_level(ShapeEnum::ALL[r.integer() as usize]);
                int(rl.remove_level() as i32);
            }
            1 => {
                let s = sh_shape(&t, r.integer());
                show(rl.remove(&s), flag);
                shape(&rl.get_result()).expect("probe digest");
            }
            2 => {
                let m = rl.modified_shape(&sh_shape(&t, r.integer()));
                int(tp_same_index(&t, &m));
                shape(&m).expect("probe digest");
            }
            3 => rl.get_modified_shapes_map(|map| {
                int(map.len() as i32);
                t.shapes.iter().for_each(|s| flag(map.seek(s).is_some()));
            }),
            _ => shape(&rl.get_result()).expect("probe digest"),
        }
    }
    end();
}

fn split_op(r: &mut Reader<'_>, sc: &ShapeUpgradeSplitCurve, op: i32) -> Result<(), Failure> {
    match op {
        0 => {
            let first = r.real();
            sc.init(first, r.real());
            Ok(())
        }
        1 => {
            let values = Handle::new(HSequence::default());
            for _ in 0..r.integer() {
                values.append(r.real());
            }
            sc.set_split_values(&values)
        }
        2 => sc.set_split_values(&Handle::default()),
        3 => sc.set_split_values(&sc.split_values()),
        4 => sc.perform(r.flag()),
        5 => sc.compute(),
        _ => sc.build(r.flag()),
    }
}

/// `su_splitcurve nops (op)...`.
pub fn splitcurve_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let sc = Handle::new(ShapeUpgradeSplitCurve::new());
    for _ in 0..r.integer() {
        let op = r.integer();
        split_op(&mut r, &sc, op).expect("probe split operation");
    }
    let values = sc.split_values();
    flag(values.is_null());
    if let Some(values) = values.get() {
        values.sequence(|items| {
            int(items.length());
            items.iter().copied().for_each(real);
        });
    }
    ShapeExtendStatus::ALL
        .iter()
        .for_each(|&k| flag(sc.status_flag(k)));
    text(sc.dynamic_type().map_or("null", |t| t.name()).as_bytes());
    end();
}
