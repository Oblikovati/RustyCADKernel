// Rust translation of ModelingAlgorithms/TKHLR/HLRAlgo/HLRAlgo_EdgesBlock.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! HLRAlgo_EdgesBlock: the edges of one wire with their orientation and flags.
use super::failures::array_failure;
use super::min_max::MinMaxIndices;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use rustycad_tkg3d::top_abs::Orientation;

const MASK_ORIENT: i32 = 15;
const MASK_OUT_LINE: i32 = 16;
const MASK_INTERNAL: i32 = 32;
const MASK_DOUBLE: i32 = 64;
const MASK_ISO_LINE: i32 = 128;

struct EdgesBlockState {
    edges: Array1<'static, i32>,
    flags: Array1<'static, i32>,
    min_max: MinMaxIndices,
}

/// HLRAlgo_EdgesBlock: the edge indices of a wire (1-based) with a flag word each (the
/// orientation in the low 4 bits, then the outline, internal, double and iso-line
/// flags) and the packed box of the wire. It is a transient shared by handles, so its
/// fields sit behind a lock and every member takes `&self` (docs/hlr-design.md
/// section 5). Natively the arrays are left uninitialized; the Rust ones start at 0.
///
/// ```
/// use rustycad_tkg3d::top_abs::Orientation;
/// use rustycad_tkhlr::hlr_algo::EdgesBlock;
/// let block = EdgesBlock::new(2).unwrap();
/// block.set_edge(2, 7).unwrap();
/// block.set_orientation(2, Orientation::Reversed).unwrap();
/// block.set_out_line(2, true).unwrap();
/// assert_eq!(block.edge(2).unwrap(), 7);
/// assert_eq!(block.orientation(2).unwrap(), Orientation::Reversed);
/// assert!(block.out_line(2).unwrap() && !block.double(2).unwrap());
/// ```
pub struct EdgesBlock {
    base: StandardTransient,
    state: SharedState<EdgesBlockState>,
}
impl_transient!(EdgesBlock, "HLRAlgo_EdgesBlock", StandardTransient, base);

impl EdgesBlock {
    /// HLRAlgo_EdgesBlock(NbEdges): a block of `nb_edges` edges; Standard_RangeError
    /// for a negative count, as the native arrays raise; `EdgesBlock::new(3)?`.
    pub fn new(nb_edges: i32) -> Result<Self, Failure> {
        let bounds = array_failure("HLRAlgo_EdgesBlock::HLRAlgo_EdgesBlock");
        let state = EdgesBlockState {
            edges: Array1::with_bounds(1, nb_edges).map_err(&bounds)?,
            flags: Array1::with_bounds(1, nb_edges).map_err(&bounds)?,
            min_max: MinMaxIndices::default(),
        };
        Ok(Self {
            base: StandardTransient::default(),
            state: SharedState::new(state),
        })
    }

    /// NbEdges: the upper bound of the edge array; `block.nb_edges()`.
    pub fn nb_edges(&self) -> i32 {
        self.state.read().edges.upper()
    }

    /// Edge(I, EI): stores the edge index `ei` at rank `i`; `block.set_edge(1, 4)?`.
    pub fn set_edge(&self, i: i32, ei: i32) -> Result<(), Failure> {
        let mut state = self.state.write();
        let slot = state.edges.change_value(i);
        *slot.map_err(array_failure("HLRAlgo_EdgesBlock::Edge"))? = ei;
        Ok(())
    }

    /// Edge(I): the edge index at rank `i`; `block.edge(1)?`.
    pub fn edge(&self, i: i32) -> Result<i32, Failure> {
        let state = self.state.read();
        let value = state.edges.value(i);
        value
            .copied()
            .map_err(array_failure("HLRAlgo_EdgesBlock::Edge"))
    }

    fn flags(&self, i: i32) -> Result<i32, Failure> {
        let state = self.state.read();
        let value = state.flags.value(i);
        value
            .copied()
            .map_err(array_failure("HLRAlgo_EdgesBlock::Flags"))
    }

    fn update_flags(&self, i: i32, edit: impl FnOnce(&mut i32)) -> Result<(), Failure> {
        let mut state = self.state.write();
        let slot = state.flags.change_value(i);
        edit(slot.map_err(array_failure("HLRAlgo_EdgesBlock::Flags"))?);
        Ok(())
    }

    fn set_mask(&self, i: i32, mask: i32, on: bool) -> Result<(), Failure> {
        self.update_flags(i, |flags| {
            if on {
                *flags |= mask;
            } else {
                *flags &= !mask;
            }
        })
    }

    /// Orientation(I, Or): stores the orientation of the edge at rank `i`;
    /// `block.set_orientation(1, Orientation::Forward)?`.
    pub fn set_orientation(&self, i: i32, or: Orientation) -> Result<(), Failure> {
        self.update_flags(i, |flags| {
            *flags &= !MASK_ORIENT;
            *flags |= (or as i32) & MASK_ORIENT;
        })
    }

    /// Orientation(I): the orientation of the edge at rank `i`, from the low 4 bits of
    /// its flags. Only the values 0..3 are orientations: the setter never stores
    /// others; `block.orientation(1)?`.
    pub fn orientation(&self, i: i32) -> Result<Orientation, Failure> {
        let bits = self.flags(i)? & MASK_ORIENT;
        Orientation::ALL.get(bits as usize).copied().ok_or_else(|| {
            Failure::with_message(
                "Standard_OutOfRange",
                &format!("HLRAlgo_EdgesBlock::Orientation: flags {bits}, expected 0..3"),
            )
        })
    }

    /// OutLine(I); `block.out_line(1)?`.
    pub fn out_line(&self, i: i32) -> Result<bool, Failure> {
        Ok(self.flags(i)? & MASK_OUT_LINE != 0)
    }

    /// OutLine(I, B); `block.set_out_line(1, true)?`.
    pub fn set_out_line(&self, i: i32, on: bool) -> Result<(), Failure> {
        self.set_mask(i, MASK_OUT_LINE, on)
    }

    /// Internal(I); `block.internal(1)?`.
    pub fn internal(&self, i: i32) -> Result<bool, Failure> {
        Ok(self.flags(i)? & MASK_INTERNAL != 0)
    }

    /// Internal(I, B); `block.set_internal(1, true)?`.
    pub fn set_internal(&self, i: i32, on: bool) -> Result<(), Failure> {
        self.set_mask(i, MASK_INTERNAL, on)
    }

    /// Double(I); `block.double(1)?`.
    pub fn double(&self, i: i32) -> Result<bool, Failure> {
        Ok(self.flags(i)? & MASK_DOUBLE != 0)
    }

    /// Double(I, B); `block.set_double(1, true)?`.
    pub fn set_double(&self, i: i32, on: bool) -> Result<(), Failure> {
        self.set_mask(i, MASK_DOUBLE, on)
    }

    /// IsoLine(I); `block.iso_line(1)?`.
    pub fn iso_line(&self, i: i32) -> Result<bool, Failure> {
        Ok(self.flags(i)? & MASK_ISO_LINE != 0)
    }

    /// IsoLine(I, B); `block.set_iso_line(1, true)?`.
    pub fn set_iso_line(&self, i: i32, on: bool) -> Result<(), Failure> {
        self.set_mask(i, MASK_ISO_LINE, on)
    }

    /// UpdateMinMax: replaces the packed box; `block.update_min_max(&packed)`.
    pub fn update_min_max(&self, tot_min_max: &MinMaxIndices) {
        self.state.write().min_max = *tot_min_max;
    }

    /// MinMax(): the packed box (a copy; the native reference is written through
    /// `update_min_max`); `block.min_max()`.
    pub fn min_max(&self) -> MinMaxIndices {
        self.state.read().min_max
    }
}
