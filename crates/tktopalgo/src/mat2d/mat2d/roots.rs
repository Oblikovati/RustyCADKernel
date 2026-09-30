// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT2d/MAT2d_Mat2d.cxx (step 3 of CreateMat and CreateMatOpen) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Step 3: the roots of the trees of bisectors. An interrupted loop gives
//! semi-infinite bisectors (the second bisector of each remaining edge); with two
//! edges left, the last two bisectors are joined when one ends on the other.
use super::Mat2dMat2d;
use super::bound;
use super::engine::{MatContour, Run};
use crate::mat::{MatBisector, MatEdge, MatListOfBisector, MatListOfEdge};
use crate::mat2d::Mat2dTool2d;
use rustycad_tkernel::standard::{Failure, Handle};

impl Mat2dMat2d {
    /// Step 3: the kind of the locus, then its roots.
    pub(super) fn make_roots(
        &mut self,
        atool: &mut Mat2dTool2d,
        run: &mut Run,
    ) -> Result<(), Failure> {
        let list: Handle<MatListOfEdge> = self.theedgelist.clone();
        self.semi_infinite = run.interrupt;
        if !run.interrupt && list.number() > 1 {
            self.join_last_bisectors(atool, run, &list)?;
        }
        if self.semi_infinite {
            run.beginbisector = run.noofbisectors;
            list.first();
            let count: i32 = if run.contour == MatContour::Open {
                list.number() - 1
            } else {
                list.number()
            };
            for _ in 0..count {
                let edge: Handle<MatEdge> = list.current()?;
                self.bisectormap
                    .insert(run.noofbisectors, edge.second_bisector());
                run.noofbisectors += 1;
                list.next()?;
            }
        }
        self.collect_roots(run)
    }

    // Two edges remain: the last bisectors separate the same edges. The bisector that
    // is not the last created is trimmed at the issue point of the other and becomes
    // its son (the locus inside), else the locus is semi-infinite.
    fn join_last_bisectors(
        &mut self,
        atool: &mut Mat2dTool2d,
        run: &mut Run,
        list: &MatListOfEdge,
    ) -> Result<(), Failure> {
        list.first();
        let edge: Handle<MatEdge> = list.current()?;
        let last_index: i32 = run.noofbisectors - 1;
        let (kept, other) = if edge.first_bisector().index_number() == last_index {
            (edge.first_bisector(), edge.second_bisector())
        } else {
            (edge.second_bisector(), edge.first_bisector())
        };
        // skv, IDEM Sep 2005: the trim may fail.
        if atool.trim_bisector_at(&other, kept.issue_point())? {
            if other.end_point() == 0 {
                other.set_end_point(kept.issue_point());
            }
            bound(&self.bisectormap, last_index, "bisector")?.add_bisector(&other);
        } else {
            self.semi_infinite = true;
        }
        if !self.semi_infinite {
            self.thenumberofbisectors -= 1;
            let last: Handle<MatBisector> = bound(&self.bisectormap, last_index, "bisector")?;
            last.set_second_edge(edge);
            last.set_bisector_number(-1);
        }
        Ok(())
    }

    // The roots: the sons of a joined last bisector (whose first edge takes its issue
    // distance), else the bisectors from `beginbisector`.
    fn collect_roots(&mut self, run: &Run) -> Result<(), Failure> {
        self.roots = Handle::new(MatListOfBisector::new());
        let last: Handle<MatBisector> =
            bound(&self.bisectormap, run.noofbisectors - 1, "bisector")?;
        if last.bisector_number() == -1 {
            self.roots = last.list();
            self.roots.first();
            self.roots
                .current()?
                .first_edge()
                .set_distance(last.dist_issue_point());
            return Ok(());
        }
        for i in run.beginbisector..run.noofbisectors {
            self.roots
                .back_add(&bound(&self.bisectormap, i, "bisector")?);
        }
        Ok(())
    }
}
