struct LineCurveEval;
impl CurveEvaluatorD2 for LineCurveEval {
    fn d2(&mut self, t: f64) -> CurveD2 {
        CurveD2 {
            point: Point3::new(0.0, 0.0, t),
            d1: Vector3::new(0.0, 0.0, 1.0),
            d2: Vector3::default(),
        }
    }
}
struct PlaneSurfaceEval;
impl SurfaceEvaluatorD2 for PlaneSurfaceEval {
    fn d2(&mut self, u: f64, v: f64) -> SurfaceD2 {
        SurfaceD2 {
            point: Point3::new(u, v, 0.0),
            du: Vector3::new(1.0, 0.0, 0.0),
            dv: Vector3::new(0.0, 1.0, 0.0),
            ..Default::default()
        }
    }
}
