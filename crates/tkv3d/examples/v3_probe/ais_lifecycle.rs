//! Compare real AIS bulk, removal and query operations with the pinned native context.
use super::{
    ais_display::{display_state_rows, operate},
    ais_load::{LoadScene, display_state},
    common::{Reader, attempt, end, int},
};
use rustycad_tkernel::{
    ncollection::List,
    standard::{Failure, Handle},
};
use rustycad_tkv3d::ais::*;
pub fn ais_lifecycle_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    let mut scene = LoadScene::new();
    for _ in 0..reader.integer() {
        let (op, index) = (reader.integer(), reader.integer());
        let arguments = std::array::from_fn(|_| reader.integer());
        attempt(|| {
            if op == 59 {
                align_object_order(&mut scene)
            } else {
                lifecycle_operation(&scene, op, index as usize, arguments)
            }
        });
        display_state_rows(&scene);
        super::ais_lifecycle_detection::detection_state(&scene);
    }
    end();
}
fn lifecycle_operation(
    scene: &LoadScene,
    op: i32,
    index: usize,
    values: [i32; 4],
) -> Result<(), Failure> {
    let [a, b, c, d] = values;
    let object = scene.objects[index].cast();
    match op {
        40 => scene.ctx.erase_all(a != 0)?,
        41 => scene.ctx.display_all(a != 0)?,
        42 => scene.ctx.display_selected(a != 0)?,
        43 => scene.ctx.erase_selected(a != 0)?,
        44 => scene.ctx.remove(&object, a != 0)?,
        45 => scene.ctx.remove_all(a != 0)?,
        46 => scene.ctx.clear_global(&object, a != 0)?,
        47 => scene.ctx.clear_global_prs(&object, a, b != 0)?,
        48 => scene.ctx.clear_prs(&object, a, b != 0)?,
        49 => scene.ctx.display_legacy(
            &object,
            a,
            b,
            c != 0,
            index.is_multiple_of(2),
            display_state(d),
        )?,
        61 => scene.objects[index].expected_kind_references.store(
            object.transient_base().ref_count(),
            std::sync::atomic::Ordering::Relaxed,
        ),
        58 => super::ais_lifecycle_detection::seed_detection(scene, a, b),
        50..=57 => query_objects(scene, op, values)?,
        _ => operate(scene, op, index, values)?,
    }
    Ok(())
}
fn query_objects(scene: &LoadScene, op: i32, values: [i32; 4]) -> Result<(), Failure> {
    let [a, b, c, d] = values;
    let kind = match a {
        0 => AisKindOfInteractive::None,
        1 => AisKindOfInteractive::Datum,
        _ => AisKindOfInteractive::Shape,
    };
    let mut objects = List::default();
    objects.append(scene.objects[2].cast());
    match op {
        50 => scene.ctx.displayed_objects(&mut objects)?,
        51 => scene.ctx.erased_objects(&mut objects)?,
        52 => scene.ctx.displayed_objects_by_kind(kind, b, &mut objects)?,
        53 => scene.ctx.erased_objects_by_kind(kind, b, &mut objects)?,
        54 => scene
            .ctx
            .objects_by_display_status(display_state(c), &mut objects)?,
        55 => {
            scene
                .ctx
                .objects_by_display_status_and_kind(kind, b, display_state(c), &mut objects)?
        }
        56 => scene.ctx.objects_inside(&mut objects, kind, b)?,
        57 => scene.ctx.objects_for_view(
            &mut objects,
            &scene.views[a as usize],
            b != 0,
            display_state(c),
        )?,
        _ => unreachable!(),
    }
    query_result(scene, &objects, d);
    Ok(())
}
fn query_result(scene: &LoadScene, objects: &List<Handle<dyn AisInteractiveObject>>, _: i32) {
    // Handle hashing follows real allocation addresses, so compare multiplicities; Rust tests verify bucket order.
    int(objects.len() as i32);
    int(i32::from(
        objects.first().unwrap() == &scene.objects[2].cast::<dyn AisInteractiveObject>(),
    ));
    for object in &scene.objects {
        int(objects
            .iter()
            .filter(|v| *v == &object.cast::<dyn AisInteractiveObject>())
            .count() as i32);
    }
}

fn align_object_order(scene: &mut LoadScene) -> Result<(), Failure> {
    for object in &scene.objects[..3] {
        scene.ctx.load(&object.cast(), -1)?;
    }
    let ordered: Vec<_> = scene
        .ctx
        .fields
        .read()
        .objects
        .items()
        .map(|(key, _)| key.cast())
        .collect();
    scene.objects[..3].clone_from_slice(&ordered);
    Ok(())
}
