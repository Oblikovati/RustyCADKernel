//! Stateful selection-object traces with a retained tree and iterator per fixture.
use super::{
    common::{Reader, attempt, end, flag, int, text},
    sensitive_entity_set::entity_set_tree,
};
#[path = "../../tests/support/boxed_selectable_scene.rs"]
mod boxed_selectable_scene;
use super::boxed_selectable;
use SelectMgrBvhSubset as Subset;
use boxed_selectable::BoxedSelectable;
use boxed_selectable_scene::{
    boxed_group_transform, boxed_object_transform, boxed_persistence, boxed_scene_groups,
    boxed_scene_manager,
};
use rustycad_tkernel::standard::{Failure, Handle, dump::DumpStream};
use rustycad_tkmath::{
    bvh::{BvhTreeHandle, BvhVec3d},
    gp::Point3,
};
use rustycad_tkservice::graphic3d::{Graphic3dCamera, Graphic3dCameraProjection};
use rustycad_tkv3d::{
    prs_mgr::{PrsMgrPresentableObject, PrsMgrPresentableObjectMembers, PrsMgrPresentationManager},
    select_mgr::*,
};
struct ObjectSetFixture<'a> {
    set: &'a SelectMgrSelectableObjectSet,
    objects: Vec<Handle<BoxedSelectable>>,
    camera: Handle<Graphic3dCamera>,
    manager: Handle<PrsMgrPresentationManager>,
    retained: Vec<BvhTreeHandle<BvhVec3d>>,
    cursor: SelectMgrSelectableObjectSetIterator<'a>,
}
impl<'a> ObjectSetFixture<'a> {
    fn new(
        set: &'a SelectMgrSelectableObjectSet,
        count: i32,
        seed: i32,
        projection: i32,
    ) -> Result<Self, Failure> {
        let mut fixture = Self {
            set,
            objects: Vec::new(),
            camera: Handle::new(Graphic3dCamera::default()),
            manager: boxed_scene_manager(),
            retained: (0..5)
                .map(|i| set.bvh(Subset::try_from(i).unwrap()))
                .collect(),
            cursor: SelectMgrSelectableObjectSetIterator::default(),
        };
        fixture.camera.set_projection_type(if projection == 0 {
            Graphic3dCameraProjection::Orthographic
        } else {
            Graphic3dCameraProjection::Perspective
        });
        fixture.camera.set_scale(12.);
        for index in 0..count {
            let object = BoxedSelectable::new(f64::from(index * 3 - 5), (index + seed) % 7 == 0);
            object.set_transform_persistence(&boxed_persistence((index + seed) % 5)?);
            fixture.objects.push(object);
        }
        Ok(fixture)
    }
    fn object_id(&self, object: &Handle<dyn SelectMgrSelectableObject>) -> i32 {
        self.objects
            .iter()
            .position(|candidate| candidate.object_address() == object.object_address())
            .unwrap() as i32
    }
    fn mutate(&mut self, op: i32, a: i32, b: i32) -> Result<(), Failure> {
        match op {
            0 => flag(self.set.append(&self.objects[a as usize].clone().cast())?),
            1 => flag(self.set.remove(&self.objects[a as usize].clone().cast())?),
            2 => self.objects[a as usize].set_transform_persistence(&boxed_persistence(b)?),
            3 => self
                .set
                .change_subset(&self.objects[a as usize].clone().cast())?,
            4 => self.set.mark_dirty(),
            5 => self.set.update_bvh(
                &if a == 0 {
                    Handle::default()
                } else {
                    self.camera.clone()
                },
                [800 + b * 17, 600 + b * 11],
            )?,
            6 => self
                .camera
                .set_eye(&Point3::new(f64::from(a), f64::from(b), 10.))?,
            7 => self.camera.set_scale(f64::from(a + 1)),
            8 => self.camera.set_projection_type(if a % 2 == 0 {
                Graphic3dCameraProjection::Orthographic
            } else {
                Graphic3dCameraProjection::Perspective
            }),
            9 => self.objects[a as usize].set_bounds(f64::from(b), b == 99),
            _ => self.other_operation(op, a, b)?,
        }
        Ok(())
    }
    fn other_operation(&mut self, op: i32, a: i32, b: i32) -> Result<(), Failure> {
        match op {
            10 => boxed_scene_groups(&self.manager, &self.objects[a as usize], b)?,
            11 => self.object_transform(a, b)?,
            12 => self.print_dump(b),
            13 => int(self.object_id(&self.set.get_object_by_id(Subset::try_from(a)?, b)?)),
            14 => {
                for object in &self.objects {
                    self.set.remove(&object.clone().cast())?;
                }
            }
            15 => self.cursor.init(self.set),
            16 => flag(self.cursor.more()),
            17 => self.cursor.next(),
            18 => int(self.object_id(&self.cursor.value()?)),
            19 => self.camera = Handle::new(self.camera.copy_members()),
            20 => boxed_group_transform(&self.objects[a as usize], b)?,
            21 => {
                let copy = self.set.clone();
                copy.remove(&self.objects[a as usize].clone().cast())?;
                copy.mark_dirty();
                copy.update_bvh(&self.camera, [800 + b * 17, 600 + b * 11])?;
            }
            _ => unreachable!(),
        }
        Ok(())
    }
    fn object_transform(&self, a: i32, b: i32) -> Result<(), Failure> {
        self.objects[a as usize].set_local_transformation(&boxed_object_transform(b)?)
    }
    fn state(&self) -> Result<(), Failure> {
        flag(self.set.is_empty());
        for object in &self.objects {
            flag(self.set.contains(&object.clone().cast()));
        }
        for value in 0..5 {
            let subset = Subset::try_from(value)?;
            flag(self.set.is_subset_empty(subset));
            let entries: Vec<_> = (0..self.objects.len() as i32)
                .map_while(|i| self.set.get_object_by_id(subset, i).ok())
                .collect();
            int(entries.len() as i32);
            for entry in entries {
                int(self.object_id(&entry));
            }
            flag(self.retained[value as usize].ptr_eq(&self.set.bvh(subset)));
            entity_set_tree(&self.retained[value as usize].read())?;
        }
        for object in &self.objects {
            int(object.bounds_calls() as i32);
        }
        Ok(())
    }
    fn print_dump(&self, depth: i32) {
        let mut stream = DumpStream::default();
        self.set.dump_json(&mut stream, depth).unwrap();
        let mut contents = String::from_utf8(stream.bytes().to_vec()).unwrap();
        let prefix = "\"SelectableObject\": \"";
        let mut offset = 0;
        while let Some(start) = contents[offset..].find(prefix) {
            let start = offset + start + prefix.len();
            let finish = start + contents[start..].find('"').unwrap();
            contents.replace_range(start..finish, "present");
            offset = start + 7;
        }
        text(contents.as_bytes());
    }
}
fn object_set_inner(r: &mut Reader<'_>) -> Result<(), Failure> {
    let (count, seed, projection, steps) = (r.integer(), r.integer(), r.integer(), r.integer());
    let set = SelectMgrSelectableObjectSet::new();
    let mut fixture = ObjectSetFixture::new(&set, count, seed, projection)?;
    fixture.state()?;
    for _ in 0..steps {
        let (op, a, b) = (r.integer(), r.integer(), r.integer());
        attempt(|| fixture.mutate(op, a, b));
        fixture.state()?;
    }
    Ok(())
}
pub fn selectable_object_set_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| object_set_inner(&mut reader));
    end();
}
