//! FilletSurf_Builder: the fillet surface of a box edge is a quarter cylinder bounded by
//! the edge length; the refused contours report their reason; the accessors check their
//! indices. The bits of the results are compared with the pinned kernel by the
//! fl_fillet_surf probe (tools/oracle/tkfillet_probe_api.cxx).
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkfillet::fillet_surf::{ErrorTypeStatus, FilletSurfBuilder, StatusDone, StatusType};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

/// The edges of the shape in map order.
fn edges(s: &Shape) -> Vec<Shape> {
    let mut m = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(s, ShapeEnum::Edge, &mut m).unwrap();
    (1..=m.extent()).map(|i| m.find_key(i).unwrap().clone()).collect()
}

fn cube(side: f64) -> Shape {
    BRepPrimAPIMakeBox::from_sizes(side, side, side).unwrap().shape().unwrap().clone()
}

#[test]
fn the_fillet_surface_of_a_box_edge_is_a_quarter_cylinder() {
    let solid = cube(20.0);
    let e = edges(&solid);
    let mut b = FilletSurfBuilder::new(&solid, &e[..1], 2.0, 1e-2, 1e-4, 1e-5).unwrap();
    b.perform().unwrap();
    assert_eq!(b.is_done(), StatusDone::IsOk);
    assert_eq!(b.nb_surface().unwrap(), 1);
    assert!(b.tol_app3d(1).unwrap() < 1e-7);
    assert_eq!(b.start_section_status().unwrap(), StatusType::TwoExtremityOnEdge);
    assert_eq!(b.end_section_status().unwrap(), StatusType::TwoExtremityOnEdge);
    // The curves on the faces run along the whole edge, 2 away from it.
    let edge = topods::edge(&e[0]).unwrap();
    let (ec, ef, el) = rustycad_tkbrep::brep::BRepTool::curve(&edge).unwrap();
    let ec = ec.get().unwrap();
    let mid = ec.value(0.5 * (ef + el)).unwrap();
    for c in [b.curve_on_face1(1).unwrap(), b.curve_on_face2(1).unwrap()] {
        let c = c.get().unwrap();
        assert!((c.last_parameter() - c.first_parameter() - 20.0).abs() < 1e-9);
        let p: Point3 = c.value(0.5 * (c.first_parameter() + c.last_parameter())).unwrap();
        assert!((p.distance(&mid) - 2.0).abs() < 1e-6, "{}", p.distance(&mid));
    }
    assert_eq!(b.first_parameter().unwrap(), 0.0);
}

#[test]
fn simulating_a_box_edge_gives_the_sections_of_the_native_walking() {
    let solid = cube(20.0);
    let e = edges(&solid);
    let mut b = FilletSurfBuilder::new(&solid, &e[..1], 2.0, 1e-2, 1e-4, 1e-5).unwrap();
    b.simulate().unwrap();
    assert_eq!(b.is_done(), StatusDone::IsOk);
    // 30 sections as the pinned kernel (fl_fillet_surf 0 1 0 0 2 1), each a quarter
    // circle.
    assert_eq!(b.nb_section(1).unwrap(), 30);
    let s = b.section(1, 1).unwrap();
    let s = s.get().unwrap();
    assert!((s.last_parameter() - s.first_parameter() - std::f64::consts::FRAC_PI_2).abs() < 1e-9);
    assert_eq!(b.section(1, 31).unwrap_err().exception_type(), "Standard_OutOfRange");
}

#[test]
fn refused_contours_report_their_reason() {
    let solid = cube(20.0);
    let e = edges(&solid);
    let empty = FilletSurfBuilder::new(&solid, &[], 1.0, 1e-2, 1e-4, 1e-5).unwrap();
    assert_eq!((empty.is_done(), empty.status_error()), (StatusDone::IsNotOk, ErrorTypeStatus::EmptyList));
    assert_eq!(empty.nb_surface().unwrap_err().exception_type(), "StdFail_NotDone");
    assert_eq!(empty.first_parameter().unwrap_err().exception_type(), "StdFail_NotDone");
    // Two edges that do not make one G1 contour.
    let pair = FilletSurfBuilder::new(&solid, &e[..2], 1.0, 1e-2, 1e-4, 1e-5).unwrap();
    assert_eq!((pair.is_done(), pair.status_error()), (StatusDone::IsNotOk, ErrorTypeStatus::EdgeNotG1));
    // An edge of another shape.
    let other = edges(&cube(5.0));
    let off = FilletSurfBuilder::new(&solid, &other[..1], 1.0, 1e-2, 1e-4, 1e-5).unwrap();
    assert_eq!(off.status_error(), ErrorTypeStatus::EdgeNotOnShape);
    // The seam of a cylinder is not sharp.
    let cyl = BRepPrimAPIMakeCylinder::new(5.0, 10.0).shape().unwrap().clone();
    let seam = edges(&cyl)[1].clone();
    let smooth = FilletSurfBuilder::new(&cyl, &[seam], 1.0, 1e-2, 1e-4, 1e-5).unwrap();
    assert_eq!(smooth.status_error(), ErrorTypeStatus::NotSharpEdge);
}

#[test]
fn accessors_refuse_surfaces_out_of_range() {
    let solid = cube(20.0);
    let e = edges(&solid);
    let mut b = FilletSurfBuilder::new(&solid, &e[..1], 2.0, 1e-2, 1e-4, 1e-5).unwrap();
    b.perform().unwrap();
    assert_eq!(b.surface_fillet(0).unwrap_err().exception_type(), "Standard_OutOfRange");
    assert_eq!(b.support_face1(2).unwrap_err().exception_type(), "Standard_OutOfRange");
    assert!(!b.support_face2(1).unwrap().is_null());
    assert!(!b.pcurve_on_face1(1).unwrap().is_null());
    assert!(!b.pcurve1_on_fillet(1).unwrap().is_null());
    assert!(b.last_parameter().unwrap().is_finite());
}
