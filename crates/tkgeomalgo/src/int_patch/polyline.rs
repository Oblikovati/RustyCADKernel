// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_PolyLine.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_PolyLine`: the 2d polygon of a walking or restriction line on one surface
//! (lbr 12 June: deflection of the lines; msv 13.03.2002: deflection of the walking
//! lines, `Error()` returns it).
use super::IntPatchIType;
use super::failures::domain_error;
use super::point_line::IntPatchPointLine;
use super::polygo::{IntPatchPolygo, IntPatchPolygoBase, polygo_polygon2d};
use super::rline::IntPatchRLine;
use super::wline::IntPatchWLine;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Point2, Xy};

/// `INITDEFLE`: the initial deflection, `Precision::PConfusion() * 100`.
fn init_defle() -> f64 {
    precision::p_confusion() * 100.0
}

/// `IntPatch_PolyLine`: the polygon of the points of a line on the first or the second
/// surface, with its deflection.
///
/// ```
/// use rustycad_tkgeomalgo::int_patch::{IntPatchPolygo, IntPatchPolyLine};
/// let polygon = IntPatchPolyLine::new();
/// assert_eq!(polygon.error(), 1e-9 * 100.0); // Precision::PConfusion() * 100.
/// assert!(polygon.nb_points().is_err());
/// ```
#[derive(Clone, Default)]
pub struct IntPatchPolyLine {
    base: IntPatchPolygoBase,
    // `typ` is uninitialized natively until a line is set.
    typ: Option<IntPatchIType>,
    onfirst: bool,
    wpoly: Handle<IntPatchWLine>,
    rpoly: Handle<IntPatchRLine>,
}

impl IntPatchPolyLine {
    /// `IntPatch_PolyLine()`: the deflection 1e-7. `IntPatchPolyLine::new()`.
    pub fn new() -> Self {
        Self::with_deflection(init_defle())
    }

    /// `IntPatch_PolyLine(InitDefle)`. `IntPatchPolyLine::with_deflection(1e-6)`.
    pub fn with_deflection(init_defle: f64) -> Self {
        Self {
            base: IntPatchPolygoBase::new(init_defle),
            ..Self::default()
        }
    }

    /// `SetWLine(OnFirst, Line)`: the polygon of a walking line; computes its deflection
    /// and box. `polygon.set_w_line(true, &line)?`.
    pub fn set_w_line(
        &mut self,
        on_first: bool,
        line: &Handle<IntPatchWLine>,
    ) -> Result<(), Failure> {
        (self.typ, self.wpoly, self.onfirst) =
            (Some(IntPatchIType::Walking), line.clone(), on_first);
        self.prepare()
    }

    /// `SetRLine(OnFirst, Line)`: the polygon of a restriction line.
    /// `polygon.set_r_line(false, &line)?`.
    pub fn set_r_line(
        &mut self,
        on_first: bool,
        line: &Handle<IntPatchRLine>,
    ) -> Result<(), Failure> {
        (self.typ, self.rpoly, self.onfirst) =
            (Some(IntPatchIType::Restriction), line.clone(), on_first);
        self.prepare()
    }

    /// `ResetError()`: back to the initial deflection 1e-7. `polygon.reset_error()`.
    pub fn reset_error(&mut self) {
        self.base.error = init_defle();
    }

    fn is_closed(&self) -> bool {
        false
    }

    /// The line of the polygon: Standard_DomainError before a line is set (the native
    /// read of an uninitialized type), Standard_NullObject for a null line.
    fn line(&self) -> Result<&dyn IntPatchPointLine, Failure> {
        let member = "IntPatch_PolyLine::NbPoints";
        let found: Option<&dyn IntPatchPointLine> = match self.typ {
            Some(IntPatchIType::Walking) => self.wpoly.get().map(|l| l as &dyn IntPatchPointLine),
            Some(_) => self.rpoly.get().map(|l| l as &dyn IntPatchPointLine),
            None => return Err(domain_error(member, "no line is set")),
        };
        found.ok_or_else(|| super::failures::null_object(member, "line"))
    }

    /// `Prepare()`: the box of the points and the deflection, the largest distance of a
    /// point to the chord of its neighbours, refined by a parabola through them.
    fn prepare(&mut self) -> Result<(), Failure> {
        self.base.bounding.set_void();
        let n: i32 = IntPatchPolygo::nb_points(self)?;
        // `eps_2` keeps the deflection of the start while the loop raises it.
        let eps_2: f64 = self.base.error * self.base.error;
        let (mut p1, mut p2) = (Point2::default(), Point2::default());
        if n >= 3 {
            (p1, p2) = (
                IntPatchPolygo::point(self, 1)?,
                IntPatchPolygo::point(self, 2)?,
            );
        }
        for i in 1..=n {
            let p3: Point2 = IntPatchPolygo::point(self, i)?;
            if i >= 3 {
                self.update_error([p1, p2, p3], eps_2);
                (p1, p2) = (p2, p3);
            }
            self.base.bounding.add_point(p3);
        }
        self.base.bounding.enlarge(self.base.error);
        Ok(())
    }

    /// The deflection of the point `p2` between `p1` and `p3`.
    fn update_error(&mut self, [p1, p2, p3]: [Point2; 3], eps_2: f64) {
        let error: f64 = self.base.error;
        let v13: Xy = p3.xy() - p1.xy();
        let v12: Xy = p2.xy() - p1.xy();
        let d13_2: f64 = v13.square_modulus();
        let mut d_2: f64 = if d13_2 > eps_2 {
            v13.cross_square_magnitude(v12) / d13_2
        } else {
            eps_2
        };
        if d_2 > error * error {
            if let Some(d1) = parabola_deflection([p1, p2, p3], v12, v13) {
                // The smaller of the linear and the parabolic deflections.
                if d1 * d1 < d_2 {
                    d_2 = d1 * d1;
                }
            }
            if d_2 > error * error {
                self.base.error = d_2.sqrt();
            }
        }
    }
}

/// The deflection of the segments `p1 p2` and `p2 p3` to the parabola through the
/// three points (the parameter of `p2` pulled toward 0.5), when that parameter is in
/// `(0.1, 0.9)`.
fn parabola_deflection([p1, p2, p3]: [Point2; 3], v12: Xy, v13: Xy) -> Option<f64> {
    let v23: Xy = p3.xy() - p2.xy();
    let (d12, d23) = (v12.modulus(), v23.modulus());
    let mut tm: f64 = d12 / (d12 + d23);
    if !(tm > 0.1 && tm < 0.9) {
        return None;
    }
    tm -= (tm - 0.5) * 0.6;
    let tm1mtm: f64 = tm * (1.0 - tm);
    // The coefficients of the parabola.
    let ax: f64 = (tm * v13.x() - v12.x()) / tm1mtm;
    let bx: f64 = (v12.x() - tm * tm * v13.x()) / tm1mtm;
    let cx: f64 = p1.x();
    let ay: f64 = (tm * v13.y() - v12.y()) / tm1mtm;
    let by: f64 = (v12.y() - tm * tm * v13.y()) / tm1mtm;
    let cy: f64 = p1.y();
    // The lines P1-P2 and P2-P3.
    let (a1, b1) = (v12.y() / d12, -v12.x() / d12);
    let c1: f64 = (p2.x() * p1.y() - p1.x() * p2.y()) / d12;
    let (a2, b2) = (v23.y() / d23, -v23.x() / d23);
    let c2: f64 = (p3.x() * p2.y() - p2.x() * p3.y()) / d23;
    // The points of the parabola with the largest deflection.
    let t1: f64 = -0.5 * (a1 * bx + b1 * by) / (a1 * ax + b1 * ay);
    let t2: f64 = -0.5 * (a2 * bx + b2 * by) / (a2 * ax + b2 * ay);
    let (xt1, yt1) = (ax * t1 * t1 + bx * t1 + cx, ay * t1 * t1 + by * t1 + cy);
    let (xt2, yt2) = (ax * t2 * t2 + bx * t2 + cx, ay * t2 * t2 + by * t2 + cy);
    let mut d1: f64 = (a1 * xt1 + b1 * yt1 + c1).abs();
    let d2: f64 = (a2 * xt2 + b2 * yt2 + c2).abs();
    if d2 > d1 {
        d1 = d2;
    }
    Some(d1)
}

impl IntPatchPolygo for IntPatchPolyLine {
    fn polygo_base(&self) -> &IntPatchPolygoBase {
        &self.base
    }
    /// `NbPoints()`: the points of the line.
    fn nb_points(&self) -> Result<i32, Failure> {
        self.line()?.nb_pnts()
    }
    /// `Point(Index)`: the parameters of the point on the surface; the first and the
    /// last points of a walking line move by 1e-7 of their segment outward.
    fn point(&self, index: i32) -> Result<Point2, Failure> {
        let line: &dyn IntPatchPointLine = self.line()?;
        let params = |i: i32| -> Result<(f64, f64), Failure> {
            let p = line.point(i)?;
            Ok(if self.onfirst {
                p.parameters_on_s1()
            } else {
                p.parameters_on_s2()
            })
        };
        let (x, y) = params(index)?;
        let (mut dx, mut dy) = (0.0, 0.0);
        if self.typ == Some(IntPatchIType::Walking) {
            let neighbour: Option<i32> = if index == 1 {
                Some(2)
            } else if index == line.nb_pnts()? {
                Some(index - 1)
            } else {
                None
            };
            if let Some(other) = neighbour {
                let (x1, y1) = params(other)?;
                (dx, dy) = (0.0000001 * (x - x1), 0.0000001 * (y - y1));
            }
        }
        Ok(Point2::new(x + dx, y + dy))
    }
}

polygo_polygon2d!(IntPatchPolyLine);
