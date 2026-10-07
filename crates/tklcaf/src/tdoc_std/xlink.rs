// Rust port of ApplicationFramework/TKLCAF/TDocStd/TDocStd_XLink, TDocStd_XLinkRoot and
// TDocStd_XLinkIterator (.hxx, .lxx, .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! External links: `TDocStd_XLink` (a link to a label of another document), the list of
//! the links of a framework kept by `TDocStd_XLinkRoot`, and its iterator.
use super::TDocStdDocument;
use crate::guid_from_text;
use crate::live;
use crate::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeDelta, TdfAttributeState, TdfData,
    TdfDeltaOnAddition, TdfDeltaOnRemoval, TdfLabel, TdfReference, TdfRelocationTable, TdfTool,
};
use rustycad_tkcdf::cdm::CdmDocumentReferences;
use rustycad_tkernel::{
    impl_transient,
    standard::{
        Failure, Guid, Handle, RuntimeTyped, SharedState, StandardTransient, WeakHandle, exceptions,
    },
    tcollection::AsciiString,
};
use std::io;

/// `TDocStd_XLink::GetID()`.
const XLINK_ID: Guid = guid_from_text("5d587400-5690-11d1-8940-080009dc3333");
/// `TDocStd_XLinkRoot::GetID()`.
const XLINK_ROOT_ID: Guid = guid_from_text("5d587401-5690-11d1-8940-080009dc3333");

#[derive(Default)]
struct XLinkState {
    document_entry: AsciiString,
    label_entry: AsciiString,
    /// Native `TDocStd_XLink* myNext`.
    next: WeakHandle<TDocStdXLink>,
}

/// `TDocStd_XLink`; `TDocStdXLink::set(&label)`.
#[derive(Default)]
pub struct TDocStdXLink {
    transient: StandardTransient,
    base: TdfAttributeBase,
    state: SharedState<XLinkState>,
}
impl_transient!(TDocStdXLink, "TDocStd_XLink", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl TDocStdXLink {
    /// `GetID`.
    pub fn get_id() -> Guid {
        XLINK_ID
    }
    /// `TDocStd_XLink()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Set(atLabel)`: finds or adds the link.
    pub fn set(label: &TdfLabel) -> Result<Handle<TDocStdXLink>, Failure> {
        let mut found: Handle<TDocStdXLink> = Handle::default();
        if label.find_attribute_as(&XLINK_ID, &mut found)? {
            return Ok(found);
        }
        let created = Handle::new(Self::new());
        label.add_attribute(&created.cast(), true)?;
        Ok(created)
    }
    /// `Update`: sets a `TDF_Reference` on the link's label to the linked label of the
    /// referenced document.
    pub fn update(&self) -> Result<Handle<TdfReference>, Failure> {
        let entry = self.document_entry().integer_value()?;
        let own = TDocStdDocument::get(&self.label())?;
        let referenced: Handle<TDocStdDocument> =
            live(&own, "TDocStd_XLink::Update")?.document(entry)?.cast();
        let data = live(&referenced, "TDocStd_XLink::Update")?.get_data();
        let mut target = TdfLabel::default();
        TdfTool::label_from_entry(&data, &self.label_entry(), &mut target, false)?;
        TdfReference::set(&self.label(), &target)
    }
    /// `DocumentEntry(aDocEntry)`: backs up, then sets.
    pub fn set_document_entry(&self, entry: &AsciiString) -> Result<(), Failure> {
        self.backup()?;
        self.state.write().document_entry = entry.clone();
        Ok(())
    }
    /// `DocumentEntry()`.
    pub fn document_entry(&self) -> AsciiString {
        self.state.read().document_entry.clone()
    }
    /// `LabelEntry(aLabel)`: backs up, then the entry of the label.
    pub fn set_label_entry_of(&self, label: &TdfLabel) -> Result<(), Failure> {
        self.backup()?;
        TdfTool::entry(label, &mut self.state.write().label_entry);
        Ok(())
    }
    /// `LabelEntry(aLabEntry)`: backs up, then sets.
    pub fn set_label_entry(&self, entry: &AsciiString) -> Result<(), Failure> {
        self.backup()?;
        self.state.write().label_entry = entry.clone();
        Ok(())
    }
    /// `LabelEntry()`.
    pub fn label_entry(&self) -> AsciiString {
        self.state.read().label_entry.clone()
    }
    /// Private `Next(anXLinkPtr)`.
    fn set_next(&self, next: &WeakHandle<TDocStdXLink>) {
        self.state.write().next = next.clone();
    }
    /// Private `Next()`.
    fn next(&self) -> WeakHandle<TDocStdXLink> {
        self.state.read().next.clone()
    }
    fn this_link(&self) -> Result<Handle<TDocStdXLink>, Failure> {
        Ok(self.base.this()?.cast())
    }
}

/// Whether a delta is of the given type (`anAttDelta->IsKind(STANDARD_TYPE(T))`).
fn delta_is_kind<T: RuntimeTyped + ?Sized>(delta: &Handle<dyn TdfAttributeDelta>) -> bool {
    delta.is_kind(T::static_type())
}

impl TdfAttribute for TDocStdXLink {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        XLINK_ID
    }
    /// `AfterAddition`: joins the framework's link list and imports the label.
    fn after_addition(&self) -> Result<(), Failure> {
        TDocStdXLinkRoot::insert(&self.this_link()?)?;
        self.label().imported(true)
    }
    /// `BeforeRemoval`: leaves the list (unless a backup) and un-imports the label.
    fn before_removal(&self) -> Result<(), Failure> {
        if self.is_backuped() {
            return Ok(());
        }
        TDocStdXLinkRoot::remove(&self.this_link()?)?;
        self.label().imported(false)
    }
    /// `BeforeUndo`: an addition to undo removes the link from the list first.
    fn before_undo(
        &self,
        delta: &Handle<dyn TdfAttributeDelta>,
        _force_it: bool,
    ) -> Result<bool, Failure> {
        if delta_is_kind::<TdfDeltaOnAddition>(delta) {
            live(
                &live(delta, "TDocStd_XLink::BeforeUndo")?.attribute(),
                "TDocStd_XLink::BeforeUndo",
            )?
            .before_removal()?;
        }
        Ok(true)
    }
    /// `AfterUndo`: a removal undone puts the link back in the list.
    fn after_undo(
        &self,
        delta: &Handle<dyn TdfAttributeDelta>,
        _force_it: bool,
    ) -> Result<bool, Failure> {
        if delta_is_kind::<dyn TdfDeltaOnRemoval>(delta) {
            live(
                &live(delta, "TDocStd_XLink::AfterUndo")?.attribute(),
                "TDocStd_XLink::AfterUndo",
            )?
            .after_addition()?;
        }
        Ok(true)
    }
    /// `BackupCopy`: a new link with the same entries (set through backed-up setters on an
    /// unattached link, which do not back up).
    fn backup_copy(&self) -> Result<Handle<dyn TdfAttribute>, Failure> {
        let copy = Self::new();
        copy.set_document_entry(&self.document_entry())?;
        copy.set_label_entry(&self.label_entry())?;
        Ok(Handle::new(copy).cast())
    }
    /// `Restore`: the entries of another link (nothing for another type).
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let Some(other) = with
            .cast::<TDocStdXLink>()
            .get()
            .map(|o| (o.document_entry(), o.label_entry()))
        else {
            return Ok(());
        };
        let mut state = self.state.write();
        state.document_entry = other.0;
        state.label_entry = other.1;
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    /// `Paste`: the entries into another link (nothing for another type).
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = into.cast::<TDocStdXLink>();
        let Some(target) = target.get() else {
            return Ok(());
        };
        target.set_document_entry(&self.document_entry())?;
        target.set_label_entry(&self.label_entry())
    }
    /// `Dump`: writes nothing.
    fn dump(&self, _stream: &mut dyn io::Write) -> io::Result<()> {
        Ok(())
    }
}

/// `TDocStd_XLinkRoot`: the head of the link list of a framework, on its root.
#[derive(Default)]
pub struct TDocStdXLinkRoot {
    transient: StandardTransient,
    base: TdfAttributeBase,
    first: SharedState<WeakHandle<TDocStdXLink>>,
}
impl_transient!(TDocStdXLinkRoot, "TDocStd_XLinkRoot", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl TDocStdXLinkRoot {
    /// `GetID`.
    pub fn get_id() -> Guid {
        XLINK_ROOT_ID
    }
    /// `Set(aDF)`: finds or adds the root of the framework.
    pub fn set(data: &Handle<TdfData>) -> Result<Handle<TDocStdXLinkRoot>, Failure> {
        let root = live(data, "TDocStd_XLinkRoot::Set")?.root();
        let mut found: Handle<TDocStdXLinkRoot> = Handle::default();
        if root.find_attribute_as(&XLINK_ROOT_ID, &mut found)? {
            return Ok(found);
        }
        let created = Handle::new(Self::default());
        root.add_attribute(&created.cast(), true)?;
        Ok(created)
    }
    /// `Insert(anXLinkPtr)`: at the head of the list (the order is not significant).
    pub fn insert(link: &Handle<TDocStdXLink>) -> Result<(), Failure> {
        let link_ref = live(link, "TDocStd_XLinkRoot::Insert")?;
        let root = Self::set(&link_ref.label().data())?;
        let root = live(&root, "TDocStd_XLinkRoot::Insert")?;
        link_ref.set_next(&root.first.read().clone());
        *root.first.write() = link.downgrade();
        Ok(())
    }
    /// `Remove(anXLinkPtr)`: unlinks it from the list of its framework.
    pub fn remove(link: &Handle<TDocStdXLink>) -> Result<(), Failure> {
        let link_ref = live(link, "TDocStd_XLinkRoot::Remove")?;
        let mut root: Handle<TDocStdXLinkRoot> = Handle::default();
        if !link_ref
            .label()
            .root()?
            .find_attribute_as(&XLINK_ROOT_ID, &mut root)?
        {
            return Ok(());
        }
        let root = live(&root, "TDocStd_XLinkRoot::Remove")?;
        let first = root.first.read().upgrade();
        if first == *link {
            *root.first.write() = link_ref.next();
            link_ref.set_next(&WeakHandle::default());
            return Ok(());
        }
        let mut previous = first;
        while let Some(current) = previous
            .get()
            .filter(|current| current.next().upgrade() != *link)
        {
            previous = current.next().upgrade();
        }
        if let Some(previous) = previous.get() {
            previous.set_next(&link_ref.next());
            link_ref.set_next(&WeakHandle::default());
        }
        Ok(())
    }
    /// Private `First()`.
    fn first(&self) -> Handle<TDocStdXLink> {
        self.first.read().upgrade()
    }
}

impl TdfAttribute for TDocStdXLinkRoot {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        XLINK_ROOT_ID
    }
    /// `BackupCopy`: an empty root (does nothing).
    fn backup_copy(&self) -> Result<Handle<dyn TdfAttribute>, Failure> {
        Ok(Handle::new(Self::default()).cast())
    }
    /// `Restore`: does nothing.
    fn restore(&self, _with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::default()).cast()
    }
    /// `Paste`: does nothing.
    fn paste(
        &self,
        _into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        Ok(())
    }
    /// `Dump`: writes nothing.
    fn dump(&self, _stream: &mut dyn io::Write) -> io::Result<()> {
        Ok(())
    }
}

/// `TDocStd_XLinkIterator`: the links of a document's framework.
#[derive(Default)]
pub struct TDocStdXLinkIterator {
    value: Handle<TDocStdXLink>,
}
impl TDocStdXLinkIterator {
    /// `TDocStd_XLinkIterator()`: no link.
    pub fn empty() -> Self {
        Self::default()
    }
    /// `TDocStd_XLinkIterator(DOC)`.
    pub fn new(document: &Handle<TDocStdDocument>) -> Result<Self, Failure> {
        let mut iterator = Self::default();
        iterator.initialize(document)?;
        Ok(iterator)
    }
    /// `Initialize(DOC)`: restarts at the first link of the document's framework.
    pub fn initialize(&mut self, document: &Handle<TDocStdDocument>) -> Result<(), Failure> {
        self.value = Handle::default();
        let data = live(document, "TDocStd_XLinkIterator")?.get_data();
        let mut root: Handle<TDocStdXLinkRoot> = Handle::default();
        if live(&data, "TDocStd_XLinkIterator")?
            .root()
            .find_attribute_as(&XLINK_ROOT_ID, &mut root)?
        {
            self.value = live(&root, "TDocStd_XLinkIterator")?.first();
        }
        Ok(())
    }
    /// `More`.
    pub fn more(&self) -> bool {
        self.value.is_some()
    }
    /// `Next`: `Standard_NoMoreObject` past the end.
    #[allow(clippy::should_implement_trait)] // the native name Next, which can fail here
    pub fn next(&mut self) -> Result<(), Failure> {
        let Some(value) = self.value.get() else {
            return Err(exceptions::no_more_object(
                "TDocStd_XLinkIterator::Next() - no more values available",
            ));
        };
        self.value = value.next().upgrade();
        Ok(())
    }
    /// `Value`: the current link.
    pub fn value(&self) -> Handle<TDocStdXLink> {
        self.value.clone()
    }
}
