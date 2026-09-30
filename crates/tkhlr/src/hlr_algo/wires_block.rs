// Rust translation of ModelingAlgorithms/TKHLR/HLRAlgo/HLRAlgo_WiresBlock.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! HLRAlgo_WiresBlock: the wires of one face, each an edges block.
use super::edges_block::EdgesBlock;
use super::failures::array_failure;
use super::min_max::MinMaxIndices;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};

struct WiresBlockState {
    wires: Array1<'static, Handle<EdgesBlock>>,
    min_max: MinMaxIndices,
}

/// HLRAlgo_WiresBlock: the edges blocks of the wires of a face (1-based, null until
/// set) and the packed box of the face. A transient shared by handles, with its fields
/// behind a lock (docs/hlr-design.md section 5).
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkhlr::hlr_algo::{EdgesBlock, WiresBlock};
/// let wires = WiresBlock::new(2).unwrap();
/// assert!(wires.wire(1).unwrap().is_null());
/// wires.set(1, Handle::new(EdgesBlock::new(3).unwrap())).unwrap();
/// assert_eq!(wires.wire(1).unwrap().get().unwrap().nb_edges(), 3);
/// ```
pub struct WiresBlock {
    base: StandardTransient,
    state: SharedState<WiresBlockState>,
}
impl_transient!(WiresBlock, "HLRAlgo_WiresBlock", StandardTransient, base);

impl WiresBlock {
    /// HLRAlgo_WiresBlock(NbWires): `nb_wires` null wires; Standard_RangeError for a
    /// negative count; `WiresBlock::new(2)?`.
    pub fn new(nb_wires: i32) -> Result<Self, Failure> {
        let wires = Array1::with_bounds(1, nb_wires)
            .map_err(array_failure("HLRAlgo_WiresBlock::HLRAlgo_WiresBlock"))?;
        let state = WiresBlockState {
            wires,
            min_max: MinMaxIndices::default(),
        };
        Ok(Self {
            base: StandardTransient::default(),
            state: SharedState::new(state),
        })
    }

    /// NbWires: the upper bound of the wire array; `wires.nb_wires()`.
    pub fn nb_wires(&self) -> i32 {
        self.state.read().wires.upper()
    }

    /// Set(I, W): stores the edges block of the wire at rank `i`; `wires.set(1, block)?`.
    pub fn set(&self, i: i32, wire: Handle<EdgesBlock>) -> Result<(), Failure> {
        let mut state = self.state.write();
        let slot = state.wires.change_value(i);
        *slot.map_err(array_failure("HLRAlgo_WiresBlock::Set"))? = wire;
        Ok(())
    }

    /// Wire(I): the edges block of the wire at rank `i` (a shared handle; null when
    /// unset); `wires.wire(1)?`.
    pub fn wire(&self, i: i32) -> Result<Handle<EdgesBlock>, Failure> {
        let state = self.state.read();
        let slot = state.wires.value(i);
        slot.cloned()
            .map_err(array_failure("HLRAlgo_WiresBlock::Wire"))
    }

    /// UpdateMinMax: replaces the packed box; `wires.update_min_max(&packed)`.
    pub fn update_min_max(&self, min_maxes: &MinMaxIndices) {
        self.state.write().min_max = *min_maxes;
    }

    /// MinMax(): the packed box (a copy, see `EdgesBlock::min_max`); `wires.min_max()`.
    pub fn min_max(&self) -> MinMaxIndices {
        self.state.read().min_max
    }
}
