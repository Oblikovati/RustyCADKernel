//! Prove that the scoped pole reader borrows the stored grid, not a cloned grid.
use super::*;
use rustycad_tkernel::standard::Handle;

fn sample_surface() -> GeomBSplineSurface {
    let mut poles = Array2::with_bounds(1, 2, 1, 2).unwrap();
    for (i, j) in [(1, 1), (1, 2), (2, 1), (2, 2)] {
        poles[(i, j)] = Point3::new(f64::from(i), f64::from(j), 0.0);
    }
    let mut knots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    let mut mults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    knots.as_mut_slice().copy_from_slice(&[0.0, 1.0]);
    mults.as_mut_slice().copy_from_slice(&[2, 2]);
    GeomBSplineSurface::new(
        &poles,
        (&knots, &mults),
        (&knots, &mults),
        (1, 1),
        (false, false),
    )
    .unwrap()
}

#[test]
fn poles_array_points_at_stored_surface_grid() {
    let surface: Handle<GeomBSplineSurface> = Handle::new(sample_surface());
    let stored = surface.state.read().poles.as_slice().as_ptr();
    let observed = surface.poles_array(|poles| poles.as_slice().as_ptr());
    assert_eq!(stored, observed);
}
