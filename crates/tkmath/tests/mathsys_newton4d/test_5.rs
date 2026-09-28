#[test]
fn MathSys_Newton4DTest__SolveSurfaceSurfaceExtrema4D_Smoke() {
    let mut aSurf1: PlaneSurfaceEval = PlaneSurfaceEval {
        z: 0.0,
        curvature: 1.0,
    };
    let mut aSurf2: PlaneSurfaceEval = PlaneSurfaceEval {
        z: 2.0,
        curvature: -1.0,
    };

    let mut aBounds: NewtonBoundsN<4> = NewtonBoundsN::default();
    aBounds.min = [-10.0, -10.0, -10.0, -10.0];
    aBounds.max = [10.0, 10.0, 10.0, 10.0];

    let mut aOptions: NewtonOptions = NewtonOptions::default();
    aOptions.solver.f_tolerance = THE_TOL;

    let aResult: NewtonResultN<4> = solve_surface_surface_extrema_4d(
        &mut aSurf1,
        &mut aSurf2,
        [0.1, -0.1, -0.1, 0.1],
        &aBounds,
        &aOptions,
    );

    assert!(aResult.is_done());
    assert!(aResult.iterations > 0);
    for aParameter in aResult.x {
        near(aParameter, 0.0, 1.0e-8);
    }
    near(aResult.residual_norm, 0.0, 1.0e-12);
}
