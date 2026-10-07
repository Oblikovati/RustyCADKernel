// Rust port of DataExchange/TKXSBase/XSControl/XSControl_SelectForTransfer and
// XSControl_ConnectedShapes (.hxx, .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The selections of the transfer: the entities the reading actor recognizes, and the
//! entities whose produced shapes are connected (by a vertex) to the shape of an entity.
use super::XSControlTransferReader;
use crate::if_select::{
    IFSelectSelectDeduct, IFSelectSelectDeductBase, IFSelectSelectExplore,
    IFSelectSelectExploreBase, IFSelectSelectExtract, IFSelectSelectExtractBase, IFSelectSelection,
    IFSelectSelectionIterator, deduct_fill_iterator, explore_label, explore_root_result,
    impl_extract_selection,
};
use crate::interface::{InterfaceEntityIterator, InterfaceGraph, InterfaceInterfaceModel};
use crate::transfer::{TransferActorOfTransientProcessOps, TransferTransientProcess};
use crate::transfer_brep::TransferBRep;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, SharedState, StandardTransient},
    tcollection::AsciiString,
};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `XSControl_SelectForTransfer`: the input entities the actor (its own, else the one of
/// the transfer reader) recognizes.
///
/// ```
/// use rustycad_tkxsbase::if_select::IFSelectSelection;
/// use rustycad_tkxsbase::xs_control::XSControlSelectForTransfer;
/// let sel = XSControlSelectForTransfer::new();
/// assert_eq!(sel.label().as_bytes(), b"Picked: Recognized for Transfer");
/// ```
#[derive(Default)]
pub struct XSControlSelectForTransfer {
    transient: StandardTransient,
    base: IFSelectSelectExtractBase,
    the_tr: SharedState<Handle<XSControlTransferReader>>,
    the_ac: SharedState<Handle<dyn TransferActorOfTransientProcessOps>>,
}
impl_transient!(XSControlSelectForTransfer, "XSControl_SelectForTransfer", dyn IFSelectSelectExtract, transient, interfaces: [dyn IFSelectSelection, dyn IFSelectSelectDeduct, dyn IFSelectSelectExtract]);
impl_extract_selection!(XSControlSelectForTransfer, base);

impl XSControlSelectForTransfer {
    /// `XSControl_SelectForTransfer()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `XSControl_SelectForTransfer(TR)`.
    pub fn with_reader(tr: &Handle<XSControlTransferReader>) -> Self {
        let sel = Self::default();
        sel.set_reader(tr);
        sel
    }
    /// `SetReader(TR)`.
    pub fn set_reader(&self, tr: &Handle<XSControlTransferReader>) {
        *self.the_tr.write() = tr.clone();
    }
    /// `SetActor(act)`.
    pub fn set_actor(&self, act: &Handle<dyn TransferActorOfTransientProcessOps>) {
        *self.the_ac.write() = act.clone();
    }
    /// `Actor`.
    pub fn actor(&self) -> Handle<dyn TransferActorOfTransientProcessOps> {
        self.the_ac.read().clone()
    }
    /// `Reader`.
    pub fn reader(&self) -> Handle<XSControlTransferReader> {
        self.the_tr.read().clone()
    }
}

impl IFSelectSelectExtract for XSControlSelectForTransfer {
    fn extract_base(&self) -> &IFSelectSelectExtractBase {
        &self.base
    }
    /// `Sort(rank, ent, model)`: the actor recognizes `ent` (false without actor).
    fn sort(
        &self,
        _rank: i32,
        ent: &Handle<StandardTransient>,
        _model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<bool, Failure> {
        let mut act = self.actor();
        let tr = self.reader();
        if act.is_null()
            && let Some(reader) = tr.get()
        {
            act = reader.actor();
        }
        Ok(act.get().is_some_and(|a| a.recognize(ent)))
    }
    /// `ExtractLabel`.
    fn extract_label(&self) -> AsciiString {
        AsciiString::new(if self.reader().is_some() {
            "Recognized for Transfer (current actor)"
        } else {
            "Recognized for Transfer"
        })
    }
}

/// `XSControl_ConnectedShapes`: explores (one level) the entities whose produced faces
/// share a vertex with the shape produced from an entity.
///
/// ```
/// use rustycad_tkxsbase::if_select::IFSelectSelection;
/// use rustycad_tkxsbase::xs_control::XSControlConnectedShapes;
/// let sel = XSControlConnectedShapes::new();
/// assert_eq!(sel.label().as_bytes(), b"(Level 1)Connected Entities through produced Shapes");
/// ```
pub struct XSControlConnectedShapes {
    transient: StandardTransient,
    base: IFSelectSelectExploreBase,
    the_tr: SharedState<Handle<XSControlTransferReader>>,
}
impl_transient!(XSControlConnectedShapes, "XSControl_ConnectedShapes", dyn IFSelectSelectExplore, transient, interfaces: [dyn IFSelectSelection, dyn IFSelectSelectDeduct, dyn IFSelectSelectExplore]);

impl Default for XSControlConnectedShapes {
    fn default() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: IFSelectSelectExploreBase::new(1),
            the_tr: SharedState::default(),
        }
    }
}

impl XSControlConnectedShapes {
    /// `XSControl_ConnectedShapes()`: one level.
    pub fn new() -> Self {
        Self::default()
    }
    /// `XSControl_ConnectedShapes(TR)`.
    pub fn with_reader(tr: &Handle<XSControlTransferReader>) -> Self {
        let sel = Self::default();
        sel.set_reader(tr);
        sel
    }
    /// `SetReader(TR)`.
    pub fn set_reader(&self, tr: &Handle<XSControlTransferReader>) {
        *self.the_tr.write() = tr.clone();
    }
    /// `AdjacentEntities(ashape, TP, type)`: the mapped entities of `TP` whose produced
    /// shape, of type `type`, has a vertex of `ashape`.
    pub fn adjacent_entities(
        ashape: &Shape,
        tp: &Handle<TransferTransientProcess>,
        kind: ShapeEnum,
    ) -> Result<Vec<Handle<StandardTransient>>, Failure> {
        let process = tp.get().ok_or_else(|| {
            rustycad_tkernel::standard::exceptions::null_object(
                "XSControl_ConnectedShapes::AdjacentEntities: null transient process",
            )
        })?;
        let mut vtx = MapOfShape::default();
        let mut vert = Explorer::new(ashape, ShapeEnum::Vertex, ShapeEnum::Shape)?;
        while vert.more() {
            vtx.add(vert.current().clone());
            vert.next()?;
        }
        let mut li = Vec::new();
        for i in 1..=process.nb_mapped() {
            let sh = TransferBRep::shape_result(&process.map_item(i)?);
            if sh.is_null() || sh.shape_type()? != kind {
                continue;
            }
            if touches(&sh, &vtx)? {
                li.push(process.mapped(i)?);
            }
        }
        Ok(li)
    }
}

/// True when a vertex of `sh` is in `vtx`.
fn touches(sh: &Shape, vtx: &MapOfShape) -> Result<bool, Failure> {
    let mut vsh = Explorer::new(sh, ShapeEnum::Vertex, ShapeEnum::Shape)?;
    while vsh.more() {
        if vtx.contains(vsh.current()) {
            return Ok(true);
        }
        vsh.next()?;
    }
    Ok(false)
}

impl IFSelectSelection for XSControlConnectedShapes {
    fn root_result(&self, g: &InterfaceGraph) -> Result<InterfaceEntityIterator, Failure> {
        explore_root_result(self, g)
    }
    fn fill_iterator(&self, iter: &mut IFSelectSelectionIterator) {
        deduct_fill_iterator(self, iter)
    }
    fn label(&self) -> AsciiString {
        explore_label(self)
    }
}

impl IFSelectSelectDeduct for XSControlConnectedShapes {
    fn deduct_base(&self) -> &IFSelectSelectDeductBase {
        &self.base.deduct
    }
}

impl IFSelectSelectExplore for XSControlConnectedShapes {
    fn explore_base(&self) -> &IFSelectSelectExploreBase {
        &self.base
    }
    /// `Explore(level, ent, G, explored)`: the entities connected to the shape produced
    /// from `ent` (false without reader process or shape).
    fn explore(
        &self,
        _level: i32,
        ent: &Handle<StandardTransient>,
        _g: &InterfaceGraph,
        explored: &mut InterfaceEntityIterator,
    ) -> Result<bool, Failure> {
        let tr = self.the_tr.read().clone();
        let tp = tr.get().map(|r| r.transient_process()).unwrap_or_default();
        if tp.is_null() {
            return Ok(false);
        }
        let shape = TransferBRep::shape_result_of(&tp, ent)?;
        if shape.is_null() {
            return Ok(false);
        }
        explored.add_list(&Self::adjacent_entities(&shape, &tp, ShapeEnum::Face)?);
        Ok(true)
    }
    /// `ExploreLabel`.
    fn explore_label(&self) -> AsciiString {
        AsciiString::new("Connected Entities through produced Shapes")
    }
}
