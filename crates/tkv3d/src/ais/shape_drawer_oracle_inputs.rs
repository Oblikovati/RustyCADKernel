//! Real BRep keys preserve independent original/reversed/located topology ownership.
use super::AisColoredDrawer;
use crate::prs3d::Prs3dDrawer;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Shape, Vertex},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::{
    gp::{Point3, Transform3, Vector3},
    toploc::Location,
};
pub(super) struct ShapeDrawerOracleInputs {
    pub key: Shape,
    pub reversed: Shape,
    pub located: Shape,
    pub parent: Handle<Prs3dDrawer>,
    pub first: Handle<AisColoredDrawer>,
    pub second: Handle<AisColoredDrawer>,
}
impl ShapeDrawerOracleInputs {
    pub fn new() -> Self {
        let key = shape_drawer_vertex(Point3::new(1., 2., 3.));
        let reversed = key.reversed();
        let located = translated_drawer_key(&key);
        let parent = Handle::new(Prs3dDrawer::new());
        let first = Handle::new(AisColoredDrawer::new(&parent));
        let second = Handle::new(AisColoredDrawer::new(&parent));
        Self {
            key,
            reversed,
            located,
            parent,
            first,
            second,
        }
    }
}
pub(super) fn shape_drawer_vertex(point: Point3) -> Shape {
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, point, 1e-7)
        .unwrap();
    vertex.into_shape()
}
fn translated_drawer_key(key: &Shape) -> Shape {
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(7., -3., 2.));
    key.located(&Location::new(transform), false).unwrap()
}
