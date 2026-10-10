//! Native selector dump order, including live object counts and depth-limited children.
use super::*;
use crate::select_basics::SelectBasicsSelectingVolumeManager;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkservice::json_dump::{
    class_begin, field_dumped, field_integer, field_pointer, field_real,
};
use std::{io, sync::Arc};

pub(super) fn dump_selector(
    base: &SelectMgrViewerSelectorBase,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    class_begin(stream, "SelectMgr_ViewerSelector")?;
    let fields = base.fields.read();
    field_integer(
        stream,
        "myToPreferClosest",
        i32::from(fields.settings.to_prefer_closest),
    )?;
    field_integer(
        stream,
        "mystored.Extent()",
        fields.results.stored.len() as i32,
    )?;
    field_dumped(stream, depth, "&mySelectingVolumeMgr", &|stream, depth| {
        base.selecting_volume.read().dump_json(stream, depth)
    })?;
    field_pointer(
        stream,
        "&mySelectableObjects",
        &base.selectable_objects as *const _ as usize,
    )?;
    dump_selector_counts(base, &fields, stream)?;
    dump_selector_camera(&fields, stream, depth)?;
    dump_selector_tail(base, &fields, stream)
}
fn dump_selector_tail(
    base: &SelectMgrViewerSelectorBase,
    fields: &SelectMgrViewerSelectorFields,
    stream: &mut DumpStream,
) -> io::Result<()> {
    field_integer(
        stream,
        "myIndexes.Size()",
        fields.results.indexes.len() as i32,
    )?;
    field_integer(
        stream,
        "myIsLeftChildQueuedFirst",
        i32::from(fields.settings.is_left_child_queued_first),
    )?;
    field_integer(
        stream,
        "myMapOfObjectSensitives.Extent()",
        base.object_sensitives.read().len() as i32,
    )?;
    field_integer(
        stream,
        "myStructs.Length()",
        fields.debug_structures.length(),
    )?;
    for structure in fields.debug_structures.iter() {
        field_pointer(stream, "aStructure", structure.object_address() as usize)?;
    }
    Ok(())
}
fn dump_selector_counts(
    base: &SelectMgrViewerSelectorBase,
    fields: &SelectMgrViewerSelectorFields,
    stream: &mut DumpStream,
) -> io::Result<()> {
    let mut count = 0;
    let mut cursor = SelectMgrSelectableObjectSetIterator::new(&base.selectable_objects);
    while cursor.more() {
        count += 1;
        cursor.next();
    }
    field_integer(stream, "aNbOfSelectableObjects", count)?;
    let tolerances = base.tolerances.read();
    field_integer(stream, "myTolerances.Tolerance()", tolerances.tolerance())?;
    field_integer(
        stream,
        "myTolerances.CustomTolerance()",
        tolerances.custom_tolerance(),
    )?;
    field_integer(
        stream,
        "myZLayerOrderMap.Extent()",
        fields.z_layer_order.len() as i32,
    )?;
    field_pointer(
        stream,
        "myEntitySetBuilder.get()",
        fields
            .entity_set_builder
            .as_ref()
            .map_or(0, |builder| Arc::as_ptr(builder) as *const () as usize),
    )
}
fn dump_selector_camera(
    fields: &SelectMgrViewerSelectorFields,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    field_dumped(stream, depth, "&myCameraEye", &|stream, _| {
        fields.settings.camera_eye.dump_json(stream)
    })?;
    field_dumped(stream, depth, "&myCameraDir", &|stream, _| {
        fields.settings.camera_direction.dump_json(stream)
    })?;
    field_real(stream, "myCameraScale", fields.settings.camera_scale)
}
