// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_Editor.cxx (RescaleGeometry: the shapes,
// properties and locations, the datums and the notes) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc_Editor::RescaleGeometry`: the parts, sub-shapes, validation properties and
//! occurrence locations of an assembly scaled by a factor, then its GD&T and notes.
use super::color_tool::entry_text;
use super::support::find;
use super::{
    XCAFDocArea, XCAFDocAssemblyGraph, XCAFDocAssemblyGraphNodeType, XCAFDocCentroid, XCAFDocDatum,
    XCAFDocDocumentTool, XCAFDocEditor, XCAFDocLocation, XCAFDocNote, XCAFDocNoteMembers,
    XCAFDocShapeTool, XCAFDocVolume,
};
use rustycad_tkernel::message::Message;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel};
use rustycad_tkmath::gp::{Axis2, Point3, Transform3, Vector3};
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPITransform};

/// `Message::SendFail(text)`.
pub(super) fn fail(text: &str) {
    Message::send_fail_text(&AsciiString::from(text));
}

/// `Message::SendWarning(text)`.
pub(super) fn warn(text: &str) {
    Message::send_warning_text(&AsciiString::from(text));
}

/// `gp_Pnt(thePnt.XYZ() * theScale)`.
pub(super) fn scaled(point: Point3, factor: f64) -> Point3 {
    let mut result = Point3::default();
    result.set_xyz(point.xyz() * factor);
    result
}

/// An axis placement with its location scaled.
pub(super) fn scaled_axis(mut axis: Axis2, factor: f64) -> Axis2 {
    axis.set_location(scaled(axis.location(), factor));
    axis
}

/// The label is a root of the shape tool (or the tool's label), or the rescale is forced.
fn is_rescalable(label: &TdfLabel, tool: &XCAFDocShapeTool, force: bool) -> Result<bool, Failure> {
    if force || tool.label() == *label || tool.get_free_shapes()?.contains(label) {
        return Ok(true);
    }
    fail(&format!(
        "Label {} is not a root. Set ForceIfNotRoot true to rescale forcibly.",
        entry_text(label)
    ));
    Ok(false)
}

/// The scaled part: its shape, sub-shapes and validation properties.
fn rescale_part(
    label: &TdfLabel,
    tool: &XCAFDocShapeTool,
    transform: &mut BRepBuilderAPITransform,
    factor: f64,
) -> Result<bool, Failure> {
    let shape = XCAFDocShapeTool::shape_of(label)?;
    transform.perform(&shape, true, true)?;
    if !transform.is_done() {
        fail(&format!("Shape {} is not scaled!", entry_text(label)));
        return Ok(false);
    }
    tool.set_shape(label, &transform.shape()?.clone())?;
    let mut sub_shapes = Vec::new();
    XCAFDocShapeTool::get_sub_shapes(label, &mut sub_shapes)?;
    for sub in &sub_shapes {
        let modified = transform.modified_shape(&XCAFDocShapeTool::shape_of(sub)?)?;
        tool.set_shape(sub, &modified)?;
    }
    rescale_properties(label, factor)?;
    Ok(true)
}

/// The area, centroid and volume of a part, scaled.
fn rescale_properties(label: &TdfLabel, factor: f64) -> Result<(), Failure> {
    if let Some(area) = find::<XCAFDocArea>(label, &XCAFDocArea::get_id())?.get() {
        area.set_value(area.get() * factor * factor)?;
    }
    if let Some(centroid) = find::<XCAFDocCentroid>(label, &XCAFDocCentroid::get_id())?.get() {
        centroid.set_point(&scaled(centroid.get(), factor))?;
    }
    if let Some(volume) = find::<XCAFDocVolume>(label, &XCAFDocVolume::get_id())?.get() {
        volume.set_value(volume.get() * factor * factor * factor)?;
    }
    Ok(())
}

/// The location of an occurrence with its translation scaled.
fn rescale_occurrence(label: &TdfLabel, factor: f64) -> Result<(), Failure> {
    let mut transform = XCAFDocShapeTool::get_location(label)?.transformation();
    let part = transform.translation_part() * factor;
    transform.set_translation_part(Vector3::new(part.x(), part.y(), part.z()));
    XCAFDocLocation::set(label, &Location::new(transform))?;
    Ok(())
}

impl XCAFDocEditor {
    /// `RescaleGeometry(theLabel, theScaleFactor, theForceIfNotRoot)`: scales the parts and
    /// occurrences of the graph of a root label (any label when forced), then its GD&T and
    /// notes; false (with a fail message) on a failure.
    pub fn rescale_geometry(
        label: &TdfLabel,
        factor: f64,
        force_if_not_root: bool,
    ) -> Result<bool, Failure> {
        if label.is_null() {
            fail("Null label.");
            return Ok(false);
        }
        if factor.abs() <= f64::MIN_POSITIVE {
            fail("Scale factor is too small.");
            return Ok(false);
        }
        let tool = XCAFDocDocumentTool::shape_tool(label)?;
        let Some(shapes) = tool.get() else {
            fail("Couldn't find XCAFDoc_ShapeTool attribute.");
            return Ok(false);
        };
        if !is_rescalable(label, shapes, force_if_not_root)? {
            return Ok(false);
        }
        let graph = XCAFDocAssemblyGraph::with_label(label)?;
        if !rescale_graph(&graph, shapes, factor)? {
            return Ok(false);
        }
        shapes.update_assemblies()?;
        let mut transform = BRepBuilderAPITransform::new(scale_transform(factor)?);
        super::editor_rescale_gdt::rescale_gdt(label, &graph, &mut transform, factor)?;
        rescale_notes(label, factor)?;
        Ok(true)
    }
}

/// A scaling transformation.
fn scale_transform(factor: f64) -> Result<Transform3, Failure> {
    let mut transform = Transform3::default();
    transform.set_scale_factor(factor)?;
    Ok(transform)
}

/// The traversal of the graph over its parts and occurrences; false when a part failed.
fn rescale_graph(
    graph: &Handle<XCAFDocAssemblyGraph>,
    tool: &XCAFDocShapeTool,
    factor: f64,
) -> Result<bool, Failure> {
    let graph = rustycad_tklcaf::live(graph, "XCAFDoc_Editor::RescaleGeometry")?;
    let mut transform = BRepBuilderAPITransform::new(scale_transform(factor)?);
    for node in 1..=graph.nb_nodes() {
        let kind = graph.get_node_type(node);
        let label = graph.get_node(node)?;
        match kind {
            XCAFDocAssemblyGraphNodeType::Part => {
                if !rescale_part(&label, tool, &mut transform, factor)? {
                    return Ok(false);
                }
            }
            XCAFDocAssemblyGraphNodeType::Occurrence => rescale_occurrence(&label, factor)?,
            _ => {}
        }
    }
    Ok(true)
}

/// The text points, points and planes of the notes of the document, scaled.
fn rescale_notes(label: &TdfLabel, factor: f64) -> Result<(), Failure> {
    let tool = XCAFDocDocumentTool::notes_tool(label)?;
    let Some(notes) = tool.get() else {
        return Ok(());
    };
    let mut labels = Vec::new();
    notes.get_notes(&mut labels)?;
    for note_label in labels {
        let note = <dyn XCAFDocNote>::get(&note_label)?;
        let Some(note) = note.get() else {
            continue;
        };
        let object = note.get_object()?;
        let o = rustycad_tklcaf::live(&object, "XCAFDoc_Editor::RescaleGeometry")?;
        if o.has_point_text() {
            o.set_point_text(&scaled(o.get_point_text(), factor));
        }
        if o.has_point() {
            o.set_point(&scaled(o.get_point(), factor));
        }
        if o.has_plane() {
            o.set_plane(&scaled_axis(o.get_plane(), factor));
        }
        note.set_object(&object)?;
    }
    Ok(())
}

/// The datums of the document, scaled (their target sizes are not, with a warning).
pub(super) fn rescale_datums(datums: &[TdfLabel], factor: f64) -> Result<(), Failure> {
    for datum_label in datums {
        let datum = find::<XCAFDocDatum>(datum_label, &XCAFDocDatum::get_id())?;
        let Some(datum) = datum.get() else {
            continue;
        };
        let object = datum.get_object()?;
        let o = rustycad_tklcaf::live(&object, "XCAFDoc_Editor::RescaleGeometry")?;
        if o.has_datum_target_params() {
            o.set_datum_target_axis(&scaled_axis(o.get_datum_target_axis(), factor));
            warn(&format!(
                "Datum PMI target length and width {} are not scaled.",
                entry_text(datum_label)
            ));
        }
        if o.has_point_text() {
            o.set_point_text_attach(&scaled(o.get_point_text_attach(), factor));
        }
        if o.has_point() {
            o.set_point(&scaled(o.get_point(), factor));
        }
        if o.has_plane() {
            o.set_plane(&scaled_axis(o.get_plane(), factor));
        }
        datum.set_object(&object)?;
    }
    Ok(())
}
