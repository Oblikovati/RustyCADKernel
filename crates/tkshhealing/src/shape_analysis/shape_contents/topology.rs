use super::*;
impl ShapeAnalysisShapeContents {
    pub(super) fn count_solids(&mut self, shape: &Shape) -> Result<(), Failure> {
        self.counts.shared_solids =
            visit_kind(shape, ShapeEnum::Solid, ShapeEnum::Shape, |solid| {
                let count = count_descendants(solid, ShapeEnum::Shell)?;
                if count > 1 {
                    self.counts.solids_with_voids += 1;
                }
                self.counts.solids += 1;
                Ok(())
            })?;
        Ok(())
    }
    pub(super) fn count_shells(&mut self, shape: &Shape) -> Result<i32, Failure> {
        let mut face_count = 0;
        self.counts.shared_shells =
            visit_kind(shape, ShapeEnum::Shell, ShapeEnum::Shape, |shell| {
                self.counts.shells += 1;
                face_count += count_descendants(shell, ShapeEnum::Face)?;
                Ok(())
            })?;
        Ok(face_count)
    }
    pub(super) fn count_faces(&mut self, shape: &Shape) -> Result<(), Failure> {
        self.counts.shared_faces = visit_kind(shape, ShapeEnum::Face, ShapeEnum::Shape, |shape| {
            let face = topods::face(shape)?;
            self.counts.faces += 1;
            let (surface, _) = BRepTool::surface_with_location(&face)?;
            let mut face = face;
            face.set_location(&Location::default(), true)?;
            self.analyze_surface(&surface, &face)?;
            self.analyze_face_wires(&face)
        })?;
        Ok(())
    }
    pub(super) fn count_edges(&mut self, shape: &Shape) -> Result<(), Failure> {
        self.counts.shared_edges = visit_kind(shape, ShapeEnum::Edge, ShapeEnum::Shape, |shape| {
            let mut edge = topods::edge(shape)?;
            edge.set_location(&Location::default(), true)?;
            self.counts.edges += 1;
            self.analyze_spatial_curve(&edge)
        })?;
        Ok(())
    }
    pub(super) fn count_remaining_topology(&mut self, shape: &Shape) -> Result<(), Failure> {
        self.counts.shared_vertices =
            visit_kind(shape, ShapeEnum::Vertex, ShapeEnum::Shape, |_| {
                self.counts.vertices += 1;
                Ok(())
            })?;
        self.counts.shared_free_edges =
            visit_kind(shape, ShapeEnum::Edge, ShapeEnum::Face, |_| {
                self.counts.free_edges += 1;
                Ok(())
            })?;
        self.counts.shared_free_wires =
            visit_kind(shape, ShapeEnum::Wire, ShapeEnum::Face, |_| {
                self.counts.free_wires += 1;
                Ok(())
            })?;
        Ok(())
    }
    fn analyze_face_wires(&mut self, face: &Face) -> Result<(), Failure> {
        let mut wires = Explorer::new(face, ShapeEnum::Wire, ShapeEnum::Shape)?;
        let (mut count, mut max_seams) = (0, 0);
        while wires.more() {
            count += 1;
            let seams = self.analyze_wire(&topods::wire(wires.current())?, face)?;
            max_seams = max_seams.max(seams);
            wires.next()?;
        }
        if max_seams == 1 {
            self.counts.wires_with_seam += 1;
        } else if max_seams > 1 {
            self.counts.wires_with_several_seams += 1;
        }
        if count > 1 {
            self.counts.faces_with_several_wires += 1;
        }
        Ok(())
    }
}
fn count_descendants(shape: &Shape, kind: ShapeEnum) -> Result<i32, Failure> {
    let mut count = 0;
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape)?;
    while explorer.more() {
        count += 1;
        explorer.next()?;
    }
    Ok(count)
}
