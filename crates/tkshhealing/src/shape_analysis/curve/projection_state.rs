//! Native projection output and candidate state, including outputs retained on failure.
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::Point3;

/// Mutable native projection outputs; `CurveProjection::default()` starts at the origin.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CurveProjection {
    /// The projected point (`proj`).
    pub point: Point3,
    /// Its curve parameter (`param`).
    pub parameter: f64,
}
impl CurveProjection {
    pub(super) fn tuple(self, distance: f64) -> (f64, Point3, f64) {
        (distance, self.point, self.parameter)
    }
}
pub(super) struct ProjectionSearch {
    pub bounds: (f64, f64),
    pub computed: CurveProjection,
    pub old: Option<CurveProjection>,
    pub minimum: f64,
    pub closed: bool,
    pub period: f64,
}
impl ProjectionSearch {
    pub fn new(curve: &dyn Adaptor3dCurve, output: CurveProjection) -> Result<Self, Failure> {
        Ok(Self {
            bounds: (curve.first_parameter()?, curve.last_parameter()?),
            computed: output,
            old: None,
            minimum: precision::infinite(),
            closed: false,
            period: 0.0,
        })
    }
    pub fn remember(&mut self, curve: &dyn Adaptor3dCurve, point: Point3) -> Result<(), Failure> {
        self.old = Some(self.computed);
        self.minimum = self.computed.point.distance(&point);
        if curve.is_closed()? {
            self.closed = true;
            self.period = self.bounds.1 - self.bounds.0;
        }
        Ok(())
    }
}
