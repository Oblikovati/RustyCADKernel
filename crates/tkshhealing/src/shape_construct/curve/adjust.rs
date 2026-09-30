use super::{ShapeConstructCurve, null_curve};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve, GeomLine};
use rustycad_tkmath::elclib::{line_parameter, line_parameter_2d, line_value, line_value_2d};
use rustycad_tkmath::gp::{Direction2, Direction3, Line2, Line3, Point2, Point3, Vector2, Vector3};

impl ShapeConstructCurve {
    /// `AdjustCurve(C3D, P1, P2, take1, take2)`: a B-spline gets its first pole at `p1`
    /// (when `take1`) and its last pole at `p2` (when `take2`); a line becomes the line
    /// through `p1` and `p2` (both are used), located at the projection of its old
    /// location; true for these two kinds, false for others, true when neither end is
    /// taken. Standard_ConstructionError when a line must pass through two equal points;
    /// `ShapeConstructCurve.adjust_curve(&c, p1, p2, true, true)?`.
    pub fn adjust_curve(
        &self,
        c3d: &Handle<dyn GeomCurve>,
        p1: Point3,
        p2: Point3,
        take1: bool,
        take2: bool,
    ) -> Result<bool, Failure> {
        if !take1 && !take2 {
            return Ok(true);
        }
        c3d.get().ok_or_else(|| null_curve("AdjustCurve"))?;
        adjust_3d(c3d, p1, p2, take1, take2)
    }
    /// `AdjustCurve2d(C2D, P1, P2, take1, take2)`: [`Self::adjust_curve`] for a p-curve
    /// (a 2D B-spline or line); `ShapeConstructCurve.adjust_curve_2d(&c, p1, p2, true, true)?`.
    pub fn adjust_curve_2d(
        &self,
        c2d: &Handle<dyn Geom2dCurve>,
        p1: Point2,
        p2: Point2,
        take1: bool,
        take2: bool,
    ) -> Result<bool, Failure> {
        if !take1 && !take2 {
            return Ok(true);
        }
        c2d.get().ok_or_else(|| null_curve("AdjustCurve2d"))?;
        adjust_2d(c2d, p1, p2, take1, take2)
    }
    /// `AdjustCurveSegment(C3D, P1, P2, U1, U2)`: a B-spline is segmented to
    /// `[max(U1, first), min(U2, last)]` (false when `U1 >= U2`) and gets its end poles at
    /// `p1` and `p2`; a line is adjusted as by `AdjustCurve` (U1 and U2 unused); false
    /// for other curves; `ShapeConstructCurve.adjust_curve_segment(&c, p1, p2, 0.0, 1.0)?`.
    pub fn adjust_curve_segment(
        &self,
        c3d: &Handle<dyn GeomCurve>,
        p1: Point3,
        p2: Point3,
        u1: f64,
        u2: f64,
    ) -> Result<bool, Failure> {
        c3d.get().ok_or_else(|| null_curve("AdjustCurveSegment"))?;
        let handle = c3d.cast::<GeomBSplineCurve>();
        match handle.get() {
            Some(spline) => adjust_segment(spline, p1, p2, u1, u2),
            None => adjust_line_3d(c3d, p1, p2),
        }
    }
}

fn adjust_3d(
    curve: &Handle<dyn GeomCurve>,
    p1: Point3,
    p2: Point3,
    take1: bool,
    take2: bool,
) -> Result<bool, Failure> {
    let handle = curve.cast::<GeomBSplineCurve>();
    let Some(spline) = handle.get() else {
        return adjust_line_3d(curve, p1, p2);
    };
    if take1 {
        spline.set_pole(1, p1)?;
    }
    if take2 {
        spline.set_pole(spline.nb_poles(), p2)?;
    }
    Ok(true)
}

/// The line branch ("P1 and P2 are both assumed relevant").
fn adjust_line_3d(curve: &Handle<dyn GeomCurve>, p1: Point3, p2: Point3) -> Result<bool, Failure> {
    let handle = curve.cast::<GeomLine>();
    let Some(line) = handle.get() else {
        return Ok(false);
    };
    line.set_lin(line_through(p1, p2, line.lin().location())?);
    Ok(true)
}

fn adjust_2d(
    curve: &Handle<dyn Geom2dCurve>,
    p1: Point2,
    p2: Point2,
    take1: bool,
    take2: bool,
) -> Result<bool, Failure> {
    let handle = curve.cast::<Geom2dBSplineCurve>();
    let Some(spline) = handle.get() else {
        return adjust_line_2d(curve, p1, p2);
    };
    if take1 {
        spline.set_pole(1, p1)?;
    }
    if take2 {
        spline.set_pole(spline.nb_poles(), p2)?;
    }
    Ok(true)
}

/// The line branch ("P1 and P2 are both assumed relevant").
fn adjust_line_2d(
    curve: &Handle<dyn Geom2dCurve>,
    p1: Point2,
    p2: Point2,
) -> Result<bool, Failure> {
    let handle = curve.cast::<Geom2dLine>();
    let Some(line) = handle.get() else {
        return Ok(false);
    };
    line.set_lin2d(line_through_2d(p1, p2, line.lin2d().location())?);
    Ok(true)
}

/// The line through `p1` and `p2`, located at the projection of `location` on it.
fn line_through(p1: Point3, p2: Point3, location: Point3) -> Result<Line3, Failure> {
    let dir = Direction3::from_vector(Vector3::between(p1, p2))?;
    let mut line = Line3::new(p1, dir);
    let param = line_parameter(line.position(), location);
    line.set_location(line_value(param, line.position()));
    Ok(line)
}

/// The same in 2D.
fn line_through_2d(p1: Point2, p2: Point2, location: Point2) -> Result<Line2, Failure> {
    let dir = Direction2::from_vector(Vector2::between(p1, p2))?;
    let mut line = Line2::new(p1, dir);
    let param = line_parameter_2d(line.position(), location);
    line.set_location(line_value_2d(param, line.position()));
    Ok(line)
}

fn adjust_segment(
    spline: &GeomBSplineCurve,
    p1: Point3,
    p2: Point3,
    u1: f64,
    u2: f64,
) -> Result<bool, Failure> {
    if u1 >= u2 {
        return Ok(false);
    }
    let first = u1.max(spline.first_parameter());
    let last = u2.min(spline.last_parameter());
    spline.segment(first, last, precision::p_confusion())?;
    spline.set_pole(1, p1)?;
    spline.set_pole(spline.nb_poles(), p2)?;
    Ok(true)
}
