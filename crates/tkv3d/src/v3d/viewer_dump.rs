//! `V3d_Viewer::DumpJson`.
use super::V3dViewer;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkservice::aspect::AspectGrid as _;
use rustycad_tkservice::graphic3d::Graphic3dGraphicDriverMembers as _;
use rustycad_tkservice::json_dump::{class_begin, field_dumped, field_integer, field_real};
use std::io;

impl V3dViewer {
    /// `DumpJson`: the driver, the manager, the views, the lights and the defaults.
    /// Example: `viewer.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "V3d_Viewer")?;
        let state = self.state.read().clone();
        let (driver, manager) = (self.driver(), self.structure_manager());
        if let Some(driver) = driver.get() {
            field_dumped(stream, depth, "myDriver.get()", &|s, d| {
                driver.dump_json(s, d)
            })?;
        }
        if let Some(manager) = manager.get() {
            field_dumped(stream, depth, "myStructureManager.get()", &|s, d| {
                manager.dump_json(s, d)
            })?;
        }
        field_dumped(stream, depth, "&myZLayerGenId", &|s, d| {
            state.z_layer_gen_id.dump_json(s, d)
        })?;
        for view in state.defined_views.iter().filter_map(|v| v.get()) {
            field_dumped(stream, depth, "aDefinedView.get()", &|s, d| {
                view.dump_json(s, d)
            })?;
        }
        for view in state.active_views.iter().filter_map(|v| v.get()) {
            field_dumped(stream, depth, "anActiveView.get()", &|s, d| {
                view.dump_json(s, d)
            })?;
        }
        for light in state.defined_lights.iter().filter_map(|l| l.get()) {
            field_dumped(stream, depth, "aDefinedLight.get()", &|s, d| {
                light.dump_json(s, d)
            })?;
        }
        for light in state.active_lights.iter().filter_map(|l| l.get()) {
            field_dumped(stream, depth, "anActiveLight.get()", &|s, d| {
                light.dump_json(s, d)
            })?;
        }
        field_dumped(stream, depth, "&myBackground", &|s, d| {
            state.background.dump_json(s, d)
        })?;
        field_dumped(stream, depth, "&myGradientBackground", &|s, d| {
            state.gradient_background.dump_json(s, d)
        })?;
        field_real(stream, "myViewSize", state.view_size)?;
        field_integer(stream, "myViewProj", state.view_proj as i32)?;
        field_integer(stream, "myVisualization", state.visualization as i32)?;
        field_integer(
            stream,
            "myDefaultTypeOfView",
            state.default_type_of_view as i32,
        )?;
        field_dumped(stream, depth, "&myDefaultRenderingParams", &|s, d| {
            state.default_rendering_params.dump_json(s, d)
        })?;
        field_integer(stream, "myComputedMode", i32::from(state.computed_mode))?;
        field_integer(
            stream,
            "myDefaultComputedMode",
            i32::from(state.default_computed_mode),
        )?;
        field_dumped(stream, depth, "&myPrivilegedPlane", &|s, _| {
            state.privileged_plane.dump_json(s)
        })?;
        dump_grid_fields(stream, depth, &state)
    }
}

/// The privileged plane structure, the grids and the grid echo fields.
fn dump_grid_fields(
    stream: &mut DumpStream,
    depth: i32,
    state: &super::viewer::V3dViewerState,
) -> io::Result<()> {
    if let Some(structure) = state.plane_structure.get() {
        field_dumped(stream, depth, "myPlaneStructure.get()", &|s, d| {
            structure.dump_json(s, d)
        })?;
    }
    field_integer(stream, "myDisplayPlane", i32::from(state.display_plane))?;
    field_real(stream, "myDisplayPlaneLength", state.display_plane_length)?;
    if let Some(grid) = state.r_grid.get() {
        field_dumped(stream, depth, "myRGrid.get()", &|s, d| grid.dump_json(s, d))?;
    }
    if let Some(grid) = state.c_grid.get() {
        field_dumped(stream, depth, "myCGrid.get()", &|s, d| grid.dump_json(s, d))?;
    }
    field_integer(stream, "myGridType", state.grid_type as i32)?;
    field_integer(stream, "myGridEcho", i32::from(state.grid_echo))?;
    if let Some(structure) = state.grid_echo_structure.get() {
        field_dumped(stream, depth, "myGridEchoStructure.get()", &|s, d| {
            structure.dump_json(s, d)
        })?;
    }
    if let Some(group) = state.grid_echo_group.get() {
        field_dumped(stream, depth, "myGridEchoGroup.get()", &|s, d| {
            group.dump_json(s, d)
        })?;
    }
    if let Some(aspect) = state.grid_echo_aspect.get() {
        field_dumped(stream, depth, "myGridEchoAspect.get()", &|s, d| {
            rustycad_tkservice::graphic3d::Graphic3dAspectsSource::dump_json(aspect, s, d)
        })?;
    }
    let vertex = state.grid_echo_last_vert;
    field_dumped(stream, depth, "&myGridEchoLastVert", &|s, d| {
        vertex.dump_json(s, d)
    })
}
