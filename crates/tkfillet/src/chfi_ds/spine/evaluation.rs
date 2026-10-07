//! The guideline as a curve of its abscissa: points, unit tangents, parameters on edges.
use super::Spine;
use crate::chfi_ds::errors::standard_failure;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::gcpnts::AbscissaPoint;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Line3, Point3, Vector3};

/// Where `Prepare` puts an abscissa: on an edge with the length along it, or on the
/// tangent extension before the start or after the end with the length along the tangent.
enum Placed {
    Edge(i32, f64),
    BeforeStart(f64),
    AfterEnd(f64),
}

impl Spine {
    /// `SetCurrent(Index)` (and the switch inside the evaluators): makes the `index`-th
    /// edge the current curve; `spine.set_current(2)?`.
    pub fn set_current(&mut self, index: i32) -> Result<(), Failure> {
        if index != self.index_of_curve {
            self.index_of_curve = index;
            self.curve.initialize(self.edges(index)?)?;
        }
        Ok(())
    }

    /// `CurrentElementarySpine(Index)`: makes the `index`-th edge current and returns its
    /// curve; `spine.current_elementary_spine(1)?`.
    pub fn current_elementary_spine(&mut self, index: i32) -> Result<&BRepAdaptorCurve, Failure> {
        self.set_current(index)?;
        Ok(&self.curve)
    }

    /// `GetType()`: the kind of the current curve; `spine.get_type()?`.
    pub fn get_type(&self) -> Result<CurveType, Failure> {
        self.curve.get_type()
    }

    /// `Line()`: the current line, oriented along the guideline and located at the start
    /// of the edge in the guideline order; `spine.line()?`.
    pub fn line(&self) -> Result<Line3, Failure> {
        let mut line = self.curve.line()?;
        if self.edges(self.index_of_curve)?.orientation() == Orientation::Reversed {
            line.reverse();
            line.set_location(self.curve.value(self.curve.last_parameter()?)?);
        } else {
            line.set_location(self.curve.value(self.curve.first_parameter()?)?);
        }
        Ok(line)
    }

    /// `Circle()`: the current circle, its X axis through the start of the edge in the
    /// guideline order and its axis reversed on a reversed edge; `spine.circle()?`.
    pub fn circle(&self) -> Result<Circle3, Failure> {
        let frame = self.curve.circle()?.position();
        let start = self.curve.value(self.curve.first_parameter()?)?;
        let mut x_direction = Direction3::from_vector(Vector3::between(frame.location(), start))?;
        let mut normal = frame.direction();
        if self.edges(self.index_of_curve)?.orientation() == Orientation::Reversed {
            let end = self.curve.value(self.curve.last_parameter()?)?;
            x_direction = Direction3::from_vector(Vector3::between(frame.location(), end))?;
            normal.reverse();
        }
        let axis = Axis2::with_x_direction(frame.location(), normal, x_direction)?;
        Circle3::new(axis, self.curve.circle()?.radius())
    }

    /// `SetFirstTgt(W)`: extends the start by the tangent at `w`; Standard_Failure on a
    /// periodic guideline; `spine.set_first_tgt(0.0)?`.
    pub fn set_first_tgt(&mut self, w: f64) -> Result<(), Failure> {
        if self.is_periodic() {
            return Err(standard_failure(
                "No extension by tangent on periodic contours",
            ));
        }
        // The flag is suspended while d1 runs, so d1 does not stop on the old extension.
        self.ends.has_first_tgt = false;
        (self.ends.first_ori, self.ends.first_tgt) = self.d1(w)?;
        self.ends.has_first_tgt = true;
        self.ends.first_tgt_par = w;
        Ok(())
    }

    /// `SetLastTgt(W)`: extends the end by the tangent at `w`; Standard_Failure on a
    /// periodic guideline; `spine.set_last_tgt(10.0)?`.
    pub fn set_last_tgt(&mut self, w: f64) -> Result<(), Failure> {
        if self.is_periodic() {
            return Err(standard_failure(
                "No extension by tangent periodic contours",
            ));
        }
        self.ends.has_last_tgt = false;
        (self.ends.last_ori, self.ends.last_tgt) = self.d1(w)?;
        self.ends.has_last_tgt = true;
        self.ends.last_tgt_par = w;
        Ok(())
    }

    /// `Absc(U)`: the abscissa of the parameter `u` on the current edge; `spine.absc(0.5)?`.
    pub fn absc(&mut self, u: f64) -> Result<f64, Failure> {
        self.absc_on_edge(u, self.index_of_curve)
    }

    /// `Absc(U, I)`: the abscissa of the parameter `u` on the `index`-th edge;
    /// `spine.absc_on_edge(0.5, 1)?`.
    pub fn absc_on_edge(&mut self, u: f64, index: i32) -> Result<f64, Failure> {
        self.set_current(index)?;
        let mut length = self.first_parameter_of(index)?;
        let curve: &dyn Adaptor3dCurve = &self.curve;
        let (first, last) = (curve.first_parameter()?, curve.last_parameter()?);
        if self.edges(index)?.orientation() == Orientation::Reversed {
            length += AbscissaPoint::length_between(curve, u, last)?;
        } else {
            length += AbscissaPoint::length_between(curve, first, u)?;
        }
        Ok(length)
    }

    /// `Parameter(AbsC, U, Oriented)`: the parameter on its edge of the abscissa `absc`
    /// (the edge is the first whose end is after it, else the last); `spine.parameter(1.0, true)?`.
    pub fn parameter(&mut self, absc: f64, oriented: bool) -> Result<f64, Failure> {
        let count = self.abscissa_length()?;
        let mut index = 1;
        while index < count && absc >= self.abscissa_value(index)? {
            index += 1;
        }
        self.parameter_on_edge(index, absc, oriented)
    }

    /// `Parameter(Index, AbsC, U, Oriented)`: the parameter on the `index`-th edge of the
    /// abscissa `absc`, mirrored on a reversed edge when `oriented`;
    /// `spine.parameter_on_edge(1, 1.0, true)?`.
    pub fn parameter_on_edge(
        &mut self,
        index: i32,
        absc: f64,
        oriented: bool,
    ) -> Result<f64, Failure> {
        self.set_current(index)?;
        let reversed = self.edges(index)?.orientation() == Orientation::Reversed;
        let length = if reversed {
            self.abscissa_value(self.index_of_curve)? - absc
        } else if self.index_of_curve == 1 {
            absc
        } else {
            absc - self.abscissa_value(self.index_of_curve - 1)?
        };
        let u = self.edge_parameter_at(index, length)?;
        if reversed && oriented {
            let curve: &dyn Adaptor3dCurve = &self.curve;
            return Ok((curve.last_parameter()? + curve.first_parameter()?) - u);
        }
        Ok(u)
    }

    /// The `GCPnts_AbscissaPoint(myCurve, L, First, uapp)` of the current `index`-th edge,
    /// started at the linear guess `uapp` of the length ratio `L / Length(index)`.
    fn edge_parameter_at(&self, index: i32, length: f64) -> Result<f64, Failure> {
        let t = length / self.length(index)?;
        let curve: &dyn Adaptor3dCurve = &self.curve;
        let first = curve.first_parameter()?;
        let uapp = (1. - t) * first + t * curve.last_parameter()?;
        AbscissaPoint::from_start(curve, length, first, uapp)?.parameter()
    }

    /// `Prepare(L, Index)`: places the abscissa `l` on an edge or on a tangent extension.
    fn prepare(&self, mut l: f64) -> Result<Placed, Failure> {
        let tol = std_max(self.tolesp, precision::confusion());
        let last = self.total_length()?;
        let len = self.abscissa_length()?;
        if self.is_periodic() && l.abs() >= tol && (l - last).abs() >= tol {
            l = elclib::in_period(l, 0.0, last);
        }
        let ends = self.ends;
        let index = if ends.has_first_tgt && l <= ends.first_tgt_par {
            if !(ends.has_ref && ends.val_ref >= l && (l - ends.first_tgt_par).abs() <= tol) {
                return Ok(Placed::BeforeStart(l - ends.first_tgt_par));
            }
            self.index_at(l, true)?
        } else if l <= 0. {
            1
        } else if ends.has_last_tgt && l >= ends.last_tgt_par {
            if !(ends.has_ref && ends.val_ref <= l && (l - ends.last_tgt_par).abs() <= tol) {
                return Ok(Placed::AfterEnd(l - ends.last_tgt_par));
            }
            self.index_at(l, true)?
        } else if l >= last {
            len
        } else {
            self.interior_index(l, len)?
        };
        Ok(Placed::Edge(index, self.length_along_edge(index, l)?))
    }

    /// The interior branch of `Prepare`: the edge holding `l`, moved off a junction toward
    /// the approximation reference.
    fn interior_index(&self, l: f64, len: i32) -> Result<i32, Failure> {
        let mut index = 1;
        while index < len && l >= self.abscissa_value(index)? {
            index += 1;
        }
        if !self.ends.has_ref {
            return Ok(index);
        }
        if l >= self.ends.val_ref && index != 1 {
            if (l - self.abscissa_value(index - 1)?).abs() <= precision::confusion() {
                index -= 1;
            }
        } else if l <= self.ends.val_ref
            && index != len
            && (l - self.abscissa_value(index)?).abs() <= precision::confusion()
        {
            index += 1;
        }
        Ok(index)
    }

    /// The end of `Prepare`: the length from the start of the `index`-th edge in its own
    /// orientation (an index outside the edges keeps `l`).
    fn length_along_edge(&self, index: i32, l: f64) -> Result<f64, Failure> {
        if index < 1 || index > self.abscissa_length()? {
            return Ok(l);
        }
        if self.edges(index)?.orientation() == Orientation::Reversed {
            return Ok(self.abscissa_value(index)? - l);
        }
        if index != 1 {
            return Ok(l - self.abscissa_value(index - 1)?);
        }
        Ok(l)
    }

    /// The point on the tangent extension: `origin + tangent * l`.
    fn extended_point(origin: Point3, tangent: Vector3, l: f64) -> Point3 {
        let mut along = tangent;
        along *= l;
        origin.translated(along)
    }

    /// `Value(AbsC)`: the point of the guideline at the abscissa `absc`; `spine.value(1.0)?`.
    pub fn value(&mut self, absc: f64) -> Result<Point3, Failure> {
        match self.prepare(absc)? {
            Placed::BeforeStart(l) => Ok(Self::extended_point(
                self.ends.first_ori,
                self.ends.first_tgt,
                l,
            )),
            Placed::AfterEnd(l) => Ok(Self::extended_point(
                self.ends.last_ori,
                self.ends.last_tgt,
                l,
            )),
            Placed::Edge(index, l) => {
                self.set_current(index)?;
                let u = self.edge_parameter_at(index, l)?;
                self.curve.value(u)
            }
        }
    }

    /// `D0(AbsC, P)`: as [`Spine::value`]; `spine.d0(1.0)?`.
    pub fn d0(&mut self, absc: f64) -> Result<Point3, Failure> {
        self.value(absc)
    }

    /// `D1(AbsC, P, V1)`: the point and the unit tangent along the guideline;
    /// `let (p, t) = spine.d1(1.0)?;`.
    pub fn d1(&mut self, absc: f64) -> Result<(Point3, Vector3), Failure> {
        match self.prepare(absc)? {
            Placed::BeforeStart(l) => Ok(self.extension_d1(true, l)),
            Placed::AfterEnd(l) => Ok(self.extension_d1(false, l)),
            Placed::Edge(index, l) => {
                self.set_current(index)?;
                let u = self.edge_parameter_at(index, l)?;
                let (point, mut v1) = self.curve.d1(u)?;
                let mut d1 = 1. / v1.magnitude();
                if self.edges(index)?.orientation() == Orientation::Reversed {
                    d1 = -d1;
                }
                v1 *= d1;
                Ok((point, v1))
            }
        }
    }

    /// The point and tangent of a tangent extension.
    fn extension_d1(&self, at_start: bool, l: f64) -> (Point3, Vector3) {
        let (origin, tangent) = if at_start {
            (self.ends.first_ori, self.ends.first_tgt)
        } else {
            (self.ends.last_ori, self.ends.last_tgt)
        };
        (Self::extended_point(origin, tangent, l), tangent)
    }

    /// `D2(AbsC, P, V1, V2)`: the point, the unit tangent and the curvature vector along
    /// the guideline (zero on a tangent extension); `let (p, t, n) = spine.d2(1.0)?;`.
    pub fn d2(&mut self, absc: f64) -> Result<(Point3, Vector3, Vector3), Failure> {
        let (index, l) = match self.prepare(absc)? {
            Placed::BeforeStart(l) => return Ok(push_zero(self.extension_d1(true, l))),
            Placed::AfterEnd(l) => return Ok(push_zero(self.extension_d1(false, l))),
            Placed::Edge(index, l) => (index, l),
        };
        self.set_current(index)?;
        let u = self.edge_parameter_at(index, l)?;
        let (point, v1, v2) = self.curve.d2(u)?;
        let reversed = self.edges(index)?.orientation() == Orientation::Reversed;
        let (v1, v2) = arc_length_derivatives(v1, v2, reversed);
        Ok((point, v1, v2))
    }
}

/// A point and tangent with a zero second derivative.
fn push_zero((point, tangent): (Point3, Vector3)) -> (Point3, Vector3, Vector3) {
    (point, tangent, Vector3::new(0., 0., 0.))
}

/// The derivatives in arc length of the curve derivatives `v1`, `v2`, in the native
/// operation order; the tangent is reversed on a reversed edge (the second derivative is
/// not, as natively).
fn arc_length_derivatives(mut v1: Vector3, mut v2: Vector3, reversed: bool) -> (Vector3, Vector3) {
    let mut n1 = v1.square_magnitude();
    let d2 = -(v1.dot(v2)) * (1. / n1) * (1. / n1);
    v2 *= 1. / n1;
    n1 = n1.sqrt();
    v2 += v1 * d2;
    let mut d1 = 1. / n1;
    if reversed {
        d1 = -d1;
    }
    v1 *= d1;
    (v1, v2)
}
