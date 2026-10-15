// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_GraphNode.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc_GraphNode`: a node of a directed graph between labels (layers, SHUO, GD&T and
//! view links), with its fathers and children in order.
use super::support::{downcast, live};
use rustycad_tkcdf::json_dump::field_pointer;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Guid, Handle, SharedState, StandardTransient};
use rustycad_tklcaf::guid_from_text;
use rustycad_tklcaf::json::dump_attribute_json_base;
use rustycad_tklcaf::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfDataSet, TdfLabel, TdfRelocationTable,
    dump_attribute,
};
use rustycad_tkservice::json_dump::{class_begin, field_dumped};
use std::io;

/// `XCAFDoc_GraphNode::GetDefaultGraphID()`.
const DEFAULT_GRAPH_ID: Guid = guid_from_text("efd212f5-6dfd-11d4-b9c8-0060b0ee281b");

/// The fields of `XCAFDoc_GraphNode`. The links are strong handles, as natively (a father
/// and its child hold each other).
#[derive(Clone, Default)]
struct GraphFields {
    fathers: Vec<Handle<XCAFDocGraphNode>>,
    children: Vec<Handle<XCAFDocGraphNode>>,
    graph_id: Guid,
}

/// `XCAFDoc_GraphNode`.
///
/// ```
/// use rustycad_tklcaf::tdf::TdfData;
/// use rustycad_tkxcaf::xcaf_doc::XCAFDocGraphNode;
/// let data = TdfData::new();
/// let (a, b) = (data.root().new_child()?, data.root().new_child()?);
/// let father = XCAFDocGraphNode::set(&a)?;
/// let child = XCAFDocGraphNode::set(&b)?;
/// father.set_child(&child)?;
/// child.set_father(&father)?;
/// assert!(father.is_father(&child) && child.is_child(&father));
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct XCAFDocGraphNode {
    transient: StandardTransient,
    base: TdfAttributeBase,
    fields: SharedState<GraphFields>,
}
rustycad_tkernel::impl_transient!(XCAFDocGraphNode, "XCAFDoc_GraphNode", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl XCAFDocGraphNode {
    /// `XCAFDoc_GraphNode()`: a null graph ID.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GetDefaultGraphID`.
    pub fn get_default_graph_id() -> Guid {
        DEFAULT_GRAPH_ID
    }
    /// `Find(L, G)`: the node of the default graph of the label.
    pub fn find(label: &TdfLabel, node: &mut Handle<XCAFDocGraphNode>) -> Result<bool, Failure> {
        label.find_attribute_as(&DEFAULT_GRAPH_ID, node)
    }
    /// `Set(L)`: the node of the default graph, added when absent.
    pub fn set(label: &TdfLabel) -> Result<Handle<XCAFDocGraphNode>, Failure> {
        Self::set_with_id(label, &DEFAULT_GRAPH_ID)
    }
    /// `Set(L, explicitID)`: the node of that graph, added when absent.
    pub fn set_with_id(label: &TdfLabel, id: &Guid) -> Result<Handle<XCAFDocGraphNode>, Failure> {
        let mut node: Handle<XCAFDocGraphNode> = Handle::default();
        if label.find_attribute_as(id, &mut node)? {
            return Ok(node);
        }
        let node = Handle::new(Self::new());
        live(&node, "XCAFDoc_GraphNode::Set")?.set_graph_id(id)?;
        label.add_attribute(&node.clone().cast(), false)?;
        Ok(node)
    }
    /// `SetGraphID(explicitID)`: backed up.
    pub fn set_graph_id(&self, id: &Guid) -> Result<(), Failure> {
        self.backup()?;
        self.fields.write().graph_id = *id;
        Ok(())
    }
    /// `SetFather(F)`: appends a father; returns its index.
    pub fn set_father(&self, father: &Handle<XCAFDocGraphNode>) -> Result<i32, Failure> {
        self.backup()?;
        let mut fields = self.fields.write();
        fields.fathers.push(father.clone());
        Ok(fields.fathers.len() as i32)
    }
    /// `SetChild(Ch)`: appends a child; returns its index.
    pub fn set_child(&self, child: &Handle<XCAFDocGraphNode>) -> Result<i32, Failure> {
        self.backup()?;
        let mut fields = self.fields.write();
        fields.children.push(child.clone());
        Ok(fields.children.len() as i32)
    }
    /// This node's own handle (the native `this`).
    fn this(&self) -> Result<Handle<XCAFDocGraphNode>, Failure> {
        Ok(self.attribute_base().this()?.cast::<XCAFDocGraphNode>())
    }
    /// `UnSetFather(F)`: removes the link both ways when `F` is a father.
    pub fn unset_father(&self, father: &Handle<XCAFDocGraphNode>) -> Result<(), Failure> {
        if self.father_index(father) == 0 {
            return Ok(());
        }
        live(father, "XCAFDoc_GraphNode::UnSetFather")?.unset_child_link(&self.this()?)?;
        self.unset_father_link(father)
    }
    /// `UnSetFather(Findex)`: the father of that index (0 does nothing).
    pub fn unset_father_index(&self, index: i32) -> Result<(), Failure> {
        if index == 0 {
            return Ok(());
        }
        self.unset_father(&self.get_father(index)?)
    }
    /// `UnSetFatherlink(F)`: backed up; removes `F` from the fathers only.
    pub fn unset_father_link(&self, father: &Handle<XCAFDocGraphNode>) -> Result<(), Failure> {
        self.backup()?;
        let index = self.father_index(father);
        if index != 0 {
            self.fields.write().fathers.remove(index as usize - 1);
        }
        Ok(())
    }
    /// `UnSetChild(Ch)`: removes the link both ways when `Ch` is a child.
    pub fn unset_child(&self, child: &Handle<XCAFDocGraphNode>) -> Result<(), Failure> {
        if self.child_index(child) == 0 {
            return Ok(());
        }
        live(child, "XCAFDoc_GraphNode::UnSetChild")?.unset_father_link(&self.this()?)?;
        self.unset_child_link(child)
    }
    /// `UnSetChild(Chindex)`: the child of that index (0 does nothing).
    pub fn unset_child_index(&self, index: i32) -> Result<(), Failure> {
        if index == 0 {
            return Ok(());
        }
        self.unset_child(&self.get_child(index)?)
    }
    /// `UnSetChildlink(Ch)`: backed up; removes `Ch` from the children only.
    pub fn unset_child_link(&self, child: &Handle<XCAFDocGraphNode>) -> Result<(), Failure> {
        self.backup()?;
        let index = self.child_index(child);
        if index != 0 {
            self.fields.write().children.remove(index as usize - 1);
        }
        Ok(())
    }
}

/// The 1-based item of a sequence, `Standard_OutOfRange` outside it.
fn sequence_value(
    items: &[Handle<XCAFDocGraphNode>],
    index: i32,
    who: &str,
) -> Result<Handle<XCAFDocGraphNode>, Failure> {
    usize::try_from(index - 1)
        .ok()
        .and_then(|position| items.get(position).cloned())
        .ok_or_else(|| {
            rustycad_tkernel::standard::exceptions::out_of_range(&format!(
                "{who}: index {index}; expected 1..={}",
                items.len()
            ))
        })
}

/// The 1-based index of a node in a sequence, 0 when absent.
fn index_of(items: &[Handle<XCAFDocGraphNode>], node: &Handle<XCAFDocGraphNode>) -> i32 {
    items
        .iter()
        .position(|item| item.object_address() == node.object_address())
        .map_or(0, |position| position as i32 + 1)
}

/// The queries.
impl XCAFDocGraphNode {
    /// `GetFather(Findex)`.
    pub fn get_father(&self, index: i32) -> Result<Handle<XCAFDocGraphNode>, Failure> {
        sequence_value(
            &self.fields.read().fathers,
            index,
            "XCAFDoc_GraphNode::GetFather",
        )
    }
    /// `GetChild(Chindex)`.
    pub fn get_child(&self, index: i32) -> Result<Handle<XCAFDocGraphNode>, Failure> {
        sequence_value(
            &self.fields.read().children,
            index,
            "XCAFDoc_GraphNode::GetChild",
        )
    }
    /// `FatherIndex(F)`: 0 when `F` is not a father.
    pub fn father_index(&self, father: &Handle<XCAFDocGraphNode>) -> i32 {
        index_of(&self.fields.read().fathers, father)
    }
    /// `ChildIndex(Ch)`: 0 when `Ch` is not a child.
    pub fn child_index(&self, child: &Handle<XCAFDocGraphNode>) -> i32 {
        index_of(&self.fields.read().children, child)
    }
    /// `IsFather(Ch)`: `Ch` is a child of this node.
    pub fn is_father(&self, child: &Handle<XCAFDocGraphNode>) -> bool {
        self.child_index(child) != 0
    }
    /// `IsChild(F)`: `F` is a father of this node.
    pub fn is_child(&self, father: &Handle<XCAFDocGraphNode>) -> bool {
        self.father_index(father) != 0
    }
    /// `NbFathers`.
    pub fn nb_fathers(&self) -> i32 {
        self.fields.read().fathers.len() as i32
    }
    /// `NbChildren`.
    pub fn nb_children(&self) -> i32 {
        self.fields.read().children.len() as i32
    }
    /// The fathers in order.
    pub fn fathers(&self) -> Vec<Handle<XCAFDocGraphNode>> {
        self.fields.read().fathers.clone()
    }
    /// The children in order.
    pub fn children(&self) -> Vec<Handle<XCAFDocGraphNode>> {
        self.fields.read().children.clone()
    }
}

/// `RT->HasRelocation(node, func)` then `if (!found && RT->AfterRelocate()) func.Nullify()`.
fn relocated(
    table: &Handle<TdfRelocationTable>,
    node: &Handle<XCAFDocGraphNode>,
) -> Result<Handle<XCAFDocGraphNode>, Failure> {
    let table = live(table, "XCAFDoc_GraphNode::Paste")?;
    let mut target: Handle<XCAFDocGraphNode> = Handle::default();
    let found = table.has_attribute_relocation_as(&node.clone().cast(), &mut target);
    if !found && table.after_relocate() {
        return Ok(Handle::default());
    }
    Ok(target)
}

impl TdfAttribute for XCAFDocGraphNode {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    /// `ID`: the graph ID.
    fn id(&self) -> Guid {
        self.fields.read().graph_id
    }
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = downcast::<XCAFDocGraphNode>(with, "XCAFDoc_GraphNode::Restore")?;
        let other = live(&other, "XCAFDoc_GraphNode::Restore")?
            .fields
            .read()
            .clone();
        *self.fields.write() = other;
        Ok(())
    }
    /// `NewEmpty`: a new node of the same graph.
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        let node = Self::new();
        node.fields.write().graph_id = self.id();
        Handle::new(node).cast()
    }
    /// `Paste`: the relocated fathers and children are appended to the target's, then the
    /// graph ID.
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = downcast::<XCAFDocGraphNode>(into, "XCAFDoc_GraphNode::Paste")?;
        let target = live(&target, "XCAFDoc_GraphNode::Paste")?;
        for father in self.fathers() {
            let moved = relocated(table, &father)?;
            if !moved.is_null() {
                target.set_father(&moved)?;
            }
        }
        for child in self.children() {
            let moved = relocated(table, &child)?;
            if !moved.is_null() {
                target.set_child(&moved)?;
            }
        }
        target.set_graph_id(&self.id())
    }
    /// `References`: the children, then the fathers.
    fn references(&self, data_set: &Handle<TdfDataSet>) {
        let Some(data_set) = data_set.get() else {
            return;
        };
        for node in self.children().into_iter().chain(self.fathers()) {
            if !node.is_null() {
                data_set.add_attribute(&node.cast());
            }
        }
    }
    /// `BeforeForget`: unlinks every father, then every child.
    fn before_forget(&self) -> Result<(), Failure> {
        while self.nb_fathers() > 0 {
            self.unset_father_index(1)?;
        }
        while self.nb_children() > 0 {
            self.unset_child_index(1)?;
        }
        Ok(())
    }
    /// `Dump`: the attribute line, then the entries of the fathers and of the children.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        dump_attribute(self, stream)?;
        dump_links(stream, "  Fathers=", &self.fathers())?;
        dump_links(stream, "  Children=", &self.children())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "XCAFDoc_GraphNode")?;
        dump_attribute_json_base(self, stream, depth)?;
        for father in self.fathers() {
            field_pointer(stream, "aFather", father.object_address())?;
        }
        for child in self.children() {
            let Some(node) = child.get() else {
                continue;
            };
            field_dumped(stream, depth, "aChild.get()", &|nested, d| {
                node.dump_json(nested, d)
            })?;
        }
        Ok(())
    }
}

/// One link list of `Dump`: the title, then each entry and a newline.
fn dump_links(
    stream: &mut dyn io::Write,
    title: &str,
    nodes: &[Handle<XCAFDocGraphNode>],
) -> io::Result<()> {
    if nodes.is_empty() {
        return Ok(());
    }
    write!(stream, "{title}")?;
    for node in nodes.iter().filter_map(Handle::get) {
        let label = node.label();
        if !label.is_null() {
            label.entry_dump(stream)?;
        }
        writeln!(stream)?;
    }
    Ok(())
}
