// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_SelectSuite,
// IFSelect_SelectExplore and IFSelect_SelectAnyList (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_SelectSuite`: a chain of deductions; `IFSelect_SelectExplore`: a recursive
//! exploration from the input entities; `IFSelect_SelectAnyList`: a rank range in a list
//! carried by one input entity.
use super::{
    IFSelectRankRange, IFSelectSelectDeduct, IFSelectSelectDeductBase, IFSelectSelection,
    IFSelectSelectionIterator, deduct_fill_iterator,
};
use crate::interface::{InterfaceEntityIterator, InterfaceGraph, interface_error};
use rustycad_tkernel::{
    impl_transient, impl_transient_interface,
    ncollection::IndexedMap,
    standard::{Failure, Handle, SharedState, StandardTransient, exceptions},
    tcollection::AsciiString,
};

/// The fields of `IFSelect_SelectSuite`.
#[derive(Default)]
struct SuiteState {
    thelab: AsciiString,
    thesel: Vec<Handle<dyn IFSelectSelectDeduct>>,
}

/// `IFSelect_SelectSuite`: the input result goes through each deduction of the chain in
/// turn (as its alternate list).
///
/// ```
/// use rustycad_tkxsbase::if_select::{IFSelectSelectSuite, IFSelectSelection};
/// assert_eq!(IFSelectSelectSuite::new().label().as_bytes(), b"Suite of 0 Selections");
/// ```
#[derive(Default)]
pub struct IFSelectSelectSuite {
    transient: StandardTransient,
    base: IFSelectSelectDeductBase,
    state: SharedState<SuiteState>,
}
impl_transient!(IFSelectSelectSuite, "IFSelect_SelectSuite", dyn IFSelectSelectDeduct, transient, interfaces: [dyn IFSelectSelection, dyn IFSelectSelectDeduct]);

impl IFSelectSelectSuite {
    /// `IFSelect_SelectSuite()`: empty.
    pub fn new() -> Self {
        Self::default()
    }
    /// `AddInput(item)`: the first deduction (prepended) or the input (false for null,
    /// or when there is an input already).
    pub fn add_input(&self, item: &Handle<dyn IFSelectSelection>) -> bool {
        if item.is_null() || self.input().is_some() {
            return false;
        }
        let first: Handle<dyn IFSelectSelectDeduct> = item.cast();
        if first.is_null() {
            self.set_input(item);
        } else {
            self.state.write().thesel.insert(0, first);
        }
        true
    }
    /// `AddPrevious(item)`: prepends a deduction.
    pub fn add_previous(&self, item: &Handle<dyn IFSelectSelectDeduct>) {
        if item.is_some() {
            self.state.write().thesel.insert(0, item.clone());
        }
    }
    /// `AddNext(item)`: appends a deduction.
    pub fn add_next(&self, item: &Handle<dyn IFSelectSelectDeduct>) {
        if item.is_some() {
            self.state.write().thesel.push(item.clone());
        }
    }
    /// `NbItems`.
    pub fn nb_items(&self) -> i32 {
        self.state.read().thesel.len() as i32
    }
    /// `Item(num)`: `Standard_OutOfRange` out of 1..NbItems.
    pub fn item(&self, num: i32) -> Result<Handle<dyn IFSelectSelectDeduct>, Failure> {
        let st = self.state.read();
        usize::try_from(num - 1)
            .ok()
            .and_then(|i| st.thesel.get(i))
            .cloned()
            .ok_or_else(|| {
                exceptions::out_of_range(&format!(
                    "IFSelect_SelectSuite::Item: {num}; expected [1, {}]",
                    st.thesel.len()
                ))
            })
    }
    /// `SetLabel(lab)`.
    pub fn set_label(&self, lab: &str) {
        self.state.write().thelab = AsciiString::new(lab);
    }
}

impl IFSelectSelectDeduct for IFSelectSelectSuite {
    fn deduct_base(&self) -> &IFSelectSelectDeductBase {
        &self.base
    }
}

impl IFSelectSelection for IFSelectSelectSuite {
    /// `RootResult(G)`: the input result (when there is one) set as the alternate list
    /// of the first deduction, whose result feeds the next one, and so on.
    fn root_result(&self, g: &InterfaceGraph) -> Result<InterfaceEntityIterator, Failure> {
        let mut iter = InterfaceEntityIterator::new();
        let mut firstin = self.has_input() || self.has_alternate();
        if firstin {
            iter = self.input_result(g)?;
        }
        let items = self.state.read().thesel.clone();
        for item in items {
            let live = item
                .get()
                .ok_or_else(|| exceptions::null_object("IFSelect_SelectSuite: null deduction"))?;
            if firstin {
                live.alternate().set_list(Some(&iter.content()));
            }
            firstin = true;
            iter = live.unique_result(g)?;
        }
        Ok(iter)
    }
    fn fill_iterator(&self, iter: &mut IFSelectSelectionIterator) {
        deduct_fill_iterator(self, iter)
    }
    /// `Label`: the label set, else "Suite of n Selections".
    fn label(&self) -> AsciiString {
        let st = self.state.read();
        if !st.thelab.is_empty() {
            return st.thelab.clone();
        }
        AsciiString::new(&format!("Suite of {} Selections", st.thesel.len()))
    }
}

/// The fields of `IFSelect_SelectExplore`: those of the deduction and the level.
pub struct IFSelectSelectExploreBase {
    /// The deduction fields.
    pub deduct: IFSelectSelectDeductBase,
    thelevel: i32,
}

impl IFSelectSelectExploreBase {
    /// The protected constructor `IFSelect_SelectExplore(level)`: 0 is recursive.
    pub fn new(level: i32) -> Self {
        Self {
            deduct: IFSelectSelectDeductBase::default(),
            thelevel: level,
        }
    }
}

/// `IFSelect_SelectExplore`, abstract: `Explore` and `ExploreLabel` are required; its
/// `RootResult` and `Label` are [`explore_root_result`] and [`explore_label`].
pub trait IFSelectSelectExplore: IFSelectSelectDeduct {
    /// The embedded fields.
    fn explore_base(&self) -> &IFSelectSelectExploreBase;
    /// `Level`.
    fn level(&self) -> i32 {
        self.explore_base().thelevel
    }
    /// `Explore(level, ent, G, explored)`: false to drop `ent`; true with nothing explored
    /// to keep it; true with entities to explore them next.
    fn explore(
        &self,
        level: i32,
        ent: &Handle<StandardTransient>,
        g: &InterfaceGraph,
        explored: &mut InterfaceEntityIterator,
    ) -> Result<bool, Failure>;
    /// `ExploreLabel`.
    fn explore_label(&self) -> AsciiString;
}
impl_transient_interface!(
    dyn IFSelectSelectExplore,
    "IFSelect_SelectExplore",
    dyn IFSelectSelectDeduct
);

/// `IFSelect_SelectExplore::RootResult`: explores the input entities level by level up to
/// the level (all levels for 0); the entities with nothing to explore, and those explored
/// at the last level, are the result.
pub fn explore_root_result<S: IFSelectSelectExplore + ?Sized>(
    sel: &S,
    g: &InterfaceGraph,
) -> Result<InterfaceEntityIterator, Failure> {
    let nb = g.size();
    let mut entrees: IndexedMap<Handle<StandardTransient>> = IndexedMap::new(nb.max(1) as usize);
    let mut result: IndexedMap<Handle<StandardTransient>> = IndexedMap::new(nb.max(1) as usize);
    for ent in sel.input_result(g)?.content() {
        entrees.add(ent);
    }
    let (thelevel, mut level, mut ilev) = (sel.level(), 1, entrees.extent());
    for i in 1..=nb.max(0) as usize {
        if i > entrees.extent() {
            break;
        }
        if i > ilev {
            level += 1;
            if level > thelevel && thelevel > 0 {
                break;
            }
            ilev = entrees.extent();
        }
        let ent = entrees.find_key(i)?.clone();
        if ent.is_null() {
            continue;
        }
        let mut exp = InterfaceEntityIterator::new();
        if !sel.explore(level, &ent, g, &mut exp)? {
            continue;
        }
        let target = if exp.nb_entities() == 0 || level == thelevel {
            &mut result
        } else {
            &mut entrees
        };
        if exp.nb_entities() == 0 {
            target.add(ent);
            continue;
        }
        exp.content().into_iter().for_each(|e| {
            target.add(e);
        });
    }
    let mut res = InterfaceEntityIterator::new();
    result.iter().for_each(|e| res.add_item(e));
    Ok(res)
}

/// `IFSelect_SelectExplore::Label`: "(Recursive)" or "(Level n)", then `ExploreLabel`.
pub fn explore_label<S: IFSelectSelectExplore + ?Sized>(sel: &S) -> AsciiString {
    let level = sel.level();
    let mut label = match level {
        0 => AsciiString::new("(Recursive)"),
        l if l > 0 => AsciiString::new(&format!("(Level {l})")),
        _ => AsciiString::default(),
    };
    label.assign_cat(sel.explore_label());
    label
}

/// The fields of `IFSelect_SelectAnyList`: those of the deduction and the rank range.
#[derive(Default)]
pub struct IFSelectSelectAnyListBase {
    /// The deduction fields.
    pub deduct: IFSelectSelectDeductBase,
    /// The rank range.
    pub range: IFSelectRankRange,
}

/// `IFSelect_SelectAnyList`, abstract: `KeepInputEntity`, `NbItems`, `FillResult` and
/// `ListLabel` are required; its `RootResult` and `Label` are [`any_list_root_result`]
/// and [`any_list_label`]; the range members are those of
/// [`IFSelectRankRange`] (`any_list_base().range`).
pub trait IFSelectSelectAnyList: IFSelectSelectDeduct {
    /// The embedded fields.
    fn any_list_base(&self) -> &IFSelectSelectAnyListBase;
    /// `KeepInputEntity(iter)`: keeps the entities of the right type.
    fn keep_input_entity(&self, iter: &mut InterfaceEntityIterator);
    /// `NbItems(ent)`: the length of the list of `ent`.
    fn nb_items(&self, ent: &Handle<StandardTransient>) -> i32;
    /// `FillResult(n1, n2, ent, res)`: the items `n1..n2` of the list of `ent`.
    fn fill_result(
        &self,
        n1: i32,
        n2: i32,
        ent: &Handle<StandardTransient>,
        res: &mut InterfaceEntityIterator,
    ) -> Result<(), Failure>;
    /// `ListLabel`.
    fn list_label(&self) -> AsciiString;
}
impl_transient_interface!(
    dyn IFSelectSelectAnyList,
    "IFSelect_SelectAnyList",
    dyn IFSelectSelectDeduct
);

/// `IFSelect_SelectAnyList::RootResult`: the items of the range in the list of the only
/// input entity; `Interface_InterfaceError` for more than one input entity.
pub fn any_list_root_result<S: IFSelectSelectAnyList + ?Sized>(
    sel: &S,
    g: &InterfaceGraph,
) -> Result<InterfaceEntityIterator, Failure> {
    let mut input = sel.input_result(g)?;
    sel.keep_input_entity(&mut input);
    if input.nb_entities() > 1 {
        return Err(interface_error(
            "SelectAnyList : more than ONE Entity in input",
        ));
    }
    let Some(ent) = input.content().last().cloned() else {
        return Ok(input);
    };
    let range = &sel.any_list_base().range;
    let rankmax = sel.nb_items(&ent);
    let rankfrom = if range.has_lower() {
        range.lower_value()
    } else {
        1
    }
    .max(1);
    let rankto = if range.has_upper() {
        range.upper_value()
    } else {
        rankmax
    }
    .min(rankmax);
    let mut iter = InterfaceEntityIterator::new();
    if rankfrom <= rankto {
        sel.fill_result(rankfrom, rankto, &ent, &mut iter)?;
    }
    Ok(iter)
}

/// `IFSelect_SelectAnyList::Label`: "In List ", `ListLabel` and the range. Native quirks
/// kept: " (-> n)" prints the lower rank (0) and " (n ->)" the upper one (0).
pub fn any_list_label<S: IFSelectSelectAnyList + ?Sized>(sel: &S) -> AsciiString {
    let range = &sel.any_list_base().range;
    let (from, to) = (range.lower_value(), range.upper_value());
    let suffix = if from == to {
        format!(" (no {from})")
    } else if from == 0 {
        format!(" (-> {from})")
    } else if to == 0 {
        format!(" ({to} ->)")
    } else {
        format!(" ({from} -> {to})")
    };
    let mut label = AsciiString::new("In List ");
    label.assign_cat(sel.list_label());
    label.assign_cat(suffix.as_str());
    label
}
