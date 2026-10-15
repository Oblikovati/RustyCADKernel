// Rust port of DataExchange/TKXCAF/XCAFPrs/XCAFPrs_DocumentExplorer.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFPrs_DocumentExplorer`: a depth-first walk over the assembly tree of an XDE document
//! (assemblies, then their components), with the merged styles and global locations of the
//! nodes and their path identifiers.
use super::{XCAFPrsDocumentIdIterator, XCAFPrsDocumentNode, XCAFPrsStyle};
use crate::xcaf_doc::{
    XCAFDocColorTool, XCAFDocColorType, XCAFDocDocumentTool, XCAFDocShapeTool,
    XCAFDocVisMaterialTool,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tklcaf::tdf::{TdfChildIterator, TdfLabel, TdfTool};
use rustycad_tklcaf::tdoc_std::TDocStdDocument;
use rustycad_tkmath::toploc::Location;

/// `XCAFPrs_DocumentExplorerFlags`: the iteration flags.
pub type XCAFPrsDocumentExplorerFlags = i32;
/// `XCAFPrs_DocumentExplorerFlags_None`: no flags.
pub const XCAF_PRS_DOCUMENT_EXPLORER_FLAGS_NONE: XCAFPrsDocumentExplorerFlags = 0x00;
/// `XCAFPrs_DocumentExplorerFlags_OnlyLeafNodes`: skip the assembly nodes.
pub const XCAF_PRS_DOCUMENT_EXPLORER_FLAGS_ONLY_LEAF_NODES: XCAFPrsDocumentExplorerFlags = 0x01;
/// `XCAFPrs_DocumentExplorerFlags_NoStyle`: do not fetch the styles.
pub const XCAF_PRS_DOCUMENT_EXPLORER_FLAGS_NO_STYLE: XCAFPrsDocumentExplorerFlags = 0x02;

/// The style of a label merged over its parent's: the material and colors of the referred
/// label, then of the label itself.
fn merged_style(
    styled: bool,
    parent: &XCAFPrsStyle,
    label: &TdfLabel,
    ref_label: &TdfLabel,
) -> Result<XCAFPrsStyle, Failure> {
    if !styled {
        return Ok(parent.clone());
    }
    let mut style = parent.clone();
    apply_label_style(&mut style, ref_label)?;
    if label != ref_label {
        apply_label_style(&mut style, label)?;
    }
    Ok(style)
}

/// The material and the generic, surface and curve colors of one label over a style.
fn apply_label_style(style: &mut XCAFPrsStyle, label: &TdfLabel) -> Result<(), Failure> {
    let material = XCAFDocVisMaterialTool::get_shape_material(label)?;
    if !material.is_null() {
        style.set_material(&material);
    }
    if let Some(color) = XCAFDocColorTool::get_label_color(label, XCAFDocColorType::ColorGen)? {
        style.set_color_curv(*color.rgb());
        style.set_color_surf_rgba(color);
    }
    if let Some(color) = XCAFDocColorTool::get_label_color(label, XCAFDocColorType::ColorSurf)? {
        style.set_color_surf_rgba(color);
    }
    if let Some(color) = XCAFDocColorTool::get_label_color(label, XCAFDocColorType::ColorCurv)? {
        style.set_color_curv(*color.rgb());
    }
    Ok(())
}

/// The referred label of a label, else the label itself.
fn referred(label: &TdfLabel) -> Result<TdfLabel, Failure> {
    let mut ref_label = label.clone();
    XCAFDocShapeTool::get_referred_shape(label, &mut ref_label)?;
    Ok(ref_label)
}

/// `XCAFPrs_DocumentExplorer`.
#[derive(Default)]
pub struct XCAFPrsDocumentExplorer {
    color_tool: Handle<XCAFDocColorTool>,
    vis_mat_tool: Handle<XCAFDocVisMaterialTool>,
    roots: Vec<TdfLabel>,
    root_index: usize,
    node_stack: Vec<XCAFPrsDocumentNode>,
    top: i32,
    has_more: bool,
    def_style: XCAFPrsStyle,
    current: XCAFPrsDocumentNode,
    flags: XCAFPrsDocumentExplorerFlags,
}

/// The path identifiers.
impl XCAFPrsDocumentExplorer {
    /// `DefineChildId(theLabel, theParentId)`: `<parent>/<entry>.`, or `<entry>.` at the root.
    pub fn define_child_id(label: &TdfLabel, parent_id: &AsciiString) -> AsciiString {
        let mut entry = AsciiString::default();
        TdfTool::entry(label, &mut entry);
        let mut id = AsciiString::default();
        if !parent_id.is_empty() {
            id.assign_cat(parent_id.as_bytes());
            id.assign_cat("/");
        }
        id.assign_cat(entry.as_bytes());
        id.assign_cat(".");
        id
    }
    /// `FindLabelFromPathId(theDocument, theId, theParentLocation, theLocation)`: the label of
    /// the last entry of the path, with the product of the locations of the entries (and of
    /// all but the last); null for a missing entry.
    pub fn find_label_from_path_id(
        document: &Handle<TDocStdDocument>,
        id: &AsciiString,
        parent_location: &mut Location,
        location: &mut Location,
    ) -> Result<TdfLabel, Failure> {
        *parent_location = Location::default();
        *location = Location::default();
        let data = rustycad_tklcaf::live(document, "XCAFPrs_DocumentExplorer")?
            .main()?
            .data();
        let mut instance = TdfLabel::default();
        let mut iterator = XCAFPrsDocumentIdIterator::new(id)?;
        while iterator.more() {
            let mut sub_label = TdfLabel::default();
            TdfTool::label_from_entry(&data, iterator.value(), &mut sub_label, false)?;
            if sub_label.is_null() {
                return Ok(TdfLabel::default());
            }
            iterator.next()?;
            if !iterator.more() {
                *parent_location = location.clone();
            }
            *location = location.multiplied(&XCAFDocShapeTool::get_location(&sub_label)?)?;
            instance = sub_label;
        }
        Ok(instance)
    }
    /// `FindShapeFromPathId(theDocument, theId)`: the referred shape of the path's label at
    /// the path's location, null when missing.
    pub fn find_shape_from_path_id(
        document: &Handle<TDocStdDocument>,
        id: &AsciiString,
    ) -> Result<Shape, Failure> {
        let (mut parent, mut location) = (Location::default(), Location::default());
        let instance = Self::find_label_from_path_id(document, id, &mut parent, &mut location)?;
        if instance.is_null() {
            return Ok(Shape::default());
        }
        let ref_label = referred(&instance)?;
        let mut shape = XCAFDocShapeTool::shape_of(&ref_label)?;
        if shape.is_null() {
            return Ok(shape);
        }
        shape.set_location(&location, true)?;
        Ok(shape)
    }
}

impl XCAFPrsDocumentExplorer {
    /// `XCAFPrs_DocumentExplorer()`: no roots.
    pub fn new() -> Self {
        Self {
            top: -1,
            ..Default::default()
        }
    }
    /// `XCAFPrs_DocumentExplorer(theDocument, theFlags, theDefStyle)`: the free shapes of the
    /// document as roots.
    pub fn with_document(
        document: &Handle<TDocStdDocument>,
        flags: XCAFPrsDocumentExplorerFlags,
        def_style: &XCAFPrsStyle,
    ) -> Result<Self, Failure> {
        let main = rustycad_tklcaf::live(document, "XCAFPrs_DocumentExplorer")?.main()?;
        let shapes = XCAFDocDocumentTool::shape_tool(&main)?;
        let roots =
            rustycad_tklcaf::live(&shapes, "XCAFPrs_DocumentExplorer")?.get_free_shapes()?;
        Self::with_roots(document, &roots, flags, def_style)
    }
    /// `XCAFPrs_DocumentExplorer(theDocument, theRoots, theFlags, theDefStyle)`.
    pub fn with_roots(
        document: &Handle<TDocStdDocument>,
        roots: &[TdfLabel],
        flags: XCAFPrsDocumentExplorerFlags,
        def_style: &XCAFPrsStyle,
    ) -> Result<Self, Failure> {
        let mut explorer = Self::new();
        explorer.init_roots(document, roots, flags, def_style)?;
        Ok(explorer)
    }
    /// `Init(theDocument, theRoot, theFlags, theDefStyle)`.
    pub fn init(
        &mut self,
        document: &Handle<TDocStdDocument>,
        root: &TdfLabel,
        flags: XCAFPrsDocumentExplorerFlags,
        def_style: &XCAFPrsStyle,
    ) -> Result<(), Failure> {
        self.init_roots(document, std::slice::from_ref(root), flags, def_style)
    }
    /// `Init(theDocument, theRoots, theFlags, theDefStyle)`: the color and material tools
    /// unless `NoStyle`, then the first node.
    pub fn init_roots(
        &mut self,
        document: &Handle<TDocStdDocument>,
        roots: &[TdfLabel],
        flags: XCAFPrsDocumentExplorerFlags,
        def_style: &XCAFPrsStyle,
    ) -> Result<(), Failure> {
        if flags & XCAF_PRS_DOCUMENT_EXPLORER_FLAGS_NO_STYLE == 0 {
            let main = rustycad_tklcaf::live(document, "XCAFPrs_DocumentExplorer")?.main()?;
            self.color_tool = XCAFDocDocumentTool::color_tool(&main)?;
            self.vis_mat_tool = XCAFDocDocumentTool::vis_material_tool(&main)?;
        } else {
            self.color_tool = Handle::default();
            self.vis_mat_tool = Handle::default();
        }
        self.def_style = def_style.clone();
        self.roots = roots.to_vec();
        self.root_index = 0;
        self.flags = flags;
        self.init_root()
    }
    /// `More`.
    pub fn more(&self) -> bool {
        self.has_more
    }
    /// `Current`.
    pub fn current(&self) -> &XCAFPrsDocumentNode {
        &self.current
    }
    /// `ChangeCurrent`.
    pub fn change_current(&mut self) -> &mut XCAFPrsDocumentNode {
        &mut self.current
    }
    /// `Current(theDepth)`: the node at that depth (`Standard_OutOfRange` outside the stack).
    pub fn current_at(&self, depth: i32) -> Result<&XCAFPrsDocumentNode, Failure> {
        if depth == self.current_depth() {
            return Ok(&self.current);
        }
        if depth < 0 || depth > self.top {
            return Err(Failure::out_of_range(
                "XCAFPrs_DocumentExplorer::Current() out of range",
            ));
        }
        Ok(&self.node_stack[depth as usize])
    }
    /// `CurrentDepth`: the stack top for an assembly, else one below.
    pub fn current_depth(&self) -> i32 {
        if self.current.is_assembly {
            self.top
        } else {
            self.top + 1
        }
    }
    /// `ColorTool`.
    pub fn color_tool(&self) -> &Handle<XCAFDocColorTool> {
        &self.color_tool
    }
    /// `VisMaterialTool`.
    pub fn vis_material_tool(&self) -> &Handle<XCAFDocVisMaterialTool> {
        &self.vis_mat_tool
    }
    /// Whether the styles are fetched (a color tool is set).
    fn styled(&self) -> bool {
        !self.color_tool.is_null()
    }
    /// `myNodeStack.SetValue(index, node)`: grows the stack as needed.
    fn set_stack(&mut self, index: i32, node: XCAFPrsDocumentNode) {
        let index = index as usize;
        if self.node_stack.len() <= index {
            self.node_stack
                .resize_with(index + 1, XCAFPrsDocumentNode::default);
        }
        self.node_stack[index] = node;
    }
    /// The root label of the root iterator.
    fn root(&self) -> Option<&TdfLabel> {
        self.roots.get(self.root_index)
    }
}

/// The stack and root steps of `Next`.
impl XCAFPrsDocumentExplorer {
    /// `myTop`.
    pub(super) fn top_index(&self) -> i32 {
        self.top
    }
    /// The node at the stack top.
    pub(super) fn top_node(&self) -> &XCAFPrsDocumentNode {
        &self.node_stack[self.top as usize]
    }
    /// The current root label.
    pub(super) fn root_label(&self) -> TdfLabel {
        self.root().cloned().unwrap_or_default()
    }
    /// `myRootIter.Next(); initRoot()`.
    pub(super) fn next_root(&mut self) -> Result<(), Failure> {
        self.root_index += 1;
        self.init_root()
    }
    /// The `OnlyLeafNodes` flag.
    pub(super) fn leaf_only(&self) -> bool {
        self.flags & XCAF_PRS_DOCUMENT_EXPLORER_FLAGS_ONLY_LEAF_NODES != 0
    }
    /// Pushes a node on the stack.
    pub(super) fn push(&mut self, node: XCAFPrsDocumentNode) {
        self.top += 1;
        self.set_stack(self.top, node);
    }
    /// Resets and pops the stack top.
    pub(super) fn clear_top(&mut self) {
        self.set_stack(self.top, XCAFPrsDocumentNode::default());
        self.top -= 1;
    }
    /// Advances the child iterator of the stack top.
    pub(super) fn advance_top(&mut self) {
        self.node_stack[self.top as usize].child_iter.next();
    }
    /// `myHasMore = value`.
    pub(super) fn set_more(&mut self, value: bool) {
        self.has_more = value;
    }
    /// `initCurrent(true)`.
    pub(super) fn make_current_assembly(&mut self) -> Result<(), Failure> {
        self.init_current(true)
    }
    /// `initCurrent(false)`.
    pub(super) fn make_current_part(&mut self) -> Result<(), Failure> {
        self.init_current(false)
    }
}

/// The walk.
impl XCAFPrsDocumentExplorer {
    /// `initRoot`: the first non-null root from the iterator (its first component for an
    /// assembly), else the end.
    fn init_root(&mut self) -> Result<(), Failure> {
        loop {
            for index in 0..=self.top {
                self.set_stack(index, XCAFPrsDocumentNode::default());
            }
            self.top = -1;
            let Some(root) = self.root().cloned() else {
                self.has_more = false;
                return self.init_current(false);
            };
            if root.is_null() {
                self.root_index += 1;
                continue;
            }
            self.has_more = true;
            if XCAFDocShapeTool::is_assembly(&referred(&root)?)? {
                return self.next();
            }
            return self.init_current(false);
        }
    }
    /// `initCurrent(theIsAssembly)`: the current node from the stack top (assembly), the root
    /// or the child of the stack top.
    fn init_current(&mut self, is_assembly: bool) -> Result<(), Failure> {
        self.current = XCAFPrsDocumentNode::default();
        if is_assembly {
            if self.top < 0 {
                return Err(Failure::with_message(
                    "Standard_ProgramError",
                    "XCAFPrs_DocumentExplorer - internal error",
                ));
            }
            self.current = self.node_stack[self.top as usize].clone();
            return Ok(());
        }
        let (label, parent) = if self.top < 0 {
            let Some(root) = self.root().cloned() else {
                return Ok(());
            };
            (root, None)
        } else {
            let top = &self.node_stack[self.top as usize];
            (top.child_iter.value(), Some(top.clone()))
        };
        self.current = self.node_of(&label, parent.as_ref())?;
        Ok(())
    }
    /// A node of a label under a parent node (none at the root).
    fn node_of(
        &self,
        label: &TdfLabel,
        parent: Option<&XCAFPrsDocumentNode>,
    ) -> Result<XCAFPrsDocumentNode, Failure> {
        let mut node = XCAFPrsDocumentNode {
            label: label.clone(),
            ref_label: referred(label)?,
            ..Default::default()
        };
        node.local_trsf = XCAFDocShapeTool::get_location(label)?;
        node.location = match parent {
            Some(parent) => parent.location.multiplied(&node.local_trsf)?,
            None => node.local_trsf.clone(),
        };
        let parent_style = parent.map_or(&self.def_style, |p| &p.style);
        node.style = merged_style(self.styled(), parent_style, label, &node.ref_label)?;
        let parent_id = parent.map(|p| p.id.clone()).unwrap_or_default();
        node.id = Self::define_child_id(label, &parent_id);
        Ok(node)
    }
    /// An assembly node of a label under a parent node, iterating the children of its
    /// referred label.
    pub(super) fn assembly_node_of(
        &self,
        label: &TdfLabel,
        parent: Option<&XCAFPrsDocumentNode>,
    ) -> Result<XCAFPrsDocumentNode, Failure> {
        let mut node = self.node_of(label, parent)?;
        node.is_assembly = true;
        node.child_iter = TdfChildIterator::new(&node.ref_label, false)?;
        Ok(node)
    }
}
