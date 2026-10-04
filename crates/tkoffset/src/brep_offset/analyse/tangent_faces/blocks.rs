use super::*;
use rustycad_tkbo::{
    bop_tools::{self, algo_tools3d},
    int_tools::Context,
};
use rustycad_tkbrep::{
    brep::BRepBuilder, topexp, topods::Compound, toptools::IndexedDataMapOfShapeListOfShape,
};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Direction3, Vector3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakePrism;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
impl BRepOffsetAnalyse {
    pub(super) fn build_tangent_blocks(
        &mut self,
        selection: &TangentSelection,
    ) -> Result<(), Failure> {
        let compound = compound(&selection.edges)?;
        let mut blocks = Vec::new();
        let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
        bop_tools::make_connexity_blocks(
            &compound,
            ShapeEnum::Vertex,
            ShapeEnum::Edge,
            &mut blocks,
            &mut ancestors,
        )?;
        let context = Context::default();
        for block in blocks {
            self.build_coplanar_blocks(&block, selection, &context)?;
        }
        Ok(())
    }
    fn build_coplanar_blocks(
        &mut self,
        edges: &[Shape],
        selection: &TangentSelection,
        context: &Context,
    ) -> Result<(), Failure> {
        let mut fence = MapOfShape::default();
        for (index, edge) in edges.iter().enumerate() {
            if !fence.add(edge.clone()) {
                continue;
            }
            let normal = self.edge_normal(edge)?;
            let mut group = vec![edge.oriented(Orientation::Forward)];
            let mut offset = *selection.offsets.find(edge)?;
            self.append_coplanar(
                &edges[index + 1..],
                normal,
                &mut fence,
                &mut group,
                &mut offset,
                selection,
            )?;
            self.extrude_group(&group, normal, offset, selection, context)?;
        }
        Ok(())
    }
    fn edge_normal(&self, edge: &Shape) -> Result<Direction3, Failure> {
        let mut normal = Direction3::default();
        algo_tools3d::get_normal_to_face_on_edge(
            &topods::edge(edge)?,
            &topods::face(&self.ancestors(edge)?[0])?,
            &mut normal,
            None,
        )?;
        Ok(normal)
    }
    fn append_coplanar(
        &self,
        edges: &[Shape],
        normal: Direction3,
        fence: &mut MapOfShape,
        group: &mut Vec<Shape>,
        offset: &mut f64,
        selection: &TangentSelection,
    ) -> Result<(), Failure> {
        for edge in edges {
            if fence.contains(edge) {
                continue;
            }
            let other = self.edge_normal(edge)?;
            if normal.xyz().crossed(other.xyz()).modulus() < self.angle.sin().abs() {
                group.push(edge.oriented(Orientation::Forward));
                fence.add(edge.clone());
                let candidate = *selection.offsets.find(edge)?;
                if *offset < candidate {
                    *offset = candidate;
                }
            }
        }
        Ok(())
    }
    fn extrude_group(
        &mut self,
        edges: &[Shape],
        normal: Direction3,
        offset: f64,
        selection: &TangentSelection,
        context: &Context,
    ) -> Result<(), Failure> {
        let compound = compound(edges)?;
        let vector = Vector3::new(
            normal.x() * offset,
            normal.y() * offset,
            normal.z() * offset,
        );
        let mut prism = BRepPrimAPIMakePrism::new(&compound, vector, false, true)?;
        if !prism.is_done() {
            return Ok(());
        }
        let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
        for (child, parent) in [
            (ShapeEnum::Edge, ShapeEnum::Face),
            (ShapeEnum::Vertex, ShapeEnum::Edge),
        ] {
            topexp::map_shapes_and_ancestors(prism.shape()?, child, parent, &mut ancestors)?;
        }
        for edge in edges {
            self.apply_tangent_face(edge, &mut prism, &ancestors, selection, context)?;
        }
        Ok(())
    }
}
fn compound(edges: &[Shape]) -> Result<Compound, Failure> {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    for edge in edges {
        builder.add(&mut compound, edge)?;
    }
    Ok(compound)
}
