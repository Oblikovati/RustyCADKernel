// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_GIter and
// TopOpeBRepBuild_GTool (.hxx/.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The iteration over the kept states of a table (`GIter`) and the tables of the
//! operations (`GTool`).
use super::gtopo::{TopOpeBRepBuildGTopo, io_failure};
use crate::top_ope_brep_ds::TopOpeBRepDSConfig;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use std::io::Write;

/// The iteration over the kept states of a table (`TopOpeBRepBuild_GIter`); it holds a
/// copy of the table where natively it points to it;
/// `let mut gi = TopOpeBRepBuildGIter::with_gtopo(&g);`.
#[derive(Clone, Copy, Debug, Default)]
pub struct TopOpeBRepBuildGIter {
    my_ii: i32,
    my_g: Option<TopOpeBRepBuildGTopo>,
}

impl TopOpeBRepBuildGIter {
    /// `TopOpeBRepBuild_GIter()`: no table; `TopOpeBRepBuildGIter::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `TopOpeBRepBuild_GIter(G)`; `TopOpeBRepBuildGIter::with_gtopo(&g)`.
    pub fn with_gtopo(g: &TopOpeBRepBuildGTopo) -> Self {
        let mut gi = Self::new();
        gi.init_with(g);
        gi
    }

    /// The table (Standard_NullObject without, a null dereference natively).
    fn g(&self) -> Result<&TopOpeBRepBuildGTopo, Failure> {
        self.my_g.as_ref().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "TopOpeBRepBuild_GIter: no GTopo",
                "TopOpeBRepBuild_GIter",
            )
        })
    }

    /// `Find()` (private): the next kept cell from the current index.
    fn find(&mut self) {
        let Some(g) = self.my_g else {
            return;
        };
        while self.my_ii <= 8 {
            if g.value_index(self.my_ii) {
                break;
            }
            self.my_ii += 1;
        }
    }

    /// `Init()`: at the first kept cell; `gi.init()`.
    pub fn init(&mut self) {
        self.my_ii = 0;
        self.find();
    }

    /// `Init(G)`; `gi.init_with(&g)`.
    pub fn init_with(&mut self, g: &TopOpeBRepBuildGTopo) {
        self.my_g = Some(*g);
        self.init();
    }

    /// `More()`; `gi.more()`.
    pub fn more(&self) -> bool {
        self.my_ii <= 8 && self.my_g.is_some_and(|g| g.value_index(self.my_ii))
    }

    /// `Next()`; `gi.next()`.
    #[allow(clippy::should_implement_trait)] // OCCT's More/Next protocol.
    pub fn next(&mut self) {
        self.my_ii += 1;
        self.find();
    }

    /// `Current(s1, s2)`: the states of the current cell (UNKNOWN at the end);
    /// `let (s1, s2) = gi.current()?;`.
    pub fn current(&self) -> Result<(State, State), Failure> {
        if !self.more() {
            return Ok((State::Unknown, State::Unknown));
        }
        let g = self.g()?;
        let (i1, i2) = g.index(self.my_ii);
        Ok((g.g_state(i1)?, g.g_state(i2)?))
    }

    /// `Dump(OS)`: the current cell; `gi.dump(&mut out)?`.
    pub fn dump(&self, os: &mut impl Write) -> Result<(), Failure> {
        if !self.more() {
            return Ok(());
        }
        let (s1, s2) = self.current()?;
        let b = self.g()?.value(s1, s2)?;
        TopOpeBRepBuildGTopo::dump_ssb(os, s1, s2, b).map_err(io_failure)?;
        writeln!(os).map_err(io_failure)
    }
}

/// The tables of the operations (`TopOpeBRepBuild_GTool`, static members only).
pub struct TopOpeBRepBuildGTool;

/// A table of the operations: rows IN, ON, OUT of the first shape.
fn table(
    t1: ShapeEnum,
    t2: ShapeEnum,
    cases: [[bool; 3]; 3],
    configs: (TopOpeBRepDSConfig, TopOpeBRepDSConfig),
) -> TopOpeBRepBuildGTopo {
    TopOpeBRepBuildGTopo::with_table(cases, t1, t2, configs.0, configs.1)
}

use TopOpeBRepDSConfig::{DiffOriented as DIFF, SameOriented as SAME, UnshGeometry as UNSH};

impl TopOpeBRepBuildGTool {
    /// `GFusUnsh(t1, t2)`: the fuse of unshared geometries (ON/OUT and OUT/ON kept);
    /// `TopOpeBRepBuildGTool::g_fus_unsh(ShapeEnum::Face, ShapeEnum::Face)`.
    pub fn g_fus_unsh(t1: ShapeEnum, t2: ShapeEnum) -> TopOpeBRepBuildGTopo {
        table(
            t1,
            t2,
            [
                [false, false, false],
                [false, false, true],
                [false, true, false],
            ],
            (UNSH, UNSH),
        )
    }

    /// `GFusSame(t1, t2)`; `TopOpeBRepBuildGTool::g_fus_same(ShapeEnum::Face, ShapeEnum::Face)`.
    pub fn g_fus_same(t1: ShapeEnum, t2: ShapeEnum) -> TopOpeBRepBuildGTopo {
        table(
            t1,
            t2,
            [
                [false, false, false],
                [false, true, true],
                [false, true, false],
            ],
            (SAME, SAME),
        )
    }

    /// `GFusDiff(t1, t2)`; `TopOpeBRepBuildGTool::g_fus_diff(ShapeEnum::Face, ShapeEnum::Face)`.
    pub fn g_fus_diff(t1: ShapeEnum, t2: ShapeEnum) -> TopOpeBRepBuildGTopo {
        table(
            t1,
            t2,
            [
                [false, false, false],
                [false, false, true],
                [false, true, false],
            ],
            (DIFF, SAME),
        )
    }

    /// `GCutUnsh(t1, t2)`; `TopOpeBRepBuildGTool::g_cut_unsh(ShapeEnum::Face, ShapeEnum::Face)`.
    pub fn g_cut_unsh(t1: ShapeEnum, t2: ShapeEnum) -> TopOpeBRepBuildGTopo {
        table(
            t1,
            t2,
            [
                [false, true, false],
                [false, false, true],
                [false, false, false],
            ],
            (UNSH, UNSH),
        )
    }

    /// `GCutSame(t1, t2)`; `TopOpeBRepBuildGTool::g_cut_same(ShapeEnum::Face, ShapeEnum::Face)`.
    pub fn g_cut_same(t1: ShapeEnum, t2: ShapeEnum) -> TopOpeBRepBuildGTopo {
        table(
            t1,
            t2,
            [
                [false, true, false],
                [false, false, true],
                [false, false, false],
            ],
            (SAME, SAME),
        )
    }

    /// `GCutDiff(t1, t2)`; `TopOpeBRepBuildGTool::g_cut_diff(ShapeEnum::Face, ShapeEnum::Face)`.
    pub fn g_cut_diff(t1: ShapeEnum, t2: ShapeEnum) -> TopOpeBRepBuildGTopo {
        table(
            t1,
            t2,
            [
                [false, true, false],
                [false, true, true],
                [false, false, false],
            ],
            (DIFF, SAME),
        )
    }

    /// `GComUnsh(t1, t2)`; `TopOpeBRepBuildGTool::g_com_unsh(ShapeEnum::Face, ShapeEnum::Face)`.
    pub fn g_com_unsh(t1: ShapeEnum, t2: ShapeEnum) -> TopOpeBRepBuildGTopo {
        table(
            t1,
            t2,
            [
                [false, true, false],
                [true, false, false],
                [false, false, false],
            ],
            (UNSH, UNSH),
        )
    }

    /// `GComSame(t1, t2)`; `TopOpeBRepBuildGTool::g_com_same(ShapeEnum::Face, ShapeEnum::Face)`.
    pub fn g_com_same(t1: ShapeEnum, t2: ShapeEnum) -> TopOpeBRepBuildGTopo {
        table(
            t1,
            t2,
            [
                [false, true, false],
                [true, true, false],
                [false, false, false],
            ],
            (SAME, SAME),
        )
    }

    /// `GComDiff(t1, t2)`; `TopOpeBRepBuildGTool::g_com_diff(ShapeEnum::Face, ShapeEnum::Face)`.
    pub fn g_com_diff(t1: ShapeEnum, t2: ShapeEnum) -> TopOpeBRepBuildGTopo {
        table(
            t1,
            t2,
            [
                [false, true, false],
                [true, false, false],
                [false, false, false],
            ],
            (DIFF, SAME),
        )
    }

    /// One table and its iteration of `Dump`.
    fn dump_table(os: &mut impl Write, g: &TopOpeBRepBuildGTopo) -> Result<(), Failure> {
        g.dump(os, "")?;
        let mut gi = TopOpeBRepBuildGIter::with_gtopo(g);
        while gi.more() {
            gi.dump(os)?;
            gi.next();
        }
        writeln!(os).map_err(io_failure)
    }

    /// `Dump(OS)`: the tables of the fuses of faces, a face/edge cut and its permutation;
    /// `TopOpeBRepBuildGTool::dump(&mut out)?`.
    pub fn dump(os: &mut impl Write) -> Result<(), Failure> {
        let (f, e) = (ShapeEnum::Face, ShapeEnum::Edge);
        Self::dump_table(os, &Self::g_fus_unsh(f, f))?;
        Self::dump_table(os, &Self::g_fus_same(f, f))?;
        Self::dump_table(os, &Self::g_fus_diff(f, f))?;
        let g = Self::g_cut_diff(f, e);
        Self::dump_table(os, &g)?;
        Self::dump_table(os, &g.copy_permuted())
    }
}
