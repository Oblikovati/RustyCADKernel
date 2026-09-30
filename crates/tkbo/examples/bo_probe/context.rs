use super::{fclass_faces, flag, integer, real};
use rustycad_tkbo::int_tools::Context;
use rustycad_tkbrep::{brep::BRepTool, topods::Face};
use rustycad_tkernel::{ncollection::BaseAllocator, standard::Failure};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
use std::sync::Arc;
fn face_cache(c: &Context, f: &Face) -> Result<(), Failure> {
    let reversed = Face::unchecked(f.oriented(Orientation::Reversed));
    flag(Arc::ptr_eq(&c.fclass2d(f)?, &c.fclass2d(&reversed)?));
    flag(Arc::ptr_eq(
        &c.surface_adaptor(f)?,
        &c.surface_adaptor(&reversed)?,
    ));
    let (a, b, d, e) = c.uv_bounds(f)?;
    for x in [a, b, d, e] {
        real(x);
    }
    localization(c, f, &reversed)?;
    flag(c.is_infinite_face(f)?);
    Ok(())
}
fn localization(c: &Context, f: &Face, reversed: &Face) -> Result<(), Failure> {
    let localization = c.surface_data(f)?;
    localization.write().set_grid_deflection(0.125);
    flag(Arc::ptr_eq(&localization, &c.surface_data(reversed)?));
    let localization = localization.read();
    integer(localization.get_nb_sample_u());
    integer(localization.get_nb_sample_v());
    real(localization.get_min_range_u());
    real(localization.get_min_range_v());
    real(localization.get_grid_deflection());
    Ok(())
}
fn classification(c: &Context, f: &Face, v: &[f64]) -> Result<(), Failure> {
    let uv = Point2::new(v[2], v[3]);
    let p = Point3::new(v[2], v[3], v[4]);
    integer(c.state_point_face(f, uv)? as i32);
    flag(c.is_point_in_face(f, uv)?);
    flag(c.is_point_in_on_face(f, uv)?);
    flag(c.is_point3d_in_face(p, f, v[5])?);
    flag(c.is_valid_point_for_face(p, f, v[5])?);
    flag(c.is_valid_point_for_faces(p, f, f, v[5])?);
    let projector = c.proj_ps(f)?;
    flag(projector.read().is_done());
    c.set_pons_projection_tolerance(1e-9);
    flag(c.proj_ps(f)?.read().is_done());
    flag(c.is_valid_point_for_face(p, f, v[5])?);
    Ok(())
}
fn hatching(c: &Context, f: &Face) -> Result<(), Failure> {
    let service = c.hatcher(f)?;
    flag(Arc::ptr_eq(&service, &c.hatcher(f)?));
    let h = service.read();
    real(h.confusion2d());
    real(h.confusion3d());
    real(h.intersector().confusion_tolerance());
    real(h.intersector().tangency_tolerance());
    flag(h.keep_points());
    flag(h.keep_segments());
    Ok(())
}
fn projection(c: &Context, p: Point3) -> Result<(), Failure> {
    let e = BRepBuilderAPIMakeEdge::from_points(Point3::new(0., 0., 0.), Point3::new(2., 3., 4.))?
        .edge()?;
    let projector = c.proj_pc(&e)?;
    flag(Arc::ptr_eq(&projector, &c.proj_pc(&e)?));
    let (curve, _, _) = BRepTool::curve(&e)?;
    let other = c.proj_pt(&curve)?;
    flag(Arc::ptr_eq(&other, &c.proj_pt(&curve.clone())?));
    for projector in [projector, other] {
        let mut pr = projector.write();
        pr.perform(p)?;
        integer(pr.nb_points());
        if pr.nb_points() > 0 {
            real(pr.lower_distance()?);
            real(pr.lower_distance_parameter()?);
        }
    }
    Ok(())
}
fn solids(c: &Context, p: Point3) -> Result<(), Failure> {
    let solid = BRepPrimAPIMakeBox::from_sizes(2., 3., 4.)?.solid()?;
    let service = c.solid_classifier(&solid)?;
    flag(Arc::ptr_eq(&service, &c.solid_classifier(&solid)?));
    let mut classifier = service.write();
    classifier.perform(&p, 1e-7)?;
    integer(classifier.state() as i32);
    drop(classifier);
    boxes(c, &solid)
}
fn boxes(c: &Context, solid: &rustycad_tkbrep::topods::Solid) -> Result<(), Failure> {
    let box3 = c.bnd_box(solid)?;
    flag(Arc::ptr_eq(&box3, &c.bnd_box(solid)?));
    for point in [box3.read().corner_min()?, box3.read().corner_max()?] {
        real(point.x());
        real(point.y());
        real(point.z());
    }
    let obb = c.obb(solid, 0.1)?;
    flag(Arc::ptr_eq(&obb, &c.obb(solid, 5.)?));
    let b = obb.read();
    real(b.x_h_size());
    real(b.y_h_size());
    real(b.z_h_size());
    Ok(())
}
fn run(v: &[f64]) -> Result<(), Failure> {
    let context = Context::default();
    let face = fclass_faces::face(v[0] as i32, v[1])?;
    face_cache(&context, &face)?;
    classification(&context, &face, v)?;
    hatching(&context, &face)?;
    let p = Point3::new(v[2], v[3], v[4]);
    projection(&context, p)?;
    solids(&context, p)?;
    let allocated = Context::with_allocator(BaseAllocator::incremental(4096));
    face_cache(&allocated, &face)?;
    classification(&allocated, &face, v)?;
    Ok(())
}
pub fn case(v: &[f64]) {
    if let Err(error) = run(v) {
        print!("{} ", error.exception_type());
    }
}
