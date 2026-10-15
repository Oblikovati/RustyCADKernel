//! The code the XCAFDoc tools that keep the document's shape tool repeat natively: the
//! `ShapeTool()` that looks it up once, and the `myShapeTool->Search(S, lab)` of their shape
//! overloads.
use super::support::live;
use super::{XCAFDocDocumentTool, XCAFDocShapeTool};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tklcaf::tdf::TdfLabel;

/// `ShapeTool()`: the cached shape tool, looked up from the label's document when null.
pub(crate) fn cached_shape_tool(
    cache: &SharedState<Handle<XCAFDocShapeTool>>,
    label: &TdfLabel,
) -> Result<Handle<XCAFDocShapeTool>, Failure> {
    if cache.read().is_null() {
        let tool = XCAFDocDocumentTool::shape_tool(label)?;
        *cache.write() = tool;
    }
    Ok(cache.read().clone())
}

/// `myShapeTool->Search(S, lab)` with the default flags: the label of the shape, `None` when
/// not found (`Standard_NullObject` without a shape tool, a null dereference natively).
pub(crate) fn search_shape(
    tool: &Handle<XCAFDocShapeTool>,
    shape: &Shape,
    who: &str,
) -> Result<Option<TdfLabel>, Failure> {
    let mut label = TdfLabel::default();
    let found = live(tool, who)?.search(shape, &mut label, [true, true, true])?;
    Ok(found.then_some(label))
}
