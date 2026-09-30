// Rust translation of ModelingAlgorithms/TKHLR/Intrv/Intrv_Intervals.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Intrv_Intervals: an ordered set of disjoint fuzzy intervals with the boolean
//! operations of intervals.
use super::interval::Interval;
use super::position::Position;
use rustycad_tkernel::standard::Failure;

/// Intrv_Intervals: a sequence of intervals kept sorted and disjoint by the boolean
/// operations. The operations walk the sequence as natively (a 1-based index that
/// a removal steps back and an early exit sets to the length), so the results,
/// including those on unsorted or overlapping input, are the native ones.
///
/// A native call whose tool is the object itself (`x.Unite(x)`) edits the sequence
/// while it reads it; Rust borrowing forbids that call, and a copy of the object as
/// the tool gives the result of a separate tool (docs/hlr-design.md section 3).
///
/// ```
/// use rustycad_tkhlr::intrv::{Interval, Intervals};
/// let mut set = Intervals::from_interval(Interval::from_bounds(0.0, 10.0));
/// set.subtract_interval(&Interval::from_bounds(2.0, 3.0));
/// assert_eq!(set.nb_intervals(), 2);
/// assert_eq!(set.value(2).unwrap().start(), 3.0);
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Intervals {
    inter: Vec<Interval>,
}

impl Intervals {
    /// Intrv_Intervals(): an empty set; `Intervals::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Intrv_Intervals(Int): the set of one interval;
    /// `Intervals::from_interval(Interval::from_bounds(0.0, 1.0))`.
    pub fn from_interval(interval: Interval) -> Self {
        Self {
            inter: vec![interval],
        }
    }

    /// NbIntervals: the number of intervals; `set.nb_intervals()`.
    pub fn nb_intervals(&self) -> i32 {
        self.inter.len() as i32
    }

    /// Value: the interval of rank `index` (1-based), or Standard_OutOfRange as the
    /// native sequence raises; `set.value(1)?`.
    pub fn value(&self, index: i32) -> Result<&Interval, Failure> {
        let found = usize::try_from(index - 1)
            .ok()
            .and_then(|offset| self.inter.get(offset));
        found.ok_or_else(|| {
            Failure::out_of_range(&format!(
                "Intrv_Intervals::Value: index {index} outside 1..={}",
                self.inter.len()
            ))
        })
    }

    /// Intersect(Interval): keeps the parts inside `tool`; `set.intersect_interval(&tool)`.
    pub fn intersect_interval(&mut self, tool: &Interval) {
        self.intersect(&Intervals::from_interval(*tool));
    }

    /// Intersect(Intervals): keeps the parts inside `tool`, as the native union minus
    /// the exclusive union; `set.intersect(&tool)`.
    pub fn intersect(&mut self, tool: &Intervals) {
        let mut exclusive = self.clone();
        exclusive.xunite(tool);
        self.unite(tool);
        self.subtract(&exclusive);
    }

    /// Subtract(Intervals): removes every interval of `tool` in turn; `set.subtract(&tool)`.
    pub fn subtract(&mut self, tool: &Intervals) {
        for interval in &tool.inter {
            self.subtract_interval(interval);
        }
    }

    /// Unite(Intervals): adds every interval of `tool` in turn; `set.unite(&tool)`.
    pub fn unite(&mut self, tool: &Intervals) {
        for interval in &tool.inter {
            self.unite_interval(interval);
        }
    }

    /// XUnite(Interval): keeps the parts in exactly one of the set and `tool`;
    /// `set.xunite_interval(&tool)`.
    pub fn xunite_interval(&mut self, tool: &Interval) {
        self.xunite(&Intervals::from_interval(*tool));
    }

    /// XUnite(Intervals): keeps the parts in exactly one of the set and `tool`
    /// (`tool` minus the set, united with the set minus `tool`); `set.xunite(&tool)`.
    pub fn xunite(&mut self, tool: &Intervals) {
        let mut tool_only = tool.clone();
        tool_only.subtract(self);
        self.subtract(tool);
        self.unite(&tool_only);
    }

    /// Subtract(Interval): removes `tool` from the set; `set.subtract_interval(&tool)`.
    pub fn subtract_interval(&mut self, tool: &Interval) {
        // `index` is the native 1-based index: a removal steps it back, an exit sets
        // it to the length, and the loop steps it forward.
        let mut index = 1_usize;
        while index <= self.inter.len() {
            index = self.subtract_step(tool, index) + 1;
        }
    }

    /// One pass of the native Subtract loop at `index`; returns the index the loop
    /// steps forward from.
    fn subtract_step(&mut self, tool: &Interval, index: usize) -> usize {
        let exit = self.inter.len();
        let current = &mut self.inter[index - 1];
        match tool.position(current) {
            Position::Before => exit,
            Position::JustBefore => {
                current.cut_at_start(tool.end, tool.tol_end);
                exit
            }
            Position::OverlappingAtStart | Position::JustOverlappingAtStart => {
                current.set_start(tool.end, tool.tol_end);
                exit
            }
            Position::JustEnclosingAtEnd
            | Position::Enclosing
            | Position::Similar
            | Position::JustEnclosingAtStart => {
                self.inter.remove(index - 1);
                index - 1
            }
            Position::Inside => self.split_around(tool, index),
            Position::JustOverlappingAtEnd | Position::OverlappingAtEnd => {
                current.set_end(tool.start, tool.tol_start);
                index
            }
            Position::JustAfter => {
                current.cut_at_end(tool.start, tool.tol_start);
                index
            }
            Position::After => index,
        }
    }

    /// The Inside case of Subtract: the interval at `index` is split into the part
    /// before `tool` and the part after it; the loop then exits (the native index is
    /// the length after the split).
    fn split_around(&mut self, tool: &Interval, index: usize) -> usize {
        let copy = self.inter[index - 1];
        self.inter.insert(index, copy);
        self.inter[index - 1].set_end(tool.start, tool.tol_start);
        self.inter[index].set_start(tool.end, tool.tol_end);
        self.inter.len()
    }

    /// Unite(Interval): adds `tool` to the set, fusing it with the intervals it meets;
    /// `set.unite_interval(&tool)`.
    pub fn unite_interval(&mut self, tool: &Interval) {
        let mut state = UniteState {
            tins: *tool,
            inserted: false,
        };
        let mut index = 1_usize;
        while index <= self.inter.len() {
            index = self.unite_step(&mut state, index) + 1;
        }
        if !state.inserted {
            self.inter.push(state.tins);
        }
    }

    /// One pass of the native Unite loop at `index`; returns the index the loop steps
    /// forward from.
    fn unite_step(&mut self, state: &mut UniteState, index: usize) -> usize {
        let exit = self.inter.len();
        let current = self.inter[index - 1];
        match state.tins.position(&current) {
            Position::Before => {
                state.inserted = true;
                self.inter.insert(index - 1, state.tins);
                // The native exit is the length after the insertion.
                exit + 1
            }
            Position::JustBefore | Position::OverlappingAtStart => {
                state.inserted = true;
                self.inter[index - 1].set_start(state.tins.start, state.tins.tol_start);
                exit
            }
            Position::JustOverlappingAtStart => {
                state.inserted = true;
                self.inter[index - 1].fuse_at_start(state.tins.start, state.tins.tol_start);
                exit
            }
            Position::Inside => {
                state.inserted = true;
                exit
            }
            Position::After => index,
            absorbed => self.absorb(state, absorbed, index),
        }
    }

    /// The Unite cases that absorb the interval at `index` into the inserted one and
    /// remove it (the native fallthrough chain Similar, JustEnclosingAtEnd, Enclosing,
    /// and the cases that take its start).
    fn absorb(&mut self, state: &mut UniteState, position: Position, index: usize) -> usize {
        let current = self.inter[index - 1];
        let tins = &mut state.tins;
        match position {
            Position::Similar => {
                tins.fuse_at_start(current.start, current.tol_start);
                tins.fuse_at_end(current.end, current.tol_end);
            }
            Position::JustEnclosingAtEnd => tins.fuse_at_end(current.end, current.tol_end),
            Position::JustOverlappingAtEnd => {
                tins.set_start(current.start, current.tol_start);
                tins.fuse_at_end(current.end, current.tol_end);
            }
            Position::JustEnclosingAtStart => tins.fuse_at_start(current.start, current.tol_start),
            Position::OverlappingAtEnd | Position::JustAfter => {
                tins.set_start(current.start, current.tol_start)
            }
            // Enclosing only removes; the other positions never reach this function.
            _ => {}
        }
        self.inter.remove(index - 1);
        index - 1
    }
}

/// The locals of the native Unite loop: the interval being inserted, as it grows,
/// and whether it has found its place.
struct UniteState {
    tins: Interval,
    inserted: bool,
}
