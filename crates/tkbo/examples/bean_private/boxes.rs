use super::{integer, real};
use crate::{bean_boxes, bean_faces};
use rustycad_tkbo::int_tools::SurfaceRangeLocalizeData;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::{ncollection::ArrayError, standard::Failure};
use rustycad_tkmath::{bnd::BoundingBox3, gp::Point3};
fn grid(cache: &mut SurfaceRangeLocalizeData, count: i32) -> Result<(), ArrayError> {
    cache.set_range_u_grid(count)?;
    cache.set_range_v_grid(count)?;
    for i in 1..=count {
        cache.set_u_param(i, f64::from(i - 2) * 0.5)?;
        cache.set_v_param(i, f64::from(i - 2) * 0.25)?;
        for j in 1..=count {
            // Deliberately distinct from surface evaluation: BuildBox must use stored points.
            cache.set_grid_point(
                i,
                j,
                Point3::new(f64::from(i), -f64::from(j), f64::from(i * j)),
            )?;
        }
    }
    Ok(())
}
fn snapshot(bounds: &BoundingBox3) -> Result<(), Failure> {
    real(bounds.gap());
    for value in bounds.get()? {
        real(value);
    }
    Ok(())
}
pub(super) fn run(values: &[f64]) -> Result<(), Failure> {
    let surface = BRepAdaptorSurface::with_face(&bean_faces::face(values[0] as i32, 2.)?, false)?;
    let mut cache = SurfaceRangeLocalizeData::new(3, 3, 0.01, 0.01);
    grid(&mut cache, values[1] as i32).map_err(|e| e.into_failure("bean box grid"))?;
    cache.set_grid_deflection(values[6]);
    let bounds = [values[2], values[3], values[4], values[5]];
    let mut output = BoundingBox3::default();
    if values[8] != 0. {
        output.add_point(Point3::new(17., -19., 23.));
        output.set_gap(0.125);
    }
    bean_boxes::build_box(&surface, bounds, &mut cache, &mut output)?;
    snapshot(&output)?;
    integer(cache.get_nb_u_points_in_frame());
    integer(cache.get_nb_v_points_in_frame());
    snapshot(&bean_boxes::surface_box(
        &surface, bounds, values[7], &mut cache,
    )?)
}
