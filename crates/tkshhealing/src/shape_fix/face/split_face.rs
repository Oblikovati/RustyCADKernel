//! Preserve OCCT's wire-map face splitting, including the last-face working state.
use super::{ShapeFixFace, ShapeFixFaceWireMap};
use crate::{
    shape_analysis::ShapeAnalysisEdge, shape_extend::ShapeExtendWireData, shape_fix::FixRoot,
};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_tools::{self, ReShape},
    topexp::Explorer,
    topods::{self, Compound, Face, Shape, ShapeIterator, Wire},
};
use rustycad_tkernel::{ncollection::List, precision, standard::Failure};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;
impl ShapeFixFace {
    /// Split outside wire groups into faces; `fixer.fix_split_face(&wire_map)?`.
    pub fn fix_split_face(&self, groups: &ShapeFixFaceWireMap) -> Result<bool, Failure> {
        let original = self.face()?;
        let source = if let Some(context) = self.context().get() {
            ReShape::apply(context, &original, ShapeEnum::Shape)?
        } else {
            original.clone().into_shape()
        };
        let Some(faces) = self.collect_split_faces(&source, groups)? else {
            return Ok(false);
        };
        if faces.len() < 2 {
            return Ok(false);
        }
        self.publish_split_faces(&original, &faces)?;
        Ok(true)
    }
    fn collect_split_faces(
        &self,
        source: &Shape,
        groups: &ShapeFixFaceWireMap,
    ) -> Result<Option<Vec<Face>>, Failure> {
        let mut faces = Vec::new();
        let mut wires = 0;
        let mut new_wires = 0;
        for child in ShapeIterator::new(source, false, true).filter(eligible_wire) {
            wires += 1;
            if !groups.is_bound(&child) {
                continue;
            }
            let wire = topods::wire(&child)?;
            let edges = ShapeExtendWireData::from_wire(&wire, true, true)?;
            if edges.nb_edges() == 0 {
                continue;
            }
            if !split_wire_closed(&edges)? {
                return Ok(None);
            }
            let internal = groups.find(&child)?;
            faces.push(self.build_split_face(source, &wire, internal)?);
            new_wires += 1 + internal.length();
        }
        Ok((wires == new_wires).then_some(faces))
    }
    fn build_split_face(
        &self,
        source: &Shape,
        wire: &Wire,
        internal: &List<Shape>,
    ) -> Result<Face, Failure> {
        let mut face = topods::face(&source.empty_copied()?)?;
        face.set_orientation(Orientation::Forward);
        BRepBuilder::default().add(&mut face, wire)?;
        for hole in internal {
            add_split_hole(&mut face, hole)?;
        }
        if !*self.forward.read() {
            face.set_orientation(Orientation::Reversed);
        }
        Ok(face)
    }
    fn publish_split_faces(&self, original: &Face, faces: &[Face]) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let mut compound = Compound::default();
        builder.make_compound(&mut compound);
        for face in faces {
            builder.add(&mut compound, face)?;
        }
        *self.result.write() = compound.clone().into_shape();
        if let Some(context) = self.context().get() {
            ReShape::replace(context, original, &compound)?;
        }
        // Native leaves myFace referring to the last explored result face.
        for shape in Explorer::new(&compound, ShapeEnum::Face, ShapeEnum::Shape)? {
            let face = topods::face(&shape)?;
            *self.face_state.write() = face.clone().into_shape();
            brep_tools::update_face(&face)?;
        }
        Ok(())
    }
}
fn eligible_wire(shape: &Shape) -> bool {
    shape.shape_type() == Ok(ShapeEnum::Wire)
        && matches!(
            shape.orientation(),
            Orientation::Forward | Orientation::Reversed
        )
}
fn split_wire_closed(edges: &ShapeExtendWireData) -> Result<bool, Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let first = analysis.first_vertex(&edges.edge(1)?);
    let last = analysis.last_vertex(&edges.edge(edges.nb_edges())?);
    Ok(first.is_same(&last))
}
fn add_split_hole(face: &mut Face, hole: &Shape) -> Result<(), Failure> {
    let mut probe = topods::face(&face.empty_copied()?)?;
    probe.set_orientation(Orientation::Forward);
    let builder = BRepBuilder::default();
    builder.add(&mut probe, hole)?;
    let classifier = BRepTopAdaptorFClass2d::new(&probe, precision::p_confusion())?;
    let oriented = if classifier.perform_infinite_point()? == State::In {
        hole.clone()
    } else {
        hole.reversed()
    };
    builder.add(face, &oriented)
}
