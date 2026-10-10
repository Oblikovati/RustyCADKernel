//! Precise native overlap matching, including client owner-slot replacement.
use super::ais_object::context_object;
#[path = "../../tests/support/overlap_sensitive.rs"]
mod overlap_sensitive;
use super::{
    boxed_selectable::BoxedSelectable,
    common::{Reader, attempt, end, flag, int},
    selection_ranking::criterion_state,
    selector_geometry::{geometry_transform, geometry_volume},
};
use context_object::ContextObject;
use overlap_sensitive::OverlapSensitivePoint;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::{graphic3d::*, null_driver::NullGraphicDriver};
use rustycad_tkv3d::{
    ais::*, prs_mgr::*, select_basics::*, select_mgr::*, select3d::*, v3d::V3dViewer,
};
use std::sync::atomic::Ordering;
fn overlap_object(
    ctx: &Handle<AisInteractiveContext>,
    kind: i32,
    persistent: i32,
    presentation: i32,
    mode: i32,
    index: i32,
) -> Handle<dyn SelectMgrSelectableObject> {
    if kind == 0 {
        return Handle::default();
    }
    let object: Handle<dyn SelectMgrSelectableObject> = if kind == 1 {
        BoxedSelectable::new(0., false).cast()
    } else {
        let ais = ContextObject::new();
        ais.set_context(ctx);
        ais.cast()
    };
    object.set_z_layer(10 + index * 10);
    object.attributes().set_display_mode(mode);
    if persistent != 0 {
        let flags = match persistent {
            1 => Graphic3dTransModeFlags::TMF_2D,
            2 => Graphic3dTransModeFlags(
                Graphic3dTransModeFlags::ORTHO_PERS.0 | Graphic3dTransModeFlags::TMF_2D.0,
            ),
            _ => Graphic3dTransModeFlags::ZOOM_PERS,
        };
        object.set_transform_persistence(
            &Handle::new(Graphic3dTransformPers::new(flags).unwrap()).cast(),
        );
    }
    if presentation != 0 {
        let manager = ctx.main_prs_mgr().clone();
        let prs = manager
            .presentation(&object.cast(), 0, true, &Handle::default())
            .unwrap();
        prs.set_display_priority(if index == 0 {
            Graphic3dDisplayPriority::Above
        } else {
            Graphic3dDisplayPriority::Topmost
        })
        .unwrap();
    }
    object
}
fn overlap_state(
    selector: &SelectMgrViewerSelector,
    owners: &[Handle<dyn SelectMgrEntityOwnerSource>],
    entities: &[Handle<OverlapSensitivePoint>],
) {
    let fields = selector.viewer_selector_base().fields.read();
    flag(fields.results.is_sorted);
    int(fields.results.stored.len() as i32);
    for (owner, criterion) in fields.results.stored.items() {
        int(owners.iter().position(|value| value == owner).unwrap() as i32);
        int(entities
            .iter()
            .position(|value| value.cast::<dyn Select3DSensitiveEntity>() == criterion.entity)
            .unwrap() as i32);
        criterion_state(criterion);
    }
    drop(fields);
    for entity in entities {
        int(owners
            .iter()
            .position(|value| *value == entity.owner_id())
            .unwrap() as i32);
        let mut events = entity.events.write();
        int(events.len() as i32);
        for event in events.iter() {
            int(*event);
        }
        events.clear();
    }
}
fn overlap_step(
    r: &mut Reader<'_>,
    selector: &SelectMgrViewerSelector,
    owners: &[Handle<dyn SelectMgrEntityOwnerSource>],
    entities: &[Handle<OverlapSensitivePoint>],
    volume: &SelectMgrSelectingVolumeManager,
) -> Result<(), Failure> {
    let (id, owner, match_owner, location_owner, decision, pick_kind, transform) = (
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
    );
    let depth = r.real();
    let distance = r.real();
    let entity = &entities[id as usize];
    entity.set(&owners[owner as usize]);
    *entity.match_owner.write() = (match_owner >= 0).then(|| owners[match_owner as usize].clone());
    *entity.location_owner.write() =
        (location_owner >= 0).then(|| owners[location_owner as usize].clone());
    entity.decision.store(decision, Ordering::Release);
    let mut pick = SelectBasicsPickResult::default();
    if pick_kind != 0 {
        pick.set_depth(depth);
    }
    pick.set_dist_to_geom_center(distance);
    if pick_kind == 2 {
        pick.set_picked_point(Point3::new(1., 2., 3.));
    }
    *entity.pick.write() = pick;
    select_mgr_check_overlap(
        selector.viewer_selector_base(),
        &entity.cast(),
        &geometry_transform(transform),
        volume,
    )
}
pub fn selector_overlap_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, persistent, presentation, mode, volume_kind, steps) = (
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
    );
    let ctx =
        AisInteractiveContext::new(&V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap())
            .unwrap();
    let first = overlap_object(&ctx, kind, persistent, presentation, mode, 0);
    let second = overlap_object(&ctx, kind, persistent, presentation, mode, 1);
    let owners: Vec<Handle<dyn SelectMgrEntityOwnerSource>> = vec![
        Handle::default(),
        SelectMgrEntityOwner::with_selectable(&first, 3).cast(),
        SelectMgrEntityOwner::with_selectable(&second, 9).cast(),
        SelectMgrEntityOwner::new(7).cast(),
    ];
    let entities = [
        OverlapSensitivePoint::new(&Handle::default()),
        OverlapSensitivePoint::new(&Handle::default()),
    ];
    let selector = Handle::new(SelectMgrViewerSelector::new());
    for entity in &entities {
        *entity.selector.write() = selector
            .cast::<dyn SelectMgrViewerSelectorSource>()
            .downgrade();
    }
    {
        let mut fields = selector.viewer_selector_base().fields.write();
        fields.z_layer_order.bind(10, 4);
        fields.z_layer_order.bind(20, 7);
        fields.z_layer_order.bind(0, 2);
        fields.results.is_sorted = true;
    }
    let volume = geometry_volume(volume_kind, 1);
    overlap_state(&selector, &owners, &entities);
    for _ in 0..steps {
        attempt(|| overlap_step(&mut r, &selector, &owners, &entities, &volume));
        overlap_state(&selector, &owners, &entities);
    }
    end();
}
