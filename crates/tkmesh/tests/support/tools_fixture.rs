//! Real topology fixture and recording strategies shared by tests and native probe.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Compound, Edge, Face, Shape, Wire},
};
use rustycad_tkernel::{
    impl_transient,
    message::ProgressRange,
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use rustycad_tkg3d::{
    geom::{GeomCurve, GeomCylindricalSurface, GeomLine, GeomPlane, GeomSurface},
    top_abs::Orientation,
};
use rustycad_tkmath::{
    gp::{Axis1, Axis3, Transform3, Vector3},
    toploc::Location,
};
use rustycad_tkmesh::{
    imesh_data::MeshModel,
    imesh_tools::{MeshParameters, ModelAlgo, ShapeVisitor},
};
/// Records all visits as actual topology objects.
#[derive(Default)]
pub struct RecordingShapeVisitor {
    transient: StandardTransient,
    pub shapes: SharedState<Vec<Shape>>,
}
impl_transient!(RecordingShapeVisitor, "OracleShapeVisitor", dyn ShapeVisitor, transient, interfaces: [dyn ShapeVisitor]);
impl ShapeVisitor for RecordingShapeVisitor {
    fn visit_face(&self, face: &Face) -> Result<(), Failure> {
        self.shapes.write().push(face.clone().into_shape());
        Ok(())
    }
    fn visit_edge(&self, edge: &Edge) -> Result<(), Failure> {
        self.shapes.write().push(edge.clone().into_shape());
        Ok(())
    }
}
/// ModelAlgo exception-boundary test implementation, with no simulated meshing.
pub struct OracleModelAlgorithm {
    transient: StandardTransient,
    pub outcome: i32,
}
impl_transient!(OracleModelAlgorithm, "OracleModelAlgorithm", dyn ModelAlgo, transient, interfaces: [dyn ModelAlgo]);
impl OracleModelAlgorithm {
    pub fn new(outcome: i32) -> Self {
        Self {
            transient: StandardTransient::default(),
            outcome,
        }
    }
}
impl ModelAlgo for OracleModelAlgorithm {
    fn perform_internal(
        &self,
        _: &Handle<dyn MeshModel>,
        _: &MeshParameters,
        _: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        if self.outcome < 0 {
            return Err(Failure::new("deliberate oracle exception"));
        }
        Ok(self.outcome != 0)
    }
}
fn explorer_face(builder: &BRepBuilder, edge: &Edge, cylinder: bool) -> Face {
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, edge).unwrap();
    let surface: Handle<dyn GeomSurface> = if cylinder {
        Handle::new(GeomCylindricalSurface::new(Axis3::default(), 2.0).unwrap()).cast()
    } else {
        Handle::new(GeomPlane::new(Axis3::default())).cast()
    };
    let mut face = Face::default();
    builder
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    builder.add(&mut face, &wire).unwrap();
    face
}
fn explorer_edges(builder: &BRepBuilder) -> (Edge, Edge) {
    let mut edge = Edge::default();
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::default())).cast();
    builder
        .make_edge_with_curve(&mut edge, &curve, 1e-7)
        .unwrap();
    let mut bare = Edge::default();
    builder.make_edge(&mut bare).unwrap();
    (edge, bare)
}
/// The native oracle's compound; `explorer_fixture(Orientation::Forward, true)`.
pub fn explorer_fixture(orientation: Orientation, duplicate: bool) -> (Shape, Vec<Shape>) {
    let builder = BRepBuilder::default();
    let (edge, bare) = explorer_edges(&builder);
    let plane = explorer_face(&builder, &edge, false);
    let cylinder = explorer_face(&builder, &edge, true);
    let mut shape = Compound::default();
    builder.make_compound(&mut shape);
    let placement = Location::new(Transform3::translation_vector(Vector3::new(
        7.0, 11.0, 13.0,
    )));
    builder
        .add(
            &mut shape,
            &edge
                .oriented(orientation)
                .located(&placement, false)
                .unwrap(),
        )
        .unwrap();
    builder.add(&mut shape, &bare).unwrap();
    builder
        .add(
            &mut shape,
            &plane
                .oriented(orientation)
                .located(&placement, false)
                .unwrap(),
        )
        .unwrap();
    builder.add(&mut shape, &cylinder).unwrap();
    append_duplicate(&builder, &mut shape, &plane, duplicate);
    (
        shape.into_shape(),
        vec![
            edge.into_shape(),
            bare.into_shape(),
            plane.into_shape(),
            cylinder.into_shape(),
        ],
    )
}
fn append_duplicate(builder: &BRepBuilder, shape: &mut Compound, plane: &Face, duplicate: bool) {
    if !duplicate {
        return;
    }
    let placement = Location::new(Transform3::translation_vector(Vector3::new(
        17.0, 23.0, 29.0,
    )));
    builder
        .add(shape, &plane.located(&placement, false).unwrap())
        .unwrap();
}
