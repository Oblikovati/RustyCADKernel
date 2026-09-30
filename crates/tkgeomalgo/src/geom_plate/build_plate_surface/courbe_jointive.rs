// Rust translation of TrierTab and CourbeJointive of
// ModelingAlgorithms/TKGeomAlgo/GeomPlate/GeomPlate_BuildPlateSurface.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The chaining of the boundary curves into a contour.
use super::{GeomPlateBuildPlateSurface, sequence_index};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::gp::Point3;

/// Inverts the permutation `tab` (`TrierTab`): after `CourbeJointive` reorders the
/// curves, `tab(i)` holds the initial index of curve `i`; after the call it holds
/// the new position of initial curve `i`.
pub(super) fn trier_tab(tab: &mut [i32]) -> Result<(), Failure> {
    let mut tab_tri: Vec<i32> = vec![0; tab.len()];
    for (i, &position) in (1..).zip(tab.iter()) {
        let slot = usize::try_from(position - 1)
            .ok()
            .and_then(|p| tab_tri.get_mut(p));
        *slot.ok_or_else(|| {
            errors::out_of_range(&format!(
                "TrierTab: index {position} outside 1..={}",
                tab.len()
            ))
        })? = i;
    }
    tab.copy_from_slice(&tab_tri);
    Ok(())
}

impl GeomPlateBuildPlateSurface {
    /// `mySense->Value(i)`.
    pub(super) fn sense_at(&self, i: i32) -> Result<i32, Failure> {
        let sense = self.sense.as_deref().unwrap_or(&[]);
        Ok(sense[sequence_index(i, sense.len(), "Sense")?])
    }

    fn set_sense(&mut self, i: i32, value: i32) -> Result<(), Failure> {
        let sense = self.sense.get_or_insert_with(Vec::new);
        let index: usize = sequence_index(i, sense.len(), "Sense")?;
        sense[index] = value;
        Ok(())
    }

    /// Swaps curves `a` and `b` and their initial indices.
    fn swap_curves(&mut self, a: i32, b: i32) -> Result<(), Failure> {
        let (ia, ib) = (
            sequence_index(a, self.lin_cont.len(), "LinCont")?,
            sequence_index(b, self.lin_cont.len(), "LinCont")?,
        );
        self.lin_cont.swap(ia, ib);
        let order = self.init_order.get_or_insert_with(Vec::new);
        let (oa, ob) = (
            sequence_index(a, order.len(), "InitOrder")?,
            sequence_index(b, order.len(), "InitOrder")?,
        );
        order.swap(oa, ob);
        Ok(())
    }

    /// The end of curve `j` in its sense (its first point when reversed).
    fn chain_end(&self, j: i32) -> Result<Point3, Failure> {
        let lin = self.lin(j)?;
        let (uinit1, mut ufinal1) = (lin.first_parameter()?, lin.last_parameter()?);
        if self.sense_at(j)? == 1 {
            ufinal1 = uinit1;
        }
        lin.d0(ufinal1)
    }

    /// Looks for the curve after `j` among `j + 1..=myNbBounds`, puts it at `j + 1`
    /// with its sense; false when none joins.
    fn chain_next(&mut self, j: i32, tolerance: f64) -> Result<bool, Failure> {
        for i in j + 1..=self.nb_bounds {
            let p1: Point3 = self.chain_end(j)?;
            let lin = self.lin(i)?;
            let (uinit2, ufinal2) = (lin.first_parameter()?, lin.last_parameter()?);
            let (start, end) = (lin.d0(uinit2)?, lin.d0(ufinal2));
            let sense: i32 = if p1.distance(&start) < tolerance {
                0
            } else if p1.distance(&end?) < tolerance {
                1
            } else {
                continue;
            };
            if i != j + 1 {
                self.swap_curves(j + 1, i)?;
            }
            self.set_sense(j + 1, sense)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Chains the first `myNbBounds` curves end to start within `tolerance`,
    /// reordering them and recording their senses (`CourbeJointive`); true when they
    /// form a closed contour.
    pub(super) fn courbe_jointive(&mut self, tolerance: f64) -> Result<bool, Failure> {
        self.sense = Some(vec![0; self.lin_cont.len()]);
        let mut result: bool = true;
        for j in 1..self.nb_bounds {
            if !self.chain_next(j, tolerance)? {
                result = false;
            }
        }
        let last = self.lin(self.nb_bounds)?;
        let (uinit1, ufinal1) = (last.first_parameter()?, last.last_parameter()?);
        let first = self.lin(1)?;
        let (uinit2, _) = (first.first_parameter()?, first.last_parameter()?);
        let p1: Point3 = last.d0(ufinal1)?;
        let p2: Point3 = first.d0(uinit2)?;
        if self.sense_at(self.nb_bounds)? == 0 && p1.distance(&p2) < tolerance {
            return Ok(result);
        }
        let p1: Point3 = last.d0(uinit1)?;
        Ok(self.sense_at(self.nb_bounds)? == 1 && p1.distance(&p2) < tolerance && result)
    }
}
