//! FaceFace's spline construction fixes tangents without GeomInt's periodic recentering.
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve, arrays};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkgeombase::{
    app_par_curves::{MultiBSpCurve, MultiCurveEval},
    geom_lib::{Check2dBSplineCurve, CheckBSplineCurve},
};
use rustycad_tkmath::{
    elslib,
    gp::{Plane3, Point2, Point3},
};
pub(super) struct WalkingSpline<'a> {
    curve: &'a MultiBSpCurve,
    knots: Array1<'static, f64>,
    multiplicities: Array1<'static, i32>,
    degree: i32,
}
impl<'a> WalkingSpline<'a> {
    pub(super) fn new(curve: &'a MultiBSpCurve) -> Result<Self, Failure> {
        Ok(Self {
            curve,
            knots: curve.knots()?,
            multiplicities: curve.multiplicities()?,
            degree: MultiCurveEval::degree(curve)?,
        })
    }
    pub(super) fn spatial(&self) -> Result<Handle<dyn GeomCurve>, Failure> {
        let mut poles = arrays::one_based(self.curve.nb_poles())?;
        self.curve.curve_3d(1, &mut poles)?;
        self.spatial_poles(&poles)
    }
    pub(super) fn on_plane(
        &self,
        plane: Plane3,
        index: i32,
    ) -> Result<(Handle<dyn GeomCurve>, Array1<'static, Point2>), Failure> {
        let planar = self.planar_poles(index)?;
        let mut poles = arrays::one_based(self.curve.nb_poles())?;
        for index in 1..=self.curve.nb_poles() {
            let point = arrays::at(&planar, index)?;
            arrays::put(
                &mut poles,
                index,
                elslib::value(point.x(), point.y(), &plane),
            )?;
        }
        Ok((self.spatial_poles(&poles)?, planar))
    }
    fn spatial_poles(&self, poles: &Array1<'_, Point3>) -> Result<Handle<dyn GeomCurve>, Failure> {
        let curve = Handle::new(GeomBSplineCurve::new(
            poles,
            &self.knots,
            &self.multiplicities,
            self.degree,
            false,
        )?);
        CheckBSplineCurve::new(&curve, 1e-7, 1e-6)?.fix_tangent(true, true)?;
        Ok(curve.cast())
    }
    pub(super) fn pcurve(&self, index: i32) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let poles = self.planar_poles(index)?;
        self.curve_from_planar_poles(&poles)
    }
    pub(super) fn curve_from_planar_poles(
        &self,
        poles: &Array1<'_, Point2>,
    ) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let curve = Handle::new(Geom2dBSplineCurve::new(
            poles,
            &self.knots,
            &self.multiplicities,
            self.degree,
            false,
        )?);
        Check2dBSplineCurve::new(&curve, 1e-7, 1e-6)?.fix_tangent(true, true)?;
        Ok(curve.cast())
    }
    fn planar_poles(&self, index: i32) -> Result<Array1<'static, Point2>, Failure> {
        let mut poles = arrays::one_based(self.curve.nb_poles())?;
        self.curve.curve_2d(index, &mut poles)?;
        Ok(poles)
    }
}
