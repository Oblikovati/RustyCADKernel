// Rust translation of ModelingAlgorithms/TKFeat/LocOpe/LocOpe_Prism.hxx/.cxx,
// LocOpe_Revol.hxx/.cxx, LocOpe_LinearForm.hxx/.lxx/.cxx and LocOpe_RevolutionForm.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LocOpe_Prism`, `LocOpe_LinearForm` (BRepSweep_Prism), `LocOpe_Revol` and
//! `LocOpe_RevolutionForm` (BRepSweep_Revol): the swept shapes of the prism, linear form,
//! revolution and revolution form features, with the faces swept by the base edges and
//! the curves through the base that the features intersect with the basis shape.
use super::swept::Swept;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::{List, Sequence};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomLine, GeomTrimmedCurve};
use rustycad_tkmath::gp::{Axis1, Axis2, Circle3, Direction3, Point3, Transform3, Vector3};
use rustycad_tkprim::brep_sweep::{BRepSweepPrism, BRepSweepRevol};

/// A translation by `v`.
fn translation(v: Vector3) -> Transform3 {
    let mut t = Transform3::default();
    t.set_translation(v);
    t
}

/// A rotation around `axis` by `angle`.
fn rotation(axis: Axis1, angle: f64) -> Result<Transform3, Failure> {
    let mut t = Transform3::default();
    t.set_rotation(axis, angle)?;
    Ok(t)
}

/// The prism sweep of a base along `v` (not copied, canonized).
fn prism(v: Vector3) -> impl FnOnce(&Shape) -> Result<BRepSweepPrism, Failure> {
    move |base| BRepSweepPrism::new(base, v, false, true)
}

/// The revolution sweep of a base around `axis` by `angle` (not copied).
fn revol(axis: Axis1, angle: f64) -> impl FnOnce(&Shape) -> Result<BRepSweepRevol, Failure> {
    move |base| BRepSweepRevol::new(base, axis, angle, false)
}

/// The line along `v` through `p`.
fn line_along(p: Point3, v: Vector3) -> Result<Handle<dyn GeomCurve>, Failure> {
    let axis = Axis1::new(p, Direction3::from_vector(v)?);
    Ok(Handle::new(GeomLine::new(axis)).cast())
}

/// `FindCircle(Ax, Pt, Ci)`: the circle around the axis through the point; None on the
/// axis.
fn find_circle(ax: Axis1, pt: Point3) -> Result<Option<Circle3>, Failure> {
    let dax = ax.direction();
    let op = Vector3::between(ax.location(), pt);
    let prm = op.dot(Vector3::from(dax));
    let prj = Point3::new(
        ax.location().x() + prm * dax.x(),
        ax.location().y() + prm * dax.y(),
        ax.location().z() + prm * dax.z(),
    );
    let axx = Vector3::between(prj, pt);
    let radius = axx.magnitude();
    if radius < precision::confusion() {
        return Ok(None);
    }
    let frame = Axis2::with_x_direction(prj, dax, Direction3::from_vector(axx)?)?;
    Ok(Some(Circle3::new(frame, radius)?))
}

/// The circle `ci` as a Geom_Circle.
fn geom_circle(ci: Circle3) -> Result<Handle<dyn GeomCurve>, Failure> {
    Ok(Handle::new(GeomCircle::new(ci.position(), ci.radius())?).cast())
}

macro_rules! swept_accessors {
    ($native:literal) => {
        /// `FirstShape()`: the base (moved) of the sweep; `op.first_shape()`.
        pub fn first_shape(&self) -> &Shape {
            &self.swept.first_shape
        }

        /// `LastShape()`: the end of the sweep; `op.last_shape()`.
        pub fn last_shape(&self) -> &Shape {
            &self.swept.last_shape
        }

        /// `Shape()`: the swept shape; StdFail_NotDone before Perform; `op.shape()?`.
        pub fn shape(&self) -> Result<&Shape, Failure> {
            self.swept.shape(concat!($native, "::Shape"))
        }

        /// `Shapes(S)`: the faces swept by the edge `s` of the base;
        /// Standard_NoSuchObject for another shape; `op.shapes(&edge)?`.
        pub fn shapes(&self, s: &Shape) -> Result<&List<Shape>, Failure> {
            self.swept.shapes(concat!($native, "::Shapes"), s)
        }
    };
}

/// `LocOpe_Prism`: the prism of a base along a vector, the base moved first by a
/// translation when given.
///
/// ```
/// use rustycad_tkfeat::loc_ope::Prism;
/// let prism = Prism::new();
/// assert!(prism.shape().is_err());
/// ```
#[derive(Clone, Default)]
pub struct Prism {
    swept: Swept,
    vec: Vector3,
    tra: Vector3,
    is_trans: bool,
}

impl Prism {
    /// `LocOpe_Prism()`; `Prism::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `LocOpe_Prism(Base, V)`: performed; `Prism::with_vector(&base, v)?`.
    pub fn with_vector(base: &Shape, v: Vector3) -> Result<Self, Failure> {
        let mut op = Self::new();
        op.perform(base, v)?;
        Ok(op)
    }

    /// `LocOpe_Prism(Base, V, Vectra)`: performed on the translated base;
    /// `Prism::with_translation(&base, v, vtra)?`.
    pub fn with_translation(base: &Shape, v: Vector3, vtra: Vector3) -> Result<Self, Failure> {
        let mut op = Self::new();
        op.perform_translated(base, v, vtra)?;
        Ok(op)
    }

    /// `Perform(Base, V)`; `prism.perform(&base, v)?`.
    pub fn perform(&mut self, base: &Shape, v: Vector3) -> Result<(), Failure> {
        self.swept.reset(base);
        self.vec = v;
        self.is_trans = false;
        self.swept.int_perf(None, prism(v))
    }

    /// `Perform(Base, V, Vtra)`: the base translated by `vtra` first;
    /// `prism.perform_translated(&base, v, vtra)?`.
    pub fn perform_translated(
        &mut self,
        base: &Shape,
        v: Vector3,
        vtra: Vector3,
    ) -> Result<(), Failure> {
        self.swept.reset(base);
        self.vec = v;
        self.tra = vtra;
        self.is_trans = true;
        self.swept.int_perf(Some(translation(vtra)), prism(v))
    }

    swept_accessors!("LocOpe_Prism");

    /// `Curves(SCurves)`: the segments of length four times the height, along the
    /// vector, through the sample points of the first shape; `prism.curves(&mut c)?`.
    pub fn curves(&self, scurves: &mut Sequence<Handle<dyn GeomCurve>>) -> Result<(), Failure> {
        scurves.clear(None);
        let (spt, _) = self.swept.first_shape_samples()?;
        let v = self.vec;
        let height = (v.x() * v.x() + v.y() * v.y() + v.z() * v.z()).sqrt();
        let (u1, u2) = (-2. * height, 2. * height);
        for p in spt.iter() {
            let the_lin = line_along(*p, v)?;
            let trlin = GeomTrimmedCurve::new(&the_lin, u1, u2, true, true)?;
            scurves.append(Handle::new(trlin).cast());
        }
        Ok(())
    }

    /// `BarycCurve()`: the line along the vector through the barycentre of the sample
    /// points of the first shape; `prism.baryc_curve()?`.
    pub fn baryc_curve(&self) -> Result<Handle<dyn GeomCurve>, Failure> {
        let (_, bar) = self.swept.first_shape_samples()?;
        line_along(bar, self.vec)
    }
}

/// `LocOpe_LinearForm`: the prism of a base along a vector for the linear form feature,
/// the base moved first by a translation when given.
///
/// ```
/// use rustycad_tkfeat::loc_ope::LinearForm;
/// let form = LinearForm::new();
/// assert!(form.shape().is_err());
/// ```
#[derive(Clone, Default)]
pub struct LinearForm {
    swept: Swept,
    vec: Vector3,
    tra: Vector3,
    is_trans: bool,
    pnt1: Point3,
    pnt2: Point3,
}

impl LinearForm {
    /// `LocOpe_LinearForm()`; `LinearForm::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `LocOpe_LinearForm(Base, V, Pnt1, Pnt2)`: performed;
    /// `LinearForm::with_vector(&base, v, p1, p2)?`.
    pub fn with_vector(
        base: &Shape,
        v: Vector3,
        pnt1: Point3,
        pnt2: Point3,
    ) -> Result<Self, Failure> {
        let mut op = Self::new();
        op.perform(base, v, pnt1, pnt2)?;
        Ok(op)
    }

    /// `LocOpe_LinearForm(Base, V, Vectra, Pnt1, Pnt2)`: performed on the translated
    /// base; `LinearForm::with_translation(&base, v, vtra, p1, p2)?`.
    pub fn with_translation(
        base: &Shape,
        v: Vector3,
        vectra: Vector3,
        pnt1: Point3,
        pnt2: Point3,
    ) -> Result<Self, Failure> {
        let mut op = Self::new();
        op.perform_translated(base, v, vectra, pnt1, pnt2)?;
        Ok(op)
    }

    /// `Perform(Base, V, Pnt1, Pnt2)`; `form.perform(&base, v, p1, p2)?`.
    pub fn perform(
        &mut self,
        base: &Shape,
        v: Vector3,
        pnt1: Point3,
        pnt2: Point3,
    ) -> Result<(), Failure> {
        self.is_trans = false;
        self.swept.reset(base);
        self.vec = v;
        (self.pnt1, self.pnt2) = (pnt1, pnt2);
        self.swept.int_perf(None, prism(v))
    }

    /// `Perform(Base, V, Vectra, Pnt1, Pnt2)`: the base translated by `vectra` first;
    /// `form.perform_translated(&base, v, vtra, p1, p2)?`.
    pub fn perform_translated(
        &mut self,
        base: &Shape,
        v: Vector3,
        vectra: Vector3,
        pnt1: Point3,
        pnt2: Point3,
    ) -> Result<(), Failure> {
        self.is_trans = true;
        self.tra = vectra;
        self.swept.reset(base);
        self.vec = v;
        (self.pnt1, self.pnt2) = (pnt1, pnt2);
        self.swept.int_perf(Some(translation(vectra)), prism(v))
    }

    swept_accessors!("LocOpe_LinearForm");
}

/// `LocOpe_Revol`: the revolution of a base around an axis, the base turned first by an
/// angle when given.
///
/// ```
/// use rustycad_tkfeat::loc_ope::Revol;
/// let revol = Revol::new();
/// assert!(revol.shape().is_err());
/// ```
#[derive(Clone, Default)]
pub struct Revol {
    swept: Swept,
    axis: Axis1,
    angle: f64,
    ang_tra: f64,
    is_trans: bool,
}

impl Revol {
    /// `LocOpe_Revol()`; `Revol::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `LocOpe_Revol(Base, Axis, Angle)`: performed; `Revol::with_axis(&base, axis, a)?`.
    pub fn with_axis(base: &Shape, axis: Axis1, angle: f64) -> Result<Self, Failure> {
        let mut op = Self::new();
        op.perform(base, axis, angle)?;
        Ok(op)
    }

    /// `LocOpe_Revol(Base, Axis, Angle, angledec)`: performed on the turned base;
    /// `Revol::with_offset_angle(&base, axis, a, dec)?`.
    pub fn with_offset_angle(
        base: &Shape,
        axis: Axis1,
        angle: f64,
        angledec: f64,
    ) -> Result<Self, Failure> {
        let mut op = Self::new();
        op.perform_turned(base, axis, angle, angledec)?;
        Ok(op)
    }

    /// `Perform(Base, Axis, Angle)`; `revol.perform(&base, axis, angle)?`.
    pub fn perform(&mut self, base: &Shape, axis: Axis1, angle: f64) -> Result<(), Failure> {
        self.swept.reset(base);
        (self.angle, self.axis, self.ang_tra, self.is_trans) = (angle, axis, 0., false);
        self.swept.int_perf(None, revol(axis, angle))
    }

    /// `Perform(Base, Axis, Angle, angledec)`: the base turned by `angledec` first;
    /// `revol.perform_turned(&base, axis, angle, dec)?`.
    pub fn perform_turned(
        &mut self,
        base: &Shape,
        axis: Axis1,
        angle: f64,
        angledec: f64,
    ) -> Result<(), Failure> {
        self.swept.reset(base);
        (self.angle, self.axis, self.ang_tra, self.is_trans) = (angle, axis, angledec, true);
        self.swept
            .int_perf(Some(rotation(axis, angledec)?), revol(axis, angle))
    }

    swept_accessors!("LocOpe_Revol");

    /// `Curves(SCurves)`: the circles around the axis through the sample points of the
    /// first shape (none for a point on the axis); `revol.curves(&mut c)?`.
    pub fn curves(&self, scurves: &mut Sequence<Handle<dyn GeomCurve>>) -> Result<(), Failure> {
        scurves.clear(None);
        let (spt, _) = self.swept.first_shape_samples()?;
        for p in spt.iter() {
            if let Some(cax) = find_circle(self.axis, *p)? {
                scurves.append(geom_circle(cax)?);
            }
        }
        Ok(())
    }

    /// `BarycCurve()`: the circle around the axis through the barycentre of the sample
    /// points of the first shape (null on the axis); `revol.baryc_curve()?`.
    pub fn baryc_curve(&self) -> Result<Handle<dyn GeomCurve>, Failure> {
        let (_, bar) = self.swept.first_shape_samples()?;
        match find_circle(self.axis, bar)? {
            Some(cax) => geom_circle(cax),
            None => Ok(Handle::default()),
        }
    }
}

/// `LocOpe_RevolutionForm`: the revolution of a base around an axis for the revolution
/// form feature.
///
/// ```
/// use rustycad_tkfeat::loc_ope::RevolutionForm;
/// let form = RevolutionForm::new();
/// assert!(form.shape().is_err());
/// ```
#[derive(Clone, Default)]
pub struct RevolutionForm {
    swept: Swept,
    axis: Axis1,
    angle: f64,
    ang_tra: f64,
    is_trans: bool,
}

impl RevolutionForm {
    /// `LocOpe_RevolutionForm()`; `RevolutionForm::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `LocOpe_RevolutionForm(Base, Axe, Angle)`: performed;
    /// `RevolutionForm::with_axis(&base, axis, angle)?`.
    pub fn with_axis(base: &Shape, axe: Axis1, angle: f64) -> Result<Self, Failure> {
        let mut op = Self::new();
        op.perform(base, axe, angle)?;
        Ok(op)
    }

    /// `Perform(Base, Axe, Angle)`; `form.perform(&base, axis, angle)?`.
    pub fn perform(&mut self, base: &Shape, axis: Axis1, angle: f64) -> Result<(), Failure> {
        self.swept.reset(base);
        (self.angle, self.axis, self.ang_tra, self.is_trans) = (angle, axis, 0., false);
        let trsf = if self.is_trans {
            Some(rotation(axis, self.ang_tra)?)
        } else {
            None
        };
        self.swept.int_perf(trsf, revol(axis, angle))
    }

    swept_accessors!("LocOpe_RevolutionForm");
}
