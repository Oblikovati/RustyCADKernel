// Rust translation of ModelingAlgorithms/TKShHealing/ShapeAnalysis/ShapeAnalysis_WireOrder.hxx and the
// members of ShapeAnalysis_WireOrder.cxx other than Perform at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeAnalysis_WireOrder`: the order of a list of edges given by their end points.
mod perform;

use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Xy, Xyz};

/// The private `ModeType` of the class.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum ModeType {
    Mode2D,
    #[default]
    Mode3D,
    ModeBoth,
}

impl ModeType {
    fn of(mode_3d: bool, mode_both: bool) -> Self {
        if mode_both {
            ModeType::ModeBoth
        } else if mode_3d {
            ModeType::Mode3D
        } else {
            ModeType::Mode2D
        }
    }
}

/// `ShapeAnalysis_WireOrder`: proposes an order (and reversals) for edges given as
/// couples of points (start, end) in 3D, in 2D, or both, so that each edge starts where
/// the previous one ends; then reports gaps, chains and couples of that order.
///
/// Numbers are 1-based; a negative number in `Ordered` means the edge is reversed.
///
/// ```
/// use rustycad_tkmath::gp::Xyz;
/// use rustycad_tkshhealing::shape_analysis::ShapeAnalysisWireOrder;
/// let mut order = ShapeAnalysisWireOrder::new();
/// order.add_3d(Xyz::new(1.0, 0.0, 0.0), Xyz::new(1.0, 1.0, 0.0));
/// order.add_3d(Xyz::new(0.0, 0.0, 0.0), Xyz::new(1.0, 0.0, 0.0));
/// order.perform(true).unwrap();
/// assert_eq!(order.status(), 3);
/// assert_eq!(order.ordered(1).unwrap(), 2);
/// ```
#[derive(Clone, Debug)]
pub struct ShapeAnalysisWireOrder {
    /// `myOrd`: the new order (null natively until `Perform`).
    ord: Option<Vec<i32>>,
    /// `myChains`: the first edge of each chain.
    chains: Option<Vec<i32>>,
    /// `myCouples`: never filled (`SetCouples` is not implemented natively).
    couples: Option<Vec<i32>>,
    /// `myXYZ`: the 3D points (the 2D points with Z = 0 in the 2D mode), two per edge.
    xyz: Vec<Xyz>,
    /// `myXY`: the 2D points of the miscible mode.
    xy: Vec<Xy>,
    /// `myTol`: kept and returned, not used by the algorithm.
    tol: f64,
    /// `myGap`: never computed natively (always 0).
    gap: f64,
    /// `myStat`.
    stat: i32,
    /// `myKeepLoops`.
    keep_loops: bool,
    /// `myMode`.
    mode: ModeType,
}

impl Default for ShapeAnalysisWireOrder {
    /// `ShapeAnalysis_WireOrder()`: the 3D mode with the tolerance `Precision::Confusion()`.
    fn default() -> Self {
        Self::with_mode(true, precision::confusion(), false)
    }
}

/// The 1-based item `index` of the point list, or Standard_OutOfRange as the native
/// sequence raises.
fn item<T: Copy>(list: &[T], index: i32, member: &str) -> Result<T, Failure> {
    usize::try_from(index)
        .ok()
        .filter(|&i| i >= 1 && i <= list.len())
        .map(|i| list[i - 1])
        .ok_or_else(|| {
            Failure::with_message(
                "Standard_OutOfRange",
                &format!("{member}: point {index}; expected 1..={}", list.len()),
            )
        })
}

/// The indices of the start and end points of the edge number `idx` (reversed when
/// negative).
fn point_indices(idx: i32) -> (i32, i32) {
    if idx > 0 {
        (2 * idx - 1, 2 * idx)
    } else {
        (-2 * idx, -2 * idx - 1)
    }
}

impl ShapeAnalysisWireOrder {
    /// `ShapeAnalysis_WireOrder()`; `ShapeAnalysisWireOrder::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `ShapeAnalysis_WireOrder(theMode3D, theTolerance, theModeBoth)`: the miscible mode
    /// when `mode_both`, else 3D or 2D; the tolerance is kept but not used;
    /// `ShapeAnalysisWireOrder::with_mode(true, 1e-7, false)`.
    pub fn with_mode(mode_3d: bool, tolerance: f64, mode_both: bool) -> Self {
        Self {
            ord: None,
            chains: None,
            couples: None,
            xyz: Vec::new(),
            xy: Vec::new(),
            tol: tolerance,
            gap: 0.0,
            stat: 0,
            keep_loops: false,
            mode: ModeType::of(mode_3d, mode_both),
        }
    }

    /// `SetMode(theMode3D, theTolerance, theModeBoth)`: clears the points when the mode
    /// changes, forgets the order, and keeps the tolerance (1e-8 when it is not positive);
    /// `order.set_mode(false, 1e-7, false)`.
    pub fn set_mode(&mut self, mode_3d: bool, tolerance: f64, mode_both: bool) {
        let new_mode = ModeType::of(mode_3d, mode_both);
        if self.mode != new_mode {
            self.clear();
        }
        self.mode = new_mode;
        self.ord = None;
        self.stat = 0;
        self.gap = 0.0;
        self.tol = if tolerance > 0.0 { tolerance } else { 1.0e-08 };
    }

    /// `Tolerance()`; `order.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.tol
    }

    /// `Clear()`: forgets the points (not the mode, the tolerance nor the last order);
    /// `order.clear()`.
    pub fn clear(&mut self) {
        self.xyz = Vec::new();
        self.xy = Vec::new();
        self.stat = 0;
        self.gap = 0.0;
    }

    /// `Add(theStart3d, theEnd3d)`: adds an edge in the 3D mode only;
    /// `order.add_3d(start, end)`.
    pub fn add_3d(&mut self, start: Xyz, end: Xyz) {
        if self.mode == ModeType::Mode3D {
            self.xyz.push(start);
            self.xyz.push(end);
        }
    }

    /// `Add(theStart2d, theEnd2d)`: adds an edge in the 2D mode only (as 3D points with
    /// Z = 0); `order.add_2d(start, end)`.
    pub fn add_2d(&mut self, start: Xy, end: Xy) {
        if self.mode == ModeType::Mode2D {
            self.xyz.push(Xyz::new(start.x(), start.y(), 0.0));
            self.xyz.push(Xyz::new(end.x(), end.y(), 0.0));
        }
    }

    /// `Add(theStart3d, theEnd3d, theStart2d, theEnd2d)`: adds an edge in the miscible
    /// mode only; `order.add_both(s3, e3, s2, e2)`.
    pub fn add_both(&mut self, start_3d: Xyz, end_3d: Xyz, start_2d: Xy, end_2d: Xy) {
        if self.mode == ModeType::ModeBoth {
            self.xyz.push(start_3d);
            self.xyz.push(end_3d);
            self.xy.push(start_2d);
            self.xy.push(end_2d);
        }
    }

    /// `NbEdges()`: the count of added couples; `order.nb_edges()`.
    pub fn nb_edges(&self) -> i32 {
        (self.xyz.len() / 2) as i32
    }

    /// `KeepLoopsMode()` read: the loops are kept one after another instead of merged;
    /// `order.keep_loops_mode()`.
    pub fn keep_loops_mode(&self) -> bool {
        self.keep_loops
    }

    /// `KeepLoopsMode() = mode` (the native member returns a reference);
    /// `order.set_keep_loops_mode(true)`.
    pub fn set_keep_loops_mode(&mut self, mode: bool) {
        self.keep_loops = mode;
    }

    /// `IsDone()`: an order was computed (it stays after `Clear`, and a `Perform` on no
    /// edge keeps the previous one); `order.is_done()`.
    pub fn is_done(&self) -> bool {
        self.ord.is_some()
    }

    /// `Status()`: 0 in order, 1 reordered, -1 some edges reversed, 3 only shifted;
    /// `order.status()`.
    pub fn status(&self) -> i32 {
        self.stat
    }

    /// `Ordered(theIdx)`: the original edge at the new rank `idx` (negative when
    /// reversed); `idx` itself without an order or beyond it; Standard_OutOfRange for a
    /// rank below 1 once an order exists; `order.ordered(1)?`.
    pub fn ordered(&self, idx: i32) -> Result<i32, Failure> {
        let Some(ord) = &self.ord else {
            return Ok(idx);
        };
        if ord.len() < idx.max(0) as usize {
            return Ok(idx);
        }
        let old = item(ord, idx, "ShapeAnalysis_WireOrder::Ordered")?;
        Ok(if old == 0 { idx } else { old })
    }

    /// `XYZ(theIdx, theStart3D, theEnd3D)`: the 3D points of edge `idx` (swapped when
    /// negative); Standard_OutOfRange for an edge that was not added;
    /// `order.xyz(-2)?`.
    pub fn xyz(&self, idx: i32) -> Result<(Xyz, Xyz), Failure> {
        let (s, e) = point_indices(idx);
        let member = "ShapeAnalysis_WireOrder::XYZ";
        Ok((item(&self.xyz, s, member)?, item(&self.xyz, e, member)?))
    }

    /// `XY(theIdx, theStart2D, theEnd2D)`: the 2D points of edge `idx` (the X and Y of the
    /// 3D points outside the miscible mode); `order.xy(1)?`.
    pub fn xy(&self, idx: i32) -> Result<(Xy, Xy), Failure> {
        let (s, e) = point_indices(idx);
        let member = "ShapeAnalysis_WireOrder::XY";
        if self.mode == ModeType::ModeBoth {
            return Ok((item(&self.xy, s, member)?, item(&self.xy, e, member)?));
        }
        let flat = |p: Xyz| Xy::new(p.x(), p.y());
        Ok((
            flat(item(&self.xyz, s, member)?),
            flat(item(&self.xyz, e, member)?),
        ))
    }

    /// `Gap(num)`: `myGap` (never computed, 0) for `num == 0`, else the sum of the
    /// coordinate distances between the end of the ordered edge before `num` and the start
    /// of `num`; `order.gap(2)?`.
    pub fn gap(&self, num: i32) -> Result<f64, Failure> {
        if num == 0 {
            return Ok(self.gap);
        }
        let n1 = self.ordered(num)?;
        let n0 = self.ordered(if num == 1 { self.nb_edges() } else { num - 1 })?;
        let member = "ShapeAnalysis_WireOrder::Gap";
        let end = item(&self.xyz, point_indices(n0).1, member)?;
        let start = item(&self.xyz, point_indices(n1).0, member)?;
        Ok(dist_abs(end, start))
    }

    /// `SetChains(gap)`: splits the order into chains where the start of an edge is not
    /// within `gap` (per coordinate) of the end of the previous one; `order.set_chains(1e-3)?`.
    pub fn set_chains(&mut self, gap: f64) -> Result<(), Failure> {
        let nb = self.nb_edges();
        if nb == 0 {
            return Ok(());
        }
        let mut chain = vec![1];
        let mut n0 = 0;
        let mut l13d = Xyz::default();
        for n1 in 1..=nb {
            if n0 == 0 {
                n0 = n1;
                l13d = self.xyz(self.ordered(n0)?)?.1;
            }
            let n2 = if n1 == nb { n0 } else { n1 + 1 };
            let (f3d, l3d) = self.xyz(self.ordered(n2)?)?;
            if !f3d.is_equal(&l13d, gap) {
                chain.push(n2);
                n0 = 0;
            }
            l13d = l3d;
        }
        self.chains = Some(chain);
        Ok(())
    }

    /// `NbChains()`; `order.nb_chains()`.
    pub fn nb_chains(&self) -> i32 {
        self.chains.as_ref().map_or(0, |c| c.len() as i32)
    }

    /// `Chain(num, n1, n2)`: the first and last ordered ranks of chain `num`, (0, 0)
    /// without chains or for `num` 0 or beyond them; Standard_OutOfRange for a negative
    /// `num` (the native array raises); `order.chain(1)?`.
    pub fn chain(&self, num: i32) -> Result<(i32, i32), Failure> {
        let Some(chains) = &self.chains else {
            return Ok((0, 0));
        };
        let nb = chains.len() as i32;
        if num == 0 || num > nb {
            return Ok((0, 0));
        }
        let n1 = item(chains, num, "ShapeAnalysis_WireOrder::Chain")?;
        let n2 = if num == nb {
            self.nb_edges()
        } else {
            chains[num as usize] - 1
        };
        Ok((n1, n2))
    }

    /// `SetCouples(gap)`: not implemented natively (a debug message only), so there is
    /// never a couple; `order.set_couples(1e-3)`.
    pub fn set_couples(&mut self, _gap: f64) {}

    /// `NbCouples()`: always 0 (see [`Self::set_couples`]); `order.nb_couples()`.
    pub fn nb_couples(&self) -> i32 {
        self.couples.as_ref().map_or(0, |c| c.len() as i32)
    }

    /// `Couple(num, n1, n2)`: the two edges of couple `num`, (0, 0) when there is none;
    /// `order.couple(1)`.
    pub fn couple(&self, num: i32) -> (i32, i32) {
        let Some(couples) = &self.couples else {
            return (0, 0);
        };
        if num <= 0 || num as usize * 2 > couples.len() {
            return (0, 0);
        }
        let n = num as usize;
        (couples[2 * n - 2], couples[2 * n - 1])
    }
}

/// The static `DISTABS`: the sum of the coordinate distances.
fn dist_abs(v1: Xyz, v2: Xyz) -> f64 {
    (v1.x() - v2.x()).abs() + (v1.y() - v2.y()).abs() + (v1.z() - v2.z()).abs()
}
