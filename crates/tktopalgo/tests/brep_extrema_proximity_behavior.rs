//! Behaviour tests of the BRepExtrema proximity group: triangle sets, the overlap tool,
//! self-intersection, the element filter, the proximity distance and value tools and
//! shape proximity in overlap mode, on faces meshed by the stand-in of
//! `brep_extrema_support`. The numeric parity with OCCT is checked by
//! tools/verify_tktopalgo_numeric.py.

mod brep_extrema_support;

use brep_extrema_support::{BREPMESH_LIKE, mesh_shape};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bvh::{BvhArray3d, BvhVec3d};
use rustycad_tkmath::gp::{Direction3, Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;
use rustycad_tktopalgo::brep_extrema::{
    BRepExtremaElementFilter, BRepExtremaFilterResult, BRepExtremaOverlapTool,
    BRepExtremaProxPntStatus, BRepExtremaProximityDistTool, BRepExtremaProximityInputs,
    BRepExtremaProximityValueTool, BRepExtremaSelfIntersection, BRepExtremaShapeProximity,
    BRepExtremaTriangleSet,
};

/// A meshed plane square of side `size` centred at `centre` with the normal `normal`.
fn meshed_square(centre: [f64; 3], normal: [f64; 3], size: f64) -> Shape {
    let direction = Direction3::new(normal[0], normal[1], normal[2]).unwrap();
    let plane = Plane3::new(Point3::new(centre[0], centre[1], centre[2]), direction).unwrap();
    let h: f64 = 0.5 * size;
    let face = BRepBuilderAPIMakeFace::from_plane_bounds(plane, (-h, h, -h, h))
        .unwrap()
        .face()
        .unwrap();
    let shape = face.into_shape();
    mesh_shape(&shape, 0.1, BREPMESH_LIKE).unwrap();
    shape
}

fn compound(shapes: &[Shape]) -> Shape {
    let builder = BRepBuilder::default();
    let mut result = Compound::default();
    builder.make_compound(&mut result);
    for shape in shapes {
        builder.add(&mut result, shape).unwrap();
    }
    result.into_shape()
}

#[test]
fn a_triangle_set_holds_the_face_triangles() -> Result<(), Failure> {
    let set = BRepExtremaTriangleSet::with_shapes(&[meshed_square(
        [1.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        2.0,
    )])?;
    assert_eq!(set.size(), 2);
    assert_eq!(set.vertices().len(), 4);
    assert_eq!(set.get_face_id(1)?, 0);
    assert_eq!(set.get_vtx_idx_in_shape(3)?, 3);
    let whole = set.bounding_box()?;
    assert_eq!(whole.corner_max().x, 2.0);
    assert!(set.bvh()?.length() >= 1);
    let [a, b, c] = set.get_vtx_indices(0)?;
    assert!(a != b && b != c);
    assert_eq!(
        set.get_face_id(2).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        set.get_shape_id_of_vtx(9).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    set.clear();
    assert_eq!(set.size(), 0);
    Ok(())
}

#[test]
fn init_stops_at_a_shape_without_mesh_and_keeps_the_vertex_data() -> Result<(), Failure> {
    let plain = BRepExtremaTriangleSet::new();
    let unmeshed =
        BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (0.0, 1.0, 0.0, 1.0))?
            .face()?
            .into_shape();
    let meshed = meshed_square([1.0, 1.0, 0.0], [0.0, 0.0, 1.0], 1.0);
    // Init answers true even when it stops at the face without a mesh (native quirk).
    assert!(plain.init(&[unmeshed, meshed.clone()])?);
    assert_eq!(plain.size(), 0);
    assert!(plain.init(&[meshed])?);
    assert_eq!(plain.size(), 2);
    Ok(())
}

#[test]
fn the_overlap_tool_finds_crossing_faces_only() -> Result<(), Failure> {
    let flat = Handle::new(BRepExtremaTriangleSet::with_shapes(&[meshed_square(
        [1.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        2.0,
    )])?);
    let crossing = Handle::new(BRepExtremaTriangleSet::with_shapes(&[meshed_square(
        [1.0, 1.0, 0.0],
        [1.0, 0.0, 0.0],
        3.0,
    )])?);
    let apart = Handle::new(BRepExtremaTriangleSet::with_shapes(&[meshed_square(
        [1.0, 1.0, 5.0],
        [0.0, 0.0, 1.0],
        2.0,
    )])?);
    let mut tool = BRepExtremaOverlapTool::with_sets(&flat, &crossing);
    tool.perform(0.0)?;
    assert!(tool.is_done());
    assert!(tool.overlap_sub_shapes1().find(&0)?.contains(0));
    let mut none = BRepExtremaOverlapTool::with_sets(&flat, &apart);
    none.perform(0.0)?;
    assert!(!none.is_done());
    // Within the tolerance the leaves are accepted (done), whether their prisms meet or not.
    none.perform(6.0)?;
    assert!(none.is_done());
    none.mark_dirty();
    assert!(!none.is_done());
    let mut unloaded = BRepExtremaOverlapTool::new();
    assert_eq!(
        unloaded.perform(0.0).unwrap_err().exception_type(),
        "Standard_NullObject"
    );
    Ok(())
}

/// A filter that marks every pair as overlapping.
struct EveryPairOverlaps;

impl BRepExtremaElementFilter for EveryPairOverlaps {
    fn pre_check_elements(
        &mut self,
        _index1: i32,
        _index2: i32,
    ) -> Result<BRepExtremaFilterResult, Failure> {
        Ok(BRepExtremaFilterResult::Overlap)
    }
}

#[test]
fn an_element_filter_decides_before_the_triangle_test() -> Result<(), Failure> {
    let flat = Handle::new(BRepExtremaTriangleSet::with_shapes(&[meshed_square(
        [1.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        2.0,
    )])?);
    let apart = Handle::new(BRepExtremaTriangleSet::with_shapes(&[meshed_square(
        [1.0, 1.0, 0.5],
        [0.0, 0.0, 1.0],
        2.0,
    )])?);
    let mut tool = BRepExtremaOverlapTool::with_sets(&flat, &apart);
    tool.set_element_filter(Some(Box::new(EveryPairOverlaps)));
    tool.perform(1.0)?;
    assert!(tool.overlap_sub_shapes2().find(&0)?.contains(0));
    Ok(())
}

#[test]
fn self_intersection_ignores_adjacent_triangles() -> Result<(), Failure> {
    let crossing = compound(&[
        meshed_square([1.0, 1.0, 0.0], [0.0, 0.0, 1.0], 2.0),
        meshed_square([1.0, 1.0, 0.0], [1.0, 0.0, 0.0], 3.0),
    ]);
    let mut tool = BRepExtremaSelfIntersection::with_shape(&crossing, 0.0)?;
    tool.perform()?;
    assert!(tool.is_done());
    assert!(tool.overlap_elements().find(&0)?.contains(1));
    assert_eq!(tool.element_set().size(), 4);
    assert!(tool.get_sub_shape(1).is_ok());
    assert_eq!(
        tool.get_sub_shape(2).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    let single = meshed_square([1.0, 1.0, 0.0], [0.0, 0.0, 1.0], 2.0);
    let mut alone = BRepExtremaSelfIntersection::new(0.0);
    assert!(alone.load_shape(&single)?);
    alone.perform()?;
    // Done means that pairs of leaves were tested; the two triangles of one face are
    // never reported.
    assert!(alone.is_done());
    assert_eq!(alone.overlap_elements().extent(), 0);
    assert_eq!(
        alone.pre_check_elements(0, 1)?,
        BRepExtremaFilterResult::NoCheck
    );
    Ok(())
}

#[test]
fn shared_vertex_and_edge_tests_follow_the_triangle_sides() -> Result<(), Failure> {
    let tool = BRepExtremaSelfIntersection::new(0.0);
    let v = |x: f64, y: f64, z: f64| BvhVec3d::new(x, y, z);
    let origin = v(0.0, 0.0, 0.0);
    let opposite = tool.is_regular_shared_vertex(
        &origin,
        [&v(1.0, 0.0, 0.0), &v(0.0, 1.0, 0.0)],
        [&v(-1.0, 0.0, 0.0), &v(0.0, -1.0, 0.0)],
    )?;
    assert_eq!(opposite, BRepExtremaFilterResult::NoCheck);
    let folded = tool.is_regular_shared_vertex(
        &origin,
        [&v(1.0, 0.0, 0.0), &v(0.0, 1.0, 0.0)],
        [&v(1.0, 1.0, 0.0), &v(2.0, 1.0, 0.0)],
    )?;
    assert_eq!(folded, BRepExtremaFilterResult::Overlap);
    let sides = tool.is_regular_shared_edge(
        &origin,
        &v(1.0, 0.0, 0.0),
        &v(0.5, 1.0, 0.0),
        &v(0.5, -1.0, 0.0),
    );
    assert_eq!(sides, BRepExtremaFilterResult::NoCheck);
    let same_side = tool.is_regular_shared_edge(
        &origin,
        &v(1.0, 0.0, 0.0),
        &v(0.5, 1.0, 0.0),
        &v(0.7, 2.0, 0.0),
    );
    assert_eq!(same_side, BRepExtremaFilterResult::Overlap);
    Ok(())
}

#[test]
fn proximity_tools_measure_parallel_squares() -> Result<(), Failure> {
    let low = [meshed_square([1.0, 1.0, 0.0], [0.0, 0.0, 1.0], 2.0)];
    let high = [meshed_square([1.0, 1.0, 1.5], [0.0, 0.0, 1.0], 2.0)];
    let s1 = Handle::new(BRepExtremaTriangleSet::with_shapes(&low)?);
    let s2 = Handle::new(BRepExtremaTriangleSet::with_shapes(&high)?);
    let inputs = BRepExtremaProximityInputs {
        set1: &s1,
        nb_samples1: 0,
        add_vertices1: &BvhArray3d::default(),
        add_status1: &[],
        set2: &s2,
        shape_list1: &low,
        shape_list2: &high,
    };
    let mut dist_tool = BRepExtremaProximityDistTool::with_inputs(&inputs);
    dist_tool.perform()?;
    assert!(dist_tool.is_done());
    assert_eq!(dist_tool.proximity_distance(), 1.5);
    assert_eq!(
        dist_tool.proximity_points_status(),
        (
            BRepExtremaProxPntStatus::Border,
            BRepExtremaProxPntStatus::Border
        )
    );
    let mut value_tool = BRepExtremaProximityValueTool::with_sets(&s1, &s2, &low, &high)?;
    let mut tolerance: f64 = -1.0;
    value_tool.perform(&mut tolerance)?;
    assert!(value_tool.is_done());
    assert_eq!((tolerance, value_tool.distance()), (1.5, 1.5));
    Ok(())
}

#[test]
fn shape_proximity_reports_overlaps_within_its_tolerance() -> Result<(), Failure> {
    let a = meshed_square([1.0, 1.0, 0.0], [0.0, 0.0, 1.0], 2.0);
    let b = meshed_square([1.0, 1.0, 0.05], [0.0, 0.0, 1.0], 2.0);
    let mut near = BRepExtremaShapeProximity::with_shapes(&a, &b, 0.1)?;
    near.perform()?;
    assert!(near.is_done());
    assert_eq!(near.overlap_sub_shapes1().extent(), 1);
    assert!(near.get_sub_shape1(0).is_ok());
    let mut infinite = BRepExtremaShapeProximity::new(precision::infinite());
    infinite.load_shape1(&a)?;
    infinite.load_shape2(&b)?;
    infinite.perform()?;
    assert!(infinite.is_done());
    assert!((infinite.proximity() - 0.05).abs() < 1e-12);
    Ok(())
}
