// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_ColorTool.hxx and .cxx (the colors, the
// color links of labels and shapes and the visibility flags) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc_ColorTool`: the colors of an XDE document (one `XCAFDoc_Color` per child of the
//! tool's label) and the links of shapes to them (tree nodes of `ColorRefGUID(type)`).
use super::generic_empty_tool::generic_empty_attribute;
use super::support::{find, find_or_add, live};
use super::tool_support::{cached_shape_tool, search_shape};
use super::{
    XCAFDoc, XCAFDocColor, XCAFDocColorType, XCAFDocDocumentTool, XCAFDocGraphNode,
    XCAFDocShapeTool,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::quantity::{Color, ColorRgba};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
use rustycad_tklcaf::tdata_std::{TDataStdName, TDataStdTreeNode, TDataStdUAttribute};
use rustycad_tklcaf::tdf::{
    TdfAttributeState, TdfChildIdIterator, TdfLabel, TdfTagSource, TdfTool,
};
use rustycad_tkservice::json_dump::field_string;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};

/// `XCAFDoc_ColorTool_AutoNaming`: the file-static flag of `SetAutoNaming`, true by default.
static COLOR_AUTO_NAMING: AtomicBool = AtomicBool::new(true);

generic_empty_attribute!(
    /// `XCAFDoc_ColorTool`.
    XCAFDocColorTool, "XCAFDoc_ColorTool", "efd212ed-6dfd-11d4-b9c8-0060b0ee281b",
    state: Handle<XCAFDocShapeTool>,
    dump: |tool: &XCAFDocColorTool, stream: &mut DumpStream, depth: i32| -> io::Result<()> {
        rustycad_tkservice::json_dump::class_begin(stream, "XCAFDoc_ColorTool")?;
        rustycad_tklcaf::json::dump_attribute_json_base(tool, stream, depth)?;
        for label in tool.get_colors().map_err(io::Error::other)? {
            field_string(stream, "aColorLabel", entry_text(&label).as_bytes())?;
        }
        Ok(())
    }
);

/// The entry text of a label.
pub(crate) fn entry_text(label: &TdfLabel) -> String {
    let mut entry = AsciiString::default();
    TdfTool::entry(label, &mut entry);
    String::from_utf8_lossy(entry.as_bytes()).into_owned()
}

impl XCAFDocColorTool {
    /// `Set(L)`: the tool of the label, created (with the document's shape tool) when absent.
    pub fn set(label: &TdfLabel) -> Result<Handle<XCAFDocColorTool>, Failure> {
        let existing = find::<XCAFDocColorTool>(label, &Self::get_id())?;
        if !existing.is_null() {
            return Ok(existing);
        }
        let tool = find_or_add(label, &Self::get_id(), Self::new)?;
        *live(&tool, "XCAFDoc_ColorTool::Set")?.state.write() =
            XCAFDocDocumentTool::shape_tool(label)?;
        Ok(tool)
    }
    /// `SetAutoNaming(theIsAutoNaming)`: whether new colors get names (process-wide).
    pub fn set_auto_naming(value: bool) {
        COLOR_AUTO_NAMING.store(value, Ordering::SeqCst);
    }
    /// `AutoNaming`.
    pub fn auto_naming() -> bool {
        COLOR_AUTO_NAMING.load(Ordering::SeqCst)
    }
    /// `BaseLabel`.
    pub fn base_label(&self) -> TdfLabel {
        self.label()
    }
    /// `ShapeTool()`: the document's shape tool (looked up once).
    pub fn shape_tool(&self) -> Result<Handle<XCAFDocShapeTool>, Failure> {
        cached_shape_tool(&self.state, &self.label())
    }
    /// `IsColor(lab)`: the label holds a color.
    pub fn is_color(&self, label: &TdfLabel) -> Result<bool, Failure> {
        Ok(Self::get_color_rgba(label)?.is_some())
    }
    /// `GetColor(lab, col)` as RGB: the color of a color label.
    pub fn get_color(label: &TdfLabel) -> Result<Option<Color>, Failure> {
        Ok(Self::get_color_rgba(label)?.map(|color| *color.rgb()))
    }
    /// `GetColor(lab, col)` as RGBA: the color of a color label.
    pub fn get_color_rgba(label: &TdfLabel) -> Result<Option<ColorRgba>, Failure> {
        let color = find::<XCAFDocColor>(label, &XCAFDocColor::get_id())?;
        Ok(color.get().map(XCAFDocColor::get_color_rgba))
    }
    /// `FindColor(col, lab)`: the first color label with an equal RGBA color.
    pub fn find_color(&self, color: &ColorRgba, label: &mut TdfLabel) -> Result<bool, Failure> {
        let mut iterator = TdfChildIdIterator::new(&self.label(), &XCAFDocColor::get_id(), false)?;
        while iterator.more() {
            let found = live(&iterator.value(), "XCAFDoc_ColorTool::FindColor")?.label();
            if Self::get_color_rgba(&found)?.is_some_and(|c| c.is_equal(*color)) {
                *label = found;
                return Ok(true);
            }
            iterator.next()?;
        }
        Ok(false)
    }
    /// `FindColor(col)`: the color label, null when not found.
    pub fn find_color_label(&self, color: &ColorRgba) -> Result<TdfLabel, Failure> {
        let mut label = TdfLabel::default();
        self.find_color(color, &mut label)?;
        Ok(label)
    }
    /// `AddColor(col)`: the label of an equal color, or a new color label (named
    /// `<name> (#hex)` when naming automatically).
    pub fn add_color(&self, color: &ColorRgba) -> Result<TdfLabel, Failure> {
        let mut label = TdfLabel::default();
        if self.find_color(color, &mut label)? {
            return Ok(label);
        }
        let label = TdfTagSource::new_child_of(&self.label())?;
        XCAFDocColor::set_rgba(&label, *color)?;
        if Self::auto_naming() {
            let name = format!(
                "{} ({})",
                Color::string_name(color.rgb().name()),
                String::from_utf8_lossy(ColorRgba::to_hex(*color, true).as_bytes())
            );
            TDataStdName::set(&label, &ExtendedString::from_utf8(&name))?;
        }
        Ok(label)
    }
    /// `RemoveColor(lab)`: forgets every attribute of the color label.
    pub fn remove_color(&self, label: &TdfLabel) -> Result<(), Failure> {
        label.forget_all_attributes(true)
    }
    /// `GetColors(Labels)`: the color labels.
    pub fn get_colors(&self) -> Result<Vec<TdfLabel>, Failure> {
        let mut labels = Vec::new();
        let mut iterator = TdfChildIdIterator::new(&self.label(), &XCAFDocColor::get_id(), false)?;
        while iterator.more() {
            let label = live(&iterator.value(), "XCAFDoc_ColorTool::GetColors")?.label();
            if self.is_color(&label)? {
                labels.push(label);
            }
            iterator.next()?;
        }
        Ok(labels)
    }
}

/// The color links of labels.
impl XCAFDocColorTool {
    /// `SetColor(L, colorL, type)`: the label's node of that type, first among the users of
    /// the color label's node.
    pub fn set_color_label(
        &self,
        label: &TdfLabel,
        color_label: &TdfLabel,
        kind: XCAFDocColorType,
    ) -> Result<(), Failure> {
        let guid = XCAFDoc::color_ref_guid(kind);
        let main = TDataStdTreeNode::set_with_id(color_label, &guid)?;
        let reference = TDataStdTreeNode::set_with_id(label, &guid)?;
        // abv: fix against bug in TreeNode::Append()
        live(&reference, "XCAFDoc_ColorTool::SetColor")?.remove()?;
        live(&main, "XCAFDoc_ColorTool::SetColor")?.prepend(&reference)?;
        Ok(())
    }
    /// `SetColor(L, Color, type)`: the link to the (added) color.
    pub fn set_color(
        &self,
        label: &TdfLabel,
        color: &ColorRgba,
        kind: XCAFDocColorType,
    ) -> Result<(), Failure> {
        let color_label = self.add_color(color)?;
        self.set_color_label(label, &color_label, kind)
    }
    /// `UnSetColor(L, type)`: forgets the link of that type.
    pub fn unset_color(&self, label: &TdfLabel, kind: XCAFDocColorType) -> Result<(), Failure> {
        label.forget_attribute_id(&XCAFDoc::color_ref_guid(kind))?;
        Ok(())
    }
    /// `IsSet(L, type)`: a link of that type with a color.
    pub fn is_set(&self, label: &TdfLabel, kind: XCAFDocColorType) -> Result<bool, Failure> {
        let node = find::<TDataStdTreeNode>(label, &XCAFDoc::color_ref_guid(kind))?;
        Ok(node.get().is_some_and(TDataStdTreeNode::has_father))
    }
    /// `GetColor(L, type, colorL)`: the color label linked with that type.
    pub fn get_color_label(
        label: &TdfLabel,
        kind: XCAFDocColorType,
        color_label: &mut TdfLabel,
    ) -> Result<bool, Failure> {
        let node = find::<TDataStdTreeNode>(label, &XCAFDoc::color_ref_guid(kind))?;
        let Some(node) = node.get().filter(|node| node.has_father()) else {
            return Ok(false);
        };
        *color_label = live(&node.father(), "XCAFDoc_ColorTool::GetColor")?.label();
        Ok(true)
    }
    /// `GetColor(L, type, color)`: the RGBA color linked with that type.
    pub fn get_label_color(
        label: &TdfLabel,
        kind: XCAFDocColorType,
    ) -> Result<Option<ColorRgba>, Failure> {
        let mut color_label = TdfLabel::default();
        if !Self::get_color_label(label, kind, &mut color_label)? {
            return Ok(None);
        }
        Self::get_color_rgba(&color_label)
    }
}

/// The color links of shapes (through the shape tool's `Search`).
impl XCAFDocColorTool {
    /// The label of a shape, `None` when the search fails.
    fn search(&self, shape: &Shape) -> Result<Option<TdfLabel>, Failure> {
        search_shape(&self.shape_tool()?, shape, "XCAFDoc_ColorTool")
    }
    /// `SetColor(S, colorL, type)`.
    pub fn set_shape_color_label(
        &self,
        shape: &Shape,
        color_label: &TdfLabel,
        kind: XCAFDocColorType,
    ) -> Result<bool, Failure> {
        let Some(label) = self.search(shape)? else {
            return Ok(false);
        };
        self.set_color_label(&label, color_label, kind)?;
        Ok(true)
    }
    /// `SetColor(S, Color, type)`: the color is added even when the shape is not found.
    pub fn set_shape_color(
        &self,
        shape: &Shape,
        color: &ColorRgba,
        kind: XCAFDocColorType,
    ) -> Result<bool, Failure> {
        let color_label = self.add_color(color)?;
        self.set_shape_color_label(shape, &color_label, kind)
    }
    /// `UnSetColor(S, type)`.
    pub fn unset_shape_color(
        &self,
        shape: &Shape,
        kind: XCAFDocColorType,
    ) -> Result<bool, Failure> {
        let Some(label) = self.search(shape)? else {
            return Ok(false);
        };
        self.unset_color(&label, kind)?;
        Ok(true)
    }
    /// `IsSet(S, type)`.
    pub fn is_shape_set(&self, shape: &Shape, kind: XCAFDocColorType) -> Result<bool, Failure> {
        let Some(label) = self.search(shape)? else {
            return Ok(false);
        };
        self.is_set(&label, kind)
    }
    /// `GetColor(S, type, colorL)`.
    pub fn get_shape_color_label(
        &self,
        shape: &Shape,
        kind: XCAFDocColorType,
        color_label: &mut TdfLabel,
    ) -> Result<bool, Failure> {
        let Some(label) = self.search(shape)? else {
            return Ok(false);
        };
        Self::get_color_label(&label, kind, color_label)
    }
    /// `GetColor(S, type, color)`.
    pub fn get_shape_color(
        &self,
        shape: &Shape,
        kind: XCAFDocColorType,
    ) -> Result<Option<ColorRgba>, Failure> {
        let mut color_label = TdfLabel::default();
        if !self.get_shape_color_label(shape, kind, &mut color_label)? {
            return Ok(None);
        }
        Self::get_color_rgba(&color_label)
    }
}

/// Visibility and color by layer.
impl XCAFDocColorTool {
    /// `IsVisible(L)`: no invisibility flag.
    pub fn is_visible(label: &TdfLabel) -> Result<bool, Failure> {
        Ok(!label.is_attribute(&XCAFDoc::invisible_guid())?)
    }
    /// `SetVisibility(L, isvisible)`: the invisibility flag of a shape or SHUO label, or
    /// none.
    pub fn set_visibility(&self, label: &TdfLabel, visible: bool) -> Result<(), Failure> {
        self.set_flag(label, &XCAFDoc::invisible_guid(), !visible)
    }
    /// `IsColorByLayer(L)`.
    pub fn is_color_by_layer(&self, label: &TdfLabel) -> Result<bool, Failure> {
        label.is_attribute(&XCAFDoc::color_by_layer_guid())
    }
    /// `SetColorByLayer(L, isColorByLayer)`.
    pub fn set_color_by_layer(&self, label: &TdfLabel, by_layer: bool) -> Result<(), Failure> {
        self.set_flag(label, &XCAFDoc::color_by_layer_guid(), by_layer)
    }
    /// Sets a flag on a shape or SHUO label (when absent), or forgets it.
    fn set_flag(
        &self,
        label: &TdfLabel,
        guid: &rustycad_tkernel::standard::Guid,
        on: bool,
    ) -> Result<(), Failure> {
        if !on {
            label.forget_attribute_id(guid)?;
            return Ok(());
        }
        let mut shuo: Handle<XCAFDocGraphNode> = Handle::default();
        let flagged =
            XCAFDocShapeTool::is_shape(label)? || XCAFDocShapeTool::get_shuo(label, &mut shuo)?;
        if flagged && !label.is_attribute(guid)? {
            TDataStdUAttribute::set(label, guid)?;
        }
        Ok(())
    }
}
