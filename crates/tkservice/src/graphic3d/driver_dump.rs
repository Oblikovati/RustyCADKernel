//! The `DumpJson` members of the driver model: `Graphic3d_CStructure`, `Graphic3d_Group`,
//! `Graphic3d_Structure`, `Graphic3d_StructureManager`, `Graphic3d_GraphicDriver`,
//! `Graphic3d_Layer`, `Graphic3d_DataStructureManager` and `Graphic3d_CView`.
use super::layer::Graphic3dLayer;
use super::{
    Graphic3dCStructure, Graphic3dCView, Graphic3dGraphicDriver,
    Graphic3dGraphicDriverMembers as _, Graphic3dGroup, Graphic3dStructureManager,
    Graphic3dStructureSource,
};
use crate::json_dump::{
    base_class, class_begin, field_dumped, field_integer, field_pointer, field_real,
};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Handle, WeakHandle};
use std::io;

/// The address of an object, as `OCCT_DUMP_FIELD_VALUE_POINTER` prints it.
fn address_of<T: ?Sized>(object: &T) -> usize {
    std::ptr::from_ref(object).cast::<()>() as usize
}

/// `OCCT_DUMP_FIELD_VALUES_DUMPED` of a handle: nothing for a null handle.
fn dumped_handle<T: ?Sized + rustycad_tkernel::standard::RuntimeTyped>(
    stream: &mut DumpStream,
    depth: i32,
    field: &str,
    handle: &Handle<T>,
    dump: &dyn Fn(&T, &mut DumpStream, i32) -> io::Result<()>,
) -> io::Result<()> {
    match handle.get() {
        Some(value) => field_dumped(stream, depth, field, &|o, d| dump(value, o, d)),
        None => Ok(()),
    }
}

/// The body of `Graphic3d_CStructure::DumpJson`.
pub(crate) fn dump_cstructure<C: Graphic3dCStructure + ?Sized>(
    cstructure: &C,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    let f = cstructure.cstructure_base().fields.read().clone();
    class_begin(stream, "Graphic3d_CStructure")?;
    for group in &f.groups {
        dumped_handle(stream, depth, "aGroup.get()", group, &|g, o, d| {
            g.dump_json(o, d)
        })?;
    }
    for (name, value) in [
        ("myId", f.id),
        ("myZLayer", f.z_layer),
        ("myPriority", f.priority as i32),
        ("myPreviousPriority", f.previous_priority as i32),
    ] {
        field_integer(stream, name, value)?;
    }
    let flags = [
        ("IsInfinite", f.is_infinite),
        ("stick", f.stick),
        ("highlight", f.highlight),
        ("visible", f.visible),
        ("HLRValidation", f.hlr_validation),
        ("IsForHighlight", f.is_for_highlight),
        ("IsMutable", f.is_mutable),
        ("Is2dText", f.is_2d_text),
    ];
    for (name, value) in flags {
        field_integer(stream, name, i32::from(value))?;
    }
    field_dumped(stream, depth, "&myBndBox", &|o, _| f.bnd_box.dump_json(o))?;
    dumped_handle(stream, depth, "myTrsf.get()", &f.trsf, &|t, o, d| {
        t.dump_json(o, d)
    })?;
    dumped_handle(
        stream,
        depth,
        "myTrsfPers.get()",
        &f.trsf_pers,
        &|t, o, d| t.dump_json(o, d),
    )?;
    dumped_handle(
        stream,
        depth,
        "myClipPlanes.get()",
        &f.clip_planes,
        &|t, o, d| t.dump_json(o, d),
    )?;
    dumped_handle(
        stream,
        depth,
        "myHighlightStyle.get()",
        &f.highlight_style,
        &|t, o, d| t.dump_json(o, d),
    )?;
    field_integer(stream, "myIsCulled", i32::from(f.is_culled))?;
    field_integer(stream, "myBndBoxClipCheck", i32::from(f.bnd_box_clip_check))
}

/// The body of `Graphic3d_Group::DumpJson`.
pub(crate) fn dump_group<G: Graphic3dGroup + ?Sized>(
    group: &G,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    let f = group.group_base().fields.read().clone();
    class_begin(stream, "Graphic3d_Group")?;
    field_pointer(stream, "this", address_of(group))?;
    dumped_handle(
        stream,
        depth,
        "myTrsfPers.get()",
        &f.trsf_pers,
        &|t, o, d| t.dump_json(o, d),
    )?;
    field_pointer(stream, "myStructure", f.structure.object_address() as usize)?;
    field_dumped(stream, depth, "&myBounds", &|o, _| f.bounds.dump_json(o))?;
    field_integer(stream, "myIsClosed", i32::from(f.is_closed))
}

/// The body of `Graphic3d_Structure::DumpJson`.
pub(crate) fn dump_structure<S: Graphic3dStructureSource + ?Sized>(
    structure: &S,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    let f = structure.structure_base().fields.read().clone();
    class_begin(stream, "Graphic3d_Structure")?;
    field_pointer(
        stream,
        "myStructureManager",
        f.structure_manager.object_address() as usize,
    )?;
    dumped_handle(
        stream,
        depth,
        "myCStructure.get()",
        &f.cstructure,
        &|c, o, d| c.dump_json(o, d),
    )?;
    for ancestor in &f.ancestors {
        field_pointer(stream, "anAncestor", ancestor.object_address() as usize)?;
    }
    for descendant in &f.descendants {
        field_pointer(stream, "aDescendant", descendant.object_address() as usize)?;
    }
    field_pointer(stream, "myOwner", f.owner.object_address() as usize)?;
    field_integer(stream, "myVisual", f.visual as i32)?;
    field_integer(stream, "myComputeVisual", f.compute_visual as i32)
}

/// The body of `Graphic3d_StructureManager::DumpJson`.
pub(crate) fn dump_structure_manager(
    manager: &Graphic3dStructureManager,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    class_begin(stream, "Graphic3d_StructureManager")?;
    for structure in manager.displayed_structures() {
        dumped_handle(
            stream,
            depth,
            "aDisplayedStructure.get()",
            &structure,
            &|s, o, d| s.dump_json(o, d),
        )?;
    }
    for structure in manager.highlighted_structures() {
        dumped_handle(
            stream,
            depth,
            "aHighlightedStructure.get()",
            &structure,
            &|s, o, d| s.dump_json(o, d),
        )?;
    }
    let affinities: Vec<_> = manager
        .fields
        .read()
        .registered_objects
        .values()
        .cloned()
        .collect();
    for affinity in &affinities {
        dumped_handle(
            stream,
            depth,
            "aRegisteredObject.get()",
            affinity,
            &|a, o, d| a.dump_json(o, d),
        )?;
    }
    dumped_handle(
        stream,
        depth,
        "myGraphicDriver.get()",
        &manager.graphic_driver(),
        &|g, o, d| g.dump_json(o, d),
    )?;
    for view in manager.defined_views() {
        dumped_handle(stream, depth, "aDefinedView", &view, &|v, o, d| {
            v.cview_dump_json(o, d)
        })?;
    }
    field_integer(
        stream,
        "myDeviceLostFlag",
        i32::from(manager.is_device_lost()),
    )
}

/// The body of `Graphic3d_GraphicDriver::DumpJson`.
pub(crate) fn dump_graphic_driver<D: Graphic3dGraphicDriver + ?Sized>(
    driver: &D,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    let (gen_id, layers) = {
        let fields = driver.graphic_driver_base().fields.read();
        (fields.struct_gen_id.clone(), fields.layers.clone())
    };
    class_begin(stream, "Graphic3d_GraphicDriver")?;
    field_dumped(stream, depth, "&myStructGenId", &|o, d| {
        gen_id.dump_json(o, d)
    })?;
    for layer in &layers {
        dumped_handle(stream, depth, "aLayer.get()", layer, &|l, o, d| {
            l.dump_json(o, d)
        })?;
    }
    Ok(())
}

impl Graphic3dLayer {
    /// `DumpJson`: the counts, the structure addresses, the settings and the cached boxes;
    /// `layer.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let state = self.state.read();
        class_begin(stream, "Graphic3d_Layer")?;
        field_pointer(stream, "this", address_of(self))?;
        field_integer(stream, "myLayerId", self.layer_id())?;
        field_integer(stream, "myNbStructures", state.nb_structures)?;
        field_integer(
            stream,
            "myNbStructuresNotCulled",
            state.nb_structures_not_culled,
        )?;
        for structure in state.array.iter().flat_map(|map| map.iter()) {
            field_pointer(
                stream,
                "aStructure",
                WeakHandle::object_address(structure) as usize,
            )?;
        }
        field_dumped(stream, depth, "&myLayerSettings", &|o, d| {
            state.layer_settings.dump_json(o, d)
        })?;
        field_integer(
            stream,
            "myBVHIsLeftChildQueuedFirst",
            i32::from(state.bvh_is_left_child_queued_first),
        )?;
        field_integer(
            stream,
            "myIsBVHPrimitivesNeedsReset",
            i32::from(state.is_bvh_primitives_needs_reset),
        )?;
        field_integer(
            stream,
            "myIsBoundingBoxNeedsReset[0]",
            i32::from(state.is_bounding_box_needs_reset[0]),
        )?;
        field_integer(
            stream,
            "myIsBoundingBoxNeedsReset[1]",
            i32::from(state.is_bounding_box_needs_reset[1]),
        )?;
        field_dumped(stream, depth, "&myBoundingBox[0]", &|o, _| {
            state.bounding_box[0].dump_json(o)
        })?;
        field_dumped(stream, depth, "&myBoundingBox[1]", &|o, _| {
            state.bounding_box[1].dump_json(o)
        })
    }
}

/// The body of `Graphic3d_DataStructureManager::DumpJson`.
pub(crate) fn dump_data_structure_manager(stream: &mut DumpStream) -> io::Result<()> {
    class_begin(stream, "Graphic3d_DataStructureManager")
}

/// The body of `Graphic3d_CView::DumpJson`.
pub(crate) fn dump_cview<V: Graphic3dCView + ?Sized>(
    view: &V,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    let f = view.cview_base().fields.read();
    class_begin(stream, "Graphic3d_CView")?;
    base_class(stream, depth, &|o, _| dump_data_structure_manager(o))?;
    field_integer(stream, "myId", f.id)?;
    field_dumped(stream, depth, "&myRenderParams", &|o, d| {
        f.render_params.dump_json(o, d)
    })?;
    field_dumped(stream, depth, "&myBgColor", &|o, _| f.bg_color.dump_json(o))?;
    field_pointer(
        stream,
        "myStructureManager",
        f.structure_manager.object_address() as usize,
    )?;
    dumped_handle(stream, depth, "myCamera.get()", &f.camera, &|c, o, d| {
        c.dump_json(o, d)
    })?;
    for structure in &f.structs_to_compute {
        dumped_handle(
            stream,
            depth,
            "aStructToCompute.get()",
            structure,
            &|s, o, d| s.dump_json(o, d),
        )?;
    }
    for structure in &f.structs_computed {
        dumped_handle(
            stream,
            depth,
            "aStructComputed.get()",
            structure,
            &|s, o, d| s.dump_json(o, d),
        )?;
    }
    let flags = [
        ("myIsInComputedMode", f.is_in_computed_mode),
        ("myIsActive", f.is_active),
        ("myIsRemoved", f.is_removed),
    ];
    for (name, value) in flags {
        field_integer(stream, name, i32::from(value))?;
    }
    field_integer(stream, "myVisualization", f.visualization as i32)?;
    let cameras = [
        ("myBackXRCamera.get()", &f.back_xr_camera),
        ("myBaseXRCamera.get()", &f.base_xr_camera),
        ("myPosedXRCamera.get()", &f.posed_xr_camera),
        ("myPosedXRCameraCopy.get()", &f.posed_xr_camera_copy),
    ];
    for (name, camera) in cameras {
        dumped_handle(stream, depth, name, camera, &|c, o, d| c.dump_json(o, d))?;
    }
    field_real(stream, "myUnitFactor", f.unit_factor)
}
