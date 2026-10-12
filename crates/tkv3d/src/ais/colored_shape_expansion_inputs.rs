//! Finite native planar topology recipes; styles are compared by actual identity, never geometry substitutes.
use rustycad_tkbrep::{
    topexp::Explorer,
    topods::{Builder, CompSolid, Compound, Shape, Shell, Solid, Wire},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Direction3, Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;
pub(super) struct ColoredExpansionInputs {
    pub faces: [Shape; 2],
    pub edges: [Shape; 2],
    pub vertex: Shape,
    pub base: Shape,
    pub own: Shape,
    pub external: Shape,
    pub wire: Shape,
    pub shell: Shape,
    pub solid: Shape,
    pub compsolid: Shape,
}
impl ColoredExpansionInputs {
    pub(super) fn new() -> Self {
        let (faces, edges, vertex) = expansion_geometry();
        let (wire, shell, solid, compsolid) = expansion_aggregates(&faces, &edges);
        let own = expansion_compound(&[&wire, &shell]);
        let base = expansion_compound(&[&own]);
        let external = expansion_compound(&[&wire, &faces[0]]);
        Self {
            faces,
            edges,
            vertex,
            base,
            own,
            external,
            wire,
            shell,
            solid,
            compsolid,
        }
    }
    pub(super) fn report_keys(&self) -> [&Shape; 8] {
        [
            &self.faces[0],
            &self.faces[1],
            &self.edges[0],
            &self.edges[1],
            &self.vertex,
            &self.own,
            &self.base,
            &self.external,
        ]
    }
}
fn bounded_expansion_face(plane: Plane3) -> Shape {
    BRepBuilderAPIMakeFace::from_plane_bounds(plane, (-2., 3., -1., 4.))
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}
fn first_expansion_leaf(shape: &Shape, kind: ShapeEnum) -> Shape {
    let explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    explorer.current().clone()
}
fn expansion_wire(edges: &[Shape; 2]) -> Shape {
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    for edge in edges {
        Builder.add(&mut wire, edge).unwrap();
    }
    wire.into_shape()
}
fn expansion_shell(faces: &[Shape; 2]) -> Shape {
    let mut shell = Shell::default();
    Builder.make_shell(&mut shell);
    for face in faces {
        Builder.add(&mut shell, face).unwrap();
    }
    shell.into_shape()
}
fn expansion_solid(shell: &Shape) -> Shape {
    let mut solid = Solid::default();
    Builder.make_solid(&mut solid);
    Builder.add(&mut solid, shell).unwrap();
    solid.into_shape()
}
fn expansion_compsolid(solid: &Shape) -> Shape {
    let mut compound = CompSolid::default();
    Builder.make_comp_solid(&mut compound);
    Builder.add(&mut compound, solid).unwrap();
    compound.into_shape()
}
fn expansion_compound(children: &[&Shape]) -> Shape {
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    for child in children {
        Builder.add(&mut compound, child).unwrap();
    }
    compound.into_shape()
}

fn expansion_geometry() -> ([Shape; 2], [Shape; 2], Shape) {
    let raised = Plane3::new(
        Point3::new(0., 0., 2.),
        Direction3::new(0., 0., 1.).unwrap(),
    )
    .unwrap();
    let faces = [
        bounded_expansion_face(Plane3::default()),
        bounded_expansion_face(raised),
    ];
    let edges = [
        first_expansion_leaf(&faces[0], ShapeEnum::Edge),
        first_expansion_leaf(&faces[1], ShapeEnum::Edge),
    ];
    let vertex = first_expansion_leaf(&edges[0], ShapeEnum::Vertex);
    (faces, edges, vertex)
}
fn expansion_aggregates(faces: &[Shape; 2], edges: &[Shape; 2]) -> (Shape, Shape, Shape, Shape) {
    let wire = expansion_wire(edges);
    let shell = expansion_shell(faces);
    let solid = expansion_solid(&shell);
    let compsolid = expansion_compsolid(&solid);
    (wire, shell, solid, compsolid)
}
