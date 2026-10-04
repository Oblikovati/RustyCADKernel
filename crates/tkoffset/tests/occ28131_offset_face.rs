//! Native differential for the offset-face stage used by OCC28131.
#[path = "../../tktopalgo/tests/support/occ28131.rs"]
mod fixture;

use rustycad_tkbrep::{
    brep::BRepTool,
    brep_tools::uv_bounds,
    topexp::Explorer,
    topods::{Edge, Face, Shape},
};
use rustycad_tkg3d::{geom::GeomOffsetSurface, top_abs::ShapeEnum};
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkoffset::brep_offset::{BRepOffsetOffsetFace, BRepOffsetStatus};
use rustycad_tktopalgo::{
    brep_builder_api::BRepBuilderAPIMakeShape, brep_check::BRepCheckAnalyzer, brep_gprop::BRepGProp,
};

#[test]
fn occ28131_non_singular_offset_face_matches_native_constructor() {
    let source = occ28131_face();
    let offset = BRepOffsetOffsetFace::from_face(&source, 10.0).unwrap();
    assert_eq!(offset.status(), BRepOffsetStatus::Good);
    assert_offset_face_shape(offset.face());
    assert_edge_history(&source, &offset);
}

fn occ28131_face() -> Face {
    let mut maker = fixture::occ28131_surface()
        .map(|surface| {
            rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace::from_surface(
                &surface.cast(),
                0.0,
            )
            .unwrap()
        })
        .unwrap();
    Face::unchecked(maker.shape().unwrap().clone())
}

fn assert_offset_face_shape(face: &Face) {
    let surface = BRepTool::surface(face).unwrap();
    assert!(surface.cast::<GeomOffsetSurface>().get().is_some());
    assert_eq!(count(&face.clone().into_shape(), ShapeEnum::Edge), 4);
    assert_eq!(count(&face.clone().into_shape(), ShapeEnum::Vertex), 8);
    assert_bounds(face);
    assert!(
        !BRepCheckAnalyzer::new(&face.clone().into_shape(), true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
    let mut properties = GProps::new();
    BRepGProp::surface_properties(&face.clone().into_shape(), &mut properties, false, false)
        .unwrap();
    assert_near(properties.mass(), 1_695.525_423_839_735_1, 1.0e-7);
}

fn assert_bounds(face: &Face) {
    let (umin, umax, vmin, vmax) = uv_bounds(face).unwrap();
    assert_near(umin, 0.0, 1.0e-12);
    assert_near(umax, 24.479_218_771_134_022, 1.0e-10);
    assert_near(vmin, 0.0, 1.0e-12);
    assert_near(vmax, 35.190_943_437_197_02, 1.0e-10);
    assert_near(BRepTool::face_tolerance(face).unwrap(), 1.0e-7, 1.0e-15);
}

fn assert_edge_history(source: &Face, offset: &BRepOffsetOffsetFace) {
    let mut edges = Explorer::new(
        &source.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )
    .unwrap();
    while edges.more() {
        let source_edge = Edge::unchecked(edges.current().clone());
        assert!(
            !offset
                .generated(&source_edge.clone().into_shape())
                .is_null()
        );
        edges.next().unwrap();
    }
}

fn count(shape: &Shape, kind: ShapeEnum) -> usize {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    let mut result = 0;
    while explorer.more() {
        result += 1;
        explorer.next().unwrap();
    }
    result
}

fn assert_near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected} +/- {tolerance}"
    );
}
