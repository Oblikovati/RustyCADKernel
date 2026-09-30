// Rust translation of ModelingData/TKG3d/Geom/Geom_BSplineCurve.cxx, Geom_BSplineCurve_1.cxx (virtual members, DumpJson) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The virtual members of Geom_Geometry, Geom_Curve and Geom_BoundedCurve.
use super::GeomBSplineCurve;
use crate::geom::{
    GeomBoundedCurve, GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3, GeomGeometry,
    dump_bounded_curve,
};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};

impl GeomGeometry for GeomBSplineCurve {
    rustycad_tkg2d::spline_geometry_members!(
        Transform3,
        dyn GeomGeometry,
        "Geom_BSplineCurve",
        dump_bounded_curve
    );
}
impl GeomCurve for GeomBSplineCurve {
    fn reverse(&self) -> Result<(), Failure> {
        self.state.write().reverse()
    }
    fn reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        Ok(self.first_parameter() + self.last_parameter() - u)
    }
    fn first_parameter(&self) -> f64 {
        self.state.read().first_parameter()
    }
    fn last_parameter(&self) -> f64 {
        self.state.read().last_parameter()
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        let state = self.state.read();
        let (start, end) = (state.start_point()?, state.end_point()?);
        Ok(start.square_distance(&end) <= precision::computational())
    }
    fn is_periodic(&self) -> bool {
        self.state.read().periodic
    }
    fn continuity(&self) -> Shape {
        self.state.read().smooth
    }
    fn is_cn(&self, n: i32) -> Result<bool, Failure> {
        self.state.read().is_cn(n)
    }
    rustycad_tkg2d::spline_state_evaluators!(
        Point3,
        Vector3,
        GeomCurveD1,
        GeomCurveD2,
        GeomCurveD3
    );
}
impl GeomBoundedCurve for GeomBSplineCurve {
    fn end_point(&self) -> Result<Point3, Failure> {
        self.state.read().end_point()
    }
    fn start_point(&self) -> Result<Point3, Failure> {
        self.state.read().start_point()
    }
}
