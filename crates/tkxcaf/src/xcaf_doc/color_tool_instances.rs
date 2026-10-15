// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_ColorTool.cxx (the `Quantity_Color`
// overloads, the instance colors and visibility and ReverseChainsOfTreeNodes) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The colors of the instances of shapes in assemblies (through the SHUO of their
//! components), and the RGB overloads of the color members (an RGBA color with alpha 1).
use super::color_tool::XCAFDocColorTool;
use super::shape_tool_add::unlocated;
use super::support::{find, live};
use super::{XCAFDoc, XCAFDocColor, XCAFDocColorType, XCAFDocGraphNode, XCAFDocShapeTool};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::quantity::{Color, ColorRgba};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tklcaf::tdata_std::TDataStdTreeNode;
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfChildIdIterator, TdfLabel};

/// `Quantity_ColorRGBA aCol; aCol.SetRGB(col)`: an opaque RGBA color.
fn opaque(color: &Color) -> ColorRgba {
    ColorRgba::from_rgb(*color, 1.0)
}

/// The `Quantity_Color` overloads.
impl XCAFDocColorTool {
    /// `FindColor(col, lab)` with an RGB color.
    pub fn find_color_rgb(&self, color: &Color, label: &mut TdfLabel) -> Result<bool, Failure> {
        self.find_color(&opaque(color), label)
    }
    /// `FindColor(col)` with an RGB color.
    pub fn find_color_rgb_label(&self, color: &Color) -> Result<TdfLabel, Failure> {
        self.find_color_label(&opaque(color))
    }
    /// `AddColor(col)` with an RGB color.
    pub fn add_color_rgb(&self, color: &Color) -> Result<TdfLabel, Failure> {
        self.add_color(&opaque(color))
    }
    /// `SetColor(L, Color, type)` with an RGB color.
    pub fn set_color_rgb(
        &self,
        label: &TdfLabel,
        color: &Color,
        kind: XCAFDocColorType,
    ) -> Result<(), Failure> {
        self.set_color(label, &opaque(color), kind)
    }
    /// `GetColor(L, type, color)` with an RGB color.
    pub fn get_label_color_rgb(
        label: &TdfLabel,
        kind: XCAFDocColorType,
    ) -> Result<Option<Color>, Failure> {
        Ok(Self::get_label_color(label, kind)?.map(|color| *color.rgb()))
    }
    /// `SetColor(S, Color, type)` with an RGB color.
    pub fn set_shape_color_rgb(
        &self,
        shape: &Shape,
        color: &Color,
        kind: XCAFDocColorType,
    ) -> Result<bool, Failure> {
        self.set_shape_color(shape, &opaque(color), kind)
    }
    /// `GetColor(S, type, color)` with an RGB color.
    pub fn get_shape_color_rgb(
        &self,
        shape: &Shape,
        kind: XCAFDocColorType,
    ) -> Result<Option<Color>, Failure> {
        Ok(self.get_shape_color(shape, kind)?.map(|color| *color.rgb()))
    }
}

/// The instance colors.
impl XCAFDocColorTool {
    /// The shape tool, dereferenced.
    fn with_shape_tool<R>(
        &self,
        action: impl FnOnce(&XCAFDocShapeTool) -> Result<R, Failure>,
    ) -> Result<R, Failure> {
        let tool = self.shape_tool()?;
        action(live(&tool, "XCAFDoc_ColorTool::ShapeTool")?)
    }
    /// `SetInstanceColor(theShape, type, color, IsCreateSHUO)`: the color of the single
    /// component of the instance, else of its SHUO (created on request).
    pub fn set_instance_color(
        &self,
        shape: &Shape,
        kind: XCAFDocColorType,
        color: &ColorRgba,
        create_shuo: bool,
    ) -> Result<bool, Failure> {
        let mut labels = Vec::new();
        if !self.with_shape_tool(|tool| tool.find_component(shape, &mut labels))? {
            return Ok(false);
        }
        let mut shuo: Handle<XCAFDocGraphNode> = Handle::default();
        if !XCAFDocShapeTool::find_shuo(&labels, &mut shuo)? {
            if labels.len() == 1 {
                self.set_color(&labels[0], color, kind)?;
                return Ok(true);
            }
            if !create_shuo || !self.with_shape_tool(|tool| tool.set_shuo(&labels, &mut shuo))? {
                return Ok(false);
            }
        }
        let shuo_label = live(&shuo, "XCAFDoc_ColorTool::SetInstanceColor")?.label();
        self.set_color(&shuo_label, color, kind)?;
        Ok(true)
    }
    /// `SetInstanceColor` with an RGB color.
    pub fn set_instance_color_rgb(
        &self,
        shape: &Shape,
        kind: XCAFDocColorType,
        color: &Color,
        create_shuo: bool,
    ) -> Result<bool, Failure> {
        self.set_instance_color(shape, kind, &opaque(color), create_shuo)
    }
    /// `GetInstanceColor(theShape, type, color)`: the color of the longest SHUO of the
    /// instance that has one, else of its last component, else of its top-level shape.
    pub fn get_instance_color(
        &self,
        shape: &Shape,
        kind: XCAFDocColorType,
    ) -> Result<Option<ColorRgba>, Failure> {
        let mut labels = Vec::new();
        if !self.with_shape_tool(|tool| tool.find_component(shape, &mut labels))? {
            return Ok(None);
        }
        let component = labels[labels.len() - 1].clone();
        while labels.len() > 1 {
            let mut shuo: Handle<XCAFDocGraphNode> = Handle::default();
            if XCAFDocShapeTool::find_shuo(&labels, &mut shuo)? {
                let shuo_label = live(&shuo, "XCAFDoc_ColorTool::GetInstanceColor")?.label();
                if let Some(color) = Self::get_label_color(&shuo_label, kind)? {
                    return Ok(Some(color));
                }
            }
            labels.pop();
        }
        if let Some(color) = Self::get_label_color(&component, kind)? {
            return Ok(Some(color));
        }
        let referred =
            self.with_shape_tool(|tool| tool.find_shape_label(&unlocated(shape)?, false))?;
        if referred.is_null() {
            return Ok(None);
        }
        Self::get_label_color(&referred, kind)
    }
    /// `GetInstanceColor` with an RGB color.
    pub fn get_instance_color_rgb(
        &self,
        shape: &Shape,
        kind: XCAFDocColorType,
    ) -> Result<Option<Color>, Failure> {
        Ok(self
            .get_instance_color(shape, kind)?
            .map(|color| *color.rgb()))
    }
    /// `IsInstanceVisible(theShape)`: the top-level shape, the last component and each SHUO
    /// of the components (from the last one up) are visible.
    pub fn is_instance_visible(&self, shape: &Shape) -> Result<bool, Failure> {
        let referred =
            self.with_shape_tool(|tool| tool.find_shape_label(&unlocated(shape)?, false))?;
        if !referred.is_null() && !Self::is_visible(&referred)? {
            return Ok(false);
        }
        let mut labels = Vec::new();
        if !self.with_shape_tool(|tool| tool.find_component(shape, &mut labels))? {
            return Ok(true);
        }
        let component = labels[labels.len() - 1].clone();
        if !Self::is_visible(&component)? {
            return Ok(false);
        }
        let mut current = vec![component];
        for upper in labels[..labels.len() - 1].iter().rev() {
            current.insert(0, upper.clone());
            let mut shuo: Handle<XCAFDocGraphNode> = Handle::default();
            if !XCAFDocShapeTool::find_shuo(&current, &mut shuo)? {
                continue;
            }
            if !Self::is_visible(&live(&shuo, "XCAFDoc_ColorTool::IsInstanceVisible")?.label())? {
                return Ok(false);
            }
        }
        Ok(true) // visible, cause cannot find invisibility status
    }
    /// `ReverseChainsOfTreeNodes`: reverses the order of the users of every color (surface,
    /// curve, then generic links).
    pub fn reverse_chains_of_tree_nodes(&self) -> Result<bool, Failure> {
        let mut iterator = TdfChildIdIterator::new(&self.label(), &XCAFDocColor::get_id(), false)?;
        while iterator.more() {
            let label = live(
                &iterator.value(),
                "XCAFDoc_ColorTool::ReverseChainsOfTreeNodes",
            )?
            .label();
            for kind in [
                XCAFDocColorType::ColorSurf,
                XCAFDocColorType::ColorCurv,
                XCAFDocColorType::ColorGen,
            ] {
                let node = find::<TDataStdTreeNode>(&label, &XCAFDoc::color_ref_guid(kind))?;
                if !node.is_null() {
                    reverse_tree_nodes(&node)?;
                }
            }
            iterator.next()?;
        }
        Ok(true)
    }
}

/// The file-static `ReverseTreeNodes(mainNode)`: swaps the previous and next links of each
/// child and makes the last child the first.
fn reverse_tree_nodes(main: &Handle<TDataStdTreeNode>) -> Result<(), Failure> {
    let main_node = live(main, "ReverseTreeNodes")?;
    if !main_node.has_first() {
        return Ok(());
    }
    let mut current = main_node.first();
    let mut next = live(&current, "ReverseTreeNodes")?.next();
    while !next.is_null() {
        let node = live(&current, "ReverseTreeNodes")?;
        let previous = node.previous();
        node.set_previous(&next)?;
        node.set_next(&previous)?;
        current = next;
        next = live(&current, "ReverseTreeNodes")?.next();
    }
    let node = live(&current, "ReverseTreeNodes")?;
    let previous = node.previous();
    node.set_previous(&next)?;
    node.set_next(&previous)?;
    main_node.set_first(&current)
}
