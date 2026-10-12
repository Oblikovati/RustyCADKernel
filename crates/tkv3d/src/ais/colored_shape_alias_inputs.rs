//! Select real geometric topology in later actual buckets; never replace native hashing with fixture IDs.
use rustycad_tkbrep::topods::{Builder, Compound, Shape, Wire};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
pub(super) struct ColoredAliasInputs {
    pub wire: Shape,
    pub edge: Shape,
    pub base: Shape,
    rejected: Vec<Shape>,
}
impl ColoredAliasInputs {
    pub(super) fn new() -> Self {
        let mut input = Self {
            wire: Shape::default(),
            edge: Shape::default(),
            base: Shape::default(),
            rejected: Vec::new(),
        };
        input.select_wire();
        input.select_edge();
        Builder.add(&mut input.wire, &input.edge).unwrap();
        let mut compound = Compound::default();
        Builder.make_compound(&mut compound);
        Builder.add(&mut compound, &input.wire).unwrap();
        input.base = compound.into_shape();
        input
    }
    fn select_wire(&mut self) {
        for _ in 0..32 {
            let mut wire = Wire::default();
            Builder.make_wire(&mut wire);
            self.wire = wire.into_shape();
            if self.wire.hash_code() % 101 < 76 {
                return;
            }
            self.rejected.push(self.wire.clone());
        }
        panic!("wire allocation attempts=32; expected native bucket below 76");
    }
    fn select_edge(&mut self) {
        let wire_bucket = self.wire.hash_code() % 101;
        for attempt in 0..4096 {
            let start = Point3::new(f64::from(attempt), 0., 0.);
            let end = Point3::new(f64::from(attempt) + 1., 0., 0.);
            self.edge = BRepBuilderAPIMakeEdge::from_points(start, end)
                .unwrap()
                .edge()
                .unwrap()
                .into_shape();
            if self.edge.hash_code() % 101 > wire_bucket {
                return;
            }
            self.rejected.push(self.edge.clone());
        }
        panic!("edge allocation attempts=4096; expected a later native bucket");
    }
}
