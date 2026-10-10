//! Native set mutation traces, live tree identity and inherited runtime behavior.
use super::common::{Reader, attempt, end, flag, int, real, text};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkmath::{
    bvh::*,
    gp::{Axis2, Point3},
};
use rustycad_tkservice::graphic3d::{Graphic3dTransModeFlags, Graphic3dTransformPers};
use rustycad_tkv3d::{
    select_mgr::*,
    select3d::{Select3DSensitiveEntity, Select3DSensitivePoint},
};
use std::sync::Arc;
#[path = "../../tests/support/rejected_sensitive.rs"]
mod rejected_sensitive;
fn entity_set_builder(kind: i32) -> BvhBuilderHandle<BvhVec3d> {
    match kind {
        1 => Some(Arc::new(BvhBinnedBuilder::<BvhVec3d, 32>::new(
            2, 32, false, 1,
        ))),
        2 => Some(Arc::new(BvhLinearBuilder::<BvhVec3d>::new(2, 32))),
        _ => None,
    }
}
fn entity_set_flags(entity: &Handle<SelectMgrSensitiveEntity>, flags: i32) -> Result<(), Failure> {
    let persistence = if flags & 1 != 0 {
        Handle::new(Graphic3dTransformPers::with_point(
            Graphic3dTransModeFlags::ZOOM_PERS,
            &Point3::default(),
        )?)
        .cast()
    } else {
        Handle::default()
    };
    entity
        .base_sensitive()
        .set_transform_persistence(&persistence);
    entity
        .base_sensitive()
        .set_flipping_options(flags & 2 != 0, &Axis2::default());
    Ok(())
}
struct EntitySetFixture {
    set: Handle<SelectMgrSensitiveEntitySet>,
    entities: Vec<Handle<SelectMgrSensitiveEntity>>,
    owners: Vec<Handle<dyn SelectMgrEntityOwnerSource>>,
    selection: Handle<SelectMgrSelection>,
    retained: BvhTreeHandle<BvhVec3d>,
}
impl EntitySetFixture {
    fn new(count: i32, builder: i32, rejected: bool, flags: i32) -> Result<Self, Failure> {
        let set = Handle::new(SelectMgrSensitiveEntitySet::new(entity_set_builder(
            builder,
        )));
        let mut fixture = Self {
            retained: set.bvh()?,
            set,
            entities: Vec::new(),
            owners: vec![
                SelectMgrEntityOwner::new(1).cast(),
                SelectMgrEntityOwner::new(2).cast(),
                Handle::default(),
            ],
            selection: Handle::new(SelectMgrSelection::new(0)),
        };
        for index in 0..count {
            fixture.add_fixture_entity(index, rejected && index == count - 1, flags)?;
        }
        Ok(fixture)
    }
    fn add_fixture_entity(
        &mut self,
        index: i32,
        rejected: bool,
        flags: i32,
    ) -> Result<(), Failure> {
        let point = Point3::new(
            f64::from((index * 17) % 13 - 6),
            f64::from(index % 3),
            f64::from(index % 5),
        );
        let owner = &self.owners[index as usize % 3];
        let base: Handle<dyn Select3DSensitiveEntity> = if rejected {
            Handle::new(rejected_sensitive::RejectedSensitivePoint::new(
                owner, point,
            ))
            .cast()
        } else {
            Handle::new(Select3DSensitivePoint::new(owner, point)).cast()
        };
        let entity = Handle::new(SelectMgrSensitiveEntity::new(&base));
        entity.set_active_for_selection();
        entity_set_flags(&entity, (index + flags) % 4)?;
        self.entities.push(entity);
        Ok(())
    }
    fn choose_selection(&self, first: i32, count: i32) {
        self.selection.change_entities(&mut |items| {
            items.clear();
            for offset in 0..count {
                items.push(self.entities[(first + offset) as usize % self.entities.len()].clone());
            }
        });
    }
    fn build(&self) -> Result<(), Failure> {
        // Native null-builder access is undefined; only compare its clean-tree path.
        let defined = self.set.builder().is_some() || !self.set.is_dirty();
        flag(defined);
        if defined {
            flag(self.retained.ptr_eq(&self.set.bvh()?));
        }
        Ok(())
    }
    fn mutate(&self, operation: i32, first: i32, second: i32) -> Result<(), Failure> {
        match operation {
            0 => self.set.append(&self.entities[first as usize])?,
            1 | 2 => {
                self.choose_selection(first, second);
                if operation == 1 {
                    self.set.append_selection(&self.selection)?;
                } else {
                    self.set.remove(&self.selection)?;
                }
            }
            3 => entity_set_flags(&self.entities[first as usize], second)?,
            4 => self.set.swap(first, second)?,
            5 => self.set.mark_dirty(),
            6 => self.build()?,
            7 => self.set.set_builder(entity_set_builder(first)),
            8 => self.set.add_owner(&self.owners[first as usize]),
            9 => self.set.remove_owner(&self.owners[first as usize]),
            10 => real(self.set.center(first, second)?),
            11 => entity_set_box(&self.set.element_box(first)?),
            12 => entity_set_box(&self.set.bounding_box()?),
            13 => self.entities[first as usize]
                .base_sensitive()
                .set(&self.owners[second as usize]),
            14 => self.set.set_properties(if first != 0 {
                Some(Arc::new(BvhTransform::<f64>::default()))
            } else {
                None
            }),
            _ => panic!("entity set operation={operation}; expected 0..=14"),
        }
        Ok(())
    }
    fn state(&self) -> Result<(), Failure> {
        int(self.set.size());
        flag(self.set.is_dirty());
        flag(self.set.has_entity_with_persistence());
        flag(self.set.has_entity_with_flipping());
        flag(self.set.properties().is_some());
        for entity in self.set.sensitives().iter() {
            int(self
                .entities
                .iter()
                .position(|known| known == entity)
                .unwrap() as i32);
        }
        for owner in &self.owners {
            int(self.set.owners().seek(owner).copied().unwrap_or(0));
        }
        for entity in &self.entities {
            flag(entity.is_active_for_selection());
        }
        entity_set_tree(&self.retained.read())
    }
}
fn entity_set_box(bounds: &BvhBox<BvhVec3d>) {
    flag(bounds.is_valid());
    for corner in [bounds.corner_min(), bounds.corner_max()] {
        real(corner.x);
        real(corner.y);
        real(corner.z);
    }
}
pub(super) fn entity_set_tree(tree: &BvhTree<BvhVec3d>) -> Result<(), Failure> {
    int(tree.length());
    int(tree.depth());
    for index in 0..tree.length() {
        let node = rustycad_tkmath::bvh::array::value(tree.node_info_buffer(), index)?;
        for value in [node.x, node.y, node.z, node.w] {
            int(value);
        }
        for point in [tree.min_point(index)?, tree.max_point(index)?] {
            real(point.x);
            real(point.y);
            real(point.z);
        }
    }
    Ok(())
}
fn entity_set_inner(r: &mut Reader<'_>) -> Result<(), Failure> {
    let (count, builder, rejected, flags) = (r.integer(), r.integer(), r.flag(), r.integer());
    let fixture = EntitySetFixture::new(count, builder, rejected, flags)?;
    text(fixture.set.dynamic_type().unwrap().name().as_bytes());
    text(
        SelectMgrSensitiveEntitySet::static_type()
            .parent()
            .unwrap()
            .name()
            .as_bytes(),
    );
    flag(fixture.set.is_kind_name("BVH_ObjectTransient"));
    flag(fixture.set.is_kind_name("Standard_Transient"));
    fixture.state()?;
    for _ in 0..r.integer() {
        let (operation, first, second) = (r.integer(), r.integer(), r.integer());
        attempt(|| fixture.mutate(operation, first, second));
        fixture.state()?;
    }
    Ok(())
}
/// Compare one independent mutation trace; `sensitive_entity_set_case(&values)`.
pub fn sensitive_entity_set_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| entity_set_inner(&mut r));
    end();
}
