use super::*;
impl BRepOffsetAnalyse {
    pub(super) fn select_different_offsets(
        &self,
        edges: &[Shape],
    ) -> Result<TangentSelection, Failure> {
        let mut selection = TangentSelection::default();
        for edge in edges {
            let parents = self.ancestors(edge)?;
            let (first, last) = (
                self.face_offset(&parents[0]),
                self.face_offset(parents.last().unwrap()),
            );
            if first == last {
                continue;
            }
            selection.edges.push(edge.clone());
            selection
                .offsets
                .bind(edge.clone(), if first < last { last } else { first });
            let minimum = if first < last {
                &parents[0]
            } else {
                parents.last().unwrap()
            };
            self.find_minimum_edges(edge, minimum, &mut selection)?;
        }
        self.collect_vertex_faces(&mut selection)?;
        Ok(selection)
    }
    fn find_minimum_edges(
        &self,
        edge: &Shape,
        minimum: &Shape,
        selection: &mut TangentSelection,
    ) -> Result<(), Failure> {
        for vertex in children(edge) {
            if self.ancestors(&vertex)?.len() != 3 {
                continue;
            }
            for candidate in edges_in_face(minimum)? {
                if candidate.is_same(edge) {
                    continue;
                }
                if children(&candidate)
                    .iter()
                    .any(|child| child.is_same(&vertex))
                {
                    selection.minimum_edges.bind(vertex.clone(), candidate);
                }
            }
        }
        Ok(())
    }
    fn collect_vertex_faces(&self, selection: &mut TangentSelection) -> Result<(), Failure> {
        for edge in &selection.edges {
            let mut fence = MapOfShape::default();
            for face in self.ancestors(edge)? {
                fence.add(face.clone());
            }
            for vertex in children(edge) {
                if !selection.vertex_faces.is_bound(&vertex) {
                    selection.vertex_faces.bind(vertex.clone(), Vec::new());
                }
                self.append_other_faces(
                    &vertex,
                    &mut fence,
                    selection.vertex_faces.change_find(&vertex)?,
                )?;
            }
        }
        Ok(())
    }
    fn append_other_faces(
        &self,
        vertex: &Shape,
        fence: &mut MapOfShape,
        result: &mut Vec<Shape>,
    ) -> Result<(), Failure> {
        for edge in self.ancestors(vertex)? {
            let Ok(intervals) = self.intervals.find(edge) else {
                continue;
            };
            let Some(first) = intervals.first() else {
                continue;
            };
            if first.get_type()? == TypeOfConcavity::Tangential {
                continue;
            }
            for face in self.ancestors(edge)? {
                if fence.add(face.clone()) {
                    result.push(face.clone());
                }
            }
        }
        Ok(())
    }
}
