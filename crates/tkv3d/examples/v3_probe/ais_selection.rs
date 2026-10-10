//! Stateful selection traces compare callbacks and every public state observer.
#[path = "../../tests/support/ais_selection_fixture.rs"]
pub(super) mod fixture;
use super::{
    boxed_selectable::BoxedSelectable,
    common::{Reader, attempt, end, flag, int},
};
use fixture::{AisStateFilter, AisStateOwner, selection_cursor_tag};
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{Failure, Handle},
};
use rustycad_tkv3d::{ais::*, select_mgr::*};
use std::sync::atomic::Ordering;
struct SelectionScene {
    selection: Handle<AisSelection>,
    object: Handle<dyn SelectMgrSelectableObject>,
    owners: Vec<Handle<AisStateOwner>>,
    filter: Handle<AisStateFilter>,
}
impl SelectionScene {
    fn new() -> Self {
        let selection = Handle::new(AisSelection::new());
        let object = BoxedSelectable::new(0., false).cast();
        let owners: Vec<_> = (0..4).map(|i| AisStateOwner::new(&object, i)).collect();
        for owner in &owners {
            *owner.selection.write() = selection.cast::<dyn AisSelectionSource>().downgrade();
        }
        let filter = AisStateFilter::new(&selection.cast(), &owners[3].cast());
        Self {
            selection,
            object,
            owners,
            filter,
        }
    }
    fn owner(&self, index: i32) -> AisSelectedOwner {
        self.owners
            .get(index as usize)
            .map_or_else(Handle::default, Handle::cast)
    }
    fn operation(&self, op: i32, a: i32, b: i32, c: i32) -> Result<(), Failure> {
        let owner = self.owner(a);
        match op {
            0 => int(self.selection.add_select(&owner)? as i32),
            1 => int(self
                .selection
                .select(&owner, &self.filter.cast(), scheme(b), c != 0)?
                as i32),
            2 => self.selection.clear()?,
            3 => self
                .selection
                .clear_and_select(&owner, &self.filter.cast(), c != 0)?,
            4 => int(self.selection.append_owner(&owner, &self.filter.cast())? as i32),
            5 => self.selection.init(),
            6 => self.filter.mode.store(a, Ordering::Release),
            7 => self.selection.next(),
            8 => self.owners[a as usize]
                .forced
                .store(c != 0, Ordering::Release),
            9 => owner.set_selectable(&if c != 0 {
                self.object.clone()
            } else {
                Handle::default()
            }),
            10 => owner.set_selected(c != 0),
            11 => self.batch(a, b, c)?,
            _ => unreachable!(),
        }
        Ok(())
    }
    fn batch(&self, a: i32, b: i32, c: i32) -> Result<(), Failure> {
        let owners = [self.owner(a), self.owner(c)];
        let picked = Array1::from_shared(&owners, -3);
        self.selection
            .select_owners(&picked, scheme(b), a % 2 == 0, &self.filter.cast())
    }
    fn state(&self) {
        int(self.selection.extent());
        flag(self.selection.is_empty());
        int(selection_cursor_tag(&*self.selection));
        let fields = self.selection.selection_base().fields.read();
        int(fields.positions.len() as i32);
        int(fields.positions.bucket_count() as i32);
        drop(fields);
        for owner in self.selection.objects().iter() {
            int(owner.priority());
        }
        for owner in &self.owners {
            flag(self.selection.is_selected(&owner.cast()));
            flag(owner.is_selected());
            flag(owner.has_selectable());
            let mut observed = owner.observed.write();
            int(observed.len() as i32);
            for tag in observed.iter() {
                int(*tag);
            }
            observed.clear();
        }
        let mut observed = self.filter.observed.write();
        int(observed.len() as i32);
        for (tag, size, cursor) in observed.iter() {
            int(*tag);
            int(*size);
            int(*cursor);
        }
        observed.clear();
    }
}
fn scheme(value: i32) -> AisSelectionScheme {
    match value {
        -1 => AisSelectionScheme::Unknown,
        0 => AisSelectionScheme::Replace,
        1 => AisSelectionScheme::Add,
        2 => AisSelectionScheme::Remove,
        3 => AisSelectionScheme::Xor,
        4 => AisSelectionScheme::Clear,
        5 => AisSelectionScheme::ReplaceExtra,
        _ => unreachable!(),
    }
}
pub fn ais_selection_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let scene = SelectionScene::new();
    scene.state();
    for _ in 0..r.integer() {
        let (op, a, b, c) = (r.integer(), r.integer(), r.integer(), r.integer());
        attempt(|| scene.operation(op, a, b, c));
        scene.state();
    }
    end();
}
