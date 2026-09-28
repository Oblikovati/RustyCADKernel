#[test]
fn MathSys_Newton3DTest__SolveCurveSurfaceExtrema3D_Smoke() {
    let mut aCurve: LineCurveEval = LineCurveEval;
    let mut aSurface: PlaneSurfaceEval = PlaneSurfaceEval;

    let mut aBounds: NewtonBoundsN<3> = NewtonBoundsN::default();
    aBounds.min = [-10.0, -10.0, -10.0];
    aBounds.max = [10.0, 10.0, 10.0];

    let mut aOptions: NewtonOptions = NewtonOptions::default();
    aOptions.solver.f_tolerance = THE_TOL;
    aOptions.solver.max_iterations = 20;

    let aResult: NewtonResultN<3> = solve_curve_surface_extrema_3d(
        &mut aCurve,
        &mut aSurface,
        [1.0, 2.0, -3.0],
        &aBounds,
        &aOptions,
    );

    assert!(aResult.is_done());
    assert!(aResult.iterations > 0);
    near(aResult.x[0], 0.0, 1.0e-12);
    near(aResult.x[1], 0.0, 1.0e-12);
    near(aResult.x[2], 0.0, 1.0e-12);
    near(aResult.residual_norm, 0.0, 1.0e-12);
}
