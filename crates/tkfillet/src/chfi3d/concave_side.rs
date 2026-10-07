// Rust translation of ChFi3d::ConcaveSide (ChFi3d.cxx) and ChFi3d_Coefficient
// (ChFi3d_Builder_0.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The concave side of two faces along their common edge, the choice of fillet that
//! the builders read.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorCurve2d, BRepAdaptorSurface};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Point2, Point3, Vector3};
use std::f64::consts::PI;

/// `ChFi3d_Coefficient(V3d, D1u, D1v, DU, DV)`: the components `(DU, DV)` of the
/// projection of `V3d` on the tangent plane spanned by `D1u` and `D1v`.
///
/// ```
/// use rustycad_tkfillet::chfi3d::coefficient;
/// use rustycad_tkmath::gp::Vector3;
/// let (du, dv) = coefficient(Vector3::new(2.0, 3.0, 5.0), Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0));
/// assert_eq!((du, dv), (2.0, 3.0));
/// ```
pub fn coefficient(v3d: Vector3, d1u: Vector3, d1v: Vector3) -> (f64, f64) {
    let aa = d1u.square_magnitude();
    let bb = d1u.dot(d1v);
    let cc = d1v.square_magnitude();
    let dd = d1u.dot(v3d);
    let ee = d1v.dot(v3d);
    let delta = aa * cc - bb * bb;
    ((dd * cc - ee * bb) / delta, (aa * ee - bb * dd) / delta)
}

/// One face of the edge: its surface, the point of the edge on it, the tangent of the
/// edge as the face orients it, and the surface derivatives with the oriented normal.
struct FaceSide<'a> {
    surface: &'a BRepAdaptorSurface,
    reversed: bool,
    p2d: Point2,
    tangent: Vector3,
    du: Vector3,
    dv: Vector3,
    normal: Vector3,
    point: Point3,
}

impl FaceSide<'_> {
    /// The derivatives at the point and the normal `DU ^ DV` (normalized when asked),
    /// reversed for a reversed face.
    fn evaluate(&mut self, normalize: bool) -> Result<(), Failure> {
        let d1 = self.surface.d1(self.p2d.x(), self.p2d.y())?;
        (self.point, self.du, self.dv) = (d1.point, d1.d1u, d1.d1v);
        self.set_normal(normalize)
    }

    fn set_normal(&mut self, normalize: bool) -> Result<(), Failure> {
        self.normal = self.du.crossed(self.dv);
        if normalize {
            self.normal.normalize()?;
        }
        if self.reversed {
            self.normal.reverse();
        }
        Ok(())
    }

    /// The direction into the face, `normal ^ tangent`.
    fn inward(&self) -> Vector3 {
        self.normal.crossed(self.tangent)
    }

    /// The derivatives pushed by the second derivatives toward `inward` (a forgotten
    /// regularity), and the normal of the pushed derivatives.
    fn push_second_order(&mut self, inward: Vector3) -> Result<(), Failure> {
        let d2 = self.surface.d2(self.p2d.x(), self.p2d.y())?;
        (self.point, self.du, self.dv) = (d2.point, d2.d1u, d2.d1v);
        self.du += if self.du.dot(inward) < 0.0 {
            -d2.d2u
        } else {
            d2.d2u
        };
        self.dv += if self.dv.dot(inward) < 0.0 {
            -d2.d2v
        } else {
            d2.d2v
        };
        self.set_normal(true)
    }

    /// The point moved by `inward` in the parameters of the face, and its normal
    /// (not normalized).
    fn shift(&mut self, inward: Vector3) -> Result<(), Failure> {
        let (u, v) = coefficient(inward, self.du, self.dv);
        self.p2d = Point2::new(self.p2d.x() + u, self.p2d.y() + v);
        self.evaluate(false)
    }
}

/// The orientations of the two faces toward the concave side, from their normals and
/// inward directions; `None` when the faces are tangent at the point.
fn transversal_sides(
    side1: &mut FaceSide<'_>,
    side2: &mut FaceSide<'_>,
) -> Option<(Orientation, Orientation)> {
    let (dint1, dint2) = (side1.inward(), side2.inward());
    // Not `ang > tol`: a NaN angle takes the tangent branch, as natively.
    let ang = side1.normal.cross_magnitude(side2.normal);
    if ang.partial_cmp(&(0.0001 * PI)) != Some(std::cmp::Ordering::Greater) {
        return None;
    }
    let mut or2 = Orientation::Forward;
    if side2.normal.dot(dint1) <= 0.0 {
        side2.normal.reverse();
        or2 = Orientation::Reversed;
    }
    let mut or1 = Orientation::Forward;
    if side1.normal.dot(dint2) <= 0.0 {
        side1.normal.reverse();
        or1 = Orientation::Reversed;
    }
    Some((or1, or2))
}

/// The tangent faces: by the second order when the faces turn back to back
/// (`None`, choice 10, when they stay tangent), else by the points moved into the faces.
fn tangent_sides(
    side1: &mut FaceSide<'_>,
    side2: &mut FaceSide<'_>,
) -> Result<Option<(Orientation, Orientation)>, Failure> {
    let (dint1, dint2) = (side1.inward(), side2.inward());
    if dint1.dot(dint2) < 0.0 {
        side1.push_second_order(dint1)?;
        side2.push_second_order(dint2)?;
        return Ok(transversal_sides(side1, side2));
    }
    // here it turns back, the points are taken in faces neither too close nor too far
    side1.shift(dint1)?;
    side2.shift(dint1)?;
    let vref = Vector3::between(side1.point, side2.point);
    let or1 = if side1.normal.dot(vref) < 0.0 {
        Orientation::Reversed
    } else {
        Orientation::Forward
    };
    let or2 = if side2.normal.dot(vref) > 0.0 {
        Orientation::Reversed
    } else {
        Orientation::Forward
    };
    Ok(Some((or1, or2)))
}

/// The tangent of the edge reversed when the face holds the edge reversed; `None` when
/// the face does not hold the edge.
fn tangent_in_face(e: &Edge, f: &Face, tangent: Vector3) -> Result<Option<Vector3>, Failure> {
    let mut exp = Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while exp.more() {
        if e.is_same(exp.current()) {
            let reversed = exp.current().orientation() == Orientation::Reversed;
            return Ok(Some(if reversed {
                tangent.reversed()
            } else {
                tangent
            }));
        }
        exp.next()?;
    }
    Ok(None)
}

/// The tangents of the edge in both faces and the edges whose pcurves are read (the
/// second one reversed on a seam); `None` when a face does not hold the edge.
fn edge_tangents(
    e: &Edge,
    f1: &Face,
    f2: &Face,
    tangent: Vector3,
) -> Result<Option<([Vector3; 2], Edge)>, Failure> {
    let mut e2 = e.clone();
    e2.set_orientation(Orientation::Forward);
    if f1.is_same(f2) && BRepTool::is_closed_on_face(e, f1)? {
        e2.set_orientation(Orientation::Reversed);
        return Ok(Some(([tangent, tangent.reversed()], e2)));
    }
    let Some(tg1) = tangent_in_face(e, f1, tangent)? else {
        return Ok(None);
    };
    let Some(tg2) = tangent_in_face(e, f2, tangent)? else {
        return Ok(None);
    };
    Ok(Some(([tg1, tg2], e2)))
}

/// The choice of fillet from the orientations and the turn of the normals around the
/// oriented tangent of the edge.
fn choice(or1: Orientation, or2: Orientation, turn: f64) -> i32 {
    let base = match (or1, or2) {
        (Orientation::Forward, Orientation::Forward) => 1,
        (Orientation::Forward, _) => 7,
        (_, Orientation::Forward) => 3,
        _ => 5,
    };
    if turn >= 0.0 { base + 1 } else { base }
}

/// `ChFi3d::ConcaveSide(S1, S2, E, Or1, Or2)`: `(choice, Or1, Or2)`, the orientations
/// of the faces toward the concave side and the choice of fillet (1 to 8); 0 when a
/// face does not hold the edge, 10 when the faces stay tangent (the face at the end is
/// in the extension of a base face).
///
/// ```no_run
/// # use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
/// # use rustycad_tkbrep::topods::Edge;
/// # fn f(s1: &BRepAdaptorSurface, s2: &BRepAdaptorSurface, e: &Edge) -> Result<(), rustycad_tkernel::standard::Failure> {
/// let (choice, or1, or2) = rustycad_tkfillet::chfi3d::concave_side(s1, s2, e)?;
/// # Ok(()) }
/// ```
pub fn concave_side(
    s1: &BRepAdaptorSurface,
    s2: &BRepAdaptorSurface,
    e: &Edge,
) -> Result<(i32, Orientation, Orientation), Failure> {
    let ce = BRepAdaptorCurve::with_edge(e)?;
    let par = 0.691254 * ce.first_parameter()? + 0.308746 * ce.last_parameter()?;
    let (_, mut tg_e) = ce.d1(par)?;
    tg_e.normalize()?;
    let tangent = tg_e;
    if e.orientation() == Orientation::Reversed {
        tg_e.reverse();
    }
    let (f1, f2) = (s1.face(), s2.face());
    let Some(([tg1, tg2], e2)) = edge_tangents(e, &f1, &f2, tangent)? else {
        return Ok((0, Orientation::Forward, Orientation::Forward));
    };
    let mut e1 = e.clone();
    e1.set_orientation(Orientation::Forward);
    let mut side1 = face_side(s1, &e1, &f1, par, tg1)?;
    let mut side2 = face_side(s2, &e2, &f2, par, tg2)?;
    side1.evaluate(true)?;
    side2.evaluate(true)?;
    let sides = match transversal_sides(&mut side1, &mut side2) {
        Some(sides) => Some(sides),
        None => tangent_sides(&mut side1, &mut side2)?,
    };
    let Some((or1, or2)) = sides else {
        return Ok((10, Orientation::Forward, Orientation::Forward));
    };
    let turn = side1.normal.crossed(side2.normal).dot(tg_e);
    Ok((choice(or1, or2, turn), or1, or2))
}

/// The side of the face at the parameter of the edge (evaluated by the caller once
/// both pcurves are read, as natively).
fn face_side<'a>(
    surface: &'a BRepAdaptorSurface,
    e: &Edge,
    f: &Face,
    par: f64,
    tangent: Vector3,
) -> Result<FaceSide<'a>, Failure> {
    let pcurve = BRepAdaptorCurve2d::with_edge_on_face(e, f)?;
    Ok(FaceSide {
        surface,
        reversed: f.orientation() == Orientation::Reversed,
        p2d: pcurve.value(par)?,
        tangent,
        du: Vector3::default(),
        dv: Vector3::default(),
        normal: Vector3::default(),
        point: Point3::default(),
    })
}
