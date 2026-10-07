//! The anonymous-namespace helpers of BOPAlgo_MakePeriodic_Test.cxx.
#![allow(dead_code)]

use rustycad_tkbo::bop_algo::{
    BOPAlgoMakeConnected, BOPAlgoMakePeriodic, BOPAlgoMakerVolume, PeriodicityParams,
};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::History;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Edge, Face, Shape};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBSplineSurface, GeomCurve, GeomRectangularTrimmedSurface, GeomSurface,
    GeomTrimmedCurve,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakePrism;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeShape,
};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_gprop::BRepGProp;

fn count_in_map(map: &IndexedMapOfShape, kind: ShapeEnum) -> i32 {
    (1..=map.extent())
        .filter(|&index| map.find_key(index).unwrap().shape_type().unwrap() == kind)
        .count() as i32
}

pub fn CountShapes(shape: &Shape, kind: ShapeEnum) -> i32 {
    let mut map = IndexedMapOfShape::default();
    topexp::map_shapes(shape, &mut map, true, true);
    count_in_map(&map, kind)
}

pub fn CountShapesList(shapes: &[Shape], kind: ShapeEnum) -> i32 {
    let mut map = IndexedMapOfShape::default();
    for shape in shapes {
        topexp::map_shapes(shape, &mut map, true, true);
    }
    count_in_map(&map, kind)
}

pub fn MakeCompound(shapes: &[Shape]) -> Shape {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    for shape in shapes {
        builder.add(&mut compound, shape).unwrap();
    }
    compound.into_shape()
}

fn IsValid(shape: &Shape) -> bool {
    BRepCheckAnalyzer::new(shape, true, false, false)
        .unwrap()
        .is_valid()
        .unwrap()
}

pub fn ExpectShapeCounts(shape: &Shape, counts: [i32; 6]) {
    assert!(!shape.is_null());
    assert!(IsValid(shape));
    let kinds = [
        ShapeEnum::Vertex,
        ShapeEnum::Edge,
        ShapeEnum::Wire,
        ShapeEnum::Face,
        ShapeEnum::Shell,
        ShapeEnum::Solid,
    ];
    for (kind, count) in kinds.into_iter().zip(counts) {
        assert_eq!(CountShapes(shape, kind), count, "{kind:?}");
    }
}

pub fn ExpectWireFaceShellSolidCounts(
    shape: &Shape,
    wires: i32,
    faces: i32,
    shells: i32,
    solids: i32,
) {
    assert!(!shape.is_null());
    assert!(IsValid(shape));
    assert_eq!(CountShapes(shape, ShapeEnum::Wire), wires);
    assert_eq!(CountShapes(shape, ShapeEnum::Face), faces);
    assert_eq!(CountShapes(shape, ShapeEnum::Shell), shells);
    assert_eq!(CountShapes(shape, ShapeEnum::Solid), solids);
}

pub fn ExpectValidShape(shape: &Shape) {
    assert!(!shape.is_null());
    assert!(IsValid(shape));
}

pub fn ExpectMaterials(actual: &[Shape], expected1: &Shape, expected2: &Shape) {
    let expected_extent = if expected2.is_null() { 1 } else { 2 };
    assert_eq!(actual.len(), expected_extent);
    for material in actual {
        assert!(
            material.is_same(expected1) || (!expected2.is_null() && material.is_same(expected2))
        );
    }
}

fn near(first: f64, second: f64) {
    assert!(
        (first - second).abs() <= 1.0e-6 * f64::max(1.0, second.abs()),
        "{first} vs {second}"
    );
}

pub fn ExpectEqualProperties(first: &Shape, second: &Shape, skip_shared: bool) {
    let (mut first_props, mut second_props) = (GProps::default(), GProps::default());
    BRepGProp::linear_properties(first, &mut first_props, skip_shared, false).unwrap();
    BRepGProp::linear_properties(second, &mut second_props, skip_shared, false).unwrap();
    near(first_props.mass(), second_props.mass());
    BRepGProp::surface_properties_eps(first, &mut first_props, 1.0, skip_shared).unwrap();
    BRepGProp::surface_properties_eps(second, &mut second_props, 1.0, skip_shared).unwrap();
    near(first_props.mass(), second_props.mass());
    BRepGProp::volume_properties_eps(first, &mut first_props, 1.0, false, skip_shared).unwrap();
    BRepGProp::volume_properties_eps(second, &mut second_props, 1.0, false, skip_shared).unwrap();
    near(first_props.mass(), second_props.mass());
}

/// The materials of each modified face match the expected ones on each side.
pub fn ExpectModifiedMaterials(
    connected: &BOPAlgoMakeConnected,
    modified: &[Shape],
    positive: (&Shape, &Shape),
    negative: (&Shape, &Shape),
) {
    let modified_faces = GetFaces(&MakeCompound(modified));
    assert!(!modified_faces.is_empty());
    let (mut has_positive, mut has_negative) = (false, false);
    for face in &modified_faces {
        let on_positive: Vec<Shape> = connected
            .materials_on_positive_side(face)
            .iter()
            .cloned()
            .collect();
        let on_negative: Vec<Shape> = connected
            .materials_on_negative_side(face)
            .iter()
            .cloned()
            .collect();
        if !on_positive.is_empty() {
            has_positive = true;
            ExpectMaterials(&on_positive, positive.0, positive.1);
        }
        if !on_negative.is_empty() {
            has_negative = true;
            ExpectMaterials(&on_negative, negative.0, negative.1);
        }
    }
    assert!(has_positive);
    assert_eq!(has_negative, !negative.0.is_null());
}

pub fn MakePeriodicityProfile() -> Shape {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    for (x, z) in [
        (0.0, 0.0),
        (10.0, 0.0),
        (10.0, 10.0),
        (5.0, 10.0),
        (5.0, 5.0),
        (0.0, 5.0),
    ] {
        polygon.add_point(Point3::new(x, 0.0, z)).unwrap();
    }
    polygon.close().unwrap();
    let face = BRepBuilderAPIMakeFace::from_wire(&polygon.wire().unwrap(), false)
        .unwrap()
        .face()
        .unwrap();
    let mut prism =
        BRepPrimAPIMakePrism::new(&face.into_shape(), Vector3::new(0.0, 5.0, 0.0), false, true)
            .unwrap();
    prism.shape().unwrap().clone()
}

/// The distinct sub-shapes of a type, the shape itself excluded.
fn distinct(shape: &Shape, kind: ShapeEnum) -> Vec<Shape> {
    let mut map = MapOfShape::default();
    map.add(shape.clone());
    let mut found = Vec::new();
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    while explorer.more() {
        if map.add(explorer.current().clone()) {
            found.push(explorer.current().clone());
        }
        explorer.next().unwrap();
    }
    found
}

pub fn GetFaces(shape: &Shape) -> Vec<Face> {
    distinct(shape, ShapeEnum::Face)
        .iter()
        .map(|face| topods::face(face).unwrap())
        .collect()
}

pub fn GetEdges(shape: &Shape) -> Vec<Edge> {
    distinct(shape, ShapeEnum::Edge)
        .iter()
        .map(|edge| topods::edge(edge).unwrap())
        .collect()
}

/// The face at `index` in exploration order, null past the end.
pub fn GetFace(shape: &Shape, index: usize) -> Face {
    let mut explorer = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    for _ in 0..index {
        if !explorer.more() {
            break;
        }
        explorer.next().unwrap();
    }
    if !explorer.more() {
        return Face::default();
    }
    topods::face(explorer.current()).unwrap()
}

fn BasisCurve(edge: &Edge) -> Handle<dyn GeomCurve> {
    let mut curve = BRepTool::curve(edge).unwrap().0;
    loop {
        let trimmed: Handle<GeomTrimmedCurve> = curve.cast();
        let Some(trimmed) = trimmed.get() else {
            return curve;
        };
        curve = trimmed.basis_curve();
    }
}

pub fn CurveTypeName(edge: &Edge) -> Option<&'static str> {
    BasisCurve(edge).dynamic_type().map(|kind| kind.name())
}

pub fn IsBSplineCurve(edge: &Edge) -> bool {
    !BasisCurve(edge).cast::<GeomBSplineCurve>().is_null()
}

fn BasisSurface(face: &Face) -> Handle<dyn GeomSurface> {
    let mut surface = BRepTool::surface(face).unwrap();
    loop {
        let trimmed: Handle<GeomRectangularTrimmedSurface> = surface.cast();
        let Some(trimmed) = trimmed.get() else {
            return surface;
        };
        surface = trimmed.basis_surface();
    }
}

pub fn SurfaceTypeName(face: &Face) -> Option<&'static str> {
    BasisSurface(face).dynamic_type().map(|kind| kind.name())
}

pub fn IsBSplineSurface(face: &Face) -> bool {
    !BasisSurface(face).cast::<GeomBSplineSurface>().is_null()
}

pub fn MakeVolume(
    maker: &mut BOPAlgoMakerVolume,
    modified_shape: &Shape,
    faces: &[Face],
) -> History {
    let mut arguments = vec![modified_shape.clone()];
    arguments.extend(faces.iter().map(|face| (**face).clone()));
    maker.set_arguments(&arguments.iter().cloned().collect::<List<Shape>>());
    maker.set_to_fill_history(true);
    maker.options_mut().set_fuzzy_value(precision::confusion());
    maker.set_non_destructive(false);
    maker.set_intersect(true);
    maker.options_mut().set_run_parallel(false);
    maker.perform();
    History::from_algorithm(&arguments, &mut ***maker).unwrap()
}

/// `MakeBoxPeriodic`: the periods, then the trimming of each direction when `set_trimmed`.
pub fn MakeBoxPeriodic(
    maker: &mut BOPAlgoMakePeriodic,
    shape: &Shape,
    periods: [f64; 3],
    is_input_trimmed: [bool; 3],
    firsts: [f64; 3],
    set_trimmed: bool,
) {
    maker.set_shape(shape);
    maker.make_x_periodic(true, periods[0]);
    maker.make_y_periodic(true, periods[1]);
    maker.make_z_periodic(true, periods[2]);
    if set_trimmed {
        for direction in 0..3 {
            maker.set_trimmed(
                direction as i32,
                is_input_trimmed[direction],
                firsts[direction],
            );
        }
    }
    maker.options_mut().set_run_parallel(false);
    maker.perform().unwrap();
}

pub fn MakeXYZPeriodicity(x_period: f64, y_period: f64, z_period: f64) -> PeriodicityParams {
    let mut params = PeriodicityParams::new();
    params.periodic = [true; 3];
    params.period = [x_period, y_period, z_period];
    params
}

pub fn PerformConnected(connected: &mut BOPAlgoMakeConnected, arguments: &[Shape]) {
    connected.set_arguments(&arguments.iter().cloned().collect::<List<Shape>>());
    connected.options_mut().set_run_parallel(false);
    connected.perform().unwrap();
}
