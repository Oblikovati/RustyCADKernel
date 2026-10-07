// Rust port of ApplicationFramework/TKLCAF/TDocStd/TDocStd_XLinkTool.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDocStd_XLinkTool`: copies a label (with its descendants and references) onto another
//! label, possibly of another document, and records the copy as an external link.
use super::{TDocStdDocument, TDocStdXLink};
use crate::live;
use crate::tdata_std::TDataStdTreeNode;
use crate::tdf::{
    TdfClosureMode, TdfClosureTool, TdfCopyTool, TdfDataSet, TdfIdFilter, TdfLabel, TdfReference,
    TdfRelocationTable, TdfTool,
};
use rustycad_tkcdf::cdm::{CdmDocument, CdmDocumentReferences};
use rustycad_tkernel::{
    standard::{Failure, Handle, exceptions},
    tcollection::AsciiString,
};

/// The links of a tree node detached for the copy, to reconnect it afterwards.
struct DetachedNode {
    node: Handle<TDataStdTreeNode>,
    father: Handle<TDataStdTreeNode>,
    previous: Handle<TDataStdTreeNode>,
    next: Handle<TDataStdTreeNode>,
}

impl DetachedNode {
    /// Finds the label's node of the default tree and removes it from its tree.
    fn detach(label: &TdfLabel) -> Result<Option<Self>, Failure> {
        let mut node = Handle::default();
        if !TDataStdTreeNode::find(label, &mut node)? {
            return Ok(None);
        }
        let current = live(&node, "TDocStd_XLinkTool::Copy")?;
        let (father, previous, next) = (current.father(), current.previous(), current.next());
        current.remove()?;
        Ok(Some(Self {
            node,
            father,
            previous,
            next,
        }))
    }
    /// Puts `node` back after its previous brother, before its next one, or under its father.
    fn reconnect(&self, node: &Handle<TDataStdTreeNode>) -> Result<(), Failure> {
        if let Some(previous) = self.previous.get() {
            previous.insert_after(node)?;
        } else if let Some(next) = self.next.get() {
            next.insert_before(node)?;
        } else if self.father.is_some() {
            live(node, "TDocStd_XLinkTool::Copy")?.set_father(&self.father)?;
        }
        Ok(())
    }
}

/// `TDocStd_XLinkTool`; `TDocStdXLinkTool::new().copy(&target, &source)`.
///
/// ```
/// use rustycad_tklcaf::{tdata_std::TDataStdInteger, tdoc_std::{TDocStdDocument, TDocStdXLinkTool}};
/// use rustycad_tkernel::tcollection::ExtendedString;
/// let document = TDocStdDocument::new(&ExtendedString::new("BinOcaf")).unwrap();
/// let source = document.main().unwrap().find_child(1, true).unwrap();
/// let target = document.main().unwrap().find_child(2, true).unwrap();
/// TDataStdInteger::set(&source, 7).unwrap();
/// let mut tool = TDocStdXLinkTool::new();
/// tool.copy(&target, &source).unwrap();
/// assert!(tool.is_done() && target.is_attribute(&TDataStdInteger::get_id()).unwrap());
/// ```
pub struct TDocStdXLinkTool {
    is_done: bool,
    data_set: Handle<TdfDataSet>,
    relocation_table: Handle<TdfRelocationTable>,
}

impl Default for TDocStdXLinkTool {
    /// `TDocStd_XLinkTool()`: not done, an empty relocation table, no data set.
    fn default() -> Self {
        Self {
            is_done: false,
            data_set: Handle::default(),
            relocation_table: Handle::new(TdfRelocationTable::new(false)),
        }
    }
}

impl TDocStdXLinkTool {
    /// `TDocStd_XLinkTool()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Copy(intarget, fromsource)` (virtual natively, never overridden): copies `source`
    /// with its descendants and references onto `target`; the tree nodes of both labels
    /// are detached during the copy and reconnected after it. Across documents,
    /// `Standard_DomainError` unless `source` is self-contained.
    pub fn copy(&mut self, target: &TdfLabel, source: &TdfLabel) -> Result<(), Failure> {
        if TDocStdDocument::get(target)? != TDocStdDocument::get(source)?
            && !TdfTool::is_self_contained(source)?
        {
            let message = format!("TDocStd_XLinkTool::Copy : not self-contained source {source:?}");
            return Err(exceptions::domain_error(&message));
        }
        let source_node = DetachedNode::detach(source)?;
        let target_node = DetachedNode::detach(target)?;
        self.copy_closure(target, source)?;
        if let Some(detached) = &source_node {
            detached.reconnect(&detached.node)?;
        }
        if let Some(detached) = &target_node {
            let mut node = Handle::default();
            if TDataStdTreeNode::find(target, &mut node)? {
                detached.reconnect(&node)?;
            }
        }
        self.is_done = true;
        Ok(())
    }
    /// The copy itself: a self-relocating table from `source` to `target`, the closure of
    /// `source` (descendants and references, every attribute) and `TDF_CopyTool::Copy`.
    fn copy_closure(&mut self, target: &TdfLabel, source: &TdfLabel) -> Result<(), Failure> {
        self.relocation_table = Handle::new(TdfRelocationTable::new(true));
        self.data_set = Handle::new(TdfDataSet::new());
        live(&self.data_set, "TDocStd_XLinkTool::Copy")?.add_label(source);
        live(&self.relocation_table, "TDocStd_XLinkTool::Copy")?
            .set_label_relocation(source, target);
        TdfClosureTool::closure_filtered(
            &self.data_set,
            &TdfIdFilter::new(false),
            &TdfClosureMode::new(true),
        )?;
        TdfCopyTool::copy(&self.data_set, &self.relocation_table)
    }
    /// `CopyWithLink(intarget, fromsource)`: `Copy`, then an XLink (label and document
    /// entries) and a `TDF_Reference` to `source` on `target`; `Standard_DomainError` when
    /// `target` already has a reference.
    pub fn copy_with_link(&mut self, target: &TdfLabel, source: &TdfLabel) -> Result<(), Failure> {
        if target.is_attribute(&TdfReference::get_id())? {
            let message = format!(" TDocStd_CopyWithLink : already a ref on {target:?}");
            return Err(exceptions::domain_error(&message));
        }
        self.copy(target, source)?;
        if !self.is_done {
            return Ok(());
        }
        let mut label_entry = AsciiString::default();
        TdfTool::entry(source, &mut label_entry);
        let document_entry = AsciiString::from_integer(document_entry(target, source)?);
        let link = TDocStdXLink::set(target)?;
        let link = live(&link, "TDocStd_XLinkTool::CopyWithLink")?;
        link.set_label_entry(&label_entry)?;
        link.set_document_entry(&document_entry)?;
        TdfReference::set(target, source)?;
        self.is_done = true;
        Ok(())
    }
    /// `UpdateLink(L)`: copies again the label the reference of `L` points to;
    /// `Standard_DomainError` without reference.
    pub fn update_link(&mut self, label: &TdfLabel) -> Result<(), Failure> {
        let mut reference: Handle<TdfReference> = Handle::default();
        if !label.find_attribute_as(&TdfReference::get_id(), &mut reference)? {
            let message =
                format!(" TDocStd_XLinkTool::UpdateLink : not ref registered on {label:?}");
            return Err(exceptions::domain_error(&message));
        }
        let source = live(&reference, "TDocStd_XLinkTool::UpdateLink")?.get();
        self.copy(label, &source)
    }
    /// `IsDone`.
    pub fn is_done(&self) -> bool {
        self.is_done
    }
    /// `RelocationTable`: the table of the last copy.
    pub fn relocation_table(&self) -> Handle<TdfRelocationTable> {
        self.relocation_table.clone()
    }
    /// `DataSet`: the data set of the last copy (null before any copy).
    pub fn data_set(&self) -> Handle<TdfDataSet> {
        self.data_set.clone()
    }
}

/// The document entry of `CopyWithLink`: 0 within one document, else the reference the
/// target document creates to the source one.
fn document_entry(target: &TdfLabel, source: &TdfLabel) -> Result<i32, Failure> {
    let (source_document, target_document) =
        (TDocStdDocument::get(source)?, TDocStdDocument::get(target)?);
    if source_document == target_document {
        return Ok(0);
    }
    let target_document = live(&target_document, "TDocStd_XLinkTool::CopyWithLink")?;
    let source_document: Handle<dyn CdmDocument> = source_document.cast();
    target_document.create_reference(&source_document)
}
