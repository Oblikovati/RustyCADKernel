//! Revolution forms (ribs and grooves of revolution) around the Z axis: the open wire
//! `(2, 0, 10) (2, 0, h) (4, 0, h) (4, 0, 10)` in the plane `y = 0`, standing on the top
//! of a cylinder of radius 5 (or of the box `[-5, 5]^2 x [0, 10]`), cut by the planes
//! `y = +-1`. The volumes are those libTKFeat gives for the same cases (its sliding
//! grooves exhaust the memory natively and are left out); every result is valid.
mod support;

use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Handle;
use rustycad_tkfeat::brep_feat::{BRepFeatMakeRevolutionForm, StatusError};
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis1, Direction3, Plane3, Point3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeCylinder;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeShape};
use support::{assert_close, is_valid, make_box, sub_shapes, volume};

fn basis(boxed: bool) -> Shape {
    if boxed {
        return make_box((-5., -5., 0.), (10., 10., 10.));
    }
    BRepPrimAPIMakeCylinder::new(5., 10.)
        .shape()
        .unwrap()
        .clone()
}

fn form(boxed: bool, mode: i32, slide: bool) -> BRepFeatMakeRevolutionForm {
    let h = if mode != 0 { 12. } else { 8. };
    let mut poly = BRepBuilderAPIMakePolygon::new();
    for (x, z) in [(2., 10.), (2., h), (4., h), (4., 10.)] {
        poly.add_point(Point3::new(x, 0., z)).unwrap();
    }
    let pln = Handle::new(GeomPlane::from_pln(
        Plane3::new(
            Point3::new(0., 0., 0.),
            Direction3::new(0., 1., 0.).unwrap(),
        )
        .unwrap(),
    ));
    let axis = Axis1::new(
        Point3::new(0., 0., 0.),
        Direction3::new(0., 0., 1.).unwrap(),
    );
    let w = poly.wire().unwrap();
    BRepFeatMakeRevolutionForm::with_wire(&basis(boxed), &w, &pln, axis, 1., 1., mode, slide)
        .unwrap()
}

/// The result: its volume, after checking it is done, valid and of `nf` faces.
fn result(f: &mut BRepFeatMakeRevolutionForm, nf: usize) -> f64 {
    f.perform().unwrap();
    assert!(f.is_done(), "status {:?}", f.rib().current_status_error());
    assert_eq!(f.rib().current_status_error(), StatusError::Ok);
    let s = f.shape().unwrap().clone();
    assert!(is_valid(&s));
    // the faces as the explorer meets them (a face shared by two solids twice)
    assert_eq!(sub_shapes(&s, ShapeEnum::Face).len(), nf);
    volume(&s)
}

#[test]
fn rib_of_revolution_on_a_cylinder() {
    for slide in [false, true] {
        let mut f = form(false, 1, slide);
        assert_close(result(&mut f, 8), 793.5770064344855, 1e-12);
    }
}

#[test]
fn rib_of_revolution_on_a_box() {
    for slide in [false, true] {
        let mut f = form(true, 1, slide);
        assert_close(result(&mut f, 11), 1008.1788430370369, 1e-12);
    }
}

#[test]
fn fixed_groove_keeps_the_basis_as_natively() {
    let mut f = form(false, 0, false);
    assert_close(result(&mut f, 22), 250. * std::f64::consts::PI, 1e-12);
    let mut f = form(true, 0, false);
    assert_close(result(&mut f, 25), 1000., 1e-12);
}
