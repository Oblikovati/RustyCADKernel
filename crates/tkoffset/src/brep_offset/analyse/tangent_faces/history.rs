use super::*;
use rustycad_tkbo::{
    bop_tools::{self, algo_tools3d},
    int_tools::Context,
};
use rustycad_tkbrep::{brep::BRepTool, toptools::IndexedDataMapOfShapeListOfShape};
use rustycad_tkmath::gp::{Direction3, Point2, Point3, Vector3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakePrism;
impl BRepOffsetAnalyse {
    pub(super) fn apply_tangent_face(
        &mut self,
        edge: &Shape,
        prism: &mut BRepPrimAPIMakePrism,
        ancestors: &IndexedDataMapOfShapeListOfShape,
        selection: &TangentSelection,
        context: &Context,
    ) -> Result<(), Failure> {
        let mut face = topods::face(&first_generated(prism, edge)?)?;
        let (removed, opposite) = self.ordered_offset_parents(edge)?;
        orient_artificial_face(edge, &opposite, &mut face)?;
        self.replace_parent(edge, &removed, &face)?;
        self.reclassify(&topods::edge(edge)?)?;
        let mut used = MapOfShape::default();
        used.add(edge.clone());
        for vertex in children(edge) {
            self.apply_side_edge(&vertex, &face, prism, ancestors, selection, &mut used)?;
        }
        self.apply_opposite_edge(edge, &face, &removed, &used, ancestors, context)?;
        self.new_faces.push(face.clone().into());
        self.generated.bind(edge.clone(), face.into());
        Ok(())
    }
    fn ordered_offset_parents(&self, edge: &Shape) -> Result<(Shape, Shape), Failure> {
        let parents = self.ancestors(edge)?;
        let (first, last) = (&parents[0], parents.last().unwrap());
        Ok(if self.face_offset(first) > self.face_offset(last) {
            (first.clone(), last.clone())
        } else {
            (last.clone(), first.clone())
        })
    }
    fn replace_parent(
        &mut self,
        edge: &Shape,
        removed: &Shape,
        face: &Face,
    ) -> Result<(), Failure> {
        let parents = self.ancestors.change_find(edge)?;
        if let Some(index) = parents.iter().position(|parent| parent.is_same(removed)) {
            parents.remove(index);
        }
        parents.push(face.clone().into());
        Ok(())
    }
    fn apply_opposite_edge(
        &mut self,
        source: &Shape,
        face: &Face,
        removed: &Shape,
        used: &MapOfShape,
        ancestors: &IndexedDataMapOfShapeListOfShape,
        context: &Context,
    ) -> Result<(), Failure> {
        let mut opposite = unused_edge(face, used)?;
        orient_opposite_edge(source, removed, &mut opposite, context)?;
        self.insert_ancestors(&opposite, vec![face.clone().into(), removed.clone()]);
        self.reclassify(&opposite)?;
        self.bind_replacement(removed, source, &opposite)?;
        for vertex in children(&opposite) {
            self.insert_ancestors(
                &vertex,
                ancestors.find_from_key(&vertex)?.iter().cloned().collect(),
            );
        }
        Ok(())
    }
    fn bind_replacement(
        &mut self,
        face: &Shape,
        source: &Shape,
        replacement: &Edge,
    ) -> Result<(), Failure> {
        if !self.replacements.is_bound(face) {
            self.replacements
                .bind(face.clone(), ShapeBindings::default());
        }
        self.replacements
            .change_find(face)?
            .bind(source.clone(), replacement.clone());
        Ok(())
    }
}
fn orient_artificial_face(edge: &Shape, opposite: &Shape, face: &mut Face) -> Result<(), Failure> {
    let mut normal = Direction3::default();
    algo_tools3d::get_normal_to_face_on_edge(&topods::edge(edge)?, face, &mut normal, None)?;
    let binormal = face_binormal(opposite, edge)?;
    if binormal.dot(Vector3::new(normal.x(), normal.y(), normal.z())) < 0. {
        face.reverse();
    }
    Ok(())
}

fn unused_edge(face: &Face, used: &MapOfShape) -> Result<Edge, Failure> {
    for edge in edges_in_face(face)? {
        if !used.contains(&edge) {
            return topods::edge(&edge);
        }
    }
    Ok(Edge::default())
}
fn orient_opposite_edge(
    source: &Shape,
    removed: &Shape,
    opposite: &mut Edge,
    context: &Context,
) -> Result<(), Failure> {
    for edge in edges_in_face(removed)? {
        if !source.is_same(&edge) {
            continue;
        }
        if bop_tools::is_split_edge_to_reverse(opposite, &topods::edge(&edge)?, context, &mut 0)? {
            opposite.reverse();
        }
        break;
    }
    Ok(())
}
fn face_binormal(opposite: &Shape, edge: &Shape) -> Result<Vector3, Failure> {
    let incident = occurrence(opposite, edge)?;
    let (curve, first, last) = BRepTool::curve(&incident)?;
    let parameter = (first + last) / 2.;
    let on_edge = curve.value(parameter)?;
    let mut uv = Point2::default();
    let mut near = Point3::default();
    algo_tools3d::point_near_edge_step(
        &incident,
        &topods::face(opposite)?,
        parameter,
        1e-5,
        &mut uv,
        &mut near,
    )?;
    let binormal = Vector3::new(
        near.x() - on_edge.x(),
        near.y() - on_edge.y(),
        near.z() - on_edge.z(),
    );
    Ok(binormal)
}
