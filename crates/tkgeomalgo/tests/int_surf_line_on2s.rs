//! Direct translation of IntSurf_LineOn2S_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::Handle;
use rustycad_tkgeomalgo::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use rustycad_tkmath::gp::{Point2, Point3};

fn build_point(
    the_point: Point3,
    the_u1: f64,
    the_v1: f64,
    the_u2: f64,
    the_v2: f64,
) -> IntSurfPntOn2S {
    let mut a_pnt = IntSurfPntOn2S::new();
    a_pnt.set_value_all(the_point, the_u1, the_v1, the_u2, the_v2);
    a_pnt
}

#[test]
fn IntSurf_LineOn2S_Test__EmptyLineBoxesAreNotOut() {
    let a_line: Handle<IntSurfLineOn2S> = Handle::new(IntSurfLineOn2S::new(None));
    assert!(!a_line.is_out_box(Point3::new(10.0, 10.0, 10.0)));
    assert!(!a_line.is_out_surf1_box(Point2::new(10.0, 10.0)));
    assert!(!a_line.is_out_surf2_box(Point2::new(10.0, 10.0)));
}

#[test]
fn IntSurf_LineOn2S_Test__PointReplacementInvalidatesCachedBoxes() {
    let a_line: Handle<IntSurfLineOn2S> = Handle::new(IntSurfLineOn2S::new(None));
    a_line.add(&build_point(Point3::new(0.0, 0.0, 0.0), 0.0, 0.0, 0.0, 0.0));
    a_line.add(&build_point(Point3::new(1.0, 1.0, 1.0), 1.0, 1.0, 1.0, 1.0));

    assert!(a_line.is_out_box(Point3::new(100.0, 100.0, 100.0)));
    assert!(a_line.is_out_surf1_box(Point2::new(50.0, 50.0)));

    a_line
        .set_point(2, Point3::new(100.0, 100.0, 100.0))
        .unwrap();
    assert!(!a_line.is_out_box(Point3::new(100.0, 100.0, 100.0)));

    a_line
        .set_value(
            2,
            &build_point(Point3::new(100.0, 100.0, 100.0), 50.0, 50.0, 60.0, 60.0),
        )
        .unwrap();
    assert!(!a_line.is_out_surf1_box(Point2::new(50.0, 50.0)));
    assert!(!a_line.is_out_surf2_box(Point2::new(60.0, 60.0)));
}

#[test]
fn IntSurf_LineOn2S_Test__Split_DividesCorrectly() {
    let a_line: Handle<IntSurfLineOn2S> = Handle::new(IntSurfLineOn2S::new(None));
    a_line.add(&build_point(Point3::new(0.0, 0.0, 0.0), 0.0, 0.0, 0.0, 0.0));
    a_line.add(&build_point(Point3::new(1.0, 0.0, 0.0), 1.0, 0.0, 1.0, 0.0));
    a_line.add(&build_point(Point3::new(2.0, 0.0, 0.0), 2.0, 0.0, 2.0, 0.0));
    a_line.add(&build_point(Point3::new(3.0, 0.0, 0.0), 3.0, 0.0, 3.0, 0.0));

    // NCollection_Sequence::Split(Index, SS) moves items Index..N into SS,
    // keeping 1..Index-1 in original.
    let a_split: Handle<IntSurfLineOn2S> = a_line.split(2).unwrap();

    assert_eq!(a_line.nb_points(), 1);
    assert_eq!(a_split.nb_points(), 3);
    assert!((a_line.value(1).unwrap().value().x() - 0.0).abs() <= 1e-15);
    assert!((a_split.value(1).unwrap().value().x() - 1.0).abs() <= 1e-15);
}
