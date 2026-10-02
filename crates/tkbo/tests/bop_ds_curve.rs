//! Regression coverage for native `BOPDS_Curve` storage behavior.
use rustycad_tkbo::bop_ds::{BopdsCurve, PaveBlock, pave_block_handle};
use rustycad_tkbo::int_tools::Curve;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Axis1, Axis2d, Direction2, Direction3, Point2, Point3};
use std::sync::Arc;

#[test]
fn default_curve_storage_and_empty_initial_block_match_native() {
    let mut record = BopdsCurve::default();
    assert_eq!(record.tolerance(), 0.0);
    assert_eq!(record.tangential_tolerance(), 0.0);
    assert!(record.box_bounds().is_void());
    assert!(record.pave_blocks().is_empty());
    assert!(record.techno_vertices().is_empty());
    assert!(!record.has_edge());
    assert_eq!(
        record.change_pave_block1().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );

    record.init_pave_block1();
    let first = Arc::clone(record.change_pave_block1().unwrap());
    record.init_pave_block1();
    assert_eq!(record.pave_blocks().len(), 1);
    assert!(Arc::ptr_eq(&record.pave_blocks()[0], &first));
    assert!(!record.has_edge());
}

#[test]
fn curve_box_blocks_vertices_and_tolerances_preserve_native_mutations() {
    let mut record = BopdsCurve::default();
    let spatial: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::new(
        Point3::new(1.0, 2.0, 3.0),
        Direction3::X,
    )))
    .into_cast();
    let first_2d: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::default(),
        Direction2::X,
    )))
    .into_cast();
    let second_2d = first_2d.clone();
    let mut curve = Curve::new(&spatial, &first_2d, &second_2d, 1.25e-7, 2.5e-6);
    curve.set_tangential_tolerance(4.5e-5);
    record.set_curve(&curve);
    assert!(record.curve().curve() == &spatial);
    assert!(record.curve().first_curve2d() == &first_2d);
    assert!(record.curve().second_curve2d() == &second_2d);
    assert_eq!(record.curve().tolerance(), 1.25e-7);
    assert_eq!(record.tangential_tolerance(), 4.5e-5);

    let mut bounds = BoundingBox3::new(Point3::new(-1.0, -2.0, -3.0), Point3::new(4.0, 5.0, 6.0));
    bounds.set_gap(0.125);
    record.set_box(&bounds);
    bounds.set_gap(9.0);
    assert_eq!(record.box_bounds().gap(), 0.125);
    record.change_box().set_gap(0.25);
    assert_eq!(record.box_bounds().gap(), 0.25);

    let first = block(2);
    let edge = block(5);
    let duplicate = Arc::clone(&edge);
    record.set_pave_blocks(&[first.clone(), edge.clone(), duplicate.clone()]);
    assert_eq!(record.pave_blocks().len(), 3);
    assert!(Arc::ptr_eq(&record.pave_blocks()[0], &first));
    assert!(Arc::ptr_eq(&record.pave_blocks()[1], &edge));
    assert!(Arc::ptr_eq(&record.pave_blocks()[2], &duplicate));
    assert!(!record.has_edge());
    edge.lock().unwrap().set_edge(12);
    assert!(record.has_edge());
    record.change_pave_blocks().pop();
    assert_eq!(record.pave_blocks().len(), 2);

    record.change_techno_vertices().extend([8, 3, 8]);
    assert_eq!(record.techno_vertices(), &[8, 3, 8]);
    record.set_tolerance(-0.0);
    assert_eq!(record.tolerance().to_bits(), (-0.0_f64).to_bits());
}

fn block(edge: i32) -> rustycad_tkbo::bop_ds::PaveBlockHandle {
    let mut block = PaveBlock::default();
    block.set_original_edge(edge);
    pave_block_handle(block)
}
