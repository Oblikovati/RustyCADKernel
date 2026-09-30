// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_PWalking.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntWalk_PWalking`: the intersection line of two parametric surfaces, marched from
//! a start point in both directions until the natural bounds, a tangent point or the
//! closure of the line.
mod arret;
mod boundary;
mod construct;
mod deflection;
mod extend;
mod frame;
mod march;
mod minimize;
mod perform;
mod steps;

use super::IntWalkTheInt2S;
use crate::int_imp::IntImpConstIsoparametric;
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::gp::{Direction2, Direction3};

/// The marching of the intersection of two parametric surfaces.
///
/// `let mut walk = IntWalkPWalking::new(&s1, &s2, 1e-6, 1e-8, 0.01, 0.1)?;
/// walk.perform(&start)?; let n = walk.nb_points()?;`
pub struct IntWalkPWalking {
    done: bool,
    line: Handle<IntSurfLineOn2S>,
    close: bool,
    tgfirst: bool,
    tglast: bool,
    /// Index of the point on the surface boundary, used for the transitions.
    my_tangent_idx: i32,
    /// The tangent to the line at `my_tangent_idx`.
    tgdir: Direction3,
    fleche: f64,
    pas_max: f64,
    tolconf: f64,
    my_tol_tang: f64,
    pasuv: [f64; 4],
    my_step_min: [f64; 4],
    pas_sav: [f64; 4],
    pas_init: [f64; 4],
    /// `Um1, Vm1, Um2, Vm2`: the lower walking bounds.
    lower: [f64; 4],
    /// `UM1, VM1, UM2, VM2`: the upper walking bounds.
    upper: [f64; 4],
    /// `ResoU1, ResoV1, ResoU2, ResoV2`.
    reso: [f64; 4],
    sens_cheminement: i32,
    choix_iso_sav: IntImpConstIsoparametric,
    previous_point: IntSurfPntOn2S,
    previoustg: bool,
    previousd: Direction3,
    previousd1: Direction2,
    previousd2: Direction2,
    firstd1: Direction2,
    firstd2: Direction2,
    my_intersection_on2s: IntWalkTheInt2S,
    static_blocage_sur_pas_trop_grand: i32,
    static_precedent_inflexion: i32,
}

/// The not-done error of the consultation members.
fn not_done(member: &str) -> Failure {
    Failure::with_message("StdFail_NotDone", &format!("IntWalk_PWalking::{member}"))
}

impl IntWalkPWalking {
    /// `IsDone()`: true after the constructor, false during and after a failed
    /// Perform; `walk.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// The line; before any Perform it is a null handle natively (a crash), here
    /// Standard_NullObject.
    fn line_ref(&self) -> Result<&IntSurfLineOn2S, Failure> {
        self.line.get().ok_or_else(|| {
            errors::null_object("IntWalk_PWalking: the line is not computed (no Perform)")
        })
    }
    /// A shared handle on the line, Standard_NullObject before any Perform (so that the
    /// line can be edited while the walking fields change).
    fn line_handle(&self) -> Result<Handle<IntSurfLineOn2S>, Failure> {
        self.line_ref()?;
        Ok(self.line.clone())
    }
    /// `NbPoints()`: StdFail_NotDone unless done; `walk.nb_points()?`.
    pub fn nb_points(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(not_done("NbPoints"));
        }
        Ok(self.line_ref()?.nb_points())
    }
    /// `Value(Index)`: the point of rank `index` (a copy); `walk.value(1)?`.
    pub fn value(&self, index: i32) -> Result<IntSurfPntOn2S, Failure> {
        if !self.done {
            return Err(not_done("Value"));
        }
        self.line_ref()?.value(index)
    }
    /// `Line()`: the polyline; `walk.line()?`.
    pub fn line(&self) -> Result<&Handle<IntSurfLineOn2S>, Failure> {
        if !self.done {
            return Err(not_done("Line"));
        }
        Ok(&self.line)
    }
    /// `TangentAtFirst()`; `walk.tangent_at_first()?`.
    pub fn tangent_at_first(&self) -> Result<bool, Failure> {
        if !self.done {
            return Err(not_done("TangentAtFirst"));
        }
        Ok(self.tgfirst)
    }
    /// `TangentAtLast()`; `walk.tangent_at_last()?`.
    pub fn tangent_at_last(&self) -> Result<bool, Failure> {
        if !self.done {
            return Err(not_done("TangentAtLast"));
        }
        Ok(self.tglast)
    }
    /// `IsClosed()`; `walk.is_closed()?`.
    pub fn is_closed(&self) -> Result<bool, Failure> {
        if !self.done {
            return Err(not_done("IsClosed"));
        }
        Ok(self.close)
    }
    /// `TangentAtLine(Index)`: the tangent at the boundary point and its index;
    /// `let (tangent, index) = walk.tangent_at_line()?;`.
    pub fn tangent_at_line(&self) -> Result<(Direction3, i32), Failure> {
        if !self.done {
            return Err(not_done("TangentAtLine"));
        }
        Ok((self.tgdir, self.my_tangent_idx))
    }
    /// `AddAPoint(thePOn2S)`: appends the point; the boundary index becomes at least 1.
    /// `walk.add_a_point(&point)?`.
    pub fn add_a_point(&mut self, pon2s: &IntSurfPntOn2S) -> Result<(), Failure> {
        self.line_ref()?.add(pon2s);
        self.my_tangent_idx = self.my_tangent_idx.max(1);
        Ok(())
    }
    /// `RemoveAPoint(theIndex)`: removes the point of rank `index` (the last one when
    /// beyond), keeping the boundary index on its point (at least 1); nothing below 1.
    /// `walk.remove_a_point(1)?`.
    pub fn remove_a_point(&mut self, index: i32) -> Result<(), Failure> {
        let line = self.line_handle()?;
        let idx: i32 = index.min(line.nb_points());
        if idx < 1 {
            return Ok(());
        }
        if idx <= self.my_tangent_idx {
            self.my_tangent_idx -= 1;
            if self.my_tangent_idx < 1 {
                self.my_tangent_idx = 1;
            }
        }
        line.remove_point(idx)
    }
    /// `MaxStep(theIndex)`: the initial step of parameter `index` (0..=3, in the order
    /// u1 v1 u2 v2), Standard_OutOfRange outside; `walk.max_step(0)?`.
    pub fn max_step(&self, index: i32) -> Result<f64, Failure> {
        if !(0..=3).contains(&index) {
            return Err(errors::out_of_range(&format!(
                "IntWalk_PWalking::MaxStep() - index {index} is out of range 0..=3"
            )));
        }
        Ok(self.pas_init[index as usize])
    }
    /// The two surfaces of the intersection (`Function().AuxillarSurface1/2()`).
    fn surfaces(&mut self) -> (Adaptor3dSurfaceHandle, Adaptor3dSurfaceHandle) {
        let function = self.my_intersection_on2s.function();
        (
            function.auxillar_surface1().clone(),
            function.auxillar_surface2().clone(),
        )
    }
}
