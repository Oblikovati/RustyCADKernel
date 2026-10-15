// Rust port of DataExchange/TKXCAF/XCAFPrs/XCAFPrs.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFPrs`: the styles of the shapes of an XDE label (colors, materials, layers,
//! visibility and the styles of the specified higher usage occurrences), and the view name
//! mode.
use super::XCAFPrsStyle;
use crate::xcaf_doc::{
    XCAFDocColorTool, XCAFDocColorType, XCAFDocDocumentTool, XCAFDocGraphNode, XCAFDocLayerTool,
    XCAFDocShapeTool, XCAFDocVisMaterialTool,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::IndexedDataMap;
use rustycad_tkernel::quantity::ColorRgba;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel};
use rustycad_tkmath::toploc::Location;
use std::sync::atomic::{AtomicBool, Ordering};

/// The styles of shapes (`NCollection_IndexedDataMap<TopoDS_Shape, XCAFPrs_Style>`).
pub type XCAFPrsStyleMap = IndexedDataMap<Shape, XCAFPrsStyle, ShapeMapHasher>;

/// The file-static `viewnameMode`.
static VIEW_NAME_MODE: AtomicBool = AtomicBool::new(false);

/// `XCAFPrs`: a class of static members only.
pub struct XCAFPrs;

/// The file-static `fillStyleColors`: the generic color as both colors, then the surface
/// and curve colors.
fn fill_style_colors(style: &mut XCAFPrsStyle, label: &TdfLabel) -> Result<(), Failure> {
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

/// Adds the location unless an equal one is there (`NCollection_IndexedMap::Add`).
fn add_location(locations: &mut Vec<Location>, location: Location) {
    if !locations.contains(&location) {
        locations.push(location);
    }
}

/// The file-static `getShapesOfSHUO`: the shapes of the last next usages of a SHUO, moved
/// by the product of the locations of the chain.
fn shapes_of_shuo(
    locations: &[Location],
    shuo: &TdfLabel,
    shapes: &mut Vec<Shape>,
) -> Result<bool, Failure> {
    let mut next = Vec::new();
    XCAFDocShapeTool::get_shuo_next_usage(shuo, &mut next)?;
    if next.is_empty() {
        return last_usage_shape(locations, shuo, shapes);
    }
    for sub_component in next {
        let mut chain = Vec::new();
        locations
            .iter()
            .for_each(|location| add_location(&mut chain, location.clone()));
        add_location(
            &mut chain,
            XCAFDocShapeTool::get_location(&sub_component.father())?,
        );
        shapes_of_shuo(&chain, &sub_component, shapes)?;
    }
    Ok(!shapes.is_empty())
}

/// The shape of the component of the last usage, moved by the product of the chain.
fn last_usage_shape(
    locations: &[Location],
    shuo: &TdfLabel,
    shapes: &mut Vec<Shape>,
) -> Result<bool, Failure> {
    let mut shape = XCAFDocShapeTool::shape_of(&shuo.father())?;
    let Some(last) = locations.last() else {
        return Ok(false); // should not be, but to avoid exception...?
    };
    if shape.is_null() {
        return Ok(false);
    }
    let mut product = last.clone();
    for location in locations[..locations.len() - 1].iter().rev() {
        product = location.multiplied(&product)?;
    }
    shape.set_location(&product, false)?;
    shapes.push(shape);
    Ok(!shapes.is_empty())
}

/// Adds or replaces the style of the shape.
fn bind_style(settings: &mut XCAFPrsStyleMap, shape: Shape, style: &XCAFPrsStyle) {
    match settings.change_seek(&shape) {
        Some(known) => *known = style.clone(),
        None => {
            settings.add(shape, style.clone());
        }
    }
}

impl XCAFPrs {
    /// `SetViewNameMode(viewNameMode)`: process-wide.
    pub fn set_view_name_mode(mode: bool) {
        VIEW_NAME_MODE.store(mode, Ordering::SeqCst);
    }
    /// `GetViewNameMode`.
    pub fn get_view_name_mode() -> bool {
        VIEW_NAME_MODE.load(Ordering::SeqCst)
    }
    /// `CollectStyleSettings(theLabel, theLoc, theSettings, theLayerColor)`: the styles of the
    /// referred shape, of the components and of the sub-shapes and the shape of the label
    /// (the native default layer color is white).
    pub fn collect_style_settings(
        label: &TdfLabel,
        location: &Location,
        settings: &mut XCAFPrsStyleMap,
        layer_color: &ColorRgba,
    ) -> Result<(), Failure> {
        let mut referred = TdfLabel::default();
        if XCAFDocShapeTool::get_referred_shape(label, &mut referred)? {
            let color = single_layer_color(label, layer_color)?;
            let sub_location = location.multiplied(&XCAFDocShapeTool::get_location(label)?)?;
            Self::collect_style_settings(&referred, &sub_location, settings, &color)?;
        }
        let mut components = Vec::new();
        if XCAFDocShapeTool::get_components(label, &mut components, false)? {
            for component in &components {
                Self::collect_style_settings(component, location, settings, layer_color)?;
            }
        }
        // The tools are looked up (and created when absent) as natively.
        XCAFDocDocumentTool::color_tool(label)?;
        XCAFDocDocumentTool::vis_material_tool(label)?;
        let mut labels = Vec::new();
        XCAFDocShapeTool::get_sub_shapes(label, &mut labels)?;
        labels.push(label.clone());
        for sub_label in &labels {
            collect_label_style(sub_label, location, settings, layer_color)?;
        }
        Ok(())
    }
}

/// The color of the only layer of the label when it has one, else the given color.
fn single_layer_color(label: &TdfLabel, layer_color: &ColorRgba) -> Result<ColorRgba, Failure> {
    let layers = XCAFDocLayerTool::get_layers(label)?;
    if layers.len() != 1 {
        return Ok(*layer_color);
    }
    let tool = XCAFDocDocumentTool::layer_tool(label)?;
    let layer = rustycad_tklcaf::live(&tool, "XCAFPrs::CollectStyleSettings")?
        .find_layer_with(&layers[0], false, true)?;
    Ok(
        XCAFDocColorTool::get_label_color(&layer, XCAFDocColorType::ColorGen)?
            .unwrap_or(*layer_color),
    )
}

/// The visibility of a label: visible itself, and not all of its (non-empty) layers hidden.
fn label_visibility(label: &TdfLabel) -> Result<bool, Failure> {
    if !XCAFDocColorTool::is_visible(label)? {
        return Ok(false);
    }
    let tool = XCAFDocDocumentTool::layer_tool(label)?;
    let tool = rustycad_tklcaf::live(&tool, "XCAFPrs::CollectStyleSettings")?;
    let layers = XCAFDocLayerTool::get_layers(label)?;
    let mut hidden = 0;
    for layer in &layers {
        if !tool.is_visible(&tool.find_layer_with(layer, false, true)?)? {
            hidden += 1;
        }
    }
    Ok(hidden == 0 || hidden != layers.len())
}

/// The style of one label of `CollectStyleSettings`, with the styles of its SHUO.
fn collect_label_style(
    label: &TdfLabel,
    location: &Location,
    settings: &mut XCAFPrsStyleMap,
    layer_color: &ColorRgba,
) -> Result<(), Failure> {
    let color_tool = XCAFDocDocumentTool::color_tool(label)?;
    let colors = rustycad_tklcaf::live(&color_tool, "XCAFPrs::CollectStyleSettings")?;
    let mut style = XCAFPrsStyle::new();
    style.set_visibility(label_visibility(label)?);
    style.set_material(&XCAFDocVisMaterialTool::get_shape_material(label)?);
    if colors.is_color_by_layer(label)? {
        let color = single_layer_color(label, layer_color)?;
        style.set_color_curv(*color.rgb());
        style.set_color_surf_rgba(color);
    } else {
        fill_style_colors(&mut style, label)?;
    }
    if XCAFDocShapeTool::is_component(label)? {
        collect_shuo_styles(label, location, settings)?;
    }
    if style.is_empty() {
        return Ok(());
    }
    let mut shape = XCAFDocShapeTool::shape_of(label)?;
    if shape.shape_type()? == ShapeEnum::Compound && shape.nb_children() == 0 {
        return Ok(());
    }
    shape.move_by(location, false)?;
    bind_style(settings, shape, &style);
    Ok(())
}

/// The styles of the SHUO of a component with next usages and a style.
fn collect_shuo_styles(
    component: &TdfLabel,
    location: &Location,
    settings: &mut XCAFPrsStyleMap,
) -> Result<(), Failure> {
    let mut nodes = Vec::new();
    XCAFDocShapeTool::get_all_component_shuo(component, &mut nodes)?;
    for node in nodes {
        let Some(shuo) = node.cast::<XCAFDocGraphNode>().get().map(|n| n.label()) else {
            continue;
        };
        let mut next = Vec::new();
        XCAFDocShapeTool::get_shuo_next_usage(&shuo, &mut next)?;
        if next.is_empty() {
            continue;
        }
        let mut style = XCAFPrsStyle::new();
        style.set_material(&XCAFDocVisMaterialTool::get_shape_material(&shuo)?);
        style.set_visibility(XCAFDocColorTool::is_visible(&shuo)?);
        fill_style_colors(&mut style, &shuo)?;
        if style.is_empty() {
            continue;
        }
        let mut locations = Vec::new();
        if !location.is_identity() {
            add_location(&mut locations, location.clone());
        }
        add_location(&mut locations, XCAFDocShapeTool::get_location(component)?);
        let mut shapes = Vec::new();
        shapes_of_shuo(&locations, &shuo, &mut shapes)?;
        shapes
            .into_iter()
            .for_each(|shape| bind_style(settings, shape, &style));
    }
    Ok(())
}
