use super::{integer, real};
use crate::bean_grid;
use rustycad_tkbo::int_tools::SurfaceRangeLocalizeData;
use rustycad_tkbrep::{brep::BRepBuilder, brep_adaptor::BRepAdaptorSurface, topods::Face};
use rustycad_tkernel::{
    ncollection::{Array1, Array2},
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::GeomBSplineSurface;
use rustycad_tkmath::gp::Point3;
fn spline(degree: i32, spans: i32, height: f64) -> Handle<GeomBSplineSurface> {
    let count = degree + spans;
    let mut poles = Array2::with_bounds(1, count, 1, count).unwrap();
    for i in 1..=count {
        for j in 1..=count {
            poles
                .set_value(
                    i,
                    j,
                    Point3::new(
                        f64::from(i),
                        f64::from(j),
                        if (i + j) % 2 == 0 { height } else { -height },
                    ),
                )
                .unwrap();
        }
    }
    let knots: Vec<f64> = (0..=spans).map(f64::from).collect();
    let mult: Vec<i32> = (0..=spans)
        .map(|i| if i == 0 || i == spans { degree + 1 } else { 1 })
        .collect();
    let knots = Array1::from_shared(&knots, 1);
    let mult = Array1::from_shared(&mult, 1);
    Handle::new(
        GeomBSplineSurface::new(
            &poles,
            (&knots, &mult),
            (&knots, &mult),
            (degree, degree),
            (false, false),
        )
        .unwrap(),
    )
}
fn snapshot(cache: &SurfaceRangeLocalizeData) {
    let nu = cache.get_range_u_grid();
    let nv = cache.get_range_v_grid();
    integer(nu);
    integer(nv);
    real(cache.get_grid_deflection());
    for i in 1..=nu {
        real(cache.get_u_param(i).unwrap());
    }
    for j in 1..=nv {
        real(cache.get_v_param(j).unwrap());
    }
    for i in 1..=nu {
        for j in 1..=nv {
            let point = cache.get_grid_point(i, j).unwrap();
            real(point.x());
            real(point.y());
            real(point.z());
        }
    }
}
pub(super) fn run(values: &[f64]) -> Result<(), Failure> {
    let spline = spline(values[0] as i32, values[1] as i32, values[2]);
    let mut face = Face::default();
    BRepBuilder::default().make_face_on_surface(&mut face, &spline.clone().cast(), 1e-7)?;
    let surface = BRepAdaptorSurface::with_face(&face, false)?;
    let mut cache = SurfaceRangeLocalizeData::new(3, 3, 0.01, 0.01);
    let bounds = [values[3], values[4], values[5], values[6]];
    bean_grid::compute_grid(&surface, &spline, bounds, values[7], &mut cache)?;
    snapshot(&cache);
    bean_grid::compute_grid(
        &surface,
        &spline,
        [0., values[1], 0., values[1]],
        values[7],
        &mut cache,
    )?;
    snapshot(&cache);
    Ok(())
}
