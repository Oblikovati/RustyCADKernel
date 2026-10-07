// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_GeneralModifier,
// IFSelect_Modifier, IFSelect_ModifReorder and IFSelect_AppliedModifiers (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_GeneralModifier`: a modification of the output (of a dispatch, on a
//! selection); `IFSelect_Modifier`: a modification of the copied model;
//! `IFSelect_ModifReorder`: reorders the entities; `IFSelect_AppliedModifiers`: the
//! modifiers applied to a file, with their entities.
use super::{IFSelectContextModif, IFSelectDispatch, IFSelectSelection};
use crate::interface::{
    InterfaceCopyTool, InterfaceIntList, InterfaceInterfaceModel, InterfaceProtocol,
    InterfaceShareTool,
};
use rustycad_tkernel::{
    impl_transient, impl_transient_interface,
    standard::{Failure, Handle, SharedState, StandardTransient, Transient, exceptions},
    tcollection::AsciiString,
};

/// The fields of `IFSelect_GeneralModifier`: the selection, the dispatch and whether the
/// graph may change.
pub struct IFSelectGeneralModifierBase {
    thesel: SharedState<Handle<dyn IFSelectSelection>>,
    thedisp: SharedState<Handle<dyn IFSelectDispatch>>,
    thechgr: bool,
}

impl IFSelectGeneralModifierBase {
    /// The protected constructor `IFSelect_GeneralModifier(maychangegraph)`.
    pub fn new(maychangegraph: bool) -> Self {
        Self {
            thesel: SharedState::new(Handle::default()),
            thedisp: SharedState::new(Handle::default()),
            thechgr: maychangegraph,
        }
    }
}

/// `IFSelect_GeneralModifier`, abstract: `Label` is required.
pub trait IFSelectGeneralModifier: Transient {
    /// The embedded fields.
    fn general_modifier_base(&self) -> &IFSelectGeneralModifierBase;
    /// `MayChangeGraph`.
    fn may_change_graph(&self) -> bool {
        self.general_modifier_base().thechgr
    }
    /// `SetDispatch(disp)`.
    fn set_dispatch(&self, disp: &Handle<dyn IFSelectDispatch>) {
        *self.general_modifier_base().thedisp.write() = disp.clone();
    }
    /// `Dispatch`.
    fn dispatch(&self) -> Handle<dyn IFSelectDispatch> {
        self.general_modifier_base().thedisp.read().clone()
    }
    /// `Applies(disp)`: no dispatch (all), or this one.
    fn applies(&self, disp: &Handle<dyn IFSelectDispatch>) -> bool {
        let own = self.dispatch();
        own.is_null() || own == *disp
    }
    /// `SetSelection(sel)`.
    fn set_selection(&self, sel: &Handle<dyn IFSelectSelection>) {
        *self.general_modifier_base().thesel.write() = sel.clone();
    }
    /// `ResetSelection`.
    fn reset_selection(&self) {
        *self.general_modifier_base().thesel.write() = Handle::default();
    }
    /// `HasSelection`.
    fn has_selection(&self) -> bool {
        self.selection().is_some()
    }
    /// `Selection`.
    fn selection(&self) -> Handle<dyn IFSelectSelection> {
        self.general_modifier_base().thesel.read().clone()
    }
    /// `Label`.
    fn label(&self) -> AsciiString;
}
impl_transient_interface!(
    dyn IFSelectGeneralModifier,
    "IFSelect_GeneralModifier",
    StandardTransient
);

/// `IFSelect_Modifier`, abstract: `Perform` is required (its protected constructor is
/// that of [`IFSelectGeneralModifierBase`]).
pub trait IFSelectModifier: IFSelectGeneralModifier {
    /// `Perform(ctx, target, protocol, TC)`: modifies the target model.
    fn perform(
        &self,
        ctx: &mut IFSelectContextModif,
        target: &Handle<dyn InterfaceInterfaceModel>,
        protocol: &Handle<dyn InterfaceProtocol>,
        tc: &mut InterfaceCopyTool,
    ) -> Result<(), Failure>;
}
impl_transient_interface!(
    dyn IFSelectModifier,
    "IFSelect_Modifier",
    dyn IFSelectGeneralModifier
);

/// `IFSelect_ModifReorder`: puts the entities in the order of the share tool (roots
/// last or first).
///
/// ```
/// use rustycad_tkxsbase::if_select::{IFSelectGeneralModifier, IFSelectModifReorder};
/// let modif = IFSelectModifReorder::new(true);
/// assert!(modif.may_change_graph() && modif.label().as_bytes() == b"Reorder, Roots last");
/// ```
pub struct IFSelectModifReorder {
    transient: StandardTransient,
    base: IFSelectGeneralModifierBase,
    thertl: bool,
}
impl_transient!(IFSelectModifReorder, "IFSelect_ModifReorder", dyn IFSelectModifier, transient, interfaces: [dyn IFSelectGeneralModifier, dyn IFSelectModifier]);

impl IFSelectModifReorder {
    /// `IFSelect_ModifReorder(rootlast = true)`.
    pub fn new(rootlast: bool) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: IFSelectGeneralModifierBase::new(true),
            thertl: rootlast,
        }
    }
}

impl IFSelectGeneralModifier for IFSelectModifReorder {
    fn general_modifier_base(&self) -> &IFSelectGeneralModifierBase {
        &self.base
    }
    /// `Label`: "Reorder, Roots last" or "Reorder, Roots first".
    fn label(&self) -> AsciiString {
        AsciiString::new(if self.thertl {
            "Reorder, Roots last"
        } else {
            "Reorder, Roots first"
        })
    }
}

impl IFSelectModifier for IFSelectModifReorder {
    /// `Perform`: the target gets the entities of the original model in the order of
    /// `Interface_ShareTool::All`.
    fn perform(
        &self,
        ctx: &mut IFSelectContextModif,
        target: &Handle<dyn InterfaceInterfaceModel>,
        _protocol: &Handle<dyn InterfaceProtocol>,
        _tc: &mut InterfaceCopyTool,
    ) -> Result<(), Failure> {
        let sht = InterfaceShareTool::from_graph(ctx.original_graph())?;
        let list = sht.all(&ctx.original_model().cast(), self.thertl)?;
        let live = target.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_ModifReorder::Perform: null target model")
        })?;
        live.clear_entities();
        list.content()
            .iter()
            .try_for_each(|ent| live.add_entity(ent))
    }
}

/// A modifier of `IFSelect_AppliedModifiers` with its count of entities.
pub type IFSelectAppliedItem = (Handle<dyn IFSelectGeneralModifier>, i32);

/// The fields of `IFSelect_AppliedModifiers`.
struct AppliedState {
    themodifs: Vec<Handle<dyn IFSelectGeneralModifier>>,
    thelists: InterfaceIntList,
    thenbent: i32,
    theentcnt: i32,
}

/// `IFSelect_AppliedModifiers`: up to `nbmax` modifiers, each with the numbers of its
/// entities (all when it has none).
///
/// ```
/// use rustycad_tkxsbase::if_select::{IFSelectAppliedModifiers, IFSelectModifReorder};
/// use rustycad_tkernel::standard::Handle;
/// let applied = IFSelectAppliedModifiers::new(2, 5).unwrap();
/// assert!(applied.add_modif(&Handle::new(IFSelectModifReorder::new(true)).cast()).unwrap());
/// assert_eq!(applied.count(), 1);
/// ```
pub struct IFSelectAppliedModifiers {
    transient: StandardTransient,
    state: SharedState<AppliedState>,
}
impl_transient!(
    IFSelectAppliedModifiers,
    "IFSelect_AppliedModifiers",
    StandardTransient,
    transient
);

impl IFSelectAppliedModifiers {
    /// `IFSelect_AppliedModifiers(nbmax, nbent)`.
    pub fn new(nbmax: i32, nbent: i32) -> Result<Self, Failure> {
        let state = AppliedState {
            themodifs: Vec::new(),
            thelists: InterfaceIntList::with_entities(nbmax + 1)?,
            thenbent: nbent,
            theentcnt: 0,
        };
        Ok(Self {
            transient: StandardTransient::default(),
            state: SharedState::new(state),
        })
    }
    /// `AddModif(modif)`: a new current modifier (false beyond the maximum).
    pub fn add_modif(&self, modif: &Handle<dyn IFSelectGeneralModifier>) -> Result<bool, Failure> {
        let mut st = self.state.write();
        if st.themodifs.len() as i32 >= st.thelists.nb_entities() {
            return Ok(false);
        }
        st.themodifs.push(modif.clone());
        let number = st.themodifs.len() as i32;
        st.thelists.set_number(number)?;
        Ok(true)
    }
    /// `AddNum(nument)`: an entity of the current modifier.
    pub fn add_num(&self, nument: i32) -> Result<bool, Failure> {
        self.state.write().thelists.add(nument)?;
        Ok(true)
    }
    /// `Count`.
    pub fn count(&self) -> i32 {
        self.state.read().themodifs.len() as i32
    }
    /// `Item(num, modif, entcount)`: the modifier `num` and its count of entities (all
    /// without list); `None` out of range. The modifier becomes the current one.
    pub fn item(&self, num: i32) -> Result<Option<IFSelectAppliedItem>, Failure> {
        let mut st = self.state.write();
        let Some(modif) = usize::try_from(num - 1)
            .ok()
            .and_then(|i| st.themodifs.get(i))
            .cloned()
        else {
            return Ok(None);
        };
        st.thelists.set_number(num)?;
        st.theentcnt = st.thelists.length();
        let entcount = if st.theentcnt > 0 {
            st.theentcnt
        } else {
            st.thenbent
        };
        Ok(Some((modif, entcount)))
    }
    /// `ItemNum(nument)`: the entity number `nument` of the current modifier (itself
    /// when it applies to all).
    pub fn item_num(&self, nument: i32) -> Result<i32, Failure> {
        let st = self.state.read();
        if st.theentcnt > 0 {
            return st.thelists.value(nument);
        }
        Ok(nument)
    }
    /// `ItemList`: the entity numbers of the current modifier.
    pub fn item_list(&self) -> Result<Vec<i32>, Failure> {
        let nb = {
            let st = self.state.read();
            if st.theentcnt > 0 {
                st.theentcnt
            } else {
                st.thenbent
            }
        };
        (1..=nb).map(|i| self.item_num(i)).collect()
    }
    /// `IsForAll`: the current modifier applies to all the entities.
    pub fn is_for_all(&self) -> bool {
        self.state.read().theentcnt == 0
    }
}
