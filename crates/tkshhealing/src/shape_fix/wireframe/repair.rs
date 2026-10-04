use super::{
    FixRoot, ShapeFixWireframe, ShapeFixWireframeShapeLists, merge_faces, merge_free_wires,
};
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
        replacement.set_location(&location, false)?;
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
        )
    }
    /// Merge the supported unlinked 3D edge map; `tool.merge_small_edges(&mut small,&mut edge_faces,&mut face_small,&mut multiple,false,-1.0)?`.
    pub fn merge_small_edges(
        &self,
        small: &mut MapOfShape,
        edge_faces: &mut ShapeFixWireframeShapeLists,
        face_small: &mut ShapeFixWireframeShapeLists,
        multiple: &mut MapOfShape,
        mode_drop: bool,
        limit_angle: f64,
    ) -> Result<bool, Failure> {
        let drop_mode = mode_drop || self.mode_drop_small_edges();
        if small.is_empty() {
            return Ok(false);
        }
        if drop_mode && !face_small.is_empty() {
            return Err(unported_small_edge_merge());
        }
        if !multiple.is_empty() {
            return Err(unported_small_edge_merge());
        }
        let angle_limit = effective_angle_limit(limit_angle, self.limit_angle());
        let face_changed =
            merge_faces::merge_face_small_edges(self, small, edge_faces, face_small, angle_limit)?;
        merge_free_wires::merge_small_edges(self, small, multiple, angle_limit, drop_mode)
            .map(|free_changed| face_changed || free_changed)
    }
}

fn effective_angle_limit(explicit: f64, configured: f64) -> Option<f64> {
    if !(explicit > -1.0 || configured > -1.0) {
        return None;
    }
    if explicit < configured {
        return Some(configured);
    }
    Some(explicit)
}

pub(super) fn unported_small_edge_merge() -> Failure {
    Failure::with_message(
        "Standard_NotImplemented",
        "ShapeFix_Wireframe::MergeSmallEdges found a small-edge map; expected a merged or mode-dropped replacement",
    )
}
