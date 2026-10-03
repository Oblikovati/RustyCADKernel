//! Native gap-repair orchestration, including recursive compound status ordering.
use super::{FixRoot, ShapeFixWireframe};
use crate::{
    shape_build::ShapeBuildReShape,
    shape_extend::{ShapeExtend, ShapeExtendStatus},
    shape_fix::{ShapeFix, ShapeFixEdge, ShapeFixWire},
};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_tools::ReShape,
    topexp::Explorer,
    topods::{self, Compound, Face, Shape, ShapeIterator, Wire},
    toptools::ShapeMapHasher,
};
use rustycad_tkernel::{
    message::{Msg, ProgressRange},
    ncollection::DataMap,
    precision,
    standard::{Failure, Handle},
    tcollection::AsciiString,
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::toploc::Location;
impl ShapeFixWireframe {
    /// Repair wire curve gaps through the shared reshape context; `tool.fix_wire_gaps()?`.
    pub fn fix_wire_gaps(&self) -> Result<bool, Failure> {
        self.state.write().gaps = 0;
        if self.shape().is_null() {
            return Ok(false);
        }
        self.prepare_gap_context()?;
        if self.shape().shape_type()? == ShapeEnum::Compound {
            return self.fix_compound_gaps();
        }
        let precision = if self.precision() > 0. {
            self.precision()
        } else {
            precision::confusion()
        };
        let wire = ShapeFixWire::new();
        wire.set_context(&self.context());
        wire.set_precision(precision);
        self.visit_gap_wires(&wire, precision, false)?;
        if !self.status_wire_gaps(ShapeExtendStatus::Done) {
            return Ok(false);
        }
        self.apply_gap_context()?;
        ShapeFix::same_parameter(
            &self.shape(),
            false,
            0.,
            &ProgressRange::default(),
            &Handle::default(),
        )?;
        self.visit_gap_wires(&wire, precision, true)?;
        Ok(true)
    }
    fn prepare_gap_context(&self) -> Result<(), Failure> {
        if self.context().is_null() {
            self.set_context(&Handle::new(ShapeBuildReShape::new()));
            return Ok(());
        }
        let original = self.shape();
        self.state.write().shape = Shape::default();
        let applied = ReShape::apply(&*self.context(), &original, ShapeEnum::Shape)?;
        self.state.write().shape = applied;
        Ok(())
    }
    fn apply_gap_context(&self) -> Result<(), Failure> {
        let applied = ReShape::apply(&*self.context(), &self.shape(), ShapeEnum::Shape)?;
        self.state.write().shape = applied;
        Ok(())
    }
    fn fix_compound_gaps(&self) -> Result<bool, Failure> {
        let original = self.shape();
        let builder = BRepBuilder::default();
        let mut compound = Compound::default();
        builder.make_compound(&mut compound);
        let mut replacements = DataMap::<Shape, Shape, ShapeMapHasher>::default();
        let mut modified = false;
        for child in ShapeIterator::new(&original, true, true) {
            let (replacement, changed) = self.fix_gap_child(child, &mut replacements)?;
            modified |= changed;
            builder.add(&mut compound, &replacement)?;
        }
        if modified {
            compound.set_orientation(original.orientation());
            ReShape::replace(&*self.context(), &original, &compound)?;
        }
        let applied = ReShape::apply(&*self.context(), &original, ShapeEnum::Shape)?;
        self.state.write().shape = applied;
        // Recursive calls clear the status; native reports the final child's flags.
        Ok(self.status_wire_gaps(ShapeExtendStatus::Done))
    }
    fn fix_gap_child(
        &self,
        mut child: Shape,
        replacements: &mut DataMap<Shape, Shape, ShapeMapHasher>,
    ) -> Result<(Shape, bool), Failure> {
        let location = child.location().clone();
        child.set_location(&Location::default(), false)?;
        let mut result = if replacements.is_bound(&child) {
            replacements.find(&child)?.oriented(child.orientation())
        } else {
            self.state.write().shape = child.clone();
            self.fix_wire_gaps()?;
            let result = self.shape();
            replacements.bind(self.shape(), result.clone());
            result
        };
        let modified = !result.is_same(&child);
        result.set_location(&location, false)?;
        Ok((result, modified))
    }
    fn visit_gap_wires(
        &self,
        tool: &ShapeFixWire,
        precision: f64,
        cleanup: bool,
    ) -> Result<(), Failure> {
        let shape = self.shape();
        for face in Explorer::new(&shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            self.visit_face_gap_wires(tool, &face, precision, cleanup)?;
        }
        for wire in Explorer::new(&shape, ShapeEnum::Wire, ShapeEnum::Face)? {
            let loaded = if cleanup {
                wire.clone()
            } else {
                ReShape::apply(&*self.context(), &wire, ShapeEnum::Shape)?
            };
            tool.load(&topods::wire(&loaded)?)?;
            tool.set_precision(precision);
            self.repair_gap_wire(tool, &wire, false, cleanup)?;
        }
        Ok(())
    }
    fn visit_face_gap_wires(
        &self,
        tool: &ShapeFixWire,
        source: &Shape,
        precision: f64,
        cleanup: bool,
    ) -> Result<(), Failure> {
        let applied = if cleanup {
            source.clone()
        } else {
            ReShape::apply(&*self.context(), source, ShapeEnum::Shape)?
        };
        let mut face = topods::face(&applied)?;
        if face.orientation() == Orientation::Reversed {
            face.set_orientation(Orientation::Forward);
        }
        for wire in ShapeIterator::new(&face, true, true) {
            if wire.shape_type()? != ShapeEnum::Wire {
                continue;
            }
            self.load_face_gap_wire(tool, &wire, &face, precision, cleanup)?;
            self.repair_gap_wire(tool, &wire, true, cleanup)?;
        }
        Ok(())
    }
    fn load_face_gap_wire(
        &self,
        tool: &ShapeFixWire,
        source: &Shape,
        face: &Face,
        precision: f64,
        cleanup: bool,
    ) -> Result<(), Failure> {
        let applied = if cleanup {
            source.clone()
        } else {
            ReShape::apply(&*self.context(), source, ShapeEnum::Shape)?
        };
        tool.load(&topods::wire(&applied)?)?;
        tool.set_face(face)?;
        tool.set_precision(precision);
        Ok(())
    }
    fn repair_gap_wire(
        &self,
        tool: &ShapeFixWire,
        source: &Shape,
        on_face: bool,
        cleanup: bool,
    ) -> Result<(), Failure> {
        tool.fix_reorder()?;
        if cleanup {
            return cleanup_gap_wire(tool, &topods::wire(source)?);
        }
        tool.fix_gaps_3d()?;
        self.record_gap_status(tool, true);
        if on_face {
            tool.fix_gaps_2d()?;
            self.record_gap_status(tool, false);
        }
        if tool.status_gaps_3d(ShapeExtendStatus::Done)
            || (on_face && tool.status_gaps_2d(ShapeExtendStatus::Done))
        {
            self.send_warning(
                source,
                &Msg::from_key(&AsciiString::from("FixWireframe.FixFixWireGaps.MSG0")),
            );
        }
        Ok(())
    }
    fn record_gap_status(&self, tool: &ShapeFixWire, spatial: bool) {
        let flags = if spatial {
            (
                tool.status_gaps_3d(ShapeExtendStatus::Done),
                tool.status_gaps_3d(ShapeExtendStatus::Fail),
            )
        } else {
            (
                tool.status_gaps_2d(ShapeExtendStatus::Done),
                tool.status_gaps_2d(ShapeExtendStatus::Fail),
            )
        };
        if flags.0 {
            self.state.write().gaps |= ShapeExtend::encode_status(if spatial {
                ShapeExtendStatus::Done1
            } else {
                ShapeExtendStatus::Done2
            });
        }
        if flags.1 {
            self.state.write().gaps |= ShapeExtend::encode_status(if spatial {
                ShapeExtendStatus::Fail1
            } else {
                ShapeExtendStatus::Fail2
            });
        }
    }
}
fn cleanup_gap_wire(tool: &ShapeFixWire, wire: &Wire) -> Result<(), Failure> {
    tool.fix_self_intersection()?;
    let edges = ShapeFixEdge::new();
    for edge in ShapeIterator::new(wire, true, true) {
        edges.fix_vertex_tolerance(&topods::edge(&edge)?)?;
    }
    Ok(())
}
