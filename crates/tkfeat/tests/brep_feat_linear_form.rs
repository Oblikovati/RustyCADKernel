//! Linear forms (ribs and slots) on the box `[0, 10]^3`: the open wire
//! `(2, 5, 10) (2, 5, h) (8, 5, h) (8, 5, 10)` in the plane `y = 5`, swept along +Y
//! (and back along -Y). A rib of height 3 adds its 6 x 3 profile times its thickness
//! (1, or 2 with the second direction); a slot closes its profile on the bounding box
//! of the basis, so it removes more than the wire encloses (the volumes libTKFeat
//! gives for the same cases).
mod support;

use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Handle;
use rustycad_tkfeat::brep_feat::{BRepFeatMakeLinearForm, StatusError};
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Direction3, Plane3, Point3, Vector3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeShape};
use support::{assert_close, is_valid, make_box, sub_shapes, volume};

fn form(mode: i32, slide: bool, back: bool) -> BRepFeatMakeLinearForm {
    let sbase = make_box((0., 0., 0.), (10., 10., 10.));
    let h = if mode != 0 { 13. } else { 7. };
    let mut poly = BRepBuilderAPIMakePolygon::new();
    for (x, z) in [(2., 10.), (2., h), (8., h), (8., 10.)] {
        poly.add_point(Point3::new(x, 5., z)).unwrap();
    }
    let frame = Plane3::new(
        Point3::new(0., 5., 0.),
        Direction3::new(0., 1., 0.).unwrap(),
    )
    .unwrap();
    let pln = Handle::new(GeomPlane::from_pln(frame));
    let dir1 = if back {
        Vector3::new(0., -1., 0.)
    } else {
        Vector3::new(0., 0., 0.)
    };
    let w = poly.wire().unwrap();
    BRepFeatMakeLinearForm::with_wire(
        &sbase,
        &w,
        &pln,
        Vector3::new(0., 1., 0.),
        dir1,
        mode,
        slide,
    )
    .unwrap()
}

/// The result: its volume, after checking it is done, valid and of `nf` faces.
fn result(f: &mut BRepFeatMakeLinearForm, nf: usize) -> f64 {
    f.perform().unwrap();
    assert!(f.is_done(), "status {:?}", f.rib().current_status_error());
    assert_eq!(f.rib().current_status_error(), StatusError::Ok);
    let s: Shape = f.shape().unwrap().clone();
    assert!(is_valid(&s));
    // the faces as the explorer meets them, as counted natively
    assert_eq!(sub_shapes(&s, ShapeEnum::Face).len(), nf);
    volume(&s)
}

#[test]
fn rib_centred_on_the_plane_adds_its_profile() {
    for slide in [false, true] {
        let mut f = form(1, slide, false);
        assert_close(result(&mut f, 11), 1018., 1e-12);
    }
}

#[test]
fn rib_along_both_directions_is_twice_as_thick() {
    for slide in [false, true] {
        let mut f = form(1, slide, true);
        assert_close(result(&mut f, 11), 1036., 1e-12);
    }
}

#[test]
fn slot_closes_its_profile_on_the_bounding_box() {
    for (back, removed) in [(false, 82.), (true, 164.)] {
        for slide in [false, true] {
            let mut f = form(0, slide, back);
            assert_close(result(&mut f, 14), 1000. - removed, 1e-12);
        }
    }
}
