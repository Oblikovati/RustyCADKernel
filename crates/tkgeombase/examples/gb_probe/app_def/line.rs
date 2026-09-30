//! The `ad_line` requests: AppDef_MultiLine and AppDef_MyLineTool.
use super::super::app_par_curves::{dump, read_line};
use super::super::common::{self as p, Reader, attempt, end};
use super::{array, mpc};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::app_def::{MultiLine, MultiPointConstraint};
use rustycad_tkgeombase::app_par_curves::LineTool;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

/// `gb_ad_read_line`: kind 0 the default line, 1 (NbMult) and SetValue of an AppDef
/// line's points, 2 (tabMultiP), 3 (tabP3d), 4 (tabP2d).
fn read_ad_line(r: &mut Reader<'_>) -> Result<MultiLine, Failure> {
    let kind = r.integer();
    if kind == 0 {
        return Ok(MultiLine::new());
    }
    if kind == 1 {
        let n = r.integer();
        let source = read_line(r);
        let line = MultiLine::with_nb_multi_points(n)?;
        for i in 1..=n.min(source.nb_multi_points()?) {
            line.set_value(i, &source.value(i)?)?;
        }
        return Ok(line);
    }
    let lower = r.integer();
    if kind == 2 {
        let source = read_line(r);
        let points: Vec<MultiPointConstraint> = (1..=source.nb_multi_points()?)
            .map(|i| source.value(i))
            .collect::<Result<_, _>>()?;
        return Ok(MultiLine::from_multi_points(&array(lower, &points)));
    }
    let n = r.integer();
    if kind == 3 {
        let points: Vec<Point3> = (0..n).map(|_| r.pnt()).collect();
        return Ok(MultiLine::from_points(&array(lower, &points)));
    }
    let points: Vec<Point2> = (0..n).map(|_| r.pnt2d()).collect();
    Ok(MultiLine::from_points2d(&array(lower, &points)))
}

fn print_array<T>(a: &Array1<'_, T>, print: impl Fn(&T)) {
    p::int(a.lower());
    p::int(a.upper());
    a.iter().for_each(print);
}

// A `lower..lower + n - 1` array of `fill`.
fn sentinels<T: Clone + Default>(lower: i32, n: i32, fill: T) -> Array1<'static, T> {
    array(lower, &vec![fill; n.max(0) as usize])
}

/// `gb_ad_tool_read`: `i overload lower n3 n2`; kind 0 Value, 1 Tangency, 2 Curvature.
fn tool_read(r: &mut Reader<'_>, line: &MultiLine, kind: i32) -> Result<(), Failure> {
    let (i, overload, lower, n3, n2) = (
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
    );
    if kind == 0 {
        let mut a = sentinels(lower, n3, Point3::new(-7., -7., -7.));
        let mut b = sentinels(lower, n2, Point2::new(-7., -7.));
        match overload {
            0 => line.value_3d(i, &mut a)?,
            1 => line.value_2d(i, &mut b)?,
            _ => LineTool::value(line, i, &mut a, &mut b)?,
        }
        print_array(&a, |v| p::pnt(*v));
        print_array(&b, |v| p::pnt2d(*v));
        return Ok(());
    }
    let mut a = sentinels(lower, n3, Vector3::new(-7., -7., -7.));
    let mut b = sentinels(lower, n2, Vector2::new(-7., -7.));
    let found = match (kind, overload) {
        (1, 0) => line.tangency_3d(i, &mut a)?,
        (1, 1) => line.tangency_2d(i, &mut b)?,
        (1, _) => line.tangency(i, &mut a, &mut b)?,
        (_, 0) => line.curvature_3d(i, &mut a)?,
        (_, 1) => line.curvature_2d(i, &mut b)?,
        _ => line.curvature(i, &mut a, &mut b)?,
    };
    p::flag(found);
    print_array(&a, |v| p::vec(*v));
    print_array(&b, |v| p::vec2d(*v));
    Ok(())
}

fn line_command(r: &mut Reader<'_>, line: &mut MultiLine) -> Result<(), Failure> {
    let code = r.integer();
    match code {
        1 => line.nb_multi_points().map(p::int),
        2 => line.nb_points().map(p::int),
        3 => {
            let i = r.integer();
            let source = read_line(r);
            line.set_value(i, &source.value(1)?)
        }
        4 => mpc(&line.value(r.integer())?),
        5 => {
            let mut text = String::new();
            line.dump(&mut text)?;
            dump(&text);
            Ok(())
        }
        6 => {
            p::int(line.first_point());
            Ok(())
        }
        7 => {
            p::int(line.last_point()?);
            p::int(line.nb_p3d()?);
            p::int(line.nb_p2d()?);
            Ok(())
        }
        8..=10 => tool_read(r, line, code - 8),
        11 => {
            let i1 = r.integer();
            line.what_status(i1, r.integer()).map(|s| p::int(s.code()))
        }
        12 => {
            let (i1, i2) = (r.integer(), r.integer());
            let other = line.make_ml_between(i1, i2, r.integer())?;
            other.nb_multi_points().map(p::int)
        }
        13 => {
            let (i1, i2) = (r.integer(), r.integer());
            let other = line.make_ml_one_more_point(i1, i2, r.integer())?;
            p::flag(other.is_some());
            Ok(())
        }
        _ => {
            let copy = line.clone();
            let i = r.integer();
            let source = read_line(r);
            attempt(|| copy.set_value(i, &source.value(1)?));
            attempt(|| mpc(&line.value(i)?));
            Ok(())
        }
    }
}

pub fn line_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let mut line = read_ad_line(&mut r)?;
        let count = r.integer();
        for _ in 0..count {
            attempt(|| line_command(&mut r, &mut line));
        }
        Ok(())
    });
    end();
}
