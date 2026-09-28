//=================================================================================================
// Rational surface tests
//=================================================================================================

// Create a rational biquadratic Bezier surface (can represent sphere patch)
fn sphere_patch() -> (Array2<'static, Point3>, Array2<'static, f64>) {
    let rows: [[[f64; 3]; 3]; 3] = [
        // Row 1 (v=0)
        [[1., 0., 0.], [1., 1., 0.], [0., 1., 0.]],
        // Row 2 (v=0.5)
        [[1., 0., 1.], [1., 1., 1.], [0., 1., 1.]],
        // Row 3 (v=1)
        [[0., 0., 1.], [0., 0., 1.], [0., 0., 1.]],
    ];
    let aPoles = pole_grid(3, 3, |i, j| rows[j as usize - 1][i as usize - 1]);
    let aSqrt2_2: f64 = 0.707106781186548;
    let weights: [[f64; 3]; 3] = [[1.0, aSqrt2_2, 1.0], [aSqrt2_2, 0.5, aSqrt2_2], [1.0, aSqrt2_2, 1.0]];
    let mut aWeights: Array2<'static, f64> = Array2::with_bounds(1, 3, 1, 3).unwrap();
    for i in 1..=3 {
        for j in 1..=3 {
            aWeights[(i, j)] = weights[j as usize - 1][i as usize - 1];
        }
    }
    (aPoles, aWeights)
}

#[test]
fn BSplSLib_CacheTest__D0_RationalSurface() {
    let (aPoles, aWeights) = sphere_patch();
    let (du, dv) = (bezier_direction(2), bezier_direction(2));

    // Create cache
    let mut aCache = new_cache(&du, &dv, &aPoles, Some(&aWeights));

    // Test at several parameter values
    for u in steps(0.2) {
        for v in steps(0.2) {
            // Cached evaluation
            locate(&mut aCache, (u, v), (&du, &dv), &aPoles, Some(&aWeights));
            let aCachePnt: Point3 = aCache.d0(u, v).unwrap();

            // Direct evaluation using BSplSLib::D0
            let aDirectPnt: Point3 =
                bsplslib::d0(&span(u, v, &du, &dv, true), &aPoles, Some(&aWeights)).unwrap();

            // Compare
            expect_xyz(pnt(aCachePnt), pnt(aDirectPnt), "Rational D0", u, v);
        }
    }
}

#[test]
fn BSplSLib_CacheTest__D1_RationalSurface() {
    let (aPoles, aWeights) = sphere_patch();
    let (du, dv) = (bezier_direction(2), bezier_direction(2));

    // Create cache
    let mut aCache = new_cache(&du, &dv, &aPoles, Some(&aWeights));

    // Test at several parameter values
    for u in steps(0.2) {
        for v in steps(0.2) {
            // Cached evaluation
            locate(&mut aCache, (u, v), (&du, &dv), &aPoles, Some(&aWeights));
            let (aCachePnt, aCacheTanU, aCacheTanV) = aCache.d1(u, v).unwrap();

            // Direct evaluation using BSplSLib::D1
            let (aDirectPnt, aDirectTanU, aDirectTanV) =
                bsplslib::d1(&span(u, v, &du, &dv, true), &aPoles, Some(&aWeights)).unwrap();

            // Compare point
            expect_xyz(pnt(aCachePnt), pnt(aDirectPnt), "Rational D1 point", u, v);
            // Compare tangent U
            expect_xyz(vec3(aCacheTanU), vec3(aDirectTanU), "Rational D1 tangentU", u, v);
            // Compare tangent V
            expect_xyz(vec3(aCacheTanV), vec3(aDirectTanV), "Rational D1 tangentV", u, v);
        }
    }
}

#[test]
fn BSplSLib_CacheTest__D2_RationalSurface() {
    let (aPoles, aWeights) = sphere_patch();
    let (du, dv) = (bezier_direction(2), bezier_direction(2));

    // Create cache
    let mut aCache = new_cache(&du, &dv, &aPoles, Some(&aWeights));

    // Test at several parameter values
    for u in steps(0.2) {
        for v in steps(0.2) {
            // Cached evaluation
            locate(&mut aCache, (u, v), (&du, &dv), &aPoles, Some(&aWeights));
            let aCache2: SurfaceD2 = aCache.d2(u, v).unwrap();

            // Direct evaluation using BSplSLib::D2
            let aDirect: SurfaceD2 =
                bsplslib::d2(&span(u, v, &du, &dv, true), &aPoles, Some(&aWeights)).unwrap();

            expect_d2(&aCache2, &aDirect, "Rational D2", u, v);
        }
    }
}

//=================================================================================================
// Test with different degrees in U and V
//=================================================================================================

// Fill poles: aPoles(i, j) = ((i-1), (j-1), sin((i-1)*0.5) * cos((j-1)*0.5)).
fn wavy_poles(rows: i32, cols: i32) -> Array2<'static, Point3> {
    pole_grid(rows, cols, |i, j| {
        let (x, y): (f64, f64) = (f64::from(i - 1) * 1.0, f64::from(j - 1) * 1.0);
        [x, y, (f64::from(i - 1) * 0.5).sin() * (f64::from(j - 1) * 0.5).cos()]
    })
}

// D1 of the cache against BSplSLib::D1, X components only, as the native test.
fn different_degrees(aDegreeU: i32, aDegreeV: i32, prefix: &str) {
    let aPoles = wavy_poles(aDegreeU + 1, aDegreeV + 1);
    let (du, dv) = (bezier_direction(aDegreeU), bezier_direction(aDegreeV));

    // Create cache
    let mut aCache = new_cache(&du, &dv, &aPoles, None);

    // Test at several parameter values
    for u in steps(0.25) {
        for v in steps(0.25) {
            // Cached evaluation
            locate(&mut aCache, (u, v), (&du, &dv), &aPoles, None);
            let (aCachePnt, aCacheTanU, aCacheTanV) = aCache.d1(u, v).unwrap();

            // Direct evaluation using BSplSLib::D1
            let (aDirectPnt, aDirectTanU, aDirectTanV) =
                bsplslib::d1(&span(u, v, &du, &dv, false), &aPoles, None).unwrap();

            // Compare
            let message = |what: &str| format!("{prefix} D1 {what} X mismatch at u=");
            EXPECT_NEAR(aCachePnt.x(), aDirectPnt.x(), THE_TOLERANCE, &message("point"), u, v);
            EXPECT_NEAR(aCacheTanU.x(), aDirectTanU.x(), THE_TOLERANCE, &message("tanU"), u, v);
            EXPECT_NEAR(aCacheTanV.x(), aDirectTanV.x(), THE_TOLERANCE, &message("tanV"), u, v);
        }
    }
}

#[test]
fn BSplSLib_CacheTest__D1_DifferentDegrees_UGreaterV() {
    // Create a surface with degree 3 in U, degree 2 in V
    different_degrees(3, 2, "DifferentDeg");
}

#[test]
fn BSplSLib_CacheTest__D1_DifferentDegrees_VGreaterU() {
    // Create a surface with degree 2 in U, degree 3 in V
    different_degrees(2, 3, "DifferentDeg V>U");
}
