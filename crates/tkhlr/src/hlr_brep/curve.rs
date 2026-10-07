//! HLRBRep_Curve.hxx/.lxx/.cxx (the 3D side, the parameters and the boxes), OCCT
//! 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::BCurveTool;
use crate::hlr_algo::{Projector, update_min_max};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{self, Direction3, Line3, Point2, Point3, Vector2, Vector3};
use std::cmp::Ordering;
use std::sync::Arc;

/// HLRBRep_Curve: an edge's 3D curve seen through the projector of the hidden line
/// data structure, as a 2D curve of the view plane. The native class keeps a raw
/// pointer to the projector of its `HLRBRep_Data`; the port shares it through an
/// `Arc` that the data structure replaces in every curve when it changes the
/// projector (docs/hlr-design.md section 6).
///
/// ```
/// use rustycad_tkhlr::hlr_brep::Curve;
/// let curve = Curve::new();
/// assert!(curve.parameter_2d(1.0).is_err()); // no projector yet
/// ```
pub struct Curve {
    curve: BRepAdaptorCurve,
    curve_type: CurveType,
    projector: Option<Arc<Projector>>,
    // The native members are uninitialized until Update; the port starts them at 0.
    ox: f64,
    oz: f64,
    vx: f64,
    vz: f64,
    of: f64,
}

/// The 16-direction box of HLRAlgo (`double TotMin[16]`).
pub type MinMaxBox = [f64; 16];

impl Default for Curve {
    // The native constructor is `= default`: the type and the line data stay
    // uninitialized; the port starts them at OtherCurve and 0.
    fn default() -> Self {
        Self {
            curve: BRepAdaptorCurve::default(),
            curve_type: CurveType::OtherCurve,
            projector: None,
            ox: 0.,
            oz: 0.,
            vx: 0.,
            vz: 0.,
            of: 0.,
        }
    }
}

pub(super) fn null_projector(owner: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("{owner}: no projector set; expected Projector() before use"),
        "",
    )
}

impl Curve {
    /// HLRBRep_Curve(): an undefined curve (no edge, no projector);
    /// `Curve::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Projector(Proj): the projector shared with the data structure;
    /// `curve.set_projector(Some(projector))`.
    pub fn set_projector(&mut self, projector: Option<Arc<Projector>>) {
        self.projector = projector;
    }

    // The shared projector, compared by identity where the native code compares the
    // projector pointer (HLRBRep_Data shares a scaled projector with its curves).
    pub(super) fn projector_handle(&self) -> Option<&Arc<Projector>> {
        self.projector.as_ref()
    }

    // The native copy: a new adaptor on the same edge (the adaptor keeps its state
    // behind locks, so it is not cloned) with the same type, line data and projector.
    pub(super) fn try_clone(&self) -> Result<Self, Failure> {
        let curve = BRepAdaptorCurve::default();
        let edge = self.curve.edge();
        if !edge.is_null() {
            curve.initialize(&edge)?;
        }
        Ok(Self {
            curve,
            curve_type: self.curve_type,
            projector: self.projector.clone(),
            ..*self
        })
    }

    pub(super) fn projector(&self) -> Result<&Projector, Failure> {
        self.projector
            .as_deref()
            .ok_or_else(|| null_projector("HLRBRep_Curve"))
    }

    /// Curve(): the 3D curve adaptor (interior mutability, as the native reference);
    /// `curve.curve()`.
    pub fn curve(&self) -> &BRepAdaptorCurve {
        &self.curve
    }

    /// GetCurve(): the 3D curve adaptor; `curve.get_curve()`.
    pub fn get_curve(&self) -> &BRepAdaptorCurve {
        &self.curve
    }

    /// Curve(E): sets the 3D curve to be projected; `curve.set_curve(&edge)?`.
    pub fn set_curve(&mut self, edge: &Edge) -> Result<(), Failure> {
        self.curve.initialize(edge)
    }

    /// Parameter2d(P3d): the parameter on the 2D curve from the 3D one; lines and
    /// ellipses are reparametrized; `curve.parameter_2d(0.5)?`.
    pub fn parameter_2d(&self, p3d: f64) -> Result<f64, Failure> {
        let perspective = self.projector()?.perspective();
        Ok(match self.curve_type {
            CurveType::Line if perspective => {
                let fm_oz = self.of - self.oz;
                self.of * p3d * (self.vx * fm_oz + self.ox * self.vz)
                    / (fm_oz * (fm_oz - p3d * self.vz))
            }
            CurveType::Line => p3d * self.vx,
            CurveType::Ellipse => p3d + self.ox,
            _ => p3d,
        })
    }

    /// Parameter3d(P2d): the parameter on the 3D curve from the 2D one;
    /// `curve.parameter_3d(0.5)?`.
    pub fn parameter_3d(&self, p2d: f64) -> Result<f64, Failure> {
        let perspective = self.projector()?.perspective();
        Ok(match self.curve_type {
            CurveType::Line if perspective => {
                let fm_oz = self.of - self.oz;
                p2d * fm_oz * fm_oz
                    / (fm_oz * (self.of * self.vx + p2d * self.vz) + self.of * self.ox * self.vz)
            }
            CurveType::Line if self.vx <= gp::resolution() => p2d,
            CurveType::Line => p2d / self.vx,
            CurveType::Ellipse => p2d - self.ox,
            _ => p2d,
        })
    }

    /// Update(TotMin, TotMax): classifies the projected curve, computes the line
    /// data and updates the box; returns the box enlargement tolerance;
    /// `curve.update(&mut min, &mut max)?`.
    pub fn update(
        &mut self,
        tot_min: &mut MinMaxBox,
        tot_max: &mut MinMaxBox,
    ) -> Result<f64, Failure> {
        self.curve_type = self.projected_type()?;
        if self.curve_type == CurveType::Line {
            self.update_line()?;
        }
        self.update_min_max(tot_min, tot_max)
    }

    // The switch of Update on the 3D curve type.
    fn projected_type(&mut self) -> Result<CurveType, Failure> {
        let typ = BCurveTool::get_type(&self.curve)?;
        let perspective = self.projector()?.perspective();
        Ok(match typ {
            CurveType::Line => typ,
            CurveType::Circle if !perspective => self.circle_type()?,
            CurveType::Ellipse if !perspective => self.ellipse_type()?,
            CurveType::BezierCurve if BCurveTool::degree(&self.curve)? == 1 => CurveType::Line,
            CurveType::BezierCurve | CurveType::BSplineCurve if !perspective => typ,
            _ => CurveType::OtherCurve,
        })
    }

    fn view_direction(&self, mut d: Direction3) -> Result<Direction3, Failure> {
        d.transform(self.projector()?.transformation());
        Ok(d)
    }

    // A circle seen along its axis stays a circle; seen edge-on it is another curve;
    // otherwise an ellipse with an angle offset.
    fn circle_type(&mut self) -> Result<CurveType, Failure> {
        let circle = BCurveTool::circle(&self.curve)?;
        let d1 = self.view_direction(circle.axis().direction())?;
        if d1.is_parallel(gp::dz(), precision::angular()) {
            return Ok(CurveType::Circle);
        }
        //*10: The minor radius of ellipse should not be too small.
        if d1.dot(gp::dz()).abs() < precision::angular() * 10. {
            return Ok(CurveType::OtherCurve);
        }
        let d3 = d1.crossed(gp::dz())?;
        let d2 = self.view_direction(circle.x_axis().direction())?;
        self.ox = d3.angle_with_ref(d2, d1);
        Ok(CurveType::Ellipse)
    }

    fn ellipse_type(&mut self) -> Result<CurveType, Failure> {
        let ellipse = BCurveTool::ellipse(&self.curve)?;
        let d1 = self.view_direction(ellipse.axis().direction())?;
        if !d1.is_parallel(gp::dz(), precision::angular()) {
            return Ok(CurveType::OtherCurve);
        }
        self.ox = 0.; // no offset on the angle
        Ok(CurveType::Ellipse)
    }

    // The 3D line of Update: the curve itself, or the degree 1 Bezier from 0, with
    // the 3D length of the Bezier.
    fn line_3d(&self) -> Result<(Line3, f64), Failure> {
        if BCurveTool::get_type(&self.curve)? == CurveType::Line {
            return Ok((BCurveTool::line(&self.curve)?, 1.));
        }
        let (pl, vl) = BCurveTool::d1(&self.curve, 0.)?;
        let line = Line3::new(pl, Direction3::from_vector(vl)?);
        Ok((line, pl.distance(&BCurveTool::value(&self.curve, 1.)?)))
    }

    fn update_line(&mut self) -> Result<(), Failure> {
        let (line, l3d) = self.line_3d()?;
        let projector = *self.projector()?;
        let mut p = line.location();
        let mut v = Vector3::from(line.direction());
        p.transform(projector.transformation());
        v.transform(projector.transformation());
        if !projector.perspective() {
            self.vx = (v.x() * v.x() + v.y() * v.y()).sqrt() * l3d;
            return Ok(());
        }
        self.update_perspective_line(&projector, p, v, l3d)
    }

    fn update_perspective_line(
        &mut self,
        projector: &Projector,
        p: Point3,
        v: Vector3,
        l3d: f64,
    ) -> Result<(), Failure> {
        let mut f = Point2::default();
        let mut vfx = Vector2::default();
        self.d1(0., &mut f, &mut vfx)?;
        vfx.normalize()?;
        self.vx = (vfx.x() * v.x() + vfx.y() * v.y()) * l3d;
        let l = -(vfx.x() * f.x() + vfx.y() * f.y());
        f.set_coordinates(f.x() + vfx.x() * l, f.y() + vfx.y() * l);
        self.ox = vfx.x() * (p.x() - f.x()) + vfx.y() * (p.y() - f.y());
        let mut vfz = Vector3::new(-f.x(), -f.y(), projector.focus()?);
        self.of = vfz.magnitude();
        vfz /= self.of;
        self.vz = vfz.dot(v);
        self.vz *= l3d;
        self.oz = vfz.dot(Vector3::new(p.x() - f.x(), p.y() - f.y(), p.z()));
        Ok(())
    }

    /// UpdateMinMax(TotMin, TotMax): adds the projected end points (and 30 inner
    /// samples on a curve that is not a line) to the box; returns the largest
    /// deviation of a sample from the chord of its neighbours;
    /// `curve.update_min_max(&mut min, &mut max)?`.
    pub fn update_min_max(
        &self,
        tot_min: &mut MinMaxBox,
        tot_max: &mut MinMaxBox,
    ) -> Result<f64, Failure> {
        let projector = self.projector()?;
        let mut a = BCurveTool::first_parameter(&self.curve)?;
        let b = BCurveTool::last_parameter(&self.curve)?;
        let mut tol_min_max = 0.;
        let mut xyz = projector.project_xyz(self.value_3d(a)?);
        update_min_max(xyz.0, xyz.1, xyz.2, tot_min, tot_max);
        if self.curve_type != CurveType::Line {
            let nb_pnt = 30;
            let step = (b - a) / f64::from(nb_pnt + 1);
            // The native locals `xb, yb, zb` start at 0 before the first sample.
            let mut prev1 = (0., 0., 0.);
            for i in 1..=nb_pnt {
                a += step;
                let prev2 = prev1;
                prev1 = xyz;
                xyz = projector.project_xyz(self.value_3d(a)?);
                update_min_max(xyz.0, xyz.1, xyz.2, tot_min, tot_max);
                if i >= 2 {
                    tol_min_max = chord_deviation(prev2, prev1, xyz, tol_min_max);
                }
            }
        }
        xyz = projector.project_xyz(self.value_3d(b)?);
        update_min_max(xyz.0, xyz.1, xyz.2, tot_min, tot_max);
        Ok(tol_min_max)
    }

    /// Z(U): the depth of the point of parameter `u` in the viewing frame;
    /// `curve.z(0.5)?`.
    pub fn z(&self, u: f64) -> Result<f64, Failure> {
        let mut p3d = BCurveTool::d0(&self.curve, u)?;
        p3d.transform(self.projector()?.transformation());
        Ok(p3d.z())
    }

    /// Value3D(U): the 3D point of parameter `u`; `curve.value_3d(0.5)?`.
    pub fn value_3d(&self, u: f64) -> Result<Point3, Failure> {
        BCurveTool::d0(&self.curve, u)
    }

    /// D0(U, P) on the 3D curve; `curve.d0_3d(0.5)?`.
    pub fn d0_3d(&self, u: f64) -> Result<Point3, Failure> {
        BCurveTool::d0(&self.curve, u)
    }

    /// D1(U, P, V) on the 3D curve; `curve.d1_3d(0.5)?`.
    pub fn d1_3d(&self, u: f64) -> Result<(Point3, Vector3), Failure> {
        BCurveTool::d1(&self.curve, u)
    }

    /// FirstParameter(): the 2D parameter of the first 3D parameter;
    /// `curve.first_parameter()?`.
    pub fn first_parameter(&self) -> Result<f64, Failure> {
        self.parameter_2d(BCurveTool::first_parameter(&self.curve)?)
    }

    /// LastParameter(): the 2D parameter of the last 3D parameter;
    /// `curve.last_parameter()?`.
    pub fn last_parameter(&self) -> Result<f64, Failure> {
        self.parameter_2d(BCurveTool::last_parameter(&self.curve)?)
    }

    /// Continuity() of the 3D curve; `curve.continuity()?`.
    pub fn continuity(&self) -> Result<Shape, Failure> {
        BCurveTool::continuity(&self.curve)
    }

    /// NbIntervals(S) of the 3D curve; `curve.nb_intervals(Shape::C1)?`.
    pub fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        BCurveTool::nb_intervals(&self.curve, shape)
    }

    /// Intervals(T, S): the 3D interval bounds; `curve.intervals(&mut tab, Shape::C1)?`.
    pub fn intervals(&self, tab: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        BCurveTool::intervals(&self.curve, tab, shape)
    }

    /// IsClosed() of the 3D curve; `curve.is_closed()?`.
    pub fn is_closed(&self) -> Result<bool, Failure> {
        BCurveTool::is_closed(&self.curve)
    }

    /// IsPeriodic() of the 3D curve; `curve.is_periodic()?`.
    pub fn is_periodic(&self) -> Result<bool, Failure> {
        BCurveTool::is_periodic(&self.curve)
    }

    /// Period() of the 3D curve; `curve.period()?`.
    pub fn period(&self) -> Result<f64, Failure> {
        BCurveTool::period(&self.curve)
    }

    /// Resolution(R3d) of the 3D curve; `curve.resolution(1e-7)?`.
    pub fn resolution(&self, r3d: f64) -> Result<f64, Failure> {
        BCurveTool::resolution(&self.curve, r3d)
    }

    /// GetType(): the type of the projected curve computed by Update;
    /// `curve.get_type()`.
    pub fn get_type(&self) -> CurveType {
        self.curve_type
    }

    /// IsRational() of the 3D curve; `curve.is_rational()?`.
    pub fn is_rational(&self) -> Result<bool, Failure> {
        BCurveTool::is_rational(&self.curve)
    }

    /// Degree() of the 3D curve; `curve.degree()?`.
    pub fn degree(&self) -> Result<i32, Failure> {
        BCurveTool::degree(&self.curve)
    }

    /// NbPoles() of the 3D curve; `curve.nb_poles()?`.
    pub fn nb_poles(&self) -> Result<i32, Failure> {
        BCurveTool::nb_poles(&self.curve)
    }

    /// NbKnots() of the 3D curve; `curve.nb_knots()?`.
    pub fn nb_knots(&self) -> Result<i32, Failure> {
        BCurveTool::nb_knots(&self.curve)
    }
}

type Xyz = (f64, f64, f64);

// The distance of the middle sample `b` from the line through `a` along `c - a`,
// measured at the projection parameter, kept when it is the largest so far.
fn chord_deviation(a: Xyz, b: Xyz, c: Xyz, tol_min_max: f64) -> f64 {
    let (mut dx1, mut dy1, mut dz1) = (c.0 - a.0, c.1 - a.1, c.2 - a.2);
    let mut dd1 = (dx1 * dx1 + dy1 * dy1 + dz1 * dz1).sqrt();
    // Not Greater (NaN included) skips, as the native `if (dd > 0)`.
    if dd1.partial_cmp(&0.) != Some(Ordering::Greater) {
        return tol_min_max;
    }
    let (dx2, dy2, dz2) = (b.0 - a.0, b.1 - a.1, b.2 - a.2);
    let dd2 = (dx2 * dx2 + dy2 * dy2 + dz2 * dz2).sqrt();
    if dd2.partial_cmp(&0.) != Some(Ordering::Greater) {
        return tol_min_max;
    }
    let p = (dx1 * dx2 + dy1 * dy2 + dz1 * dz2) / (dd1 * dd2);
    dx1 = a.0 + p * dx1 - b.0;
    dy1 = a.1 + p * dy1 - b.1;
    dz1 = a.2 + p * dz1 - b.2;
    dd1 = (dx1 * dx1 + dy1 * dy1 + dz1 * dz1).sqrt();
    if dd1 > tol_min_max { dd1 } else { tol_min_max }
}
