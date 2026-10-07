// Rust translation of the `transform` and `d2transform` commands of
// Draw/TKDraw/DrawTrSurf/DrawTrSurf.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `translate`, `rotate`, `pmirror`, `lmirror`, `smirror`, `pscale` and their `2d`
//! forms.
//!
//! The commands read their numbers from the end of the arguments; every argument up
//! to the last one not consumed is a variable name. Native quirk kept: `pmirror` and
//! `pscale` do not consume their point coordinates, so those are also treated (and
//! echoed) as names.
use super::CommandEntry;
use crate::draw::{DrawInterpretor, DrawSession};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{
    Axis1, Axis2d, Direction2, Direction3, Plane3, Point2, Point3, Transform2, Transform3, Vector2,
    Vector3,
};
use std::f64::consts::PI;

/// The transformation commands.
pub(super) fn commands() -> Vec<CommandEntry> {
    vec![
        (
            "translate",
            "translate name [names...] dx dy dz".into(),
            transform,
        ),
        (
            "rotate",
            "rotate name [names...] x y z dx dy dz angle".into(),
            transform,
        ),
        ("pmirror", "pmirror name [names...] x y z".into(), transform),
        (
            "lmirror",
            "lmirror name [names...] x y z dx dy dz".into(),
            transform,
        ),
        (
            "smirror",
            "smirror name [names...] x y z dx dy dz".into(),
            transform,
        ),
        ("pscale", "pscale name [names...] x y z s".into(), transform),
        (
            "2dtranslate",
            "translate name [names...] dx dy".into(),
            d2transform,
        ),
        (
            "2drotate",
            "rotate name [names...] x y dx dy  angle".into(),
            d2transform,
        ),
        (
            "2dpmirror",
            "pmirror name [names...] x y".into(),
            d2transform,
        ),
        (
            "2dlmirror",
            "lmirror name [names...] x y dx dy".into(),
            d2transform,
        ),
        (
            "2dpscale",
            "pscale name [names...] x y s".into(),
            d2transform,
        ),
    ]
}

/// Reads numbers backwards from the argument list.
struct Arguments<'a> {
    session: &'a DrawSession,
    args: &'a [&'a str],
    /// The index of the last argument not consumed (`last`).
    last: usize,
}

impl Arguments<'_> {
    /// `Draw::Atof(a[index])`.
    fn real(&self, index: usize) -> f64 {
        self.session.atof(self.args[index])
    }

    /// The three numbers ending at `last` (not consumed).
    fn triple(&self) -> (f64, f64, f64) {
        let l = self.last;
        (self.real(l - 2), self.real(l - 1), self.real(l))
    }

    /// The two numbers ending at `last` (not consumed).
    fn pair(&self) -> (f64, f64) {
        let l = self.last;
        (self.real(l - 1), self.real(l))
    }
}

/// The 3D transformation of `a[0]`, reading from the end; `None` when too few
/// arguments remain (the command then returns 1).
fn transformation_3d(args: &mut Arguments<'_>) -> Result<Option<Transform3>, Failure> {
    let mut trsf = Transform3::default();
    let command = args.args[0];
    if command == "pscale" {
        let scale = args.real(args.last);
        args.last -= 1;
        if args.last < 4 {
            return Ok(None);
        }
        let (x, y, z) = args.triple();
        trsf.set_scale(Point3::new(x, y, z), scale)?;
    } else if command.get(1..) == Some("mirror") {
        if args.last < 4 {
            return Ok(None);
        }
        if !mirror_3d(args, &mut trsf)? {
            return Ok(None);
        }
    } else if command == "translate" {
        if args.last < 4 {
            return Ok(None);
        }
        let (x, y, z) = args.triple();
        args.last -= 3;
        trsf.set_translation(Vector3::new(x, y, z));
    } else if command == "rotate" {
        if args.last < 8 {
            return Ok(None);
        }
        let angle = args.real(args.last) * (PI / 180.0);
        args.last -= 1;
        let (dx, dy, dz) = args.triple();
        let direction = Direction3::new(dx, dy, dz)?;
        args.last -= 3;
        let (x, y, z) = args.triple();
        args.last -= 3;
        trsf.set_rotation(Axis1::new(Point3::new(x, y, z), direction), angle)?;
    }
    Ok(Some(trsf))
}

/// The `pmirror`/`lmirror`/`smirror` cases; false when too few arguments remain.
fn mirror_3d(args: &mut Arguments<'_>, trsf: &mut Transform3) -> Result<bool, Failure> {
    let (px, py, pz) = args.triple();
    let kind = args.args[0].as_bytes()[0];
    if kind == b'p' {
        trsf.set_mirror_point(Point3::new(px, py, pz));
        return Ok(true);
    }
    args.last -= 3;
    if args.last < 4 {
        return Ok(false);
    }
    let (ox, oy, oz) = args.triple();
    let origin = Point3::new(ox, oy, oz);
    args.last -= 3;
    let direction = Direction3::new(px, py, pz)?;
    if kind == b'l' {
        trsf.set_mirror_axis(Axis1::new(origin, direction));
    } else if kind == b's' {
        let plane = Plane3::new(origin, direction)?;
        trsf.set_mirror_plane(plane.position().ax2()?);
    }
    Ok(true)
}

/// `translate|rotate|pmirror|lmirror|smirror|pscale name... numbers`: transforms the
/// curves and surfaces in place and replaces the points, echoing each name.
fn transform(session: &DrawSession, di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    if a.len() < 5 {
        return Ok(1);
    }
    let mut args = Arguments {
        session,
        args: a,
        last: a.len() - 1,
    };
    let Some(trsf) = transformation_3d(&mut args)? else {
        return Ok(1);
    };
    for name in &a[1..=args.last] {
        let geometry = session.get_geometry(name);
        if let Some(geometry) = geometry.get() {
            geometry.transform(trsf)?;
            session.repaint();
        } else if let Some(point) = session.get_point(name) {
            session.set_point(name, point.transformed(trsf))?;
        }
        di.append(*name)?.append(" ")?;
    }
    Ok(0)
}

/// The 2D transformation of `a[0]`, reading from the end; `None` when too few
/// arguments remain.
fn transformation_2d(args: &mut Arguments<'_>) -> Result<Option<Transform2>, Failure> {
    let mut trsf = Transform2::default();
    match args.args[0] {
        "2dpscale" => {
            let scale = args.real(args.last);
            args.last -= 1;
            if args.last < 3 {
                return Ok(None);
            }
            let (x, y) = args.pair();
            trsf.set_scale(Point2::new(x, y), scale);
        }
        "2dpmirror" | "2dlmirror" => {
            if args.last < 3 || !mirror_2d(args, &mut trsf)? {
                return Ok(None);
            }
        }
        "2dtranslate" => {
            if args.last < 3 {
                return Ok(None);
            }
            let (x, y) = args.pair();
            args.last -= 2;
            trsf.set_translation(Vector2::new(x, y));
        }
        "2drotate" => {
            if args.last < 4 {
                return Ok(None);
            }
            let angle = args.real(args.last) * (PI / 180.0);
            args.last -= 1;
            let (x, y) = args.pair();
            args.last -= 2;
            trsf.set_rotation(Point2::new(x, y), angle);
        }
        _ => {}
    }
    Ok(Some(trsf))
}

/// The `2dpmirror`/`2dlmirror` cases; false when too few arguments remain.
fn mirror_2d(args: &mut Arguments<'_>, trsf: &mut Transform2) -> Result<bool, Failure> {
    let (px, py) = args.pair();
    if args.args[0] == "2dpmirror" {
        trsf.set_mirror_point(Point2::new(px, py));
        return Ok(true);
    }
    args.last -= 2;
    if args.last < 3 {
        return Ok(false);
    }
    let (ox, oy) = args.pair();
    args.last -= 2;
    let direction = Direction2::new(px, py)?;
    trsf.set_mirror_axis(Axis2d::new(Point2::new(ox, oy), direction));
    Ok(true)
}

/// `2dtranslate|2drotate|2dpmirror|2dlmirror|2dpscale name... numbers`: transforms
/// the 2D curves in place and replaces the 2D points, echoing each name.
fn d2transform(session: &DrawSession, di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    if a.len() < 4 {
        return Ok(1);
    }
    let mut args = Arguments {
        session,
        args: a,
        last: a.len() - 1,
    };
    let Some(trsf) = transformation_2d(&mut args)? else {
        return Ok(1);
    };
    for name in &a[1..=args.last] {
        let curve = session.get_curve_2d(name);
        if let Some(curve) = curve.get() {
            curve.transform(trsf)?;
            session.repaint();
        } else if let Some(point) = session.get_point_2d(name) {
            session.set_point_2d(name, point.transformed(trsf))?;
        }
        di.append(*name)?.append(" ")?;
    }
    Ok(0)
}
