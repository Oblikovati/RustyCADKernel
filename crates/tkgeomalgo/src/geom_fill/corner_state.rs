// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_CornerState.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_CornerState`: what a constrained filling records at one corner.
use rustycad_tkernel::standard::real;

/// The state of a corner of a constrained filling (`GeomFill_CornerState`): the gap
/// between the two boundaries, the angles between their tangents and their normals,
/// and whether the tangent field must be cancelled there. The overloaded native
/// getters and setters are `gap`/`set_gap`, `tgt_ang`/`set_tgt_ang` and
/// `nor_ang`/`set_nor_ang`.
///
/// `let mut corner = GeomFillCornerState::new(); corner.set_gap(0.1); corner.gap() == 0.1`
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeomFillCornerState {
    gap: f64,
    tgtang: f64,
    isconstrained: bool,
    norang: f64,
    scal: f64,
    coonscnd: bool,
}

impl Default for GeomFillCornerState {
    fn default() -> Self {
        Self::new()
    }
}

impl GeomFillCornerState {
    /// `GeomFill_CornerState()`: an infinite gap (`RealLast`), no constraint, a
    /// scale of 1 and the Coons condition satisfied; `GeomFillCornerState::new()`.
    pub fn new() -> Self {
        Self {
            gap: real::real_last(),
            tgtang: 0.0,
            isconstrained: false,
            norang: 0.0,
            scal: 1.,
            coonscnd: true,
        }
    }
    /// `Gap()`; `corner.gap()`.
    pub fn gap(&self) -> f64 {
        self.gap
    }
    /// `Gap(G)`; `corner.set_gap(0.1)`.
    pub fn set_gap(&mut self, g: f64) {
        self.gap = g;
    }
    /// `TgtAng()`; `corner.tgt_ang()`.
    pub fn tgt_ang(&self) -> f64 {
        self.tgtang
    }
    /// `TgtAng(Ang)`; `corner.set_tgt_ang(0.2)`.
    pub fn set_tgt_ang(&mut self, ang: f64) {
        self.tgtang = ang;
    }
    /// `HasConstraint()`; `corner.has_constraint()`.
    pub fn has_constraint(&self) -> bool {
        self.isconstrained
    }
    /// `Constraint()`: marks the corner as constrained; `corner.constraint()`.
    pub fn constraint(&mut self) {
        self.isconstrained = true;
    }
    /// `NorAng()`; `corner.nor_ang()`.
    pub fn nor_ang(&self) -> f64 {
        self.norang
    }
    /// `NorAng(Ang)`; `corner.set_nor_ang(0.3)`.
    pub fn set_nor_ang(&mut self, ang: f64) {
        self.norang = ang;
    }
    /// `IsToKill(Scal)`: `(kill, scale)`, where the tangent field must be cancelled
    /// only at a constrained corner whose Coons condition failed;
    /// `let (kill, scal) = corner.is_to_kill();`.
    pub fn is_to_kill(&self) -> (bool, f64) {
        (self.isconstrained && !self.coonscnd, self.scal)
    }
    /// `DoKill(Scal)`: records the failed Coons condition and the scale;
    /// `corner.do_kill(0.5)`.
    pub fn do_kill(&mut self, scal: f64) {
        self.scal = scal;
        self.coonscnd = false;
    }
}
