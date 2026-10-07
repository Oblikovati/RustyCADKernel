// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_SelectPointed (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_SelectPointed`: a selection of entities given one by one.
use super::{
    IFSelectSelectBase, IFSelectSelection, IFSelectSelectionIterator, IFSelectTransformer,
    select_base_fill_iterator,
};
use crate::interface::{InterfaceCopyControl, InterfaceEntityIterator, InterfaceGraph};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, SharedState, StandardTransient, exceptions},
    tcollection::AsciiString,
};

/// The fields of `IFSelect_SelectPointed`.
#[derive(Default)]
struct PointedState {
    theset: bool,
    theitems: Vec<Handle<StandardTransient>>,
}

/// `IFSelect_SelectPointed`.
///
/// ```
/// use rustycad_tkxsbase::if_select::IFSelectSelectPointed;
/// use rustycad_tkernel::standard::{Handle, StandardTransient};
/// let sel = IFSelectSelectPointed::new();
/// let ent = Handle::new(StandardTransient::default());
/// assert!(sel.add(&ent) && !sel.add(&ent));
/// assert_eq!((sel.nb_items(), sel.rank(&ent)), (1, 1));
/// ```
#[derive(Default)]
pub struct IFSelectSelectPointed {
    transient: StandardTransient,
    state: SharedState<PointedState>,
}
impl_transient!(IFSelectSelectPointed, "IFSelect_SelectPointed", dyn IFSelectSelectBase, transient, interfaces: [dyn IFSelectSelection, dyn IFSelectSelectBase]);

impl IFSelectSelectPointed {
    /// `IFSelect_SelectPointed()`: not set, no item.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Clear`: no item, not set.
    pub fn clear(&self) {
        let mut st = self.state.write();
        st.theitems.clear();
        st.theset = false;
    }
    /// `IsSet`.
    pub fn is_set(&self) -> bool {
        self.state.read().theset
    }
    /// `SetEntity(item)`: set, with `item` alone (none for null).
    pub fn set_entity(&self, item: &Handle<StandardTransient>) {
        let mut st = self.state.write();
        st.theitems.clear();
        st.theset = true;
        if item.is_some() {
            st.theitems.push(item.clone());
        }
    }
    /// `SetList(list)`: set, with the items of `list` (none for null).
    pub fn set_list(&self, list: Option<&[Handle<StandardTransient>]>) {
        let mut st = self.state.write();
        st.theitems = list.map(<[_]>::to_vec).unwrap_or_default();
        st.theset = true;
    }
    /// `Add(item)`: appends a new item (false for null or present).
    pub fn add(&self, item: &Handle<StandardTransient>) -> bool {
        let mut st = self.state.write();
        if item.is_null() || st.theitems.contains(item) {
            return false;
        }
        st.theitems.push(item.clone());
        st.theset = true;
        true
    }
    /// `Remove(item)`: removes the last occurrence. Native quirk kept: true even when
    /// `item` is not present (false only for null).
    pub fn remove(&self, item: &Handle<StandardTransient>) -> bool {
        if item.is_null() {
            return false;
        }
        let mut st = self.state.write();
        if let Some(pos) = st.theitems.iter().rposition(|i| i == item) {
            st.theitems.remove(pos);
        }
        true
    }
    /// `Toggle(item)`: removes the first occurrence, or appends; true when appended.
    pub fn toggle(&self, item: &Handle<StandardTransient>) -> bool {
        if item.is_null() {
            return false;
        }
        let mut st = self.state.write();
        match st.theitems.iter().position(|i| i == item) {
            Some(pos) => {
                st.theitems.remove(pos);
                false
            }
            None => {
                st.theitems.push(item.clone());
                true
            }
        }
    }
    /// `AddList(list)`: appends the items not present before. Native quirks kept: the
    /// duplicates inside `list` are all appended, and the result is always false.
    pub fn add_list(&self, list: Option<&[Handle<StandardTransient>]>) -> bool {
        let Some(list) = list else { return false };
        let mut st = self.state.write();
        let before = st.theitems.clone();
        st.theitems
            .extend(list.iter().filter(|i| !before.contains(i)).cloned());
        st.theset = true;
        false
    }
    /// `RemoveList(list)`: `Remove` of each item.
    pub fn remove_list(&self, list: Option<&[Handle<StandardTransient>]>) -> bool {
        list.is_some_and(|l| l.iter().fold(false, |res, i| self.remove(i) | res))
    }
    /// `ToggleList(list)`: `Toggle` of each item. Native quirk kept: always true.
    pub fn toggle_list(&self, list: Option<&[Handle<StandardTransient>]>) -> bool {
        list.iter().flat_map(|l| l.iter()).for_each(|i| {
            self.toggle(i);
        });
        true
    }
    /// `Rank(item)`: the rank of the last occurrence (0 when absent or null).
    pub fn rank(&self, item: &Handle<StandardTransient>) -> i32 {
        if item.is_null() {
            return 0;
        }
        self.state
            .read()
            .theitems
            .iter()
            .rposition(|i| i == item)
            .map_or(0, |p| p as i32 + 1)
    }
    /// `NbItems`.
    pub fn nb_items(&self) -> i32 {
        self.state.read().theitems.len() as i32
    }
    /// `Item(num)`: null out of range.
    pub fn item(&self, num: i32) -> Handle<StandardTransient> {
        let st = self.state.read();
        usize::try_from(num - 1)
            .ok()
            .and_then(|i| st.theitems.get(i))
            .cloned()
            .unwrap_or_default()
    }
    /// `Update(control)`: each item replaced by its image (removed without one);
    /// `Standard_NullObject` for a null control.
    pub fn update_control(
        &self,
        control: &Handle<dyn InterfaceCopyControl>,
    ) -> Result<(), Failure> {
        let live = control.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_SelectPointed::Update: null copy control")
        })?;
        self.update_with(|from, to| live.search(from, to))
    }
    /// `Update(trf)`: each item replaced by its image (removed without one).
    pub fn update_transformer(&self, trf: &Handle<dyn IFSelectTransformer>) -> Result<(), Failure> {
        let live = trf.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_SelectPointed::Update: null transformer")
        })?;
        self.update_with(|from, to| Ok(live.updated(from, to)))
    }
    /// The items replaced from the last one by `image` (removed when it fails).
    fn update_with(
        &self,
        image: impl Fn(
            &Handle<StandardTransient>,
            &mut Handle<StandardTransient>,
        ) -> Result<bool, Failure>,
    ) -> Result<(), Failure> {
        let items = self.state.read().theitems.clone();
        let mut kept = items.clone();
        for i in (0..items.len()).rev() {
            let mut to = Handle::default();
            if image(&items[i], &mut to)? {
                kept[i] = to;
            } else {
                kept.remove(i);
            }
        }
        self.state.write().theitems = kept;
        Ok(())
    }
}

impl IFSelectSelection for IFSelectSelectPointed {
    /// `RootResult(G)`: the items of the model of the graph.
    fn root_result(&self, g: &InterfaceGraph) -> Result<InterfaceEntityIterator, Failure> {
        let mut result = InterfaceEntityIterator::new();
        for item in self.state.read().theitems.iter() {
            if g.entity_number(item) > 0 {
                result.get_one_item(item);
            }
        }
        Ok(result)
    }
    fn fill_iterator(&self, iter: &mut IFSelectSelectionIterator) {
        select_base_fill_iterator(iter)
    }
    /// `Label`: "Pointed Entities".
    fn label(&self) -> AsciiString {
        AsciiString::new("Pointed Entities")
    }
}

impl IFSelectSelectBase for IFSelectSelectPointed {}
