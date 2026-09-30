// Rust translation of Geom2dHatch_Hatcher.cxx (category Computation - Domains) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The domains of the hatchings: the parts of each hatching inside the region, from
//! the states around its intersection points.
use super::domain_walk::{DomainWalk, Flow};
use super::{Classifier, Hatcher};
use crate::hatch_gen::{Domain, ErrorStatus};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::State;

// The tolerance of the classification of a hatching without intersection point.
const CLASSIFICATION_TOLERANCE: f64 = 0.0000001;

impl Hatcher {
    /// ComputeDomains(): the domains of every hatching; `hatcher.compute_domains()?`.
    pub fn compute_domains(&mut self) -> Result<(), Failure> {
        for ind_h in 1..=self.my_nb_hatchings {
            if self.my_hatchings.is_bound(&ind_h) {
                self.compute_domains_of(ind_h)?;
            }
        }
        Ok(())
    }

    /// ComputeDomains(IndH): trims the hatching when it is not trimmed, then cuts it
    /// into domains at its intersection points (nothing when its status is not
    /// NoProblem). A hatching without intersection point is one infinite domain when
    /// a point of it classifies IN the elements. Incoherent states stop with the
    /// status IncoherentParity (when no domain was made yet, else the hatching is
    /// done with the domains made) or IncompatibleStates; Standard_NoSuchObject for
    /// an unbound index; `hatcher.compute_domains_of(1)?`.
    pub fn compute_domains_of(&mut self, ind_h: i32) -> Result<(), Failure> {
        let hatching = self.hatching_mut(ind_h)?;
        hatching.clr_domains();
        hatching.set_is_done(false);
        if !hatching.trim_done() {
            self.trim_hatching(ind_h)?;
        }
        if self.hatching(ind_h)?.status() != ErrorStatus::NoProblem {
            return Ok(());
        }
        let nb_pnt: i32 = self.hatching(ind_h)?.nb_points();
        if nb_pnt == 0 {
            return self.classify_whole(ind_h);
        }
        let mut walk = DomainWalk::new(self.my_keep_points, self.my_keep_segments);
        let hatching = self.hatching_mut(ind_h)?;
        for ipnt in 1..=nb_pnt {
            if walk.step(hatching, ipnt, nb_pnt)? == Flow::Stop {
                return Ok(());
            }
        }
        hatching.set_is_done(true);
        Ok(())
    }

    // A hatching without intersection point: one infinite domain when its
    // classification point is IN the elements.
    fn classify_whole(&mut self, ind_h: i32) -> Result<(), Failure> {
        let point = self.hatching(ind_h)?.classification_point()?;
        let classifier =
            Classifier::with_point(&mut self.my_elements, &point, CLASSIFICATION_TOLERANCE)?;
        let hatching = self.hatching_mut(ind_h)?;
        if classifier.state() == State::In {
            hatching.add_domain(Domain::new());
        }
        hatching.set_is_done(true);
        Ok(())
    }
}
