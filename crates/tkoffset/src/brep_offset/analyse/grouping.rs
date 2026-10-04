use super::BRepOffsetAnalyse;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topexp::Explorer,
    topods::{self, Compound, Edge, Face, Shape},
    toptools::MapOfShape,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkfillet::chfi_ds::TypeOfConcavity;
use rustycad_tkg3d::top_abs::ShapeEnum;
impl BRepOffsetAnalyse {
    pub(super) fn bind_descendant(&mut self, parent: &Shape, child: &Shape) -> Result<(), Failure> {
        if !self.descendants.is_bound(parent) {
            self.descendants.bind(parent.clone(), Vec::new());
        }
        let children = self.descendants.change_find(parent)?;
        if !children.iter().any(|old| old.is_equal(child)) {
            children.push(child.clone());
        }
        Ok(())
    }
    /// Read descendants in native ancestor insertion order; `analysis.descendants(&face, false)`.
    pub fn descendants(&self, shape: &Shape, _update: bool) -> Option<&[Shape]> {
        // Eager rebuilding on every ancestor mutation makes the native lazy update flag redundant.
        self.descendants.find(shape).ok().map(Vec::as_slice)
    }
    /// Return the generated face or a null shape; `analysis.generated(&edge)`.
    pub fn generated(&self, shape: &Shape) -> Shape {
        self.generated.find(shape).cloned().unwrap_or_default()
    }
    /// Preserve an edge when no different-offset replacement exists; `analysis.edge_replacement(&face, &edge)`.
    pub fn edge_replacement<'a>(&'a self, face: &Face, edge: &'a Edge) -> &'a Edge {
        self.replacements
            .find(face)
            .ok()
            .and_then(|edges| edges.find(edge).ok())
            .unwrap_or(edge)
    }
    /// Group faces connected by a concavity type; `analysis.explode(kind, &mut groups)?`.
    pub fn explode(&self, kind: TypeOfConcavity, groups: &mut Vec<Shape>) -> Result<(), Failure> {
        self.explode_types([kind, kind], groups)
    }
    /// Group faces connected by either type; `analysis.explode_two_types(first, second, &mut groups)?`.
    pub fn explode_two_types(
        &self,
        first: TypeOfConcavity,
        second: TypeOfConcavity,
        groups: &mut Vec<Shape>,
    ) -> Result<(), Failure> {
        self.explode_types([first, second], groups)
    }
    /// Extend a compound through one concavity type; `analysis.add_faces(&face, &mut compound, &mut visited, kind)?`.
    pub fn add_faces(
        &self,
        face: &Face,
        compound: &mut Compound,
        visited: &mut MapOfShape,
        kind: TypeOfConcavity,
    ) -> Result<(), Failure> {
        self.add_faces_types(face, compound, visited, [kind, kind])
    }
    /// Extend a compound through either type; `analysis.add_faces_two_types(&face, &mut compound, &mut visited, first, second)?`.
    pub fn add_faces_two_types(
        &self,
        face: &Face,
        compound: &mut Compound,
        visited: &mut MapOfShape,
        first: TypeOfConcavity,
        second: TypeOfConcavity,
    ) -> Result<(), Failure> {
        self.add_faces_types(face, compound, visited, [first, second])
    }
    fn explode_types(
        &self,
        kinds: [TypeOfConcavity; 2],
        groups: &mut Vec<Shape>,
    ) -> Result<(), Failure> {
        groups.clear();
        let mut visited = MapOfShape::default();
        let builder = BRepBuilder::default();
        for shape in Explorer::new(&self.shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            if !visited.add(shape.clone()) {
                continue;
            }
            let face = topods::face(&shape)?;
            let mut compound = Compound::default();
            builder.make_compound(&mut compound);
            builder.add(&mut compound, &face)?;
            self.add_faces_types(&face, &mut compound, &mut visited, kinds)?;
            groups.push(compound.into_shape());
        }
        Ok(())
    }
    fn add_faces_types(
        &self,
        face: &Face,
        compound: &mut Compound,
        visited: &mut MapOfShape,
        kinds: [TypeOfConcavity; 2],
    ) -> Result<(), Failure> {
        let Some(edges) = self.descendants(face, false) else {
            return Ok(());
        };
        for edge in edges {
            let Some(other) = self.connected_face(face, &topods::edge(edge)?, kinds)? else {
                continue;
            };
            if !visited.add(other.clone().into_shape()) {
                continue;
            }
            BRepBuilder::default().add(compound, &other)?;
            self.add_faces_types(&other, compound, visited, kinds)?;
        }
        Ok(())
    }
    fn connected_face(
        &self,
        face: &Face,
        edge: &Edge,
        kinds: [TypeOfConcavity; 2],
    ) -> Result<Option<Face>, Failure> {
        let Some(interval) = self.type_of(edge)?.first() else {
            return Ok(None);
        };
        if !kinds.contains(&interval.get_type()?) {
            return Ok(None);
        }
        let ancestors = self.ancestors(edge)?;
        if ancestors.len() != 2 {
            return Ok(None);
        }
        let index = usize::from(ancestors[0].is_same(face));
        Ok(Some(topods::face(&ancestors[index])?))
    }
}
