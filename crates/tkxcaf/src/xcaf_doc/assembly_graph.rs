// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_AssemblyGraph.hxx and .cxx and
// XCAFDoc_AssemblyTool.hxx (the graph traversal) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc_AssemblyGraph`: the "part-of" graph of the assemblies of a document: one node
//! per assembly, occurrence, part and sub-shape label (1-based, in discovery order), and
//! links from parents to children.
use super::support::{child_labels, find};
use super::{XCAFDoc, XCAFDocDocumentTool, XCAFDocShapeTool};
use rustycad_tkernel::ncollection::{DataMap, IndexedMap, PackedMap};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tklcaf::tdata_std::TDataStdTreeNode;
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel};
use rustycad_tklcaf::tdoc_std::TDocStdDocument;

/// `XCAFDoc_AssemblyGraph::NodeType`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(i32)]
pub enum XCAFDocAssemblyGraphNodeType {
    /// `NodeType_UNDEFINED`.
    #[default]
    Undefined = 0,
    /// `NodeType_AssemblyRoot`: a free assembly.
    AssemblyRoot,
    /// `NodeType_Subassembly`: an assembly used by another.
    Subassembly,
    /// `NodeType_Occurrence`: a component (reference).
    Occurrence,
    /// `NodeType_Part`: a simple shape.
    Part,
    /// `NodeType_Subshape`: a sub-shape.
    Subshape,
}

/// `XCAFDoc_AssemblyGraph::AdjacencyMap`: the children of each parent node.
pub type XCAFDocAssemblyGraphAdjacencyMap = DataMap<i32, PackedMap<i32>>;

/// The graph data.
#[derive(Default)]
struct GraphData {
    roots: PackedMap<i32>,
    nodes: IndexedMap<TdfLabel>,
    adjacency: XCAFDocAssemblyGraphAdjacencyMap,
    node_types: DataMap<i32, XCAFDocAssemblyGraphNodeType>,
    usages: DataMap<i32, i32>,
}

/// `XCAFDoc_AssemblyGraph`.
pub struct XCAFDocAssemblyGraph {
    transient: StandardTransient,
    shape_tool: Handle<XCAFDocShapeTool>,
    data: SharedState<GraphData>,
}
rustycad_tkernel::impl_transient!(
    XCAFDocAssemblyGraph,
    "XCAFDoc_AssemblyGraph",
    StandardTransient,
    transient
);

impl XCAFDocAssemblyGraph {
    /// `XCAFDoc_AssemblyGraph(theDoc)`: the graph of the free shapes of the document.
    pub fn new(document: &Handle<TDocStdDocument>) -> Result<Handle<Self>, Failure> {
        let main = rustycad_tklcaf::live(document, "Null document!")?.main()?;
        Self::build(
            XCAFDocDocumentTool::shape_tool(&main)?,
            &TdfLabel::default(),
        )
    }
    /// `XCAFDoc_AssemblyGraph(theLabel)`: the graph of one label (of the free shapes for the
    /// shape tool's label).
    pub fn with_label(label: &TdfLabel) -> Result<Handle<Self>, Failure> {
        if label.is_null() {
            return Err(Failure::with_message("Standard_NullObject", "Null label!"));
        }
        Self::build(XCAFDocDocumentTool::shape_tool(label)?, label)
    }
    /// The graph of the shape tool from a label.
    fn build(
        shape_tool: Handle<XCAFDocShapeTool>,
        label: &TdfLabel,
    ) -> Result<Handle<Self>, Failure> {
        if shape_tool.is_null() {
            return Err(Failure::with_message(
                "Standard_NoSuchObject",
                "No XCAFDoc_ShapeTool attribute!",
            ));
        }
        let graph = Self {
            transient: StandardTransient::default(),
            shape_tool,
            data: SharedState::default(),
        };
        graph.build_graph(label)?;
        Ok(Handle::new(graph))
    }
    /// `GetShapeTool`.
    pub fn get_shape_tool(&self) -> &Handle<XCAFDocShapeTool> {
        &self.shape_tool
    }
    /// `GetRoots`: the IDs of the root nodes.
    pub fn get_roots(&self) -> PackedMap<i32> {
        self.data.read().roots.clone()
    }
    /// `IsDirectLink(theNode1, theNode2)`: the second node is a child of the first.
    pub fn is_direct_link(&self, node1: i32, node2: i32) -> bool {
        self.data
            .read()
            .adjacency
            .seek(&node1)
            .is_some_and(|children| children.contains(node2))
    }
    /// `HasChildren(theNode)`.
    pub fn has_children(&self, node: i32) -> bool {
        self.data.read().adjacency.is_bound(&node)
    }
    /// `GetChildren(theNode)` (`Standard_NoSuchObject` for a node without children).
    pub fn get_children(&self, node: i32) -> Result<PackedMap<i32>, Failure> {
        self.data.read().adjacency.find(&node).cloned()
    }
    /// `GetNodeType(theNode)`: `Undefined` for an unknown node.
    pub fn get_node_type(&self, node: i32) -> XCAFDocAssemblyGraphNodeType {
        self.data
            .read()
            .node_types
            .seek(&node)
            .copied()
            .unwrap_or_default()
    }
    /// `GetNode(theNode)`: the label of a node (`Standard_OutOfRange` outside the graph).
    pub fn get_node(&self, node: i32) -> Result<TdfLabel, Failure> {
        self.data.read().nodes.find_key_int(node).cloned()
    }
    /// `GetNodes`: the labels of the nodes, in ID order.
    pub fn get_nodes(&self) -> Result<Vec<TdfLabel>, Failure> {
        let data = self.data.read();
        (1..=data.nodes.extent())
            .map(|index| data.nodes.find_key(index).cloned())
            .collect()
    }
    /// `NbNodes`.
    pub fn nb_nodes(&self) -> i32 {
        self.data.read().nodes.extent() as i32
    }
    /// `GetLinks`: the children of each parent node (a copy).
    pub fn get_links(&self) -> XCAFDocAssemblyGraphAdjacencyMap {
        let mut links = XCAFDocAssemblyGraphAdjacencyMap::default();
        links.assign(&self.data.read().adjacency);
        links
    }
    /// `NbLinks`: the number of parent-child links.
    pub fn nb_links(&self) -> i32 {
        self.data
            .read()
            .adjacency
            .items()
            .map(|(_, children)| children.extent())
            .sum()
    }
    /// `NbOccurrences(theNode)`: how many times a non-occurrence node is used.
    pub fn nb_occurrences(&self, node: i32) -> i32 {
        self.data.read().usages.seek(&node).copied().unwrap_or(0)
    }
    /// `XCAFDoc_AssemblyTool::Traverse(theGraph, theFilter, theFunc, theNode)`: the nodes from
    /// `start` passing the filter, until the function returns false.
    pub fn traverse(
        &self,
        start: i32,
        mut filter: impl FnMut(&Self, i32) -> bool,
        mut function: impl FnMut(&Self, i32) -> bool,
    ) -> Result<(), Failure> {
        let mut iterator = XCAFDocAssemblyGraphIterator::new(self, start)?;
        while iterator.more() {
            let node = iterator.current();
            if filter(self, node) && !function(self, node) {
                break;
            }
            iterator.next();
        }
        Ok(())
    }
}

/// `XCAFDoc_AssemblyGraph::Iterator`: the node IDs of a graph from a start node.
///
/// ```
/// use rustycad_tklcaf::tdoc_std::{TDocStdApplication, TDocStdDocument, TDocStdStandardApplication};
/// use rustycad_tkernel::{standard::Handle, tcollection::ExtendedString};
/// use rustycad_tkxcaf::xcaf_doc::{XCAFDocAssemblyGraph, XCAFDocAssemblyGraphIterator, XCAFDocDocumentTool};
/// let application = TDocStdStandardApplication::new();
/// let mut document: Handle<TDocStdDocument> = Handle::default();
/// application.new_tdoc_std_document(&ExtendedString::from_utf8("BinXCAF"), &mut document)?;
/// XCAFDocDocumentTool::set(&document.main()?, true)?;
/// let graph = XCAFDocAssemblyGraph::new(&document)?;
/// assert!(!XCAFDocAssemblyGraphIterator::new(graph.get().unwrap(), 1)?.more());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct XCAFDocAssemblyGraphIterator<'a> {
    graph: &'a XCAFDocAssemblyGraph,
    current: i32,
}

impl<'a> XCAFDocAssemblyGraphIterator<'a> {
    /// `Iterator(theGraph, theNode)`: from a 1-based node (`Standard_NullObject` below 1;
    /// the native default start is 1).
    pub fn new(graph: &'a XCAFDocAssemblyGraph, node: i32) -> Result<Self, Failure> {
        if node < 1 {
            return Err(Failure::with_message(
                "Standard_NullObject",
                "Node ID must be positive one-based integer!",
            ));
        }
        Ok(Self {
            graph,
            current: node,
        })
    }
    /// `More`: the current ID is a node of the graph.
    pub fn more(&self) -> bool {
        self.current <= self.graph.nb_nodes()
    }
    /// `Current`.
    pub fn current(&self) -> i32 {
        self.current
    }
    /// `Next`.
    #[allow(clippy::should_implement_trait)] // OCCT's More/Next protocol.
    pub fn next(&mut self) {
        self.current += 1;
    }
}

/// The building of the graph.
impl XCAFDocAssemblyGraph {
    /// `buildGraph(theLabel)`: the nodes of the roots (the referred shape under its
    /// reference) and of their components.
    fn build_graph(&self, label: &TdfLabel) -> Result<(), Failure> {
        let tool = rustycad_tklcaf::live(&self.shape_tool, "XCAFDoc_AssemblyGraph")?;
        let roots = if label.is_null() || tool.label() == *label {
            tool.get_free_shapes()?
        } else {
            vec![label.clone()]
        };
        for root in roots {
            let mut original = TdfLabel::default();
            let (root_id, proceed_id) =
                if XCAFDocShapeTool::get_referred_shape(&root, &mut original)? {
                    let root_id = self.add_node(&root, 0)?;
                    if root_id == 0 {
                        continue;
                    }
                    (root_id, self.add_node(&original, root_id)?)
                } else {
                    original = root.clone();
                    let root_id = self.add_node(&original, 0)?;
                    (root_id, root_id)
                };
            if root_id == 0 || proceed_id == 0 {
                continue;
            }
            self.data.write().roots.add(root_id);
            if XCAFDocShapeTool::is_assembly(&original)? {
                self.add_components(&original, proceed_id)?;
            }
        }
        Ok(())
    }
    /// `addComponents(theParent, theParentId)`: the children of a shape label and, through
    /// their shape references, their originals and components.
    fn add_components(&self, parent: &TdfLabel, parent_id: i32) -> Result<(), Failure> {
        if !XCAFDocShapeTool::is_shape(parent)? {
            return Ok(()); // We have to return here in order to prevent iterating by sub-labels.
        }
        for component in child_labels(parent)? {
            let component_id = self.add_node(&component, parent_id)?;
            if component_id == 0 {
                continue;
            }
            let jump = find::<TDataStdTreeNode>(&component, &XCAFDoc::shape_ref_guid())?;
            let Some(jump) = jump.get().filter(|node| node.has_father()) else {
                continue;
            };
            let original = rustycad_tklcaf::live(&jump.father(), "XCAFDoc_AssemblyGraph")?.label();
            if original.is_null() {
                continue;
            }
            let child_id = self.add_node(&original, component_id)?;
            if child_id != 0 {
                self.add_components(&original, child_id)?;
            }
        }
        Ok(())
    }
    /// `addNode(theLabel, theParentId)`: the node of a label (0 for an undefined type), its
    /// usage count and its link from the parent.
    fn add_node(&self, label: &TdfLabel, parent_id: i32) -> Result<i32, Failure> {
        let kind = node_type(label)?;
        if kind == XCAFDocAssemblyGraphNodeType::Undefined {
            return Ok(0);
        }
        let mut data = self.data.write();
        let id = data.nodes.add(label.clone()) as i32;
        data.node_types.bind(id, kind);
        if kind != XCAFDocAssemblyGraphNodeType::Occurrence {
            let usages = data.usages.seek(&id).copied().unwrap_or(0);
            data.usages.bind(id, usages + 1);
        }
        if parent_id > 0 {
            if !data.adjacency.is_bound(&parent_id) {
                data.adjacency.bind(parent_id, PackedMap::default());
            }
            data.adjacency.change_find(&parent_id)?.add(id);
        }
        Ok(id)
    }
}

/// The node type of a label: an assembly (root when free), an occurrence, a sub-shape or a
/// part, else undefined.
fn node_type(label: &TdfLabel) -> Result<XCAFDocAssemblyGraphNodeType, Failure> {
    use XCAFDocAssemblyGraphNodeType as T;
    Ok(if XCAFDocShapeTool::is_assembly(label)? {
        if XCAFDocShapeTool::is_free(label)? {
            T::AssemblyRoot
        } else {
            T::Subassembly
        }
    } else if XCAFDocShapeTool::is_reference(label)? {
        T::Occurrence
    } else if XCAFDocShapeTool::is_sub_shape_label(label)? {
        T::Subshape
    } else if XCAFDocShapeTool::is_simple_shape(label)? {
        T::Part
    } else {
        T::Undefined
    })
}
