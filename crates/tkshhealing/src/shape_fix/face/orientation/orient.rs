use super::classify::WireRelation;
use super::{FaceWireSet, ShapeFixFace, ShapeFixFaceWireMap, reverse_wire, send_reversed_warning};
use crate::shape_fix::FixRoot;
use rustycad_tkbrep::topods::{Face, Shape, Wire};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::message::Msg;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::State;

pub(super) fn orient_classified_wires(
    fixer: &ShapeFixFace,
    face: &Face,
    wires: &FaceWireSet,
    relations: Vec<WireRelation>,
    output_map: &mut ShapeFixFaceWireMap,
) -> Result<(Vec<Wire>, Vec<usize>, bool), Failure> {
    let internal = collect_internal_shapes(&relations);
    let mut changes = OrientationChanges {
        fixer,
        face,
        output_map,
        output: wires.oriented.clone(),
        reversed: Vec::new(),
        changed: wires.changed,
    };
    changes.apply_relations(&relations, &internal)?;
    Ok((changes.output, changes.reversed, changes.changed))
}

fn collect_internal_shapes(relations: &[WireRelation]) -> MapOfShape {
    let mut internal = MapOfShape::default();
    for relation in relations {
        for shape in relation.internal.iter() {
            internal.add(shape.clone());
        }
    }
    internal
}

struct OrientationChanges<'a> {
    fixer: &'a ShapeFixFace,
    face: &'a Face,
    output_map: &'a mut ShapeFixFaceWireMap,
    output: Vec<Wire>,
    reversed: Vec<usize>,
    changed: bool,
}

impl OrientationChanges<'_> {
    fn apply_relations(
        &mut self,
        relations: &[WireRelation],
        internal: &MapOfShape,
    ) -> Result<(), Failure> {
        for (index, relation) in relations.iter().enumerate() {
            self.orient_first(index, relation)?;
        }
        for (index, relation) in relations.iter().enumerate() {
            self.orient_nested(index, relation, internal)?;
        }
        Ok(())
    }

    fn orient_first(&mut self, index: usize, relation: &WireRelation) -> Result<(), Failure> {
        if relation.containing == State::Unknown {
            return self.warn_unresolved(index);
        }
        let wire = self.output[index].clone();
        if relation.containing == State::Out && relation.infinite == State::In {
            return self.reverse_and_map(index, &wire, relation);
        }
        if relation.containing == State::Out {
            bind_wire_children(self.output_map, &wire, &relation.internal);
        }
        Ok(())
    }

    fn warn_unresolved(&self, index: usize) -> Result<(), Failure> {
        let warning = Msg::from_key(&rustycad_tkernel::tcollection::AsciiString::from(
            "FixAdvFace.FixOrientation.MSG11",
        ));
        self.fixer
            .send_warning(&self.output[index].clone().into_shape(), &warning);
        Ok(())
    }

    fn orient_nested(
        &mut self,
        index: usize,
        relation: &WireRelation,
        internal: &MapOfShape,
    ) -> Result<(), Failure> {
        if relation.containing != State::In {
            return Ok(());
        }
        let wire = self.output[index].clone();
        let is_internal = internal.contains(&wire.clone().into_shape());
        self.apply_nested_state(index, relation, &wire, is_internal)
    }

    fn apply_nested_state(
        &mut self,
        index: usize,
        relation: &WireRelation,
        wire: &Wire,
        is_internal: bool,
    ) -> Result<(), Failure> {
        match (relation.infinite, is_internal) {
            (State::In, false) => self.reverse_and_map(index, wire, relation),
            (State::Out, true) => self.reverse_only(index, wire),
            (State::Out, false) => {
                bind_wire_children(self.output_map, wire, &relation.internal);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn reverse_only(&mut self, index: usize, wire: &Wire) -> Result<(), Failure> {
        let fixed = reverse_wire(self.face, wire)?;
        send_reversed_warning(self.fixer, &fixed);
        self.record_reversed(index, fixed);
        Ok(())
    }

    fn reverse_and_map(
        &mut self,
        index: usize,
        wire: &Wire,
        relation: &WireRelation,
    ) -> Result<(), Failure> {
        let fixed = reverse_wire(self.face, wire)?;
        send_reversed_warning(self.fixer, &fixed);
        bind_wire_children(self.output_map, &fixed, &relation.internal);
        self.record_reversed(index, fixed);
        Ok(())
    }

    fn record_reversed(&mut self, index: usize, wire: Wire) {
        self.output[index] = wire;
        self.reversed.push(index + 1);
        self.changed = true;
    }
}

fn bind_wire_children(map: &mut ShapeFixFaceWireMap, wire: &Wire, children: &List<Shape>) {
    map.bind(wire.clone().into_shape(), clone_shapes(children));
}

fn clone_shapes(shapes: &List<Shape>) -> List<Shape> {
    List::from_iter_in(shapes.iter().cloned(), shapes.allocator().clone())
}
