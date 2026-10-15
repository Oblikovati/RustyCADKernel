// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_Editor.cxx (GetChildShapeLabels,
// GetParentShapeLabels and FilterShapeTree) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The shape labels related to a label (its children and referred shapes, its parents and
//! users) and the filtering of the shape tree of a document.
use super::support::child_labels;
use super::{XCAFDocEditor, XCAFDocShapeTool};
use rustycad_tkernel::ncollection::Map;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfChildIterator, TdfLabel};

/// A set of labels (`NCollection_Map<TDF_Label>`).
pub type XCAFDocLabelSet = Map<TdfLabel>;

impl XCAFDocEditor {
    /// `GetChildShapeLabels(theLabel, theRelatedLabels)`: adds the shape label, the labels
    /// under an assembly or simple shape and the shape a reference refers to, recursively.
    pub fn get_child_shape_labels(
        label: &TdfLabel,
        related: &mut XCAFDocLabelSet,
    ) -> Result<(), Failure> {
        if label.is_null() || !XCAFDocShapeTool::is_shape(label)? || !related.add(label.clone()) {
            return Ok(()); // Label already processed
        }
        if XCAFDocShapeTool::is_assembly(label)? || XCAFDocShapeTool::is_simple_shape(label)? {
            for child in child_labels(label)? {
                Self::get_child_shape_labels(&child, related)?;
            }
        }
        if XCAFDocShapeTool::is_reference(label)? {
            let mut referred = TdfLabel::default();
            XCAFDocShapeTool::get_referred_shape(label, &mut referred)?;
            Self::get_child_shape_labels(&referred, related)?;
        }
        Ok(())
    }
    /// `GetParentShapeLabels(theLabel, theRelatedLabels)`: adds the shape label and, through
    /// the father of a sub-shape or component or else the users of the shape, its parents.
    pub fn get_parent_shape_labels(
        label: &TdfLabel,
        related: &mut XCAFDocLabelSet,
    ) -> Result<(), Failure> {
        if label.is_null() || !XCAFDocShapeTool::is_shape(label)? || !related.add(label.clone()) {
            return Ok(()); // Label already processed
        }
        if XCAFDocShapeTool::is_sub_shape_label(label)? || XCAFDocShapeTool::is_component(label)? {
            return Self::get_parent_shape_labels(&label.father(), related);
        }
        let mut users = Vec::new();
        XCAFDocShapeTool::get_users(label, &mut users, false)?;
        for user in &users {
            Self::get_parent_shape_labels(user, related)?;
        }
        Ok(())
    }
    /// `FilterShapeTree(theShapeTool, theLabelsToKeep)`: forgets the attributes of every
    /// label of the shape tree that is not kept, a child or a parent of a kept label, then
    /// updates the assemblies; false for an empty set.
    pub fn filter_shape_tree(
        shape_tool: &Handle<XCAFDocShapeTool>,
        keep: &XCAFDocLabelSet,
    ) -> Result<bool, Failure> {
        if keep.is_empty() {
            return Ok(false);
        }
        let mut kept = XCAFDocLabelSet::default();
        for label in keep {
            Self::get_child_shape_labels(label, &mut kept)?;
        }
        for label in keep {
            let mut parents = XCAFDocLabelSet::default();
            Self::get_parent_shape_labels(label, &mut parents)?;
            kept.unite(&parents);
        }
        let tool = rustycad_tklcaf::live(shape_tool, "XCAFDoc_Editor::FilterShapeTree")?;
        let mut iterator = TdfChildIterator::new(&tool.label(), true)?;
        while iterator.more() {
            let label = iterator.value();
            if !kept.contains(&label) {
                label.forget_all_attributes(false)?;
            }
            iterator.next();
        }
        tool.update_assemblies()?;
        Ok(true)
    }
}
