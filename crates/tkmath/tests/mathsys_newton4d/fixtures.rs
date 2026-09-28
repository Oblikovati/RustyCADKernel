struct PlaneSurfaceEval {
    z: f64,
    curvature: f64,
}
impl SurfaceEvaluatorD2 for PlaneSurfaceEval {
    fn d2(&mut self, u: f64, v: f64) -> SurfaceD2 {
        SurfaceD2 {
            point: Point3::new(u, v, self.z + self.curvature * (u * u + v * v)),
            du: Vector3::new(1.0, 0.0, 2.0 * self.curvature * u),
            dv: Vector3::new(0.0, 1.0, 2.0 * self.curvature * v),
            duu: Vector3::new(0.0, 0.0, 2.0 * self.curvature),
            dvv: Vector3::new(0.0, 0.0, 2.0 * self.curvature),
            duv: Vector3::default(),
        }
    }
}
