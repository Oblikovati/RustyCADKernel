//! `ProjectPointOnSurf`: the nearest extremum of a point on a bounded surface.
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeombase::extrema::ExtPS;
use rustycad_tkmath::gp::Point3;

/// `ProjectPointOnSurf`: `Extrema_ExtPS` on a `GeomAdaptor_Surface` loaded with the
/// bounds, and the index of its nearest extremum.
#[derive(Default)]
pub(super) struct ProjectPointOnSurf {
    is_done: bool,
    index: i32,
    ext_ps: ExtPS<'static>,
}

impl ProjectPointOnSurf {
    /// `Init(Surface, Umin, Usup, Vmin, Vsup)`: the projector on the bounded surface
    /// with the tolerance `Precision::PConfusion()`.
    pub(super) fn init(
        &mut self,
        surface: &Handle<dyn GeomSurface>,
        (umin, usup, vmin, vsup): (f64, f64, f64, f64),
    ) -> Result<(), Failure> {
        let tolerance: f64 = precision::p_confusion();
        let adaptor = GeomAdaptorSurface::default();
        adaptor.load_bounds(surface, (umin, usup, vmin, vsup))?;
        let adaptor: Handle<dyn Adaptor3dSurface> = Handle::new(adaptor).cast();
        self.ext_ps.initialize_shared(
            &adaptor,
            (umin, usup),
            (vmin, vsup),
            (tolerance, tolerance),
        )?;
        self.is_done = false;
        Ok(())
    }

    /// `Perform(P)` and `Init()`: done when some extremum is found, with the index of
    /// the nearest one (the first of equal distances).
    pub(super) fn perform(&mut self, p: Point3) -> Result<(), Failure> {
        self.ext_ps.perform(p)?;
        self.is_done = self.ext_ps.is_done() && self.ext_ps.nb_ext()? > 0;
        if !self.is_done {
            return Ok(());
        }
        let mut dist2_min: f64 = self.ext_ps.square_distance(1)?;
        self.index = 1;
        for i in 2..=self.ext_ps.nb_ext()? {
            let dist2: f64 = self.ext_ps.square_distance(i)?;
            if dist2 < dist2_min {
                dist2_min = dist2;
                self.index = i;
            }
        }
        Ok(())
    }

    /// `IsDone()`.
    pub(super) fn is_done(&self) -> bool {
        self.is_done
    }

    /// `LowerDistanceParameters(U, V)`: StdFail_NotDone when not done.
    pub(super) fn lower_distance_parameters(&self) -> Result<(f64, f64), Failure> {
        self.check_done("LowerDistanceParameters")?;
        Ok(self.ext_ps.point(self.index)?.parameter())
    }

    /// `LowerDistance()`: StdFail_NotDone when not done.
    pub(super) fn lower_distance(&self) -> Result<f64, Failure> {
        self.check_done("LowerDistance")?;
        Ok(self.ext_ps.square_distance(self.index)?.sqrt())
    }

    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if self.is_done {
            return Ok(());
        }
        Err(Failure::with_message(
            "StdFail_NotDone",
            &format!("GeomInt_IntSS::ProjectPointOnSurf::{member}: no extremum was found"),
        ))
    }
}
