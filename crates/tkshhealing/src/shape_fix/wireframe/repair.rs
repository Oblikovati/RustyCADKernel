use super::{FixRoot, ShapeFixWireframe, ShapeFixWireframeShapeLists};
use crate::{shape_build::ShapeBuildReShape, shape_extend::ShapeExtendStatus};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_tools::ReShape,
    topods::{Compound, Shape, ShapeIterator},
    toptools::{MapOfShape, ShapeMapHasher},
};
use rustycad_tkernel::{
    ncollection::DataMap,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::toploc::Location;

impl ShapeFixWireframe {
    /// Merge detectable unlinked 3D small edges; face-linked and pcurve cases return an error.
    /// `let changed = wireframe.fix_small_edges()?;`.
    pub fn fix_small_edges(&self) -> Result<bool, Failure> {
        self.state.write().small = 0;
        if self.shape().is_null() {
            return Ok(false);
        }
        self.prepare_small_edge_context()?;
        if self.shape().shape_type()? == ShapeEnum::Compound {
            return self.fix_small_edge_compound();
        }
        self.scan_and_merge_small_edges()
    }
    fn prepare_small_edge_context(&self) -> Result<(), Failure> {
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
    fn fix_small_edge_compound(&self) -> Result<bool, Failure> {
        let original = self.shape();
        let builder = BRepBuilder::default();
        let mut compound = Compound::default();
        builder.make_compound(&mut compound);
        let mut replacements = DataMap::<Shape, Shape, ShapeMapHasher>::default();
        let mut modified = false;
        for child in ShapeIterator::new(&original, true, true) {
            let (replacement, changed) = self.fix_small_edge_child(child, &mut replacements)?;
            modified |= changed;
            // check if resulting shape if not empty
            if replacement.is_null() {
                continue;
            }
            builder.add(&mut compound, &replacement)?;
        }
        self.publish_small_edge_compound(original, compound, modified)
    }
    fn fix_small_edge_child(
        &self,
        mut child: Shape,
        replacements: &mut DataMap<Shape, Shape, ShapeMapHasher>,
    ) -> Result<(Shape, bool), Failure> {
        let location = child.location().clone();
        child.set_location(&Location::default(), false)?;
        let source = child.clone();
        let mut replacement = self.cached_or_repair_child(child, replacements)?;
        let changed = !replacement.is_same(&source);
        if !replacement.is_null() {
            replacement.set_location(&location, false)?;
        }
        Ok((replacement, changed))
    }
    fn cached_or_repair_child(
        &self,
        child: Shape,
        replacements: &mut DataMap<Shape, Shape, ShapeMapHasher>,
    ) -> Result<Shape, Failure> {
        if replacements.is_bound(&child) {
            return Ok(replacements.find(&child)?.oriented(child.orientation()));
        }
        self.state.write().shape = child;
        self.fix_small_edges()?;
        let replacement = self.shape();
        replacements.bind(self.shape(), replacement.clone());
        Ok(replacement)
    }
    fn publish_small_edge_compound(
        &self,
        original: Shape,
        mut compound: Compound,
        modified: bool,
    ) -> Result<bool, Failure> {
        if modified {
            compound.set_orientation(original.orientation());
            ReShape::replace(&*self.context(), &original, &compound.into_shape())?;
        }
        let result = ReShape::apply(&*self.context(), &original, ShapeEnum::Shape)?;
        self.state.write().shape = result;
        Ok(self.status_small_edges(ShapeExtendStatus::Done))
    }
    fn scan_and_merge_small_edges(&self) -> Result<bool, Failure> {
        let mut small = MapOfShape::default();
        let mut multiple = MapOfShape::default();
        let mut edge_faces = ShapeFixWireframeShapeLists::default();
        let mut face_small = ShapeFixWireframeShapeLists::default();
        self.check_small_edges(&mut small, &mut edge_faces, &mut face_small, &mut multiple)?;
        self.merge_small_edges(
            &mut small,
            &mut edge_faces,
            &mut face_small,
            &mut multiple,
            false,
            -1.,
        )?;
        Ok(self.status_small_edges(ShapeExtendStatus::Done))
    }
}
