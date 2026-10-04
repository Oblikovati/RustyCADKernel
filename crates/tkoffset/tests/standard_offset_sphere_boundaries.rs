//! Exact curve-type checks for native spherical box-corner faces.
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{Edge, Face},
};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg3d::{
    geom::{GeomCircle, GeomSphericalSurface},
    top_abs::ShapeEnum,
};
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkoffset::brep_offset::{BRepOffsetMakeOffset, BRepOffsetMode};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

#[test]
fn box_vertex_sphere_edges_keep_native_exact_iso_images() {
    let mut source_maker = BRepPrimAPIMakeBox::from_sizes(1.0, 2.0, 3.0).unwrap();
    let source = source_maker.shape().unwrap().clone();
    let mut maker = BRepOffsetMakeOffset::new();
    maker.initialize(
        &source,
        0.5,
        1.0e-7,
        BRepOffsetMode::Skin,
        false,
        false,
        JoinType::Arc,
        false,
        false,
    );
    maker.make_offset_shape().unwrap();

    let spheres = generated_sphere_faces(&source, &mut maker);
    assert_eq!(spheres.len(), 8);
    for face in &spheres {
        assert_spherical_boundary_types(face);
    }
    assert_shared_sphere_edges(maker.shape(), &spheres);
}

fn assert_shared_sphere_edges(result: &rustycad_tkbrep::topods::Shape, spheres: &[Face]) {
    let faces = Explorer::new(result, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .map(Face::unchecked)
        .collect::<Vec<_>>();
    for sphere in spheres {
        assert_sphere_edge_occurrences(sphere, &faces);
    }
}

fn assert_sphere_edge_occurrences(sphere: &Face, faces: &[Face]) {
    for edge in Explorer::new(
        &sphere.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )
    .unwrap()
    {
        assert_opposite_neighbor(&Edge::unchecked(edge), sphere, faces);
    }
}

fn assert_opposite_neighbor(edge: &Edge, sphere: &Face, faces: &[Face]) {
    let neighbors = faces
        .iter()
        .filter_map(|face| matching_edge(edge, face).map(|match_edge| (face, match_edge)))
        .collect::<Vec<_>>();
    assert_eq!(neighbors.len(), 2);
    let adjacent = neighbors
        .iter()
        .find(|(face, _)| !face.is_same(&sphere.clone().into_shape()));
    let (_, neighbor_edge) = adjacent.expect("sphere edge also borders a non-spherical face");
    assert!(neighbor_edge.is_equal(&edge.reversed()));
}

fn matching_edge(edge: &Edge, face: &Face) -> Option<Edge> {
    Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )
    .ok()?
    .map(Edge::unchecked)
    .find(|candidate| candidate.is_same(&edge.clone().into_shape()))
}

fn generated_sphere_faces(
    source: &rustycad_tkbrep::topods::Shape,
    maker: &mut BRepOffsetMakeOffset,
) -> Vec<Face> {
    let mut result = Vec::new();
    for vertex in Explorer::new(source, ShapeEnum::Vertex, ShapeEnum::Shape).unwrap() {
        for generated in maker.generated(&vertex).unwrap() {
            let face = Face::unchecked(generated);
            if BRepTool::surface(&face)
                .unwrap()
                .cast::<GeomSphericalSurface>()
                .is_null()
            {
                continue;
            }
            if !result
                .iter()
                .any(|seen: &Face| seen.is_same(&face.clone().into_shape()))
            {
                result.push(face);
            }
        }
    }
    result
}

fn assert_spherical_boundary_types(face: &Face) {
    let (mut lines, mut splines) = (0, 0);
    for edge in Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )
    .unwrap()
    {
        let edge = Edge::unchecked(edge);
        let curve = BRepTool::curve_with_location(&edge).unwrap();
        let pcurve = BRepTool::curve_on_face(&edge, face).unwrap().pcurve;
        let circle = curve.curve.cast::<GeomCircle>();
        let circle = circle.get().expect("native sphere boundary is a circle");
        assert!((circle.radius() - 0.5).abs() < 1.0e-12);
        if assert_pcurve_type(&pcurve_basis(&pcurve)) {
            splines += 1;
        } else {
            lines += 1;
        }
    }
    assert_eq!((lines, splines), (1, 2));
}

fn pcurve_basis(
    pcurve: &rustycad_tkernel::standard::Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
) -> rustycad_tkernel::standard::Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve> {
    pcurve
        .cast::<Geom2dTrimmedCurve>()
        .get()
        .map(|curve| curve.basis_curve())
        .unwrap_or_else(|| pcurve.clone())
}

fn assert_pcurve_type(
    pcurve: &rustycad_tkernel::standard::Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
) -> bool {
    if pcurve.cast::<Geom2dLine>().is_some() {
        return false;
    }
    let spline = pcurve.cast::<Geom2dBSplineCurve>();
    let spline = spline.get().expect("native p-curve is a B-spline or line");
    assert_eq!(spline.degree(), 7);
    assert_eq!(spline.nb_knots(), 4);
    true
}
