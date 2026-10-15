// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_AssemblyIterator.hxx and .cxx and
// XCAFDoc_AssemblyTool.hxx (the iterator traversal) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc_AssemblyIterator`: a depth-first walk over the items of the assemblies of a
//! document (each instance identified by its path of entries), down to a depth limit.
use super::{XCAFDocAssemblyItemId, XCAFDocDocumentTool, XCAFDocShapeTool};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tklcaf::tdf::{TdfLabel, TdfTool};
use rustycad_tklcaf::tdoc_std::TDocStdDocument;

/// An item pending in the fringe: its (referred) label and its path.
#[derive(Clone, Default)]
struct AuxAssemblyItem {
    label: TdfLabel,
    item: XCAFDocAssemblyItemId,
}

/// `XCAFDoc_AssemblyIterator`.
pub struct XCAFDocAssemblyIterator {
    shape_tool: Handle<XCAFDocShapeTool>,
    fringe: Vec<AuxAssemblyItem>,
    max_level: i32,
    seed_level: i32,
}

/// The shape tool of a document (`Standard_NullObject` for a null document,
/// `Standard_NoSuchObject` without tool) and the depth check.
fn document_shape_tool(
    document: &Handle<TDocStdDocument>,
    level: i32,
) -> Result<Handle<XCAFDocShapeTool>, Failure> {
    let main = rustycad_tklcaf::live(document, "Null document!")?.main()?;
    let tool = XCAFDocDocumentTool::shape_tool(&main)?;
    if tool.is_null() {
        return Err(Failure::with_message(
            "Standard_NoSuchObject",
            "No XCAFDoc_ShapeTool attribute!",
        ));
    }
    if level < 0 {
        return Err(Failure::with_message(
            "Standard_RangeError",
            "Null document!",
        ));
    }
    Ok(tool)
}

/// `createItem(theLabel, theParentPath, theAuxItem)`: the referred label (else the label)
/// with the parent path and the label's entry.
fn create_item(label: &TdfLabel, parent: &[AsciiString]) -> Result<AuxAssemblyItem, Failure> {
    let mut entry = AsciiString::default();
    TdfTool::entry(label, &mut entry);
    let mut original = TdfLabel::default();
    let referred = if XCAFDocShapeTool::get_referred_shape(label, &mut original)? {
        original
    } else {
        label.clone()
    };
    let mut path = parent.to_vec();
    path.push(entry);
    Ok(AuxAssemblyItem {
        label: referred,
        item: XCAFDocAssemblyItemId::from_path(&path),
    })
}

impl XCAFDocAssemblyIterator {
    /// `XCAFDoc_AssemblyIterator(theDoc, theLevel)`: the free shapes as seeds (the native
    /// default level is `INT_MAX`).
    pub fn new(document: &Handle<TDocStdDocument>, level: i32) -> Result<Self, Failure> {
        let shape_tool = document_shape_tool(document, level)?;
        let roots =
            rustycad_tklcaf::live(&shape_tool, "XCAFDoc_AssemblyIterator")?.get_free_shapes()?;
        let fringe = roots
            .iter()
            .map(|root| create_item(root, &[]))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            shape_tool,
            fringe,
            max_level: level,
            seed_level: 1,
        })
    }
    /// `XCAFDoc_AssemblyIterator(theDoc, theRoot, theLevel)`: the root item as the seed (on
    /// its referred shape for a part instance, its parent path for an assembly instance);
    /// empty when its last entry has no label.
    pub fn with_root(
        document: &Handle<TDocStdDocument>,
        root: &XCAFDocAssemblyItemId,
        level: i32,
    ) -> Result<Self, Failure> {
        let shape_tool = document_shape_tool(document, level)?;
        if root.is_null() {
            return Err(Failure::with_message(
                "Standard_NullObject",
                "Null assembly item!",
            ));
        }
        let mut iterator = Self {
            shape_tool,
            fringe: Vec::new(),
            max_level: level,
            seed_level: root.get_path().len() as i32,
        };
        let data = rustycad_tklcaf::live(document, "XCAFDoc_AssemblyIterator")?.get_data();
        let mut seed = AuxAssemblyItem {
            item: root.clone(),
            ..Default::default()
        };
        let last = root.get_path().last().cloned().unwrap_or_default();
        TdfTool::label_from_entry(&data, &last, &mut seed.label, false)?;
        if seed.label.is_null() {
            return Ok(iterator);
        }
        seed_referred(&mut seed, root)?;
        iterator.fringe.push(seed);
        Ok(iterator)
    }
    /// `More`: items are pending.
    pub fn more(&self) -> bool {
        !self.fringe.is_empty()
    }
    /// `Next`: pops the current item, then pushes its components (or the component itself)
    /// in reverse order while under the depth limit.
    #[allow(clippy::should_implement_trait)] // OCCT's More/Next protocol, fallible.
    pub fn next(&mut self) -> Result<(), Failure> {
        let Some(current) = self.fringe.pop() else {
            return Ok(()); // No next item.
        };
        let depth = current.item.get_path().len() as i32 - self.seed_level;
        if depth >= self.max_level {
            return Ok(());
        }
        let mut components = Vec::new();
        if XCAFDocShapeTool::is_assembly(&current.label)? {
            XCAFDocShapeTool::get_components(&current.label, &mut components, false)?;
        } else if XCAFDocShapeTool::is_component(&current.label)? {
            components.push(current.label.clone());
        }
        for component in components.iter().rev() {
            self.fringe
                .push(create_item(component, current.item.get_path())?);
        }
        Ok(())
    }
    /// `Current`: the item on top of the fringe (`Standard_NoSuchObject` past the end).
    pub fn current(&self) -> Result<XCAFDocAssemblyItemId, Failure> {
        self.fringe
            .last()
            .map(|aux| aux.item.clone())
            .ok_or_else(|| {
                Failure::with_message("Standard_NoSuchObject", "NCollection_Sequence::Last")
            })
    }
    /// The shape tool of the document.
    pub fn shape_tool(&self) -> &Handle<XCAFDocShapeTool> {
        &self.shape_tool
    }
    /// `XCAFDoc_AssemblyTool::Traverse(theIterator, theFunc)`: the items in order until the
    /// function returns false.
    pub fn traverse(
        mut self,
        mut function: impl FnMut(&XCAFDocAssemblyItemId) -> bool,
    ) -> Result<(), Failure> {
        while self.more() {
            if !function(&self.current()?) {
                break;
            }
            self.next()?;
        }
        Ok(())
    }
}

/// The seed of a root that refers to a shape: the referred label for a part instance, the
/// path without its last entry for an assembly instance.
fn seed_referred(seed: &mut AuxAssemblyItem, root: &XCAFDocAssemblyItemId) -> Result<(), Failure> {
    let mut original = TdfLabel::default();
    if !XCAFDocShapeTool::get_referred_shape(&seed.label, &mut original)? {
        return Ok(());
    }
    if !XCAFDocShapeTool::is_assembly(&seed.label)? {
        seed.label = original;
        return Ok(());
    }
    let mut path = root.to_string_path()?;
    let index = path.search_from_end("/");
    if index != -1 {
        path.remove(index, path.length() - index + 1)?;
    }
    seed.item.init_from_string(&path);
    Ok(())
}
