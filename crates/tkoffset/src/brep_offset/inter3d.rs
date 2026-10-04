//! Intersection-pair state from `BRepOffset_Inter3d`.
use rustycad_tkbool::brep_algo::BRepAlgoAsDes;
use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::State;
mod face_inter;

/// Stores completed face pairs and their new section edges; `BRepOffsetInter3d::new(...)`.
pub struct BRepOffsetInter3d {
    as_des: Handle<BRepAlgoAsDes>,
    touched: Vec<Shape>,
    done: Vec<(Shape, Vec<Shape>)>,
    new_edges: Vec<Shape>,
    side: State,
    tolerance: f64,
}

impl BRepOffsetInter3d {
    /// Creates empty intersection state; `BRepOffsetInter3d::new(&as_des, State::Out, 1e-7)`.
    pub fn new(as_des: &Handle<BRepAlgoAsDes>, side: State, tolerance: f64) -> Self {
        Self {
            as_des: as_des.clone(),
            touched: Vec::new(),
            done: Vec::new(),
            new_edges: Vec::new(),
            side,
            tolerance,
        }
    }

    /// Marks the face pair treated in both directions; `intersections.set_done(&f1, &f2)`.
    pub fn set_done(&mut self, first: &Face, second: &Face) {
        append_pair(&mut self.done, first, second);
        append_pair(&mut self.done, second, first);
    }

    /// Checks prior treatment using `IsSame`; `intersections.is_done(&f1, &f2)`.
    pub fn is_done(&self, first: &Face, second: &Face) -> bool {
        self.done.iter().any(|(key, values)| {
            key.is_same(first) && values.iter().any(|value| value.is_same(second))
        })
    }

    /// Returns faces touched by a stored intersection in insertion order.
    /// Example: `let touched = intersections.touched_faces();`.
    pub fn touched_faces(&self) -> &[Shape] {
        &self.touched
    }

    /// Returns unique section edges stored from the first face's list.
    /// Example: `let edges = intersections.new_edges();`.
    pub fn new_edges(&self) -> &[Shape] {
        &self.new_edges
    }

    /// Borrows the shared ancestry tool; `let as_des = intersections.as_des()`.
    pub fn as_des(&self) -> Handle<BRepAlgoAsDes> {
        self.as_des.clone()
    }

    /// Records a native Store call and then marks the pair done.
    // Native-private Store centralizes the same edge-list mutation used by FaceInter and ConnexInt.
    pub(crate) fn store(
        &mut self,
        first: &Face,
        second: &Face,
        first_edges: &[Shape],
        second_edges: &[Shape],
    ) -> Result<(), Failure> {
        if !first_edges.is_empty() {
            self.store_edges(first, second, first_edges, second_edges)?;
        }
        self.set_done(first, second);
        Ok(())
    }

    fn store_edges(
        &mut self,
        first: &Face,
        second: &Face,
        first_edges: &[Shape],
        second_edges: &[Shape],
    ) -> Result<(), Failure> {
        let Some(as_des) = self.as_des.get() else {
            return Err(Failure::with_message(
                "Standard_ConstructionError",
                "BRepOffset_Inter3d::Store requires a non-null BRepAlgo_AsDes handle",
            ));
        };
        append_unique(&mut self.touched, first);
        append_unique(&mut self.touched, second);
        store_first_edges(as_des, first, first_edges, &mut self.new_edges);
        store_face_edges(as_des, second, second_edges);
        Ok(())
    }

    /// Reads the constructor's side; `intersections.side() == State::Out`.
    pub fn side(&self) -> State {
        self.side
    }

    /// Reads the constructor's tolerance; `intersections.tolerance() == 1e-7`.
    pub fn tolerance(&self) -> f64 {
        self.tolerance
    }
}

fn append_pair(done: &mut Vec<(Shape, Vec<Shape>)>, key: &Shape, value: &Shape) {
    if let Some((_, values)) = done.iter_mut().find(|(known, _)| known.is_same(key)) {
        values.push(value.clone());
        return;
    }
    done.push((key.clone(), vec![value.clone()]));
}

fn append_unique(shapes: &mut Vec<Shape>, shape: &Shape) {
    if shapes.iter().any(|known| known.is_same(shape)) {
        return;
    }
    shapes.push(shape.clone());
}

fn store_face_edges(as_des: &BRepAlgoAsDes, face: &Face, edges: &[Shape]) {
    for edge in edges {
        as_des.add_shared(face, edge);
    }
}

fn store_first_edges(
    as_des: &BRepAlgoAsDes,
    face: &Face,
    edges: &[Shape],
    new_edges: &mut Vec<Shape>,
) {
    for edge in edges {
        as_des.add_shared(face, edge);
        append_unique(new_edges, edge);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkbrep::topods::{Edge, TEdge, TFace};
    use rustycad_tkernel::standard::{Handle, RuntimeTyped, Transient};
    use rustycad_tkg3d::top_abs::Orientation;
    use rustycad_tkmath::toploc::Location;

    #[test]
    fn store_tracks_both_face_lists_and_unique_state() {
        let first = test_face();
        let second = test_face();
        let edge = test_edge();
        let second_edge = test_edge();
        let as_des = Handle::new(BRepAlgoAsDes::new());
        let mut intersections = BRepOffsetInter3d::new(&as_des, State::Out, 1e-7);

        intersections
            .store(
                &first,
                &second,
                &[
                    edge.clone().into_shape(),
                    edge.oriented(Orientation::Reversed),
                ],
                &[second_edge.clone().into_shape()],
            )
            .unwrap();

        assert_stored_state(&intersections, &as_des, &first, &second);
    }

    #[test]
    fn empty_store_marks_pair_without_touching_shapes() {
        let first = test_face();
        let second = test_face();
        let as_des = Handle::new(BRepAlgoAsDes::new());
        let mut intersections = BRepOffsetInter3d::new(&as_des, State::In, 0.25);

        intersections.store(&first, &second, &[], &[]).unwrap();

        assert_empty_store_state(&intersections, &as_des, &first, &second);
        intersections.set_done(&first, &second);
        intersections.set_done(&first, &second);
        assert!(intersections.is_done(&first, &second));
        assert!(intersections.is_done(&second, &first));
        let reversed_first = Face::unchecked(first.reversed());
        assert!(intersections.is_done(&reversed_first, &second));
    }

    #[test]
    fn store_rejects_null_association_handle_safely() {
        let mut intersections = BRepOffsetInter3d::new(&Handle::default(), State::Out, 0.0);
        let result =
            intersections.store(&test_face(), &test_face(), &[test_edge().into_shape()], &[]);

        assert_eq!(
            result.unwrap_err().exception_type(),
            "Standard_ConstructionError"
        );
        assert!(intersections.touched_faces().is_empty());
        assert!(!intersections.is_done(&test_face(), &test_face()));
    }

    fn test_face() -> Face {
        Face::unchecked(test_shape(TFace::new()))
    }

    fn test_edge() -> Edge {
        Edge::unchecked(test_shape(TEdge::new()))
    }

    fn assert_stored_state(
        intersections: &BRepOffsetInter3d,
        as_des: &Handle<BRepAlgoAsDes>,
        first: &Face,
        second: &Face,
    ) {
        assert_eq!(intersections.touched_faces().len(), 2);
        assert_eq!(intersections.new_edges().len(), 1);
        assert!(intersections.is_done(first, second));
        assert!(intersections.is_done(second, first));
        assert_eq!(as_des.get().unwrap().descendant(first).count(), 2);
        assert_eq!(as_des.get().unwrap().descendant(second).count(), 1);
    }

    fn assert_empty_store_state(
        intersections: &BRepOffsetInter3d,
        as_des: &Handle<BRepAlgoAsDes>,
        first: &Face,
        second: &Face,
    ) {
        assert!(intersections.is_done(first, second));
        assert!(intersections.touched_faces().is_empty());
        assert!(intersections.new_edges().is_empty());
        assert_eq!(intersections.side(), State::In);
        assert_eq!(intersections.tolerance(), 0.25);
        assert_eq!(
            intersections.as_des().object_address(),
            as_des.object_address()
        );
    }

    fn test_shape<T: Transient + RuntimeTyped>(value: T) -> Shape {
        Shape::from_parts(
            Handle::new(value).into_cast(),
            Location::default(),
            Orientation::Forward,
        )
    }
}
