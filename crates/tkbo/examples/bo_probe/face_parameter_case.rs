use super::{face_boundary_parameter::BoundaryParameter, flag, real};
use rustycad_tkbo::int_tools::Context;
use rustycad_tkbrep::{brep::BRepBuilder, brep_adaptor::BRepAdaptorSurface, topods::Face};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkmath::gp::{Axis3, Cylinder3, Direction3, Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace as MakeFace;
fn run(v: &[f64]) -> Result<(), Failure> {
    let curve = super::tools_geometry::geometry(v[0] as i32, 0., std::f64::consts::TAU)?;
    let faces = [
        super::fclass_faces::face(v[1] as i32, 3.)?,
        super::fclass_faces::face(v[2] as i32, 3.)?,
    ];
    let request = BoundaryParameter {
        parameter: v[3],
        other: v[4],
        increase: v[5] != 0.,
        tolerance: v[6],
    };
    let mut parameter = 99.;
    flag(request.compute(&curve, &faces, &Context::default(), &mut parameter)?);
    real(parameter);
    Ok(())
}
pub fn case(v: &[f64]) {
    if let Err(error) = run(v) {
        print!("{} ", error.exception_type());
    }
}
fn analytic_faces(v: &[f64]) -> Result<[Face; 2], Failure> {
    let plane = Handle::new(GeomPlane::from_point(
        Point3::default(),
        Direction3::new(1., 0., v[0])?,
    )?)
    .cast();
    let mut first = Face::default();
    BRepBuilder::default().make_face_on_surface(&mut first, &plane, 1e-7)?;
    let second = MakeFace::from_cylinder_bounds(
        Cylinder3::new(Axis3::default(), 3.)?,
        (0., std::f64::consts::TAU, -v[1], v[1]),
    )?
    .face()?;
    Ok(match v[3] as i32 {
        1 => [second, first],
        2 => [second.clone(), second],
        3 => [first.clone(), first],
        _ => [first, second],
    })
}
fn analytic(v: &[f64]) -> Result<(), Failure> {
    let faces = analytic_faces(v)?;
    let first = BRepAdaptorSurface::with_face(&faces[0], true)?;
    let second = BRepAdaptorSurface::with_face(&faces[1], true)?;
    flag(super::face_analytic::treat_analytically(
        &first, &second, v[2],
    )?);
    Ok(())
}
pub fn analytic_case(v: &[f64]) {
    if let Err(error) = analytic(v) {
        print!("{} ", error.exception_type());
    }
}
