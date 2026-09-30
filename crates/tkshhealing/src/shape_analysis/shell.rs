// Rust translation of ModelingAlgorithms/TKShHealing/ShapeAnalysis/ShapeAnalysis_Shell.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeAnalysis_Shell`: the orientation of the edges of shells (bad, free and
//! connected edges).
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Compound, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// `ShapeAnalysis_Shell`: loads shells and checks that each edge of them is used once
/// (free) or twice with opposite orientations (connected); other edges are bad.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkshhealing::shape_analysis::ShapeAnalysisShell;
/// let mut tool = ShapeAnalysisShell::new();
/// assert!(!tool.check_oriented_shells(&Shape::default(), true, false).unwrap());
/// assert_eq!(tool.nb_loaded(), 0);
/// ```
#[derive(Default)]
pub struct ShapeAnalysisShell {
    /// `myShells`: the loaded shells (and the shells found with bad edges).
    shells: IndexedMapOfShape,
    /// `myBad`: edges used twice with the same orientation.
    bad: IndexedMapOfShape,
    /// `myFree`: edges used once.
    free: IndexedMapOfShape,
    /// `myConex`: an edge is used twice or more.
    conex: bool,
}

/// The edge maps `CheckEdges` fills: FORWARD, REVERSED and INTERNAL uses.
#[derive(Default)]
struct EdgeUses {
    dirs: IndexedMapOfShape,
    revs: IndexedMapOfShape,
    ints: IndexedMapOfShape,
}

/// The static `CheckEdges`: records the non-degenerated edges of `shape` by orientation;
/// an edge met twice with the same (FORWARD or REVERSED) orientation goes to `bads` and
/// makes the result true.
fn check_edges(
    shape: &Shape,
    bads: &mut IndexedMapOfShape,
    uses: &mut EdgeUses,
) -> Result<bool, Failure> {
    if shape.shape_type()? != ShapeEnum::Edge {
        let mut res = false;
        let mut it = ShapeIterator::new(shape, true, true);
        while it.more() {
            if check_edges(it.value()?, bads, uses)? {
                res = true;
            }
            it.next();
        }
        return Ok(res);
    }
    if BRepTool::degenerated(&topods::edge(shape)?)? {
        return Ok(false);
    }
    let map = match shape.orientation() {
        Orientation::Forward => &mut uses.dirs,
        Orientation::Reversed => &mut uses.revs,
        Orientation::Internal => {
            if uses.ints.find_index(shape) == 0 {
                uses.ints.add(shape.clone());
            }
            return Ok(false);
        }
        // An EXTERNAL edge is ignored.
        Orientation::External => return Ok(false),
    };
    if map.find_index(shape) == 0 {
        map.add(shape.clone());
        return Ok(false);
    }
    bads.add(shape.clone());
    Ok(true)
}

impl ShapeAnalysisShell {
    /// `ShapeAnalysis_Shell()`; `ShapeAnalysisShell::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Clear()`: forgets the loaded shells and the checked edges; `tool.clear()`.
    pub fn clear(&mut self) {
        self.shells.clear(true);
        self.bad.clear(true);
        self.free.clear(true);
        self.conex = false;
    }

    /// `LoadShells(shape)`: loads `shape` if it is a shell, else each shell it holds
    /// (nothing for a null shape); `tool.load_shells(&shape)?`.
    pub fn load_shells(&mut self, shape: &Shape) -> Result<(), Failure> {
        if shape.is_null() {
            return Ok(());
        }
        if shape.shape_type()? == ShapeEnum::Shell {
            self.shells.add(shape.clone());
            return Ok(());
        }
        for shell in Explorer::new(shape, ShapeEnum::Shell, ShapeEnum::Shape)? {
            self.shells.add(shell);
        }
        Ok(())
    }

    /// `CheckOrientedShells(shape, alsofree, checkinternaledges)`: records the bad edges
    /// of the shells of `shape` (a shell with a bad edge is loaded, and the result is
    /// true). With `also_free` the edges used once are
    /// recorded as free, and the connected flag is set; with `check_internal_edges` an
    /// edge also used INTERNAL counts as connected;
    /// `tool.check_oriented_shells(&shape, true, false)?`.
    pub fn check_oriented_shells(
        &mut self,
        shape: &Shape,
        also_free: bool,
        check_internal_edges: bool,
    ) -> Result<bool, Failure> {
        self.conex = false;
        if shape.is_null() {
            return Ok(false);
        }
        let mut res = false;
        let mut uses = EdgeUses::default();
        for shell in Explorer::new(shape, ShapeEnum::Shell, ShapeEnum::Shape)? {
            // `IndexedMap::Add` returns the (non-zero) index of the key, so the native
            // `if (myShells.Add(sh))` holds for a shell loaded before too.
            if check_edges(&shell, &mut self.bad, &mut uses)? && self.shells.add(shell) != 0 {
                res = true;
            }
        }
        if !also_free {
            return Ok(res);
        }
        self.record_free(&uses.dirs, &uses.revs, &uses.ints, check_internal_edges)?;
        self.record_free(&uses.revs, &uses.dirs, &uses.ints, check_internal_edges)?;
        Ok(res)
    }

    /// One of the two free-edge loops: an edge of `these` that is not bad and is not in
    /// `others` (nor, when asked, in `ints`) is free; otherwise it is connected.
    fn record_free(
        &mut self,
        these: &IndexedMapOfShape,
        others: &IndexedMapOfShape,
        ints: &IndexedMapOfShape,
        check_internal_edges: bool,
    ) -> Result<(), Failure> {
        for i in 1..=these.extent() {
            let edge = these.find_key(i)?;
            let free = !self.bad.contains(edge)
                && !others.contains(edge)
                && !(check_internal_edges && ints.contains(edge));
            if free {
                self.free.add(edge.clone());
            } else {
                self.conex = true;
            }
        }
        Ok(())
    }

    /// `IsLoaded(shape)`: `shape` is a loaded shell (false for a null shape);
    /// `tool.is_loaded(&shell)`.
    pub fn is_loaded(&self, shape: &Shape) -> bool {
        !shape.is_null() && self.shells.contains(shape)
    }

    /// `NbLoaded()`: the count of loaded shells; `tool.nb_loaded()`.
    pub fn nb_loaded(&self) -> i32 {
        self.shells.extent() as i32
    }

    /// `Loaded(num)`: the loaded shell of rank `num`; Standard_OutOfRange outside
    /// `1..=NbLoaded()` (natively `FindKey` raises, although the header promises a null
    /// shape); `tool.loaded(1)?`.
    pub fn loaded(&self, num: i32) -> Result<Shape, Failure> {
        find_key(&self.shells, num, "ShapeAnalysis_Shell::Loaded")
    }

    /// `HasBadEdges()`: an edge is recorded as bad; `tool.has_bad_edges()`.
    pub fn has_bad_edges(&self) -> bool {
        self.bad.extent() > 0
    }

    /// `BadEdges()`: the bad edges in a compound (empty, not null, when there is none);
    /// `tool.bad_edges()?`.
    pub fn bad_edges(&self) -> Result<Compound, Failure> {
        compound_of(&self.bad)
    }

    /// `HasFreeEdges()`: an edge is recorded as free; `tool.has_free_edges()`.
    pub fn has_free_edges(&self) -> bool {
        self.free.extent() > 0
    }

    /// `FreeEdges()`: the free edges in a compound (empty, not null, when there is none);
    /// `tool.free_edges()?`.
    pub fn free_edges(&self) -> Result<Compound, Failure> {
        compound_of(&self.free)
    }

    /// `HasConnectedEdges()`: an edge is shared twice or more; `tool.has_connected_edges()`.
    pub fn has_connected_edges(&self) -> bool {
        self.conex
    }
}

/// `map.FindKey(num)`, with Standard_OutOfRange for a rank outside the map.
fn find_key(map: &IndexedMapOfShape, num: i32, member: &str) -> Result<Shape, Failure> {
    let index = usize::try_from(num).unwrap_or(0);
    if index == 0 || index > map.extent() {
        return Err(Failure::with_message(
            "Standard_OutOfRange",
            &format!("{member}: rank {num}; expected 1..={}", map.extent()),
        ));
    }
    Ok(map.find_key(index)?.clone())
}

/// A compound of the keys of `map`, in map order.
fn compound_of(map: &IndexedMapOfShape) -> Result<Compound, Failure> {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    for i in 1..=map.extent() {
        builder.add(&mut compound, map.find_key(i)?)?;
    }
    Ok(compound)
}
