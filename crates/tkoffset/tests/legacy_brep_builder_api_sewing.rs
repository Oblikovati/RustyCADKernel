//! Exact translations of TKOffset/GTests/BRepBuilderAPI_Sewing_Test.cxx at OCCT 3d097a0.
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPISewing};

/// The first two faces of a 10-cube in explorer order.
fn first_two_box_faces() -> Result<(Face, Face), Failure> {
    let mut box_maker = BRepPrimAPIMakeBox::from_sizes(10.0, 10.0, 10.0)?;
    let a_box: Shape = box_maker.shape()?.clone();
    assert!(box_maker.is_done());
    let mut an_exp = Explorer::new(&a_box, ShapeEnum::Face, ShapeEnum::Shape)?;
    assert!(an_exp.more());
    let a_face1 = topods::face(an_exp.current())?;
    an_exp.next()?;
    assert!(an_exp.more());
    let a_face2 = topods::face(an_exp.current())?;
    Ok((a_face1, a_face2))
}

/// A default sewing of the first two faces of a 10-cube, performed.
fn sewn_two_faces() -> Result<BRepBuilderAPISewing, Failure> {
    let (a_face1, a_face2) = first_two_box_faces()?;
    let mut a_sewing = BRepBuilderAPISewing::default();
    a_sewing.add(a_face1.as_ref())?;
    a_sewing.add(a_face2.as_ref())?;
    a_sewing.perform(&ProgressRange::default())?;
    Ok(a_sewing)
}

// TEST(BRepBuilderAPI_SewingTest, SewTwoFaces)
#[test]
fn sew_two_faces() -> Result<(), Failure> {
    let a_sewing = sewn_two_faces()?;
    assert!(!a_sewing.sewed_shape().is_null());
    Ok(())
}

// TEST(BRepBuilderAPI_SewingTest, NbFreeEdges)
#[test]
fn nb_free_edges() -> Result<(), Failure> {
    let a_sewing = sewn_two_faces()?;
    // Two faces from a box share one edge; the remaining edges are free.
    assert!(a_sewing.nb_free_edges() > 0);
    Ok(())
}

// TEST(BRepBuilderAPI_SewingTest, NbDegeneratedShapes)
#[test]
fn nb_degenerated_shapes() -> Result<(), Failure> {
    let a_sewing = sewn_two_faces()?;
    // Planar faces should have no degenerated shapes.
    assert_eq!(a_sewing.nb_degenerated_shapes(), 0);
    Ok(())
}

// TEST(BRepBuilderAPI_SewingTest, SetTolerance)
#[test]
fn set_tolerance() {
    let mut a_sewing = BRepBuilderAPISewing::default();
    let a_tol = 0.01;
    a_sewing.set_tolerance(a_tol);
    assert!((a_sewing.tolerance() - a_tol).abs() <= precision::confusion());
}

// TEST(BRepBuilderAPI_SewingTest, SewAllBoxFaces)
#[test]
fn sew_all_box_faces() -> Result<(), Failure> {
    let mut box_maker = BRepPrimAPIMakeBox::from_sizes(10.0, 10.0, 10.0)?;
    let a_box: Shape = box_maker.shape()?.clone();
    assert!(box_maker.is_done());
    let mut a_sewing = BRepBuilderAPISewing::default();
    let mut an_exp = Explorer::new(&a_box, ShapeEnum::Face, ShapeEnum::Shape)?;
    while an_exp.more() {
        a_sewing.add(an_exp.current())?;
        an_exp.next()?;
    }
    a_sewing.perform(&ProgressRange::default())?;
    assert!(!a_sewing.sewed_shape().is_null());
    // All 6 faces of a box form a closed shell, so there should be no free edges.
    assert_eq!(a_sewing.nb_free_edges(), 0);
    Ok(())
}
