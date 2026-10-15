// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_Editor.cxx (RescaleGeometry: the
// dimensions, rescaleDimensionRefLabels and shouldRescaleAndCheckRefLabels) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The GD&T part of `XCAFDoc_Editor::RescaleGeometry`: the dimensions whose shapes belong
//! to the rescaled graph get their points, planes and values scaled, the datums their
//! points, and the legacy entries a warning.
use super::color_tool::entry_text;
use super::editor_rescale::{rescale_datums, scaled, scaled_axis, warn};
use super::support::find;
use super::{
    XCAFDocAssemblyGraph, XCAFDocAssemblyItemRef, XCAFDocDimTolTool, XCAFDocDimension,
    XCAFDocDocumentTool,
};
use crate::xcaf_dim_tol_objects::{
    XCAFDimTolObjectsDimensionObject, XCAFDimTolObjectsDimensionType as T,
};
use rustycad_tkcaf::tnaming::{TNamingBuilder, TNamingNamedShape};
use rustycad_tkernel::standard::c_string::{FormatArg, sprintf_into};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tklcaf::tdf::{TdfLabel, TdfTool};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPITransform};

/// The graph contains the label as a node.
fn in_graph(graph: &XCAFDocAssemblyGraph, label: &TdfLabel) -> Result<bool, Failure> {
    Ok(graph.get_nodes()?.contains(label))
}

/// The file-static `shouldRescaleAndCheckRefLabels`: a label (or the item a reference
/// refers to) is in the graph; `all_in_graph` tells whether all are.
fn should_rescale(
    labels: &[TdfLabel],
    graph: &XCAFDocAssemblyGraph,
    all_in_graph: &mut bool,
) -> Result<bool, Failure> {
    *all_in_graph = true;
    let mut rescale = false;
    for label in labels {
        if in_graph(graph, label)? {
            rescale = true;
            continue;
        }
        let reference = find::<XCAFDocAssemblyItemRef>(label, &XCAFDocAssemblyItemRef::get_id())?;
        let item = reference
            .get()
            .map(XCAFDocAssemblyItemRef::get_item)
            .unwrap_or_default();
        let Some(entry) = item.get_path().last() else {
            *all_in_graph = false;
            continue;
        };
        let mut referred = TdfLabel::default();
        TdfTool::label_from_entry(&label.data(), entry, &mut referred, false)?;
        if referred.is_null() || !in_graph(graph, &referred)? {
            *all_in_graph = false;
            continue;
        }
        rescale = true;
    }
    Ok(rescale)
}

/// The file-static `rescaleDimensionRefLabels`: the named shapes of the labels outside the
/// graph, scaled.
fn rescale_ref_labels(
    labels: &[TdfLabel],
    transform: &mut BRepBuilderAPITransform,
    graph: &XCAFDocAssemblyGraph,
    entry: &str,
) -> Result<(), Failure> {
    for label in labels {
        if in_graph(graph, label)? {
            continue;
        }
        let named = find::<TNamingNamedShape>(label, &TNamingNamedShape::get_id())?;
        let Some(named) = named.get() else {
            continue;
        };
        let shape = named.get()?;
        transform.perform(&shape, true, true)?;
        if !transform.is_done() {
            warn(&format!("Dimension PMI {entry} is not scaled."));
            continue;
        }
        let scaled_shape = transform.shape()?.clone();
        TNamingBuilder::new(label)?.generated_from(&shape, &scaled_shape)?;
    }
    Ok(())
}

/// How the first value of a dimension of that type rescales: `Some(true)` scaled with the
/// other values (linear distances and sizes), `Some(false)` kept (angles), `None` not
/// scaled with a warning (no location, curved distance, oriented, location with path,
/// common label and presentation).
fn value_policy(kind: T) -> Option<bool> {
    let value = kind as i32;
    let linear = (T::LocationLinearDistance as i32
        ..=T::LocationLinearDistanceFromInnerToInner as i32)
        .contains(&value);
    let size = (T::SizeCurveLength as i32..=T::SizeThickness as i32).contains(&value)
        || kind == T::SizeWithPath;
    if linear || size {
        return Some(true);
    }
    (kind == T::LocationAngular || kind == T::SizeAngular).then_some(false)
}

/// `Standard_SStream << double`: the `%g` text of the value.
fn stream_real(value: f64) -> Result<String, Failure> {
    let mut text = Vec::new();
    sprintf_into(&mut text, b"%g", &[FormatArg::Real(value)])
        .map_err(|e| Failure::new(&e.to_string()))?;
    Ok(String::from_utf8_lossy(&text).into_owned())
}

/// The points and plane of a dimension object, scaled.
fn rescale_dimension_points(object: &XCAFDimTolObjectsDimensionObject, factor: f64) {
    if object.has_text_point() {
        object.set_point_text_attach(&scaled(object.get_point_text_attach(), factor));
    }
    if object.has_point() {
        object.set_point(&scaled(object.get_point(), factor));
    }
    if object.has_point2() {
        object.set_point2(&scaled(object.get_point2(), factor));
    }
    if object.has_plane() {
        object.set_plane(&scaled_axis(object.get_plane(), factor));
    }
}

/// The values of a dimension object (and the reference shapes), scaled by type; the
/// semantic name records the new first value.
fn rescale_values(
    object: &XCAFDimTolObjectsDimensionObject,
    refs: (&[TdfLabel], &[TdfLabel]),
    transform: &mut BRepBuilderAPITransform,
    graph: &XCAFDocAssemblyGraph,
    factor: f64,
    entry: &str,
) -> Result<(), Failure> {
    let values = object.get_values();
    let Some(array) = values.get() else {
        warn(&format!("Dimension PMI values {entry} are not scaled."));
        return Ok(());
    };
    let policy = value_policy(object.get_type());
    match policy {
        Some(true) => array.change_array1().as_mut_slice()[0] *= factor,
        Some(false) => {}
        None => warn(&format!("Dimension PMI {entry} is not scaled.")),
    }
    rescale_ref_labels(refs.0, transform, graph, entry)?;
    rescale_ref_labels(refs.1, transform, graph, entry)?;
    if policy == Some(true) {
        array.change_array1().as_mut_slice()[1..]
            .iter_mut()
            .for_each(|value| *value *= factor);
        let name = object.get_semantic_name();
        if let Some(name) = name.get() {
            let first = array.array1().as_slice()[0];
            name.assign_cat(" (Rescaled to ");
            name.assign_cat(stream_real(first)?);
            name.assign_cat(")");
        }
    }
    Ok(())
}

/// One dimension of `RescaleGeometry`.
fn rescale_dimension(
    label: &TdfLabel,
    graph: &XCAFDocAssemblyGraph,
    transform: &mut BRepBuilderAPITransform,
    factor: f64,
) -> Result<(), Failure> {
    let entry = entry_text(label);
    let attribute = find::<XCAFDocDimension>(label, &XCAFDocDimension::get_id())?;
    let Some(attribute) = attribute.get() else {
        return Ok(());
    };
    let (mut first, mut second) = (Vec::new(), Vec::new());
    let (mut first_in, mut second_in) = (true, true);
    let mut rescale = false;
    if XCAFDocDimTolTool::get_ref_shape_label(label, &mut first, &mut second)? {
        rescale = should_rescale(&first, graph, &mut first_in)?
            || should_rescale(&second, graph, &mut second_in)?;
    }
    if !rescale {
        warn(&format!("Dimension PMI {entry} is not scaled!"));
        return Ok(());
    }
    let object = attribute.get_object()?;
    let o = rustycad_tklcaf::live(&object, "XCAFDoc_Editor::RescaleGeometry")?;
    rescale_dimension_points(o, factor);
    if !o.get_values().is_null() && (!first_in || !second_in) {
        warn(&format!(
            "Dimension PMI {entry} base shapes do not belong to the rescaled assembly!"
        ));
        return Ok(());
    }
    rescale_values(o, (&first, &second), transform, graph, factor, &entry)?;
    attribute.set_object(&object)
}

/// The GD&T of the document of a label, rescaled.
pub(super) fn rescale_gdt(
    label: &TdfLabel,
    graph: &Handle<XCAFDocAssemblyGraph>,
    transform: &mut BRepBuilderAPITransform,
    factor: f64,
) -> Result<(), Failure> {
    let tool = XCAFDocDocumentTool::dim_tol_tool(label)?;
    let Some(tool) = tool.get() else {
        return Ok(());
    };
    let graph = rustycad_tklcaf::live(graph, "XCAFDoc_Editor::RescaleGeometry")?;
    for dimension in tool.get_dimension_labels()? {
        rescale_dimension(&dimension, graph, transform, factor)?;
    }
    rescale_datums(&tool.get_datum_labels()?, factor)?;
    for dim_tol in tool.get_dim_tol_labels()? {
        warn(&format!(
            "DimTol PMI {} is not scaled.",
            entry_text(&dim_tol)
        ));
    }
    Ok(())
}
