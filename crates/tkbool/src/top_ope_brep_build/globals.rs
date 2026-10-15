//! The global variables of TopOpeBRepBuild (`GLOBAL_*` of TopOpeBRepBuild_GridFF.cxx,
//! _GridSS.cxx, _Section.cxx, _ffsfs.cxx and _Builder1.cxx), thread-local here as the
//! process-wide natives are only read and written within one build.
use crate::top_ope_brep_ds::TopOpeBRepDSHDataStructure;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Handle;
use std::cell::{Cell, RefCell};

/// `GLOBAL_issp` values: the state of the section edge split (TheIN, TheON, TheOUT).
pub(crate) const THE_IN: i32 = 1;
/// TheON.
pub(crate) const THE_ON: i32 = 2;
/// TheOUT.
pub(crate) const THE_OUT: i32 = 3;

thread_local! {
    /// `GLOBAL_faces2d`: the faces of the operation are split in 2d (same domain).
    pub(crate) static GLOBAL_FACES2D: Cell<bool> = const { Cell::new(false) };
    /// `GLOBAL_classifysplitedge`.
    pub(crate) static GLOBAL_CLASSIFYSPLITEDGE: Cell<bool> = const { Cell::new(false) };
    /// `GLOBAL_IEtoMERGE`.
    pub(crate) static GLOBAL_IETOMERGE: Cell<bool> = const { Cell::new(false) };
    /// `GLOBAL_issp`: the split of a section edge in progress (0, THE_IN, THE_ON, THE_OUT).
    pub(crate) static GLOBAL_ISSP: Cell<i32> = const { Cell::new(0) };
    /// `GLOBAL_hassd`: the section edge has same domain edges (2 in 2d, 3 in 3d).
    pub(crate) static GLOBAL_HASSD: Cell<i32> = const { Cell::new(0) };
    /// `GLOBAL_iexE`.
    pub(crate) static GLOBAL_IEXE: Cell<i32> = const { Cell::new(0) };
    /// `GLOBAL_iexF`.
    pub(crate) static GLOBAL_IEXF: Cell<i32> = const { Cell::new(0) };
    /// `GLOBAL_revownsplfacori`.
    pub(crate) static GLOBAL_REVOWNSPLFACORI: Cell<bool> = const { Cell::new(false) };
    /// `GLOBAL_lfrtoprocess`.
    pub(crate) static GLOBAL_LFRTOPROCESS: Cell<bool> = const { Cell::new(false) };
    /// `GLOBAL_USE_NEW_BUILDER`.
    pub(crate) static GLOBAL_USE_NEW_BUILDER: Cell<bool> = const { Cell::new(false) };
    /// `GLOBAL_lfr1` (a null pointer natively when None).
    pub(crate) static GLOBAL_LFR1: RefCell<Option<List<Shape>>> = const { RefCell::new(None) };
    /// `GLOBAL_SplitAnc` (a null pointer natively when None).
    pub(crate) static GLOBAL_SPLIT_ANC: RefCell<Option<DataMap<Shape, i32, ShapeMapHasher>>> = const { RefCell::new(None) };
    /// `GLOBAL_DS2d`: the 2d data structure (a null pointer natively when None).
    pub(crate) static GLOBAL_DS2D: RefCell<Option<Handle<TopOpeBRepDSHDataStructure>>> = const { RefCell::new(None) };
}
