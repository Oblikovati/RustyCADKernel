// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT2d/MAT2d_Mat2d.cxx (loop 2 of CreateMat and CreateMatOpen, Intersect and LoadBisectorsToRemove) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Loop 2 of the algorithm: the bisectors of each edge are intersected (step 2.2);
//! when a bisector meets an older one first, the bisectors between them are removed
//! and their edges come back into the contour (steps 2.3 and 2.4).
use super::Mat2dMat2d;
use super::bound;
use super::engine::Run;
use crate::mat::{MatBisector, MatEdge, MatListOfBisector, MatListOfEdge};
use crate::mat2d::Mat2dTool2d;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};

// The number of iterations of loop 2 whose edge counts are compared: when the count
// comes back to the one 10 iterations before, no bisector is removed any more.
const EVEN_NB_ITER_BIS: i32 = 10;

impl Mat2dMat2d {
    /// Loop 2: while bisectors are to be removed.
    pub(super) fn remove_loop(
        &mut self,
        atool: &mut Mat2dTool2d,
        run: &mut Run,
    ) -> Result<(), Failure> {
        let list: Handle<MatListOfEdge> = self.theedgelist.clone();
        loop {
            run.nb_iter_bis += 1;
            run.noofbisectorstoremove = 0;
            run.scan_start(&list)?;
            record_edge_count(run, list.number());
            for _ in 0..run.scan_count(list.number()) {
                let edge: Handle<MatEdge> = list.current()?;
                if edge.distance() == -1. {
                    self.intersect_edge(atool, run, &edge)?;
                }
                list.next()?;
            }
            if run.to_nullify {
                run.noofbisectorstoremove = 0;
            }
            if run.noofbisectorstoremove == 0 {
                return Ok(());
            }
            for i in 0..run.noofbisectorstoremove {
                self.remove_branch(atool, i)?;
            }
        }
    }

    // Step 2.2 for one edge: the intersection of its two bisectors, then the search
    // of the bisectors to remove.
    fn intersect_edge(
        &mut self,
        atool: &mut Mat2dTool2d,
        run: &mut Run,
        edge: &Handle<MatEdge>,
    ) -> Result<(), Failure> {
        let firstbisector: Handle<MatBisector> = edge.first_bisector();
        let secondbisector: Handle<MatBisector> = edge.second_bisector();
        edge.set_distance(atool.intersect_bisector(
            &firstbisector,
            &secondbisector,
            &mut run.intersectionpoint,
        )?);
        edge.set_intersection_point(run.intersectionpoint);
        let begin: i32 = run.beginbisector;
        let (new_first, new_second) = (
            firstbisector.index_number() >= begin,
            secondbisector.index_number() >= begin,
        );
        if edge.distance() == precision::infinite() {
            if new_first || new_second {
                self.intersect(
                    atool,
                    0,
                    &mut run.noofbisectorstoremove,
                    &firstbisector,
                    &secondbisector,
                )?;
            }
            return Ok(());
        }
        if new_first {
            self.intersect(
                atool,
                1,
                &mut run.noofbisectorstoremove,
                &firstbisector,
                &secondbisector,
            )?;
        }
        if new_second {
            self.intersect(
                atool,
                2,
                &mut run.noofbisectorstoremove,
                &firstbisector,
                &secondbisector,
            )?;
        }
        Ok(())
    }

    // Step 2.4 for the branch `i`: the bisectors from `bisectoronetoremove(i)` down to
    // `bisectortwotoremove(i)` are cancelled; the edges they separated come back and
    // their sons are extended to infinity.
    fn remove_branch(&mut self, atool: &mut Mat2dTool2d, i: i32) -> Result<(), Failure> {
        let list: Handle<MatListOfEdge> = self.theedgelist.clone();
        let mut bisectortoremove: Handle<MatBisector> =
            bound(&self.bisectoronetoremove, i, "bisector to remove")?;
        loop {
            self.thenumberofbisectors -= 1;
            let currentbisectorlist: Handle<MatListOfBisector> = bisectortoremove.list();
            currentbisectorlist.first();
            let mut previousedge: Handle<MatEdge> = currentbisectorlist.first_item()?.first_edge();
            list.init(&previousedge)?;
            reset_edge(&previousedge);
            while currentbisectorlist.more() {
                let currentbisector: Handle<MatBisector> = currentbisectorlist.current()?;
                let currentedge: Handle<MatEdge> = currentbisector.second_edge();
                list.link_after(&currentedge)?;
                list.next()?;
                currentedge.set_first_bisector(&currentbisector);
                previousedge.set_second_bisector(&currentbisector);
                currentbisector.set_first_parameter(precision::infinite());
                currentbisector.set_second_parameter(precision::infinite());
                atool.trim_bisector(&currentbisector)?;
                reset_edge(&currentedge);
                previousedge = currentedge;
                currentbisectorlist.next()?;
            }
            self.removed_edges_list.back_add(&list.current()?);
            list.unlink()?;
            let last: Handle<MatBisector> =
                bound(&self.bisectortwotoremove, i, "bisector to remove")?;
            if bisectortoremove.bisector_number() == last.bisector_number() {
                return Ok(());
            }
            bisectortoremove = if bound(&self.typeofbisectortoremove, i, "type")? == 1 {
                bisectortoremove.first_bisector()?
            } else {
                bisectortoremove.last_bisector()?
            };
        }
    }

    /// `Intersect(atool, aside, noofbisectorstoremove, bisector1, bisector2)`: with
    /// `aside` 0 the first bisector against the leftmost descendants of the second and
    /// the second against the rightmost descendants of the first; with 1 (2) the first
    /// (second) bisector against its own leftmost and rightmost descendants. The
    /// parameters that the trials change are restored.
    fn intersect(
        &mut self,
        atool: &mut Mat2dTool2d,
        aside: i32,
        noofbisectortoremove: &mut i32,
        firstbisector: &Handle<MatBisector>,
        secondbisector: &Handle<MatBisector>,
    ) -> Result<(), Failure> {
        let mut distance: [f64; 2] = [precision::infinite(); 2];
        let mut firstbisectortoremove: [Handle<MatBisector>; 2] =
            [Handle::default(), Handle::default()];
        let mut lastbisectortoremove: [Handle<MatBisector>; 2] =
            [Handle::default(), Handle::default()];
        for bisectornumber in 0..2 {
            firstbisectortoremove[bisectornumber] = match (aside, bisectornumber) {
                (0, 0) | (2, _) => secondbisector.clone(),
                _ => firstbisector.clone(),
            };
            let start: &Handle<MatBisector> = &firstbisectortoremove[bisectornumber];
            let Some(mut previousbisector) = descend_start(start, aside, bisectornumber)? else {
                continue;
            };
            let mut lastbisector: Handle<MatBisector> = start.clone();
            while !previousbisector.list().is_empty() {
                previousbisector = if bisectornumber == 0 {
                    previousbisector.first_bisector()?
                } else {
                    previousbisector.last_bisector()?
                };
                let distant: f64 = trial(
                    atool,
                    aside,
                    bisectornumber,
                    (firstbisector, secondbisector),
                    &previousbisector,
                )?;
                if distant < precision::infinite() {
                    distance[bisectornumber] = distant;
                    lastbisectortoremove[bisectornumber] = lastbisector.clone();
                }
                lastbisector = previousbisector.clone();
            }
        }
        self.load_bisectors_to_remove(
            noofbisectortoremove,
            distance,
            &firstbisectortoremove,
            &lastbisectortoremove,
        )
    }

    /// `LoadBisectorsToRemove(noofbisectorstoremove, distance1, distance2, ...)`: the
    /// branch of the one side that meets something (the other not) is recorded,
    /// unless the same first bisector already has a longer branch.
    fn load_bisectors_to_remove(
        &mut self,
        noofbisectorstoremove: &mut i32,
        distance: [f64; 2],
        firstbisectortoremove: &[Handle<MatBisector>; 2],
        lastbisectortoremove: &[Handle<MatBisector>; 2],
    ) -> Result<(), Failure> {
        let infinite: f64 = precision::infinite();
        let index: usize = if distance[0] < infinite && distance[1] == infinite {
            0
        } else if distance[1] < infinite && distance[0] == infinite {
            1
        } else {
            return Ok(());
        };
        let first: &Handle<MatBisector> = &firstbisectortoremove[index];
        let last: &Handle<MatBisector> = &lastbisectortoremove[index];
        let mut found: i32 = *noofbisectorstoremove;
        for j in 0..*noofbisectorstoremove {
            if bound(&self.bisectoronetoremove, j, "bisector to remove")?.bisector_number()
                == first.bisector_number()
            {
                found = j;
                if bound(&self.bisectortwotoremove, j, "bisector to remove")?.bisector_number()
                    < last.bisector_number()
                {
                    found = -1;
                }
                break;
            }
        }
        if found == -1 {
            return Ok(());
        }
        self.bisectoronetoremove.insert(found, first.clone());
        self.bisectortwotoremove.insert(found, last.clone());
        self.typeofbisectortoremove.insert(found, index as i32 + 1);
        if found == *noofbisectorstoremove {
            *noofbisectorstoremove += 1;
        }
        Ok(())
    }
}

// The edge counts of the last iterations of loop 2 (EdgeNumbers): when the count
// comes back to the oldest one, the removals stop for good.
fn record_edge_count(run: &mut Run, number: i32) {
    let even: usize = EVEN_NB_ITER_BIS as usize;
    if run.nb_iter_bis <= EVEN_NB_ITER_BIS + 1 {
        run.edge_numbers[run.nb_iter_bis as usize] = number;
    } else {
        run.edge_numbers.copy_within(2..=even + 1, 1);
        run.edge_numbers[even + 1] = number;
    }
    if run.edge_numbers[even + 1] == run.edge_numbers[1] {
        run.to_nullify = true;
    }
}

// An edge back in the contour: its intersection is to compute again and its
// bisectors are extended.
fn reset_edge(edge: &Handle<MatEdge>) {
    edge.set_distance(-1.);
    edge.first_bisector()
        .set_second_parameter(precision::infinite());
    edge.second_bisector()
        .set_first_parameter(precision::infinite());
}

// The start of the descent of Intersect: the bisector itself with `aside` 0, else its
// first (or last) son; `None` when it has none.
fn descend_start(
    start: &Handle<MatBisector>,
    aside: i32,
    bisectornumber: usize,
) -> Result<Option<Handle<MatBisector>>, Failure> {
    if aside == 0 {
        return Ok(Some(start.clone()));
    }
    if start.list().is_empty() {
        return Ok(None);
    }
    Ok(Some(if bisectornumber == 0 {
        start.first_bisector()?
    } else {
        start.last_bisector()?
    }))
}

// One trial of Intersect: the intersection of a descendant with the first or the
// second bisector, the parameter it changes restored.
fn trial(
    atool: &mut Mat2dTool2d,
    aside: i32,
    bisectornumber: usize,
    (firstbisector, secondbisector): (&Handle<MatBisector>, &Handle<MatBisector>),
    previousbisector: &Handle<MatBisector>,
) -> Result<f64, Failure> {
    let mut intersectionpoint: i32 = 0;
    if aside == 1 || (aside == 0 && bisectornumber == 0) {
        let saveparameter: f64 = previousbisector.first_parameter();
        let distant: f64 =
            atool.intersect_bisector(firstbisector, previousbisector, &mut intersectionpoint)?;
        previousbisector.set_first_parameter(saveparameter);
        return Ok(distant);
    }
    let saveparameter: f64 = previousbisector.second_parameter();
    let distant: f64 =
        atool.intersect_bisector(previousbisector, secondbisector, &mut intersectionpoint)?;
    previousbisector.set_second_parameter(saveparameter);
    Ok(distant)
}
