// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_Mesh::AcceptDisplayMode` and `MeshVS_Mesh::Compute`: every builder whose flags
//! share a bit with the mode adds the nodes, then the elements, of the data source.
use super::builders::builders_of;
use super::{MeshVSMeshMembers, MeshVSMeshSource};
use crate::MeshVSDrawerAttribute as Key;
use crate::data_source::MeshVSPackedMap;
use crate::prs_builder::MeshVSPrsBuilderMembers;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
use rustycad_tkv3d::prs_mgr::PrsMgrPresentationManager;
use std::time::Instant;

/// The native `MeshVS_Mesh::AcceptDisplayMode` body: a positive mode, accepted by any
/// builder (or by the mesh without builder). Example: `meshvs_mesh_accept_display_mode(&*mesh, 1)`.
pub fn meshvs_mesh_accept_display_mode<O: MeshVSMeshSource + ?Sized>(
    object: &O,
    mode: i32,
) -> bool {
    if mode <= 0 {
        return false;
    }
    let builders = builders_of(object.mesh_base());
    builders.is_empty()
        || builders
            .iter()
            .filter_map(Handle::get)
            .any(|b| b.test_flags(mode))
}

/// The native `MeshVS_Mesh::Compute` body; with `MeshVS_DA_ComputeTime` (or without the
/// key, as natively) the mode and the elapsed time are printed on standard output.
/// Example: `meshvs_mesh_compute(&*mesh, &manager, &prs, 1)?`.
pub fn meshvs_mesh_compute<O: MeshVSMeshSource + ?Sized>(
    object: &O,
    manager: &Handle<PrsMgrPresentationManager>,
    prs: &Handle<dyn Graphic3dStructureSource>,
    mode: i32,
) -> Result<(), Failure> {
    let show_time = drawer_flag(object, Key::ComputeTime as i32);
    let start = Instant::now();
    let source = object.get_data_source();
    let Some(source) = source.get().filter(|_| mode > 0) else {
        return Ok(());
    };
    let nodes = source.get_all_nodes();
    let elements = source.get_all_elements();
    let (mut nodes_to_exclude, mut elements_to_exclude) =
        (MeshVSPackedMap::default(), MeshVSPackedMap::default());
    for builder in builders_of(object.mesh_base())
        .iter()
        .filter_map(Handle::get)
    {
        if !builder.test_flags(mode) {
            continue;
        }
        builder.set_presentation_manager(manager);
        if !nodes.is_empty() {
            builder.build(prs, &nodes, &mut nodes_to_exclude, false, mode)?;
        }
        if !elements.is_empty() {
            builder.build(prs, &elements, &mut elements_to_exclude, true, mode)?;
        }
    }
    if show_time {
        print_time("DisplayMode", "Compute", mode, start);
    }
    Ok(())
}

/// A boolean drawer key that defaults to true when the drawer lacks it (native
/// `bool toShow = true; GetBoolean(key, toShow)`).
pub(super) fn drawer_flag<O: MeshVSMeshSource + ?Sized>(object: &O, key: i32) -> bool {
    let mut value = true;
    let drawer = object.get_drawer();
    if let Some(drawer) = drawer.get() {
        drawer.get_boolean(key, &mut value);
    }
    value
}

/// The native timer report (`OSD_Timer::Show` seconds and CPU seconds).
pub(super) fn print_time(mode_label: &str, label: &str, mode: i32, start: Instant) {
    let seconds = start.elapsed().as_secs_f64();
    println!("{mode_label} : {mode}");
    println!("{label} : {seconds} sec");
    println!("{label} CPU : {seconds} sec\n");
}
