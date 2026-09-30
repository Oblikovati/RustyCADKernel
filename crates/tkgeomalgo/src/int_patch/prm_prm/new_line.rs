// Rust translation of IntPatch_PrmPrmIntersection::NewLine
// (ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_PrmPrmIntersection.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `NewLine`: a walking line resampled at a constant curvilinear abscissa (measured on
//! the parameters on the first surface), the inserted points computed by the walking.
use super::{IntPatchPrmPrmIntersection, start_params};
use crate::int_patch::{IntPatchLine, IntPatchPointLine, IntPatchWLType, IntPatchWLine};
use crate::int_start::failures::sequence_value;
use crate::int_surf::IntSurfLineOn2S;
use crate::int_walk::IntWalkPWalking;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;

/// The parameters and the curvilinear abscissae of the points `Low..=High` of the line,
/// read with the Standard_OutOfRange of the native arrays.
struct Samples {
    low: i32,
    uv: Vec<[f64; 4]>,
    ac: Vec<f64>,
}

impl Samples {
    fn slot(&self, i: i32, member: &str) -> Result<usize, Failure> {
        sequence_value(&self.ac, i - self.low + 1, member)?;
        Ok((i - self.low) as usize)
    }

    /// `AC(i)`.
    fn ac(&self, i: i32) -> Result<f64, Failure> {
        Ok(self.ac[self.slot(i, "NCollection_Array1::Value")?])
    }

    /// `(U1(i), V1(i), U2(i), V2(i))`.
    fn uv(&self, i: i32) -> Result<[f64; 4], Failure> {
        Ok(self.uv[self.slot(i, "NCollection_Array1::Value")?])
    }

    /// The samples of the points `low..=high` of the line.
    fn new(line: &IntPatchWLine, low: i32, high: i32) -> Result<Self, Failure> {
        let (u1, v1, u2, v2) = line.point(low)?.parameters();
        let mut samples = Self {
            low,
            uv: vec![[u1, v1, u2, v2]],
            ac: vec![0.0],
        };
        for i in low + 1..=high {
            let (u1, v1, u2, v2) = line.point(i)?.parameters();
            let prev: [f64; 4] = samples.uv[samples.uv.len() - 1];
            let (du1, dv1) = (u1 - prev[0], v1 - prev[1]);
            let ac: f64 = samples.ac[samples.ac.len() - 1] + ((du1 * du1) + (dv1 * dv1)).sqrt();
            samples.uv.push([u1, v1, u2, v2]);
            samples.ac.push(ac);
        }
        Ok(samples)
    }
}

impl IntPatchPrmPrmIntersection {
    /// `NewLine(Caro1, Caro2, IndexLine, LowPoint, HighPoint, NbPoints)`: about `NbPoints`
    /// points of the line `index_line` between its points `low` and `high`: every point
    /// of the line in that range, and new points inserted where the existing ones are not
    /// too close. Standard_OutOfRange for a line index out of range or `low >= high`,
    /// StdFail_NotDone unless done.
    ///
    /// ```ignore
    /// let line = inter.new_line(&s1, &s2, 1, 1, 20, 40)?;
    /// ```
    pub fn new_line(
        &self,
        surf1: &Adaptor3dSurfaceHandle,
        surf2: &Adaptor3dSurfaceHandle,
        num_line: i32,
        (low, high): (i32, i32),
        nb_pnts_to_insert: i32,
    ) -> Result<Handle<dyn IntPatchLine>, Failure> {
        let nb_pnts: i32 = nb_pnts_to_insert + high - low;
        if num_line > self.nb_lines()? || num_line < 1 || low >= high {
            return Err(Failure::out_of_range(&format!(
                "IntPatch_PrmPrmIntersection::NewLine() - line {num_line} of 1..={}, points {low}..{high}; expected low < high",
                self.slin.len()
            )));
        }
        let the_line: Handle<IntPatchWLine> = self.line(num_line)?.cast::<IntPatchWLine>();
        let samples = Samples::new(&the_line, low, high)?;
        let mut pw = IntWalkPWalking::new(surf1, surf2, 0.000001, 0.000001, 0.001, 0.001)?;
        let result: Handle<IntSurfLineOn2S> = Handle::new(IntSurfLineOn2S::new(None));
        result.add(&the_line.point(low)?);
        let ds: f64 = samples.ac(high)? / f64::from(nb_pnts - 1);
        let (dsmin, smax) = (ds * 0.3, samples.ac(high)?);
        let mut indice: i32 = low;
        let (mut i, mut s) = (2, ds);
        while i < nb_pnts && s < smax {
            while samples.ac(indice + 1)? <= s {
                result.add(&the_line.point(indice)?);
                indice += 1;
            }
            let a: f64 = s - samples.ac(indice)?;
            let b: f64 = samples.ac(indice + 1)? - s;
            let nab: f64 = 1.0 / (a + b);
            // Check the distances to the next and to the previous point.
            if nab > ds && a > dsmin && b > dsmin {
                let start = interpolated(&samples, indice, (a, b, nab))?;
                if let Some(point) = pw.perform_first_point(&start)? {
                    result.add(&point);
                }
            } else {
                s += dsmin;
            }
            i += 1;
            s += ds;
        }
        result.add(&the_line.point(high)?);
        let rwl = IntPatchWLine::new_undecided(&result, false);
        rwl.set_creating_way_info(IntPatchWLType::PrmPrm);
        Ok(Handle::new(rwl).cast())
    }
}

/// The start parameters between the samples `indice` and `indice + 1`, weighted by the
/// distances `a` and `b` to them.
fn interpolated(
    samples: &Samples,
    indice: i32,
    (a, b, nab): (f64, f64, f64),
) -> Result<Array1<'static, f64>, Failure> {
    let (p, q) = (samples.uv(indice)?, samples.uv(indice + 1)?);
    start_params([0, 1, 2, 3].map(|k| (p[k] * b + q[k] * a) * nab))
}
