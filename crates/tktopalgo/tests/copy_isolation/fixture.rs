//! Complete representation hierarchy is constructed before any isolation snapshot.
use super::mesh::attach_meshes;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Builder, Compound, Edge, Face, Shape, Wire};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Direction2, Plane3, Point2, Point3};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPICopy, BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape,
};

pub(super) struct SharedCopyFixture {
    pub root: Compound,
    pub edge: Edge,
    pub faces: [Face; 2],
}

pub(super) fn shared_copy_fixture() -> SharedCopyFixture {
    let edge = BRepBuilderAPIMakeEdge::from_points(Point3::default(), Point3::new(2.0, 0.0, 0.0))
        .unwrap()
        .edge()
        .unwrap();
    let faces = [analytic_face(&edge), analytic_face(&edge)];
    attach_meshes(&edge, &faces[0]);
    let mut root = Compound::default();
    Builder.make_compound(&mut root);
    for face in &faces {
        Builder.add(&mut root, face).unwrap();
    }
    Builder.add(&mut root, &edge.reversed()).unwrap();
    SharedCopyFixture { root, edge, faces }
}

fn analytic_face(edge: &Edge) -> Face {
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomPlane::from_pln(Plane3::default())).cast();
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 0.01)
        .unwrap();
    attach_pcurve(edge, &face);
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    Builder.add(&mut wire, edge).unwrap();
    Builder.add(&mut face, &wire).unwrap();
    face
}

pub(super) fn copied_member(copier: &mut BRepBuilderAPICopy, original: &Shape) -> Shape {
    let images = copier.modified(original).unwrap();
    assert_eq!(images.len(), 1);
    images.first().unwrap().clone()
}

pub(super) fn copy_fixture(source: &SharedCopyFixture) -> (BRepBuilderAPICopy, SharedCopyFixture) {
    let mut copier = BRepBuilderAPICopy::with_shape(&source.root, true, true).unwrap();
    let edge = rustycad_tkbrep::topods::edge(&copied_member(&mut copier, &source.edge)).unwrap();
    let faces = source
        .faces
        .each_ref()
        .map(|face| rustycad_tkbrep::topods::face(&copied_member(&mut copier, face)).unwrap());
    let root = rustycad_tkbrep::topods::compound(copier.shape().unwrap()).unwrap();
    assert_eq!(
        BRepTool::edge_tolerance(&edge).unwrap(),
        BRepTool::edge_tolerance(&source.edge).unwrap()
    );
    (copier, SharedCopyFixture { root, edge, faces })
}

fn attach_pcurve(edge: &Edge, face: &Face) {
    let line = Geom2dLine::from_point(Point2::default(), Direction2::new(1.0, 0.0).unwrap());
    let pcurve: Handle<dyn Geom2dCurve> = Handle::new(line).cast();
    BRepBuilder::default()
        .update_edge_pcurve_on_face(edge, &pcurve, face, 0.02)
        .unwrap();
}
