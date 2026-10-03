use super::ShapeFixFace;
use crate::shape_extend::ShapeExtendStatus;
use crate::shape_fix::{FixRoot, need_fix};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_tools::ReShape,
    topods::{self, Face, Shape, ShapeIterator},
};
use rustycad_tkernel::{ncollection::Sequence, standard::Failure};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

impl ShapeFixFace {
    pub(super) fn perform_second_wire_pass(
        &self,
        reordered_wires: &mut Vec<(Shape, Shape)>,
    ) -> Result<(), Failure> {
        let faces = rustycad_tkbrep::topexp::Explorer::new(
            &self.result(),
            ShapeEnum::Face,
            ShapeEnum::Shape,
        )?;
        for face in faces {
            *self.face_state.write() = face;
            self.perform_one_second_face_pass(reordered_wires)?;
            self.perform_face_stages()?;
        }
        Ok(())
    }

    fn perform_one_second_face_pass(
        &self,
        reordered_wires: &mut Vec<(Shape, Shape)>,
    ) -> Result<(), Failure> {
        let source = super::super::apply_context(&self.face_state.read(), &self.context())?;
        let mut rebuilt = empty_forward_face(&source)?;
        let saved = self.disable_second_pass_modes();
        let mut pass = SecondPassChanges::default();
        for child in ShapeIterator::new(&source, false, true) {
            if child.shape_type()? != ShapeEnum::Wire {
                BRepBuilder::default().add(&mut rebuilt, &child)?;
                continue;
            }
            self.process_second_pass_wire(&child, &mut rebuilt, &mut pass, reordered_wires)?;
        }
        self.restore_second_pass_modes(saved);
        let split_needed = self.finish_second_face_pass(&source, rebuilt, pass)?;
        if split_needed {
            self.split_removed_segments()?;
        }
        Ok(())
    }

    fn disable_second_pass_modes(&self) -> SavedSecondModes {
        let wire = &self.wire_tool;
        let saved = SavedSecondModes {
            small: wire.fix_small_mode(),
            connected: wire.fix_connected_mode(),
            edge_curves: wire.fix_edge_curves_mode(),
            degenerated: wire.fix_degenerated_mode(),
        };
        wire.set_fix_small_mode(0);
        wire.set_fix_connected_mode(0);
        wire.set_fix_edge_curves_mode(0);
        wire.set_fix_degenerated_mode(0);
        saved
    }

    fn restore_second_pass_modes(&self, saved: SavedSecondModes) {
        let wire = &self.wire_tool;
        wire.set_fix_small_mode(saved.small);
        wire.set_fix_connected_mode(saved.connected);
        wire.set_fix_edge_curves_mode(saved.edge_curves);
        wire.set_fix_degenerated_mode(saved.degenerated);
    }

    fn process_second_pass_wire(
        &self,
        child: &Shape,
        rebuilt: &mut Face,
        pass: &mut SecondPassChanges,
        reordered_wires: &mut Vec<(Shape, Shape)>,
    ) -> Result<(), Failure> {
        let wire = topods::wire(child)?;
        self.wire_tool.load(&wire)?;
        if self.wire_tool.nb_edges() == 0 {
            return self.keep_or_drop_empty_wire(&wire, rebuilt, pass);
        }
        if !self.wire_tool.perform()? {
            BRepBuilder::default().add(rebuilt, &wire.clone().into_shape())?;
            return Ok(());
        }
        self.append_performed_wire(child, rebuilt, pass, reordered_wires)
    }

    fn keep_or_drop_empty_wire(
        &self,
        wire: &rustycad_tkbrep::topods::Wire,
        rebuilt: &mut Face,
        pass: &mut SecondPassChanges,
    ) -> Result<(), Failure> {
        if self.wire_tool.wire_data()?.nb_non_manifold_edges() > 0 {
            BRepBuilder::default().add(rebuilt, &wire.clone().into_shape())?;
            return Ok(());
        }
        pass.fixed = true;
        self.record_wire_face_stage(ShapeExtendStatus::Done5);
        Ok(())
    }

    fn append_performed_wire(
        &self,
        original: &Shape,
        rebuilt: &mut Face,
        pass: &mut SecondPassChanges,
        reordered_wires: &mut Vec<(Shape, Shape)>,
    ) -> Result<(), Failure> {
        pass.fixed = second_wire_was_fixed(self);
        pass.split_needed |= self.wire_tool.status_removed_segment();
        let output = self.wire_tool.wire()?;
        if pass.fixed && self.wire_tool.nb_edges() == 0 {
            self.record_wire_face_stage(ShapeExtendStatus::Done5);
            return self.replace_wire(original, &output);
        }
        if pass.fixed {
            self.replace_wire(original, &output)?;
        } else {
            remember_reordered_wire(original, &output, reordered_wires);
        }
        if !self.add_loop_wires_if_enabled(rebuilt, pass)? {
            BRepBuilder::default().add(rebuilt, &output.clone().into_shape())?;
        }
        Ok(())
    }

    fn replace_wire(
        &self,
        original: &Shape,
        output: &rustycad_tkbrep::topods::Wire,
    ) -> Result<(), Failure> {
        if !self.context().is_null() {
            self.context()
                .replace(original, &output.clone().into_shape())?;
        }
        Ok(())
    }

    fn add_loop_wires_if_enabled(
        &self,
        rebuilt: &mut Face,
        pass: &mut SecondPassChanges,
    ) -> Result<bool, Failure> {
        if !need_fix(self.fix_loop_wires_mode(), true) {
            return Ok(false);
        }
        let mut loops = Sequence::<Shape>::default();
        if !self.fix_loop_wire(&mut loops)? {
            return Ok(false);
        }
        pass.fixed = true;
        self.record_wire_face_stage(ShapeExtendStatus::Done7);
        for wire in &loops {
            BRepBuilder::default().add(rebuilt, wire)?;
        }
        Ok(true)
    }

    fn finish_second_face_pass(
        &self,
        source: &Shape,
        mut rebuilt: Face,
        pass: SecondPassChanges,
    ) -> Result<bool, Failure> {
        if !pass.fixed {
            return Ok(pass.split_needed);
        }
        if !*self.forward.read() {
            rebuilt.set_orientation(Orientation::Reversed);
        }
        if !self.context().is_null() {
            self.context()
                .replace(source, &rebuilt.clone().into_shape())?;
        }
        *self.face_state.write() = rebuilt.into_shape();
        self.record_wire_face_stage(ShapeExtendStatus::Done1);
        Ok(pass.split_needed)
    }

    fn split_removed_segments(&self) -> Result<(), Failure> {
        let source = super::super::apply_context(&self.face_state.read(), &self.context())?;
        let face = topods::face(&source)?;
        let mut rebuilt = empty_forward_face(&source)?;
        let mut wire_count = 0;
        let mut split_wires = Vec::new();
        for child in ShapeIterator::new(&source, false, true) {
            if child.shape_type()? != ShapeEnum::Wire {
                BRepBuilder::default().add(&mut rebuilt, &child)?;
                continue;
            }
            self.collect_split_wire_child(
                &face,
                child,
                &mut rebuilt,
                &mut wire_count,
                &mut split_wires,
            )?;
        }
        self.finish_split_wires(&source, rebuilt, wire_count, split_wires)
    }

    fn collect_split_wire_child(
        &self,
        face: &Face,
        child: Shape,
        rebuilt: &mut Face,
        wire_count: &mut usize,
        split_wires: &mut Vec<rustycad_tkbrep::topods::Wire>,
    ) -> Result<(), Failure> {
        if !eligible_split_wire(&child)? {
            return BRepBuilder::default().add(rebuilt, &child);
        }
        *wire_count += 1;
        split_wires.extend(super::split_wire::split_wire(face, &topods::wire(&child)?)?);
        Ok(())
    }

    fn finish_split_wires(
        &self,
        source: &Shape,
        mut rebuilt: Face,
        wire_count: usize,
        split_wires: Vec<rustycad_tkbrep::topods::Wire>,
    ) -> Result<(), Failure> {
        if wire_count >= split_wires.len() {
            return Ok(());
        }
        for wire in split_wires {
            BRepBuilder::default().add(&mut rebuilt, &wire.clone().into_shape())?;
        }
        if !self.context().is_null() {
            self.context()
                .replace(source, &rebuilt.clone().into_shape())?;
        }
        *self.face_state.write() = rebuilt.into_shape();
        self.record_wire_face_stage(ShapeExtendStatus::Done8);
        Ok(())
    }
}

#[derive(Default)]
struct SecondPassChanges {
    fixed: bool,
    split_needed: bool,
}

struct SavedSecondModes {
    small: i32,
    connected: i32,
    edge_curves: i32,
    degenerated: i32,
}

fn empty_forward_face(source: &Shape) -> Result<Face, Failure> {
    let mut face = topods::face(&source.empty_copied()?)?;
    face.set_orientation(Orientation::Forward);
    Ok(face)
}

fn second_wire_was_fixed(fixer: &ShapeFixFace) -> bool {
    let wire = &fixer.wire_tool;
    [
        wire.status_lacking(ShapeExtendStatus::Done),
        wire.status_self_intersection(ShapeExtendStatus::Done),
        wire.status_notches(ShapeExtendStatus::Done),
        wire.status_fix_tails(ShapeExtendStatus::Done),
    ]
    .into_iter()
    .any(|changed| changed)
}

fn remember_reordered_wire(
    original: &Shape,
    replacement: &rustycad_tkbrep::topods::Wire,
    reordered_wires: &mut Vec<(Shape, Shape)>,
) {
    let output = replacement.clone().into_shape();
    if !original.is_same(&output) {
        reordered_wires.push((original.clone(), output));
    }
}

fn eligible_split_wire(shape: &Shape) -> Result<bool, Failure> {
    if shape.shape_type()? != ShapeEnum::Wire {
        return Ok(false);
    }
    Ok(matches!(
        shape.orientation(),
        Orientation::Forward | Orientation::Reversed
    ))
}
