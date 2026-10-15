// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_ViewTool.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc_ViewTool`: the views of an XDE document (one `XCAFDoc_View` per child of the
//! tool's label) and their links to shapes, GD&T, clipping planes, notes and annotations
//! (graph nodes, the view the child).
use super::generic_empty_tool::generic_empty_attribute;
use super::graph_links::{
    child_labels_of, child_node, default_node, father_labels, link_father, node_of, unlink_fathers,
};
use super::support::{child_labels, find, find_or_add, live};
use super::{XCAFDoc, XCAFDocGraphNode, XCAFDocView};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Guid, Handle};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tklcaf::tdata_std::{TDataStdName, TDataStdTreeNode, TDataStdUAttribute};
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel, TdfTagSource};
use std::io;

generic_empty_attribute!(
    /// `XCAFDoc_ViewTool`.
    XCAFDocViewTool, "XCAFDoc_ViewTool", "efd213e4-6dfd-11d4-b9c8-0060b0ee281b", state: (),
    dump: |tool: &XCAFDocViewTool, stream: &mut DumpStream, depth: i32| -> io::Result<()> {
        // No override natively: the `TDataStd_GenericEmpty` dump.
        rustycad_tklcaf::tdata_std::dump_generic_empty(tool, stream, depth)
    }
);

/// The view links of `SetView`: the shapes, GD&T, clipping planes, notes and annotations.
#[derive(Clone, Debug, Default)]
pub struct XCAFDocViewLinks {
    /// The shape labels.
    pub shapes: Vec<TdfLabel>,
    /// The GD&T labels.
    pub gdts: Vec<TdfLabel>,
    /// The clipping plane labels.
    pub clipping_planes: Vec<TdfLabel>,
    /// The note labels.
    pub notes: Vec<TdfLabel>,
    /// The annotation labels.
    pub annotations: Vec<TdfLabel>,
}

/// Links the labels as fathers of the view's node of the GUID (created when there are labels).
fn link_all(view: &TdfLabel, labels: &[TdfLabel], guid: &Guid) -> Result<(), Failure> {
    if labels.is_empty() {
        return Ok(());
    }
    let child = child_node(view, guid)?;
    labels
        .iter()
        .try_for_each(|label| link_father(label, &child, guid))
}

/// Unlinks the view's node of the GUID from its fathers and forgets it; the clipping planes
/// forget the GD&T attribute of their emptied fathers, as natively.
fn detach(view: &TdfLabel, guid: &Guid) -> Result<(), Failure> {
    let father_guid = if *guid == XCAFDoc::view_ref_plane_guid() {
        XCAFDoc::view_ref_gdt_guid()
    } else {
        *guid
    };
    unlink_fathers(view, guid, &father_guid)?;
    view.forget_attribute_id(guid)?;
    Ok(())
}

impl XCAFDocViewTool {
    /// `Set(L)`: the tool of the label, added when absent.
    pub fn set(label: &TdfLabel) -> Result<Handle<XCAFDocViewTool>, Failure> {
        find_or_add(label, &Self::get_id(), Self::new)
    }
    /// `BaseLabel`.
    pub fn base_label(&self) -> TdfLabel {
        self.label()
    }
    /// `IsView(theLabel)`.
    pub fn is_view(&self, label: &TdfLabel) -> Result<bool, Failure> {
        label.is_attribute(&XCAFDocView::get_id())
    }
    /// `GetViewLabels(theLabels)`: the view children of the tool's label.
    pub fn get_view_labels(&self) -> Result<Vec<TdfLabel>, Failure> {
        let mut labels = Vec::new();
        for label in child_labels(&self.label())? {
            if self.is_view(&label)? {
                labels.push(label);
            }
        }
        Ok(labels)
    }
    /// `AddView`: a new child with an empty view, named `View`.
    pub fn add_view(&self) -> Result<TdfLabel, Failure> {
        let label = TdfTagSource::new_child_of(&self.label())?;
        XCAFDocView::set(&label)?;
        TDataStdName::set(&label, &ExtendedString::from_utf8("View"))?;
        Ok(label)
    }
    /// `SetView(theShapes, theGDTs, theClippingPlanes, theNotes, theAnnotations, theViewL)`:
    /// replaces every link of the view. The native note node is set to the clipping-plane
    /// GUID and the annotations reuse the note labels; both are kept.
    pub fn set_view(&self, links: &XCAFDocViewLinks, view: &TdfLabel) -> Result<(), Failure> {
        if !self.is_view(view)? {
            return Ok(());
        }
        let guids = [
            XCAFDoc::view_ref_shape_guid(),
            XCAFDoc::view_ref_gdt_guid(),
            XCAFDoc::view_ref_plane_guid(),
            XCAFDoc::view_ref_note_guid(),
            XCAFDoc::view_ref_annotation_guid(),
        ];
        guids.iter().try_for_each(|guid| detach(view, guid))?;
        link_all(view, &links.shapes, &guids[0])?;
        link_all(view, &links.gdts, &guids[1])?;
        link_all(view, &links.clipping_planes, &guids[2])?;
        if !links.notes.is_empty() {
            let child = default_node(view, &guids[2])?;
            links
                .notes
                .iter()
                .try_for_each(|note| link_father(note, &child, &guids[3]))?;
        }
        self.link_annotations(links, view)
    }
    /// The annotation links of `SetView`, as natively: an annotation without node links a new
    /// node of the note of the same index (`Standard_OutOfRange` past the notes), one with a
    /// node links the previous annotation node (`Standard_NullObject` at first).
    fn link_annotations(&self, links: &XCAFDocViewLinks, view: &TdfLabel) -> Result<(), Failure> {
        let guid = XCAFDoc::view_ref_annotation_guid();
        if links.annotations.is_empty() {
            return Ok(());
        }
        let child = default_node(view, &guid)?;
        let mut annotation_node: Handle<XCAFDocGraphNode> = Handle::default();
        for (index, annotation) in links.annotations.iter().enumerate() {
            if node_of(annotation, &guid)?.is_null() {
                let note = links
                    .notes
                    .get(index)
                    .ok_or_else(|| Failure::out_of_range("NCollection_Sequence::Value"))?;
                annotation_node = XCAFDocGraphNode::set(note)?;
            }
            let node = live(&annotation_node, "XCAFDoc_ViewTool::SetView")?;
            node.set_graph_id(&guid)?;
            node.set_child(&child)?;
            live(&child, "XCAFDoc_ViewTool::SetView")?.set_father(&annotation_node)?;
        }
        Ok(())
    }
    /// `SetView(theShapeLabels, theGDTLabels, theClippingPlaneLabels, theViewL)`.
    pub fn set_view_with_planes(
        &self,
        shapes: &[TdfLabel],
        gdts: &[TdfLabel],
        planes: &[TdfLabel],
        view: &TdfLabel,
    ) -> Result<(), Failure> {
        if !self.is_view(view)? {
            return Ok(());
        }
        let guids = [
            XCAFDoc::view_ref_shape_guid(),
            XCAFDoc::view_ref_gdt_guid(),
            XCAFDoc::view_ref_plane_guid(),
        ];
        guids.iter().try_for_each(|guid| detach(view, guid))?;
        link_all(view, shapes, &guids[0])?;
        link_all(view, gdts, &guids[1])?;
        link_all(view, planes, &guids[2])
    }
    /// `SetView(theShapeLabels, theGDTLabels, theViewL)`.
    pub fn set_view_shapes(
        &self,
        shapes: &[TdfLabel],
        gdts: &[TdfLabel],
        view: &TdfLabel,
    ) -> Result<(), Failure> {
        if !self.is_view(view)? {
            return Ok(());
        }
        let guids = [XCAFDoc::view_ref_shape_guid(), XCAFDoc::view_ref_gdt_guid()];
        guids.iter().try_for_each(|guid| detach(view, guid))?;
        link_all(view, shapes, &guids[0])?;
        link_all(view, gdts, &guids[1])
    }
    /// `SetClippingPlanes(theClippingPlaneLabels, theViewL)`.
    pub fn set_clipping_planes(&self, planes: &[TdfLabel], view: &TdfLabel) -> Result<(), Failure> {
        if !self.is_view(view)? {
            return Ok(());
        }
        let guid = XCAFDoc::view_ref_plane_guid();
        detach(view, &guid)?;
        link_all(view, planes, &guid)
    }
    /// `RemoveView(theViewL)`: unlinks the shapes, GD&T and clipping planes (whose emptied
    /// fathers forget their GD&T attribute) and forgets every attribute of the view.
    pub fn remove_view(&self, view: &TdfLabel) -> Result<(), Failure> {
        if !self.is_view(view)? {
            return Ok(());
        }
        let gdt = XCAFDoc::view_ref_gdt_guid();
        unlink_fathers(
            view,
            &XCAFDoc::view_ref_shape_guid(),
            &XCAFDoc::view_ref_shape_guid(),
        )?;
        unlink_fathers(view, &gdt, &gdt)?;
        unlink_fathers(view, &XCAFDoc::view_ref_plane_guid(), &gdt)?;
        view.forget_all_attributes(true)
    }
}

/// The references of the views.
impl XCAFDocViewTool {
    /// The father of the view's `ViewRefGUID` tree node, else the fathers of its graph node of
    /// the GUID; `None` without any.
    fn references(view: &TdfLabel, guid: &Guid) -> Result<Option<Vec<TdfLabel>>, Failure> {
        let node = find::<TDataStdTreeNode>(view, &XCAFDoc::view_ref_guid())?;
        if let Some(node) = node.get().filter(|node| node.has_father()) {
            return Ok(Some(vec![
                live(&node.father(), "XCAFDoc_ViewTool")?.label(),
            ]));
        }
        Ok(father_labels(view, guid)?.filter(|fathers| !fathers.is_empty()))
    }
    /// `GetRefShapeLabel(theViewL, theShapeLabels)`.
    pub fn get_ref_shape_label(&self, view: &TdfLabel) -> Result<Option<Vec<TdfLabel>>, Failure> {
        Self::references(view, &XCAFDoc::view_ref_shape_guid())
    }
    /// `GetRefGDTLabel(theViewL, theGDTLabels)`.
    pub fn get_ref_gdt_label(&self, view: &TdfLabel) -> Result<Option<Vec<TdfLabel>>, Failure> {
        Self::references(view, &XCAFDoc::view_ref_gdt_guid())
    }
    /// `GetRefClippingPlaneLabel(theViewL, theClippingPlaneLabels)`.
    pub fn get_ref_clipping_plane_label(
        &self,
        view: &TdfLabel,
    ) -> Result<Option<Vec<TdfLabel>>, Failure> {
        Self::references(view, &XCAFDoc::view_ref_plane_guid())
    }
    /// `GetRefNoteLabel(theViewL, theNoteLabels)`.
    pub fn get_ref_note_label(&self, view: &TdfLabel) -> Result<Option<Vec<TdfLabel>>, Failure> {
        Self::references(view, &XCAFDoc::view_ref_note_guid())
    }
    /// `GetRefAnnotationLabel(theViewL, theAnnotationLabels)`.
    pub fn get_ref_annotation_label(
        &self,
        view: &TdfLabel,
    ) -> Result<Option<Vec<TdfLabel>>, Failure> {
        Self::references(view, &XCAFDoc::view_ref_annotation_guid())
    }
    /// Appends the views (children of the item's node of the GUID); false without any.
    fn views_of(item: &TdfLabel, guid: &Guid, views: &mut Vec<TdfLabel>) -> Result<bool, Failure> {
        let Some(children) = child_labels_of(item, guid)?.filter(|c| !c.is_empty()) else {
            return Ok(false);
        };
        views.extend(children);
        Ok(true)
    }
    /// `GetViewLabelsForShape(theShapeL, theViews)`.
    pub fn get_view_labels_for_shape(
        &self,
        shape: &TdfLabel,
        views: &mut Vec<TdfLabel>,
    ) -> Result<bool, Failure> {
        Self::views_of(shape, &XCAFDoc::view_ref_shape_guid(), views)
    }
    /// `GetViewLabelsForGDT(theGDTL, theViews)`.
    pub fn get_view_labels_for_gdt(
        &self,
        gdt: &TdfLabel,
        views: &mut Vec<TdfLabel>,
    ) -> Result<bool, Failure> {
        Self::views_of(gdt, &XCAFDoc::view_ref_gdt_guid(), views)
    }
    /// `GetViewLabelsForClippingPlane(theClippingPlaneL, theViews)`.
    pub fn get_view_labels_for_clipping_plane(
        &self,
        plane: &TdfLabel,
        views: &mut Vec<TdfLabel>,
    ) -> Result<bool, Failure> {
        Self::views_of(plane, &XCAFDoc::view_ref_plane_guid(), views)
    }
    /// `GetViewLabelsForNote(theNoteL, theViews)`.
    pub fn get_view_labels_for_note(
        &self,
        note: &TdfLabel,
        views: &mut Vec<TdfLabel>,
    ) -> Result<bool, Failure> {
        Self::views_of(note, &XCAFDoc::view_ref_note_guid(), views)
    }
    /// `GetViewLabelsForAnnotation(theAnnotationL, theViews)`.
    pub fn get_view_labels_for_annotation(
        &self,
        annotation: &TdfLabel,
        views: &mut Vec<TdfLabel>,
    ) -> Result<bool, Failure> {
        Self::views_of(annotation, &XCAFDoc::view_ref_annotation_guid(), views)
    }
    /// `IsLocked(theViewL)`.
    pub fn is_locked(&self, view: &TdfLabel) -> Result<bool, Failure> {
        view.is_attribute(&XCAFDoc::lock_guid())
    }
    /// `Lock(theViewL)`.
    pub fn lock(&self, view: &TdfLabel) -> Result<(), Failure> {
        TDataStdUAttribute::set(view, &XCAFDoc::lock_guid())?;
        Ok(())
    }
    /// `Unlock(theViewL)`.
    pub fn unlock(&self, view: &TdfLabel) -> Result<(), Failure> {
        view.forget_attribute_id(&XCAFDoc::lock_guid())?;
        Ok(())
    }
}
