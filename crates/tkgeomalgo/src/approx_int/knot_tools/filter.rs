//! `FilterKnots`: the output knots from the draft indices, with neither too many nor
//! too few points per knot interval.
use super::{ApproxIntKnotTools, seq_at};
use rustycad_tkernel::standard::Failure;

/// `aMaxPntCoeff`: the maximum number of points per interval is this times the minimum.
const MAX_PNT_COEFF: i32 = 15;

/// The output sequence (`theLKnots`, a 0-based dynamic array) and the last kept index
/// (`anIndsPrev`).
struct Output<'a> {
    knots: &'a mut Vec<i32>,
    prev: i32,
}

impl Output<'_> {
    fn append(&mut self, value: i32) {
        self.knots.push(value);
        self.prev = value;
    }
    fn set_last(&mut self, value: i32) {
        let last: usize = self.knots.len() - 1;
        self.knots[last] = value;
        self.prev = value;
    }
}

impl ApproxIntKnotTools {
    /// `FilterKnots(theInds, theMinNbPnts, theLKnots)`: splits the intervals of more
    /// than 15 times `min_nb_pnts` points, merges those of too few points, and makes
    /// the last index the last knot; the knots are appended to `knots`;
    /// `ApproxIntKnotTools::filter_knots(&mut inds, 30, &mut knots)?`.
    pub fn filter_knots(
        inds: &mut Vec<i32>,
        min_nb_pnts: i32,
        knots: &mut Vec<i32>,
    ) -> Result<(), Failure> {
        // I: too many points per knot interval.
        let max_nb_pnts: i32 = MAX_PNT_COEFF * min_nb_pnts;
        let mut i: i32 = 1;
        while i < inds.len() as i32 {
            let nbint: i32 = seq_at(inds, i + 1)? - seq_at(inds, i)? + 1;
            if nbint <= max_nb_pnts {
                i += 1;
                continue;
            }
            let ind: i32 = seq_at(inds, i)? + nbint / 2;
            inds.insert(i as usize, ind);
        }
        // II: too few points per knot interval.
        let first: i32 = seq_at(inds, 1)?;
        let mut out = Output { knots, prev: first };
        out.knots.push(first);
        Self::merge_short_intervals(inds, min_nb_pnts, &mut out)?;
        // III: the last knot.
        let last: i32 = seq_at(inds, inds.len() as i32)?;
        if out.knots.len() < 2 {
            out.knots.push(last);
        } else if out.knots[out.knots.len() - 1] < last {
            let upper: usize = out.knots.len() - 1;
            out.knots[upper] = last;
        }
        Ok(())
    }

    /// Step II of `FilterKnots`.
    fn merge_short_intervals(
        inds: &[i32],
        min_nb_pnts: i32,
        out: &mut Output<'_>,
    ) -> Result<(), Failure> {
        let len: i32 = inds.len() as i32;
        let mut i: i32 = 2;
        while i <= len {
            if seq_at(inds, i)? - out.prev > min_nb_pnts {
                out.append(seq_at(inds, i)?);
            } else if i != len {
                i = Self::merge_from(inds, (i, min_nb_pnts), out)?;
            }
            i += 1;
        }
        Ok(())
    }

    /// The merge of a short interval starting at `i`: the index to continue from.
    fn merge_from(
        inds: &[i32],
        (i, min_nb_pnts): (i32, i32),
        out: &mut Output<'_>,
    ) -> Result<i32, Failure> {
        let len: i32 = inds.len() as i32;
        let min_nb_step: i32 = min_nb_pnts / 2;
        let mut an_idx: i32 = i + 1;
        while an_idx <= len {
            if seq_at(inds, an_idx)? - out.prev >= min_nb_pnts {
                break;
            }
            an_idx += 1;
        }
        an_idx -= 1;
        let at_idx: i32 = seq_at(inds, an_idx)?;
        let mid_idx: i32 = (at_idx + out.prev) / 2;
        if mid_idx - out.prev < min_nb_pnts
            && mid_idx - at_idx < min_nb_pnts
            && at_idx - out.prev >= min_nb_step
        {
            return Self::merge_bad_distribution(inds, (an_idx, min_nb_pnts), out);
        }
        if an_idx == len && out.knots.len() >= 2 {
            // The last point is reached and the last item can be modified: the bad
            // interval to the end is divided in two parts
            // (last good index -> last index - theMinNbPnts -> last index).
            let last_good_idx: i32 = out.knots[out.knots.len() - 2];
            let last: i32 = seq_at(inds, len)?;
            if last - 2 * min_nb_pnts >= last_good_idx {
                let upper: usize = out.knots.len() - 1;
                out.knots[upper] = last - min_nb_pnts;
                out.knots.push(last);
                out.prev = at_idx;
                return Ok(an_idx);
            }
        }
        Ok(i)
    }

    /// Bad distribution points merged into one knot interval.
    fn merge_bad_distribution(
        inds: &[i32],
        (an_idx, min_nb_pnts): (i32, i32),
        out: &mut Output<'_>,
    ) -> Result<i32, Failure> {
        let at_idx: i32 = seq_at(inds, an_idx)?;
        if at_idx - out.prev > 2 * min_nb_pnts {
            out.append(out.prev + min_nb_pnts);
            return Ok(an_idx - 1);
        }
        let before: i32 = seq_at(inds, an_idx - 1)?;
        if before - out.prev >= min_nb_pnts / 2 {
            out.append(before);
            if at_idx - before <= min_nb_pnts / 2 {
                out.set_last(at_idx);
                return Ok(an_idx);
            }
            return Ok(an_idx - 1);
        }
        out.append(at_idx);
        Ok(an_idx)
    }
}
