//! Selection, owners, highlight and sensitive entities, the twin of
//! tools/oracle/tkmeshvs_probe_select.cxx.
use super::build::{mv_ids, new_mesh};
use super::display_driver::SensitiveDisplayDriver;
use super::print::{Reader, attempt, end, flag, int, pnt, real, structure, text, trace};
use super::source::{MV_ELEMENTS, MV_NODES, mv_hmap, mv_source};
use martensite_tkmeshvs::flags::*;
use martensite_tkmeshvs::selection_policy::{
    MeshVSMeshSelectionMethod, mesh_selection_method_from_native,
};
use martensite_tkmeshvs::*;
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::prs_mgr::PrsMgrPresentationManager;
use rustycad_tkv3d::prs3d::Prs3dDrawer;
use rustycad_tkv3d::select_basics::SelectBasicsPickResult;
use rustycad_tkv3d::select_mgr::*;
use rustycad_tkv3d::select3d::*;
use rustycad_tkv3d::v3d::V3dViewer;

fn entity_line(e: &Handle<dyn Select3DSensitiveEntity>) {
    let e = e.get().expect("entity");
    text(e.dynamic_type().name().as_bytes());
    let o = e.owner_id();
    int(o.get().map_or(-1, |o| o.priority()));
    let eo = o.cast::<MeshVSMeshEntityOwner>();
    flag(!eo.is_null());
    if let Some(eo) = eo.get() {
        int(eo.id());
        int(eo.kind().0);
        flag(eo.is_group());
        flag(eo.owner() == 0);
    }
    flag(!o.cast::<dyn MeshVSMeshOwnerSource>().is_null());
    int(e.nb_sub_elements());
    int(e.sensitivity_factor());
    let b = e.bounding_box();
    flag(b.is_valid());
    if b.is_valid() {
        for v in [b.corner_min(), b.corner_max()] {
            real(v.x);
            real(v.y);
            real(v.z);
        }
    }
    pnt(&e.center_of_geometry());
}

const MV_RAYS: [[f64; 2]; 5] = [[0.7, 0.2], [1.5, 0.5], [3.5, 0.5], [1., 0.], [9., 9.]];

fn picks(e: &Handle<dyn Select3DSensitiveEntity>, allow: bool) {
    let e = e.get().expect("entity");
    attempt(|| e.bvh());
    for ray in MV_RAYS {
        let mut m = SelectMgrSelectingVolumeManager::new();
        m.allow_overlap_detection(allow);
        let axis = Axis1::new(
            Point3::new(ray[0], ray[1], 10.),
            Direction3::new(0., 0., -1.).unwrap(),
        );
        m.init_axis_selecting_volume(axis);
        attempt(|| m.build_selecting_volume());
        let mut pick = SelectBasicsPickResult::default();
        attempt(|| {
            let hit = e.matches(&m, &mut pick)?;
            flag(hit);
            if hit {
                real(pick.depth());
                real(pick.dist_to_geom_center());
            }
            Ok(())
        });
    }
}

/// `mv_select Mode Method Variant MaxNodes HiddenNodes HiddenElems Hilighter Selectable`.
pub fn select_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (mode, method, variant, max_nodes) = (r.integer(), r.integer(), r.integer(), r.integer());
    let (hidden_nodes, hidden_elems, hilighter, selectable) =
        (r.integer(), r.integer(), r.flag(), r.integer());
    let mesh = MeshVSMesh::new(false).unwrap();
    let source = mv_source(variant);
    mesh.set_data_source(&source.clone().cast());
    mesh.get_drawer()
        .get()
        .unwrap()
        .set_integer(MeshVSDrawerAttribute::MaxFaceNodes as i32, max_nodes);
    if hilighter {
        mesh.add_builder(
            &Handle::new(MeshVSMeshPrsBuilder::with_defaults(&mesh.clone().cast())).cast(),
            true,
        );
    }
    mesh.set_mesh_sel_method(mesh_selection_method_from_native(method).unwrap());
    mesh.set_hidden_nodes(&mv_ids(hidden_nodes, 1, 14));
    mesh.set_hidden_elems(&mv_ids(hidden_elems, 101, 8));
    if selectable >= 0 {
        mesh.set_selectable_nodes(&mv_ids(selectable, 1, 14));
    }
    let mut first_owner: Handle<dyn SelectMgrEntityOwnerSource> = Handle::default();
    for pass in 0..2 {
        let sel = Handle::new(SelectMgrSelection::new(mode));
        attempt(|| mesh.compute_selection(&sel, mode));
        let entities = sel.entities();
        int(entities.len() as i32);
        for entity in &entities {
            let e = entity.base_sensitive();
            let owner = e.get().unwrap().owner_id();
            if pass == 1 {
                flag(owner.object_address() == first_owner.object_address());
                continue;
            }
            if first_owner.is_null() {
                first_owner = owner;
            }
            entity_line(&e);
            picks(&e, true);
        }
        int(mesh.get_owner_maps(false).extent() as i32);
        int(mesh.get_owner_maps(true).extent() as i32);
    }
    for (key, value) in mesh.get_owner_maps(false).items() {
        int(*key);
        let mo = value.cast::<dyn MeshVSMeshOwnerSource>();
        flag(!mo.is_null());
        if let Some(mo) = mo.get() {
            flag(mo.get_data_source().object_address() == source.object_address());
        }
    }
    flag(mesh.is_whole_mesh_owner(&first_owner));
    flag(mesh.is_whole_mesh_owner(&Handle::default()));
    end();
}

fn prs_line(prs: &Handle<dyn Graphic3dStructureSource>) {
    structure(prs);
    if let Some(p) = prs.get() {
        int(p.get_z_layer());
        int(p.display_priority() as i32);
        flag(p.is_displayed());
    }
}

/// `mv_highlight Mode Variant Key Immediate`.
pub fn highlight_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (mode, variant, key, immediate) = (r.integer(), r.integer(), r.integer(), r.flag());
    let (driver, display) = SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let manager = PrsMgrPresentationManager::new(&viewer.structure_manager());
    let mesh = new_mesh(false, variant);
    mesh.add_builder(
        &Handle::new(MeshVSMeshPrsBuilder::with_defaults(&mesh.clone().cast())).cast(),
        true,
    );
    mesh.set_hidden_nodes(&mv_ids(0, 1, 14));
    let sel = Handle::new(SelectMgrSelection::new(mode));
    attempt(|| mesh.compute_selection(&sel, mode));
    let mut owner: Handle<dyn SelectMgrEntityOwnerSource> = Handle::default();
    for entity in sel.entities() {
        let o = entity.base_sensitive().get().unwrap().owner_id();
        let eo = o.cast::<MeshVSMeshEntityOwner>();
        if owner.is_null() && eo.get().is_none_or(|e| e.id() == key) {
            owner = o;
        }
    }
    flag(owner.is_null());
    if owner.is_null() {
        end();
        return;
    }
    highlight_owner(&mesh, &manager, &owner, immediate);
    trace(&display);
    end();
}

fn highlight_owner(
    mesh: &Handle<MeshVSMesh>,
    manager: &Handle<PrsMgrPresentationManager>,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    immediate: bool,
) {
    let mo = owner.cast::<dyn MeshVSMeshOwnerSource>();
    if let Some(mo) = mo.get() {
        mo.set_detected_entities(&mv_ids(0x2, 1, 14), &mv_ids(0x2, 101, 8));
        mo.add_selected_entities(&mv_ids(0x6, 1, 14), &Handle::default());
        mo.add_selected_entities(&mv_ids(0x9, 1, 14), &mv_ids(0x1, 101, 8));
        flag(mo.is_forced_hilight());
    }
    let style = Handle::new(Prs3dDrawer::new());
    style.set_color(Color::named(NamedColor::ORANGE));
    if immediate {
        manager.begin_immediate_draw();
    }
    let o = owner.get().unwrap();
    flag(o.is_hilighted(manager, 0));
    attempt(|| o.hilight_with_color(manager, &style, 0));
    if let Some(mo) = mo.get() {
        flag(mo.is_forced_hilight());
        mo.unhilight(manager, 0);
        flag(mo.is_forced_hilight());
        mv_hmap(&mo.get_selected_nodes());
        mv_hmap(&mo.get_selected_elements());
    }
    prs_line(&mesh.get_hilight_presentation(&Handle::default()).unwrap());
    attempt(|| mesh.hilight_selected(manager, std::slice::from_ref(owner)));
    prs_line(&mesh.get_select_presentation(&Handle::default()).unwrap());
    mesh.clear_selected();
    prs_line(&mesh.get_select_presentation(&Handle::default()).unwrap());
    if let Some(mo) = mo.get() {
        mo.clear_selected_entities();
        mv_hmap(&mo.get_selected_nodes());
    }
}

fn prism_points() -> Vec<Point3> {
    MV_ELEMENTS[5]
        .nodes
        .iter()
        .map(|&n| {
            let p = MV_NODES[(n - 1) as usize];
            Point3::new(p[0], p[1], p[2])
        })
        .collect()
}
fn pentagon_points() -> Vec<Point3> {
    MV_ELEMENTS[2]
        .nodes
        .iter()
        .map(|&n| {
            let p = MV_NODES[(n - 1) as usize];
            Point3::new(p[0], p[1], p[2])
        })
        .collect()
}

fn sensitive_entity(
    kind: i32,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    mesh: &Handle<MeshVSMesh>,
) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
    let p = prism_points();
    let pent = pentagon_points();
    Ok(match kind {
        0 => Handle::new(MeshVSSensitiveQuad::from_points(owner, &p)?).cast(),
        1 => Handle::new(MeshVSSensitiveQuad::new(
            owner,
            [
                Point3::new(1., 0., 0.),
                Point3::new(2., 0., 0.5),
                Point3::new(2., 1., 0.25),
                Point3::new(1., 1., 0.),
            ],
        ))
        .cast(),
        2 => Handle::new(MeshVSSensitivePolyhedron::new(
            owner,
            &p,
            1,
            &MeshVSDataSource3DBase::create_prism_topology(3),
        )?)
        .cast(),
        3 => Handle::new(MeshVSSensitiveFace::new(
            owner,
            &pent,
            Select3DTypeOfSensitivity::Interior,
        )?)
        .cast(),
        4 => Handle::new(MeshVSSensitiveSegment::new(
            owner,
            Point3::new(1., 0., 0.),
            Point3::new(1., 1., 0.),
        ))
        .cast(),
        5 => Handle::new(MeshVSDummySensitiveEntity::new(owner)).cast(),
        6 => {
            let mesh_owner = MeshVSMeshOwner::new(&mesh.clone().cast(), &mesh.get_data_source(), 5);
            Handle::new(MeshVSSensitiveMesh::new(&mesh_owner.cast(), 3)?).cast()
        }
        7 => Handle::new(MeshVSSensitiveMesh::new(owner, 1)?).cast(),
        8 => Handle::new(MeshVSCommonSensitiveEntity::new(
            owner,
            &**mesh,
            MeshVSMeshSelectionMethod::Nodes,
        )?)
        .cast(),
        9 => Handle::new(MeshVSCommonSensitiveEntity::new(
            owner,
            &**mesh,
            MeshVSMeshSelectionMethod::Precise,
        )?)
        .cast(),
        _ => Handle::new(MeshVSSensitiveFace::new(
            owner,
            &pent,
            Select3DTypeOfSensitivity::Boundary,
        )?)
        .cast(),
    })
}

/// `mv_sensitive Kind Selectable`.
pub fn sensitive_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, selectable) = (r.integer(), r.integer());
    let mesh = new_mesh(false, 3);
    if selectable >= 0 {
        mesh.set_selectable_nodes(&mv_ids(selectable, 1, 14));
    }
    let owner: Handle<dyn SelectMgrEntityOwnerSource> =
        SelectMgrEntityOwner::with_selectable(&mesh.clone().cast(), 2).cast();
    let mut e: Handle<dyn Select3DSensitiveEntity> = Handle::default();
    attempt(|| {
        e = sensitive_entity(kind, &owner, &mesh)?;
        Ok(())
    });
    flag(e.is_null());
    if e.is_null() {
        end();
        return;
    }
    entity_line(&e);
    picks(&e, true);
    picks(&e, false);
    let entity = e.get().unwrap();
    let mut c: Handle<dyn Select3DSensitiveEntity> = Handle::default();
    attempt(|| {
        c = entity.get_connected()?;
        Ok(())
    });
    flag(c.is_null());
    if !c.is_null() {
        entity_line(&c);
    }
    if kind == 6 || kind == 7 {
        int(e.cast::<MeshVSSensitiveMesh>().get().unwrap().get_mode());
    }
    entity.clear();
    flag(entity.owner_id().is_null());
    flag(entity.has_init_location());
    end();
}
