//! The radius laws of a fillet guideline, one composite law per elementary spine.
mod builder;
mod pieces;

use super::FilSpine;
use crate::chfi_ds::ElSpine;
use crate::chfi_ds::errors::domain_error;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeomalgo::law::{LawComposite, LawFunction};

impl FilSpine {
    /// `AppendLaw(Els)`: appends the law computed for `el_spine`.
    pub(super) fn append_law(&mut self, el_spine: &Handle<ElSpine>) -> Result<(), Failure> {
        let law = self.compute_law(el_spine)?;
        self.laws.push(law.cast());
        Ok(())
    }

    /// `ComputeLaw(Els)`: the composite radius law of `el_spine` from the radii of the
    /// edges it crosses; Standard_DomainError when a needed radius is missing;
    /// `spine.compute_law(&el_spine)?` (private natively).
    pub(crate) fn compute_law(
        &self,
        el_spine: &Handle<ElSpine>,
    ) -> Result<Handle<LawComposite>, Failure> {
        let el_spine = el_spine
            .get()
            .ok_or_else(|| domain_error("ChFiDS_FilSpine::ComputeLaw: null elementary spine"))?;
        builder::compute(self, el_spine)
    }

    /// The `splitdone` part of `SetRadius(UandR, IinC)`: the law of the first elementary
    /// spine when it is periodic, else of every elementary spine whose range holds `w`.
    pub(super) fn recompute_law_at(&mut self, w: f64) -> Result<(), Failure> {
        let el_spines: Vec<Handle<ElSpine>> = self.spine.el_spines().to_vec();
        let first = el_spines.first().ok_or_else(no_el_spine)?;
        if first.get().ok_or_else(no_el_spine)?.is_periodic()? {
            let law = self.compute_law(first)?;
            *self.laws.first_mut().ok_or_else(no_el_spine)? = law.cast();
            return Ok(());
        }
        for (offset, el_spine) in el_spines.iter().enumerate() {
            let curve = el_spine.get().ok_or_else(no_el_spine)?;
            let (first, last) = (curve.first_parameter()?, curve.last_parameter()?);
            if first <= w && w <= last {
                let law = self.compute_law(el_spine)?;
                *self.laws.get_mut(offset).ok_or_else(no_el_spine)? = law.cast();
            }
        }
        Ok(())
    }

    /// `Law(Els)`: the composite law of the elementary spine `el_spine` (by handle
    /// identity), or a null handle; `spine.law(&el_spine)?`.
    pub fn law(&self, el_spine: &Handle<ElSpine>) -> Result<Handle<LawComposite>, Failure> {
        for (offset, candidate) in self.spine.el_spines().iter().enumerate() {
            if candidate.object_address() == el_spine.object_address() {
                let law = self.laws.get(offset).ok_or_else(no_el_spine)?;
                return Ok(law.cast());
            }
        }
        Ok(Handle::default())
    }

    /// `ChangeLaw(E)`: lends the elementary law of the composite at the middle of `edge`
    /// to `edit` (the native reference to it); Standard_DomainError before the split or
    /// on a constant edge; `spine.change_law(&edge, |law| law.clone())?`.
    pub fn change_law<R>(
        &self,
        edge: &Edge,
        edit: impl FnOnce(&mut Handle<dyn LawFunction>) -> R,
    ) -> Result<R, Failure> {
        if !self.spine.split_done() {
            return Err(domain_error(
                "ChFiDS_FilSpine::ChangeLaw : the limits are not up-to-date",
            ));
        }
        let index = self.spine.index_of_edge(edge);
        if self.is_constant_on_edge(index)? {
            return Err(domain_error(
                "ChFiDS_FilSpine::ChangeLaw : no law on constant edges",
            ));
        }
        let el_spine = self.spine.el_spine_of_index(index)?;
        let w =
            0.5 * (self.spine.first_parameter_of(index)? + self.spine.last_parameter_of(index)?);
        let law = self.law(&el_spine)?;
        let composite = law
            .get()
            .ok_or_else(|| domain_error("ChFiDS_FilSpine::ChangeLaw : no law for the edge"))?;
        composite.change_elementary_law(w, edit)
    }

    /// `MaxRadFromSeqAndLaws()`: the largest radius among the pairs and the laws sampled
    /// at six points of their bounds (never below 0); `spine.max_rad_from_seq_and_laws()?`.
    pub fn max_rad_from_seq_and_laws(&self) -> Result<f64, Failure> {
        let mut max_rad = 0.;
        for pair in &self.par_and_rad {
            if pair.y() > max_rad {
                max_rad = pair.y();
            }
        }
        for law in &self.laws {
            let law = law.get().ok_or_else(no_el_spine)?;
            max_rad = max_sampled_radius(law, max_rad)?;
        }
        Ok(max_rad)
    }
}

/// The sampling of `MaxRadFromSeqAndLaws` over one law: five points from the start by a
/// fifth of the range, then the end.
fn max_sampled_radius(law: &dyn LawFunction, mut max_rad: f64) -> Result<f64, Failure> {
    let (fpar, lpar) = law.bounds()?;
    let delta = (lpar - fpar) * 0.2;
    for i in 0..=4 {
        let rad = law.value(fpar + f64::from(i) * delta)?;
        if rad > max_rad {
            max_rad = rad;
        }
    }
    let rad = law.value(lpar)?;
    if rad > max_rad {
        max_rad = rad;
    }
    Ok(max_rad)
}

/// The native iteration over an empty or shorter list (an exhausted iterator).
fn no_el_spine() -> Failure {
    Failure::with_message(
        "Standard_NoSuchObject",
        "ChFiDS_FilSpine: no elementary spine or law; expected one law per elementary spine",
    )
}
