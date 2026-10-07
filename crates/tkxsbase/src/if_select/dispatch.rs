// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_Dispatch, IFSelect_DispGlobal,
// IFSelect_DispPerOne, IFSelect_DispPerCount, IFSelect_DispPerFiles and
// IFSelect_DispPerSignature (.hxx, .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_Dispatch`: splits the result of a final selection into packets (one file
//! each): all in one, one per root, by count, into a number of files, per signature.
use super::{
    IFSelectIntParam, IFSelectSelection, IFSelectSelectionIterator, IFSelectSignCounterOps,
};
use crate::if_graph::{
    IFGraphCompare, IFGraphSCRoots, IFGraphSubPartsEvaluate, IFGraphSubPartsIterator,
};
use crate::interface::{InterfaceEntityIterator, InterfaceGraph};
use rustycad_tkernel::{
    impl_transient, impl_transient_interface,
    standard::{Failure, Handle, SharedState, StandardTransient, Transient, exceptions},
    tcollection::{AsciiString, HAsciiString},
};

/// The fields of `IFSelect_Dispatch`: the root name and the final selection.
#[derive(Default)]
pub struct IFSelectDispatchBase {
    thename: SharedState<Handle<HAsciiString>>,
    thefinal: SharedState<Handle<dyn IFSelectSelection>>,
}

/// The final selection of a dispatch; `Standard_NullObject` without one (a native null
/// dereference).
fn final_of<D: IFSelectDispatch + ?Sized>(
    disp: &D,
) -> Result<Handle<dyn IFSelectSelection>, Failure> {
    let sel = disp.final_selection();
    if sel.is_null() {
        return Err(exceptions::null_object(
            "IFSelect_Dispatch: no final selection; expected SetFinalSelection first",
        ));
    }
    Ok(sel)
}

/// `IFSelect_Dispatch`, abstract: `Label` and `Packets` are required.
///
/// ```
/// use rustycad_tkxsbase::if_select::{IFSelectDispGlobal, IFSelectDispatch};
/// let disp = IFSelectDispGlobal::new();
/// assert_eq!(disp.label().as_bytes(), b"One File for All Input");
/// assert_eq!(disp.limited_max(10), Some(1));
/// ```
pub trait IFSelectDispatch: Transient {
    /// The embedded fields.
    fn dispatch_base(&self) -> &IFSelectDispatchBase;
    /// `SetRootName(name)`.
    fn set_root_name(&self, name: &Handle<HAsciiString>) {
        *self.dispatch_base().thename.write() = name.clone();
    }
    /// `HasRootName`.
    fn has_root_name(&self) -> bool {
        self.root_name().is_some()
    }
    /// `RootName`.
    fn root_name(&self) -> Handle<HAsciiString> {
        self.dispatch_base().thename.read().clone()
    }
    /// `SetFinalSelection(sel)`.
    fn set_final_selection(&self, sel: &Handle<dyn IFSelectSelection>) {
        *self.dispatch_base().thefinal.write() = sel.clone();
    }
    /// `FinalSelection`.
    fn final_selection(&self) -> Handle<dyn IFSelectSelection> {
        self.dispatch_base().thefinal.read().clone()
    }
    /// `Selections`: the final selection and all its inputs, recursively. Native quirk
    /// kept: the iterator returned is the one used to collect them, at its end.
    fn selections(&self) -> Result<IFSelectSelectionIterator, Failure> {
        let mut iter = IFSelectSelectionIterator::new();
        iter.add_item(&self.final_selection());
        while iter.more() {
            let sel = iter.value()?;
            sel.get()
                .ok_or_else(|| {
                    exceptions::null_object("IFSelect_Dispatch::Selections: null input selection")
                })?
                .fill_iterator(&mut iter);
            iter.next();
        }
        Ok(iter)
    }
    /// `CanHaveRemainder`: false by default.
    fn can_have_remainder(&self) -> bool {
        false
    }
    /// `LimitedMax(nbent, max)`: the maximum count of packets, `None` when not limited
    /// (natively false, `max` 0).
    fn limited_max(&self, _nbent: i32) -> Option<i32> {
        None
    }
    /// `Label`.
    fn label(&self) -> AsciiString;
    /// `GetEntities(G)`: the unique result of the final selection.
    fn get_entities(&self, g: &InterfaceGraph) -> Result<InterfaceEntityIterator, Failure> {
        final_of(self)?.unique_result(g)
    }
    /// `Packets(G, packs)`: the packets, as parts of `packs`.
    fn packets(
        &self,
        g: &InterfaceGraph,
        packs: &mut IFGraphSubPartsIterator,
    ) -> Result<(), Failure>;
    /// `Packeted(G)`: the entities put in packets (the entities without the remainder).
    fn packeted(&self, g: &InterfaceGraph) -> Result<InterfaceEntityIterator, Failure> {
        let total = self.get_entities(g)?;
        let remain = self.remainder(g)?;
        if remain.nb_entities() == 0 {
            return Ok(total);
        }
        let mut compare = IFGraphCompare::new(g)?;
        compare.get_from_iter(&total, true)?;
        compare.get_from_iter(&remain, false)?;
        compare.first_only()
    }
    /// `Remainder(G)`: empty by default.
    fn remainder(&self, _g: &InterfaceGraph) -> Result<InterfaceEntityIterator, Failure> {
        Ok(InterfaceEntityIterator::new())
    }
}
impl_transient_interface!(dyn IFSelectDispatch, "IFSelect_Dispatch", StandardTransient);

/// Declares a dispatch with its base and the given extra fields.
macro_rules! dispatch_struct {
    ($(#[$doc:meta])* $name:ident, $native:literal { $($field:ident: $ty:ty),* }) => {
        $(#[$doc])*
        #[derive(Default)]
        pub struct $name {
            transient: StandardTransient,
            base: IFSelectDispatchBase,
            $($field: $ty,)*
        }
        impl_transient!($name, $native, dyn IFSelectDispatch, transient, interfaces: [dyn IFSelectDispatch]);
        impl $name {
            #[doc = concat!("`", $native, "()`.")]
            pub fn new() -> Self {
                Self::default()
            }
        }
    };
}

dispatch_struct!(
    /// `IFSelect_DispGlobal`: one packet with all the entities.
    IFSelectDispGlobal, "IFSelect_DispGlobal" {}
);

impl IFSelectDispatch for IFSelectDispGlobal {
    fn dispatch_base(&self) -> &IFSelectDispatchBase {
        &self.base
    }
    fn label(&self) -> AsciiString {
        AsciiString::new("One File for All Input")
    }
    /// `LimitedMax`: one packet.
    fn limited_max(&self, _nbent: i32) -> Option<i32> {
        Some(1)
    }
    /// `Packets`: one part with the unique result of the final selection.
    fn packets(
        &self,
        g: &InterfaceGraph,
        packs: &mut IFGraphSubPartsIterator,
    ) -> Result<(), Failure> {
        packs.add_part();
        packs.get_from_iter(&final_of(self)?.unique_result(g)?)
    }
}

/// The strong-component roots of the final selection result, loaded in a SCRoots.
fn roots_of<D: IFSelectDispatch + ?Sized>(
    disp: &D,
    g: &InterfaceGraph,
) -> Result<IFGraphSCRoots, Failure> {
    let mut roots = IFGraphSCRoots::new(g, false)?;
    roots.set_load();
    roots.get_from_iter(&final_of(disp)?.unique_result(g)?)?;
    Ok(roots)
}

dispatch_struct!(
    /// `IFSelect_DispPerOne`: one packet per root (strong component).
    IFSelectDispPerOne, "IFSelect_DispPerOne" {}
);

impl IFSelectDispatch for IFSelectDispPerOne {
    fn dispatch_base(&self) -> &IFSelectDispatchBase {
        &self.base
    }
    fn label(&self) -> AsciiString {
        AsciiString::new("One File per Input Entity")
    }
    /// `LimitedMax`: one packet per entity.
    fn limited_max(&self, nbent: i32) -> Option<i32> {
        Some(nbent)
    }
    /// `Packets`: the parts of the roots of the final selection.
    fn packets(
        &self,
        g: &InterfaceGraph,
        packs: &mut IFGraphSubPartsIterator,
    ) -> Result<(), Failure> {
        let mut roots = roots_of(self, g)?;
        packs.get_parts(&mut roots)
    }
}

/// The value of a count parameter: at least 1.
fn count_value(count: &SharedState<Handle<IFSelectIntParam>>) -> i32 {
    count.read().get().map_or(0, IFSelectIntParam::value).max(1)
}

/// Groups the roots of `roots` (already started) by `pcount` into the parts of `packs`.
fn group_roots(
    roots: &mut IFGraphSCRoots,
    packs: &mut IFGraphSubPartsIterator,
    pcount: i32,
) -> Result<(), Failure> {
    let mut i = 0;
    while roots.more()? {
        if i == 0 {
            packs.add_part();
        }
        i += 1;
        if i >= pcount {
            i = 0;
        }
        packs.get_from_iter(&roots.entities()?)?;
        roots.next();
    }
    Ok(())
}

dispatch_struct!(
    /// `IFSelect_DispPerCount`: packets of a count of roots.
    IFSelectDispPerCount, "IFSelect_DispPerCount" { thecount: SharedState<Handle<IFSelectIntParam>> }
);

impl IFSelectDispPerCount {
    /// `Count`.
    pub fn count(&self) -> Handle<IFSelectIntParam> {
        self.thecount.read().clone()
    }
    /// `SetCount(count)`.
    pub fn set_count(&self, count: &Handle<IFSelectIntParam>) {
        *self.thecount.write() = count.clone();
    }
    /// `CountValue`: the count, at least 1.
    pub fn count_value(&self) -> i32 {
        count_value(&self.thecount)
    }
}

impl IFSelectDispatch for IFSelectDispPerCount {
    fn dispatch_base(&self) -> &IFSelectDispatchBase {
        &self.base
    }
    /// `Label`: "One File per n Entities".
    fn label(&self) -> AsciiString {
        AsciiString::new(&format!("One File per {} Entities", self.count_value()))
    }
    fn limited_max(&self, nbent: i32) -> Option<i32> {
        Some(1 + nbent / self.count_value())
    }
    /// `Packets`: the roots of the final selection, grouped by the count.
    fn packets(
        &self,
        g: &InterfaceGraph,
        packs: &mut IFGraphSubPartsIterator,
    ) -> Result<(), Failure> {
        let mut roots = roots_of(self, g)?;
        roots.start()?;
        group_roots(&mut roots, packs, self.count_value())
    }
}

dispatch_struct!(
    /// `IFSelect_DispPerFiles`: a maximum count of packets.
    IFSelectDispPerFiles, "IFSelect_DispPerFiles" { thecount: SharedState<Handle<IFSelectIntParam>> }
);

impl IFSelectDispPerFiles {
    /// `Count`.
    pub fn count(&self) -> Handle<IFSelectIntParam> {
        self.thecount.read().clone()
    }
    /// `SetCount(count)`.
    pub fn set_count(&self, count: &Handle<IFSelectIntParam>) {
        *self.thecount.write() = count.clone();
    }
    /// `CountValue`: the count, at least 1.
    pub fn count_value(&self) -> i32 {
        count_value(&self.thecount)
    }
}

impl IFSelectDispatch for IFSelectDispPerFiles {
    fn dispatch_base(&self) -> &IFSelectDispatchBase {
        &self.base
    }
    /// `Label`: "Maximum n Files".
    fn label(&self) -> AsciiString {
        AsciiString::new(&format!("Maximum {} Files", self.count_value()))
    }
    fn limited_max(&self, _nbent: i32) -> Option<i32> {
        Some(self.count_value())
    }
    /// `Packets`: the roots of the final selection, in (at most) the count of packets.
    fn packets(
        &self,
        g: &InterfaceGraph,
        packs: &mut IFGraphSubPartsIterator,
    ) -> Result<(), Failure> {
        let mut roots = roots_of(self, g)?;
        roots.start()?;
        let per_packet = (roots.nb_parts() - 1) / self.count_value() + 1;
        group_roots(&mut roots, packs, per_packet)
    }
}

dispatch_struct!(
    /// `IFSelect_DispPerSignature`: one packet per value of a signature counter.
    IFSelectDispPerSignature, "IFSelect_DispPerSignature" { thesign: SharedState<Handle<dyn IFSelectSignCounterOps>> }
);

impl IFSelectDispPerSignature {
    /// `SignCounter`.
    pub fn sign_counter(&self) -> Handle<dyn IFSelectSignCounterOps> {
        self.thesign.read().clone()
    }
    /// `SetSignCounter(sign)`: with entity lists; `Standard_NullObject` for null (a
    /// native null dereference).
    pub fn set_sign_counter(
        &self,
        sign: &Handle<dyn IFSelectSignCounterOps>,
    ) -> Result<(), Failure> {
        *self.thesign.write() = sign.clone();
        let live = sign.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_DispPerSignature::SetSignCounter: null counter")
        })?;
        live.signature_list().set_list(true);
        Ok(())
    }
    /// `SignName`: the name of the counter, "???" without one.
    pub fn sign_name(&self) -> String {
        self.sign_counter()
            .get()
            .map_or_else(|| String::from("???"), |c| c.name())
    }
}

impl IFSelectDispatch for IFSelectDispPerSignature {
    fn dispatch_base(&self) -> &IFSelectDispatchBase {
        &self.base
    }
    /// `Label`: "One File per Signature " and the counter name.
    fn label(&self) -> AsciiString {
        AsciiString::new(&format!("One File per Signature {}", self.sign_name()))
    }
    fn limited_max(&self, nbent: i32) -> Option<i32> {
        Some(nbent)
    }
    /// `Packets`: one part per signature of the root result of the final selection (one
    /// part with it all without counter).
    fn packets(
        &self,
        g: &InterfaceGraph,
        packs: &mut IFGraphSubPartsIterator,
    ) -> Result<(), Failure> {
        let root = final_of(self)?.root_result(g)?;
        let counter = self.sign_counter();
        let Some(live) = counter.get() else {
            packs.add_part();
            return packs.get_from_iter(&root);
        };
        live.clear();
        live.add_list(Some(&root.content()), g.model())?;
        for sign in live.signature_list().list("") {
            let text = sign
                .get()
                .map(|s| String::from_utf8_lossy(s.string().as_bytes()).into_owned())
                .unwrap_or_default();
            let Some(ents) = live.signature_list().entities(&text) else {
                continue;
            };
            packs.add_part();
            for ent in ents {
                packs.get_from_entity(&ent, false)?;
            }
        }
        Ok(())
    }
}
