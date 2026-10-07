// Rust translation of the status switch of BRepBlend_Walking::InternalPerform
// (BRepBlend_Walking.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The consequences of the status of a step: the line updated, the next step, or the
//! extremities where the walking stops.
use super::BRepBlendWalking;
use super::internal::March;
use super::recadre::Vertex;
use crate::blend::Status;
use crate::chfi_ds::lock;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_min;

impl BRepBlendWalking {
    /// The switch on the status of a step.
    pub(super) fn apply_status(
        &mut self,
        state: Status,
        bound: f64,
        march: &mut March,
    ) -> Result<(), Failure> {
        match state {
            Status::Ok => self.advance(bound, march, false),
            Status::StepTooSmall => self.advance(bound, march, true),
            Status::StepTooLarge => self.shorten(march),
            Status::OnRst1 => self.stop_on_rst1(march),
            Status::OnRst2 => self.stop_on_rst2(march),
            Status::OnRst12 => self.stop_on_rst12(march),
            Status::SamePoints => {
                // On arrete
                (march.ext1, march.ext2) = self.previous_ends()?;
                march.arrive = true;
                Ok(())
            }
            Status::Backward => Ok(()),
        }
    }

    /// The previous point appended to the line (prepended when walking backward).
    fn push_previous(&self) -> Result<(), Failure> {
        let mut line = lock(self.line_handle()?);
        if self.sens > 0.0 {
            line.append(self.previous_p);
        } else {
            line.prepend(self.previous_p);
        }
        Ok(())
    }

    /// Mettre a jour la ligne, then the next parameter (the step grown by half when it
    /// was too small), or the end on the bound.
    fn advance(&mut self, bound: f64, march: &mut March, grow: bool) -> Result<(), Failure> {
        self.push_previous()?;
        march.parprec = self.param;
        if grow {
            march.stepw = std_min(1.5 * march.stepw, self.pasmax);
        }
        if self.param == bound {
            // Indiquer que fin sur Bound.
            march.arrive = true;
            (march.ext1, march.ext2) = self.previous_ends()?;
        } else {
            self.param += self.sens * march.stepw;
            if self.sens * (self.param - bound) > -self.tolgui {
                self.param = bound;
            }
        }
        self.evalpinit(march)
    }

    /// The step halved, or the stop at the previous point below the guide tolerance.
    fn shorten(&mut self, march: &mut March) -> Result<(), Failure> {
        march.stepw /= 2.0;
        if march.stepw.abs() < self.tolgui {
            (march.ext1, march.ext2) = self.previous_ends()?;
            march.arrive = true;
            return Ok(());
        }
        // on ne risque pas de depasser Bound.
        self.param = march.parprec + self.sens * march.stepw;
        self.evalpinit(march)
    }

    /// The stop on the first restriction; the singular case where the reframing on the
    /// second one failed is protected (both ends at the same point).
    fn stop_on_rst1(&mut self, march: &mut March) -> Result<(), Failure> {
        self.push_previous()?;
        let rec1 = reframing(&march.rec1)?;
        let vtx1 = rec1.vertex.clone();
        self.make_extremity(
            &mut march.ext1,
            true,
            rec1.index,
            rec1.solrst[0],
            vtx1.as_ref(),
        )?;
        let previous = self.previous_p;
        let (u2, v2) = (self.sol[2], self.sol[3]);
        if previous
            .point_on_s1()
            .is_equal(&previous.point_on_s2(), 2.0 * self.tolpoint3d)
        {
            march
                .ext2
                .set_value_on_curve(previous.point_on_s1(), u2, v2, self.tolpoint3d);
            if let Some(vtx1) = &vtx1 {
                self.make_singular_extremity(&mut march.ext2, false, vtx1)?;
            }
        } else {
            let w = previous.parameter();
            march
                .ext2
                .set_value(previous.point_on_s2(), u2, v2, w, self.tolpoint3d);
        }
        march.arrive = true;
        Ok(())
    }

    /// The stop on the second restriction, symmetric of [`Self::stop_on_rst1`].
    fn stop_on_rst2(&mut self, march: &mut March) -> Result<(), Failure> {
        self.push_previous()?;
        let rec2 = reframing(&march.rec2)?;
        let vtx2 = rec2.vertex.clone();
        let previous = self.previous_p;
        let (u1, v1) = (self.sol[0], self.sol[1]);
        if previous
            .point_on_s1()
            .is_equal(&previous.point_on_s2(), 2.0 * self.tolpoint3d)
        {
            march
                .ext1
                .set_value_on_curve(previous.point_on_s2(), u1, v1, self.tolpoint3d);
            if let Some(vtx2) = &vtx2 {
                self.make_singular_extremity(&mut march.ext1, true, vtx2)?;
            }
        } else {
            let w = previous.parameter();
            march
                .ext1
                .set_value(previous.point_on_s1(), u1, v1, w, self.tolpoint3d);
        }
        self.make_extremity(
            &mut march.ext2,
            false,
            rec2.index,
            rec2.solrst[0],
            vtx2.as_ref(),
        )?;
        march.arrive = true;
        Ok(())
    }

    /// The stop on both restrictions; when only one reframing is a vertex and both ends
    /// are at the same point, both are that vertex.
    fn stop_on_rst12(&mut self, march: &mut March) -> Result<(), Failure> {
        self.push_previous()?;
        let (rec1, rec2) = (reframing(&march.rec1)?, reframing(&march.rec2)?);
        let (mut vtx1, mut vtx2): (Option<Vertex>, Option<Vertex>) =
            (rec1.vertex.clone(), rec2.vertex.clone());
        let previous = self.previous_p;
        let same = previous
            .point_on_s1()
            .is_equal(&previous.point_on_s2(), 2.0 * self.tolpoint3d);
        if vtx1.is_some() != vtx2.is_some() && same {
            // On blinde le cas singulier ou un seul recadrage est reconnu comme vertex.
            if vtx1.is_some() {
                vtx2.clone_from(&vtx1);
            } else {
                vtx1.clone_from(&vtx2);
            }
        }
        let (index1, w1, index2, w2) = (rec1.index, rec1.solrst[0], rec2.index, rec2.solrst[0]);
        self.make_extremity(&mut march.ext1, true, index1, w1, vtx1.as_ref())?;
        self.make_extremity(&mut march.ext2, false, index2, w2, vtx2.as_ref())?;
        march.arrive = true;
        Ok(())
    }
}

/// The reframing of a step that stops on its restriction.
fn reframing(rec: &Option<super::recadre::Reframed>) -> Result<&super::recadre::Reframed, Failure> {
    rec.as_ref().ok_or_else(|| {
        Failure::with_message(
            "Standard_Failure",
            "BRepBlend_Walking::InternalPerform: no reframing; expected one for a stop on a restriction",
        )
    })
}
