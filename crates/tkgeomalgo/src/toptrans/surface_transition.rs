// Rust translation of ModelingAlgorithms/TKGeomAlgo/TopTrans/TopTrans_SurfaceTransition.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! TopTrans_SurfaceTransition: the complex transition of a reference surface crossing
//! several boundary faces along an intersection curve (eap, Mar 2002: touch case).
use super::nearest::{
    CurvedSector, NearestUpdate, SectorFace, UNDEFINED_ANGLE, face_angle, inside_normal,
    nearest_curved, nearest_planar, opposite, sector,
};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkmath::gp::Direction3;

/// The sector row of the faces after the intersection (cosine > 0, `AFTER` natively).
const AFTER: usize = 1;
/// The sector row of the faces before the intersection (cosine < 0, `BEFORE` natively).
const BEFORE: usize = 2;

/// TopTrans_SurfaceTransition: around the intersection curve, the plane normal to its
/// tangent is cut in four sectors (signs of the cosine and sine of the angle from the
/// before-after direction); each sector keeps the boundary face nearest the reference
/// surface, and the states before and after the curve follow from them.
///
/// The per-sector arrays are uninitialized natively (NCollection_Array2 of trivial
/// types) and only read after a write; the port starts them at 0 and FORWARD.
/// ```
/// use rustycad_tkgeomalgo::toptrans::SurfaceTransition;
/// use rustycad_tkg3d::top_abs::{Orientation, State};
/// use rustycad_tkmath::gp::Direction3;
/// let mut transition = SurfaceTransition::new();
/// transition.reset_line(Direction3::X, Direction3::Z).unwrap();
/// transition.compare_plane(1e-12, Direction3::Z, Orientation::Forward, Orientation::Forward).unwrap();
/// assert_ne!(transition.state_before(), State::Unknown);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct SurfaceTransition {
    tgt: Direction3,
    norm: Direction3,
    beafter: Direction3,
    curv_ref: f64,
    ang: [[f64; 2]; 2],
    curv: [[f64; 2]; 2],
    ori: [[Orientation; 2]; 2],
    is_defined: bool,
    touch_flag: bool,
}

impl Default for SurfaceTransition {
    fn default() -> Self {
        Self::new()
    }
}

impl SurfaceTransition {
    /// TopTrans_SurfaceTransition(): an undefined transition, whose states are UNKNOWN
    /// until a reset; `SurfaceTransition::new()`.
    pub fn new() -> Self {
        Self {
            tgt: Direction3::default(),
            norm: Direction3::default(),
            beafter: Direction3::default(),
            curv_ref: 0.0,
            ang: [[0.0; 2]; 2],
            curv: [[0.0; 2]; 2],
            ori: [[Orientation::Forward; 2]; 2],
            is_defined: false,
            touch_flag: false,
        }
    }

    /// Reset(Tgt, Norm, MaxD, MinD, MaxCurv, MinCurv): starts a transition for the
    /// curve tangent `tgt` on a reference surface with normal `norm` (oriented outside
    /// the matter) and principal directions and curvatures. With both curvatures null
    /// at `Precision::Angular` it is the straight case; when the tangent follows no
    /// principal direction the transition stays undefined (not implemented natively).
    /// Raises Standard_ConstructionError when `norm` and `tgt` are parallel, after
    /// storing them; `transition.reset(t, n, max_d, min_d, 1.0, 0.0)?`.
    pub fn reset(
        &mut self,
        tgt: Direction3,
        norm: Direction3,
        max_d: Direction3,
        min_d: Direction3,
        max_curv: f64,
        min_curv: f64,
    ) -> Result<(), Failure> {
        self.store_frame(tgt, norm)?;
        let tola: f64 = precision::angular();
        let (curv_is_max, curv_is_min) = principal_matches(tgt, max_d, min_d, tola);
        if max_curv.abs() < tola && min_curv.abs() < tola {
            return self.reset_line(tgt, norm);
        }
        if !curv_is_max && !curv_is_min {
            return Ok(());
        }
        if curv_is_max {
            self.curv_ref = max_curv.abs();
        }
        if curv_is_min {
            self.curv_ref = min_curv.abs();
        }
        if self.curv_ref < tola {
            self.curv_ref = 0.0;
        }
        self.clear_sectors();
        Ok(())
    }

    /// Reset(Tgt, Norm): starts a transition for a straight curve tangent `tgt` on a
    /// reference surface with normal `norm`; raises Standard_ConstructionError when
    /// they are parallel; `transition.reset_line(Direction3::X, Direction3::Z)?`.
    pub fn reset_line(&mut self, tgt: Direction3, norm: Direction3) -> Result<(), Failure> {
        self.store_frame(tgt, norm)?;
        self.curv_ref = 0.0;
        self.clear_sectors();
        Ok(())
    }

    /// Compare(Tole, Norm, MaxD, MinD, MaxCurv, MinCurv, S, O): adds a curved boundary
    /// face with normal `norm` (oriented outside the matter) and principal directions
    /// and curvatures. `transition` is the face orientation (FORWARD: OUT then IN along
    /// the curve, REVERSED: IN then OUT, INTERNAL: IN, IN); `orientation` places the
    /// curve on the face (FORWARD: the face is before, REVERSED: after, INTERNAL: the
    /// curve is inside it). A non-positive `tole` means `Precision::Angular`. Raises
    /// Standard_ConstructionError when `norm` is parallel to the tangent;
    /// `transition.compare(1e-7, n, max_d, min_d, 1.0, 0.5, s, o)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn compare(
        &mut self,
        tole: f64,
        norm: Direction3,
        max_d: Direction3,
        min_d: Direction3,
        max_curv: f64,
        min_curv: f64,
        transition: Orientation,
        orientation: Orientation,
    ) -> Result<(), Failure> {
        if !self.is_defined {
            return Ok(());
        }
        let tola: f64 = if tole > 0.0 {
            tole
        } else {
            precision::angular()
        };
        let (curv_is_max, curv_is_min) = principal_matches(self.tgt, max_d, min_d, tola);
        if !curv_is_max && !curv_is_min {
            // The face would be a boundary curve in the plane normal to the tangent:
            // not implemented natively.
            self.is_defined = false;
            return Ok(());
        }
        let curvature: f64 = if curv_is_min {
            min_curv.abs()
        } else {
            max_curv.abs()
        };
        let curvature = self.signed_curvature(norm, curvature, tola)?;
        let angle: f64 = face_angle(self.beafter, self.tgt, norm, orientation)?;
        let face = SectorFace {
            angle,
            orientation: transition,
            curvature,
        };
        let offered = OfferedFace {
            face,
            stores_curvature: true,
        };
        self.update_sectors(offered, tola, orientation, |this, i, j| {
            this.nearest_curved_at(i, j, tola, face)
        });
        Ok(())
    }

    /// Compare(Tole, Norm, S, O): adds a plane or cylindrical boundary face with normal
    /// `norm`, as `compare` without curvatures; raises Standard_ConstructionError when
    /// `norm` is parallel to the tangent;
    /// `transition.compare_plane(1e-7, Direction3::Z, s, o)?`.
    pub fn compare_plane(
        &mut self,
        tole: f64,
        norm: Direction3,
        transition: Orientation,
        orientation: Orientation,
    ) -> Result<(), Failure> {
        if !self.is_defined {
            return Ok(());
        }
        // The oriented angle from beafter to the normal of the curve inside the face.
        let angle: f64 = face_angle(self.beafter, self.tgt, norm, orientation)?;
        let tola: f64 = if tole > 0.0 {
            tole
        } else {
            precision::angular()
        };
        let face = SectorFace {
            angle,
            orientation: transition,
            curvature: 0.0,
        };
        let offered = OfferedFace {
            face,
            stores_curvature: false,
        };
        self.update_sectors(offered, tola, orientation, |this, i, j| {
            (nearest_planar(this.stored(i, j), face, tola), false)
        });
        Ok(())
    }

    /// StateBefore: the state of the reference surface just before the curve; when the
    /// before sectors are undefined or incoherent, the after sectors decide, inverted
    /// for touching faces; `transition.state_before()`.
    pub fn state_before(&self) -> State {
        self.state_at(BEFORE, AFTER, get_before)
    }

    /// StateAfter: the state of the reference surface just after the curve, with the
    /// same fallback as `state_before`; `transition.state_after()`.
    pub fn state_after(&self) -> State {
        self.state_at(AFTER, BEFORE, get_after)
    }

    fn store_frame(&mut self, tgt: Direction3, norm: Direction3) -> Result<(), Failure> {
        self.is_defined = false;
        // beafter is oriented from before to after the curve on the reference surface.
        self.norm = norm;
        self.tgt = tgt;
        self.beafter = norm.crossed(tgt)?;
        Ok(())
    }

    fn clear_sectors(&mut self) {
        self.ang = [[UNDEFINED_ANGLE; 2]; 2];
        self.touch_flag = false;
        self.is_defined = true;
    }

    /// The face curvature, null on a flat reference and negated when the face turns
    /// against the tangent.
    fn signed_curvature(
        &self,
        norm: Direction3,
        curvature: f64,
        tola: f64,
    ) -> Result<f64, Failure> {
        let curvature: f64 = if self.curv_ref < tola { 0.0 } else { curvature };
        let on_face: Direction3 = inside_normal(self.tgt, norm)?;
        let prod: f64 = on_face.crossed(norm)?.dot(self.tgt);
        Ok(if prod < 0.0 { -curvature } else { curvature })
    }

    fn stored(&self, i: usize, j: usize) -> SectorFace {
        SectorFace {
            angle: self.ang[i - 1][j - 1],
            orientation: self.ori[i - 1][j - 1],
            curvature: self.curv[i - 1][j - 1],
        }
    }

    fn nearest_curved_at(
        &self,
        i: usize,
        j: usize,
        tola: f64,
        offered: SectorFace,
    ) -> (NearestUpdate, bool) {
        let at = CurvedSector {
            i,
            j,
            reference_curvature: self.curv_ref,
            tola,
        };
        nearest_curved(at, self.stored(i, j), offered)
    }

    /// The sector loop shared by both Compare overloads: an INTERNAL curve also visits
    /// the opposite sector, and a null sign visits both neighbours. As natively, the
    /// opposite sector is taken from the indices the first pass left behind.
    fn update_sectors(
        &mut self,
        offered: OfferedFace,
        tola: f64,
        orientation: Orientation,
        decide: impl Fn(&Self, usize, usize) -> (NearestUpdate, bool),
    ) {
        let (mut i, mut j) = sector(offered.face.angle, tola);
        let passes: usize = if orientation == Orientation::Internal {
            2
        } else {
            1
        };
        for pass in 1..=passes {
            if pass == 2 {
                (i, j) = (opposite(i), opposite(j));
            }
            let (i0, j0) = (i == 0, j == 0);
            let neighbours: usize = if i0 || j0 { 2 } else { 1 };
            for n in 1..=neighbours {
                i = if i0 { n } else { i };
                j = if j0 { n } else { j };
                if !self.visit_sector(i, j, offered, &decide) {
                    return;
                }
            }
        }
    }

    /// Applies one decision to sector (i, j); returns false when the transition became
    /// undefined, which stops the loop.
    fn visit_sector(
        &mut self,
        i: usize,
        j: usize,
        offered: OfferedFace,
        decide: &impl Fn(&Self, usize, usize) -> (NearestUpdate, bool),
    ) -> bool {
        let (update, touched) = decide(self, i, j);
        self.touch_flag |= touched;
        let orientation: Orientation = match update {
            NearestUpdate::Unknown => {
                self.is_defined = false;
                return false;
            }
            NearestUpdate::Keep => return true,
            NearestUpdate::Internal => Orientation::Internal,
            NearestUpdate::Replace => offered.face.orientation,
        };
        self.ang[i - 1][j - 1] = offered.face.angle;
        self.ori[i - 1][j - 1] = orientation;
        // The plane overload leaves the sector curvature as it was.
        if offered.stores_curvature {
            self.curv[i - 1][j - 1] = offered.face.curvature;
        }
        true
    }

    /// FUN_getstate over the sector row `primary`, then over `fallback` (inverted for
    /// touching faces) when the first is undefined or incoherent.
    fn state_at(
        &self,
        primary: usize,
        fallback: usize,
        state_of: fn(Orientation) -> State,
    ) -> State {
        if !self.is_defined {
            return State::Unknown;
        }
        let state: State = self.row_state(primary, state_of);
        if state != State::Unknown {
            return state;
        }
        // Looking back in the other row for defined states (eap, 25 Mar 2002).
        let state: State = self.row_state(fallback, state_of);
        match state {
            State::Out if self.touch_flag => State::In,
            State::In if self.touch_flag => State::Out,
            _ => state,
        }
    }

    /// FUN_getstate: the state that the defined sectors of row `row` give, UNKNOWN when
    /// none is defined or when the two disagree (incoherent data).
    fn row_state(&self, row: usize, state_of: fn(Orientation) -> State) -> State {
        let angles: [f64; 2] = self.ang[row - 1];
        let orientations: [Orientation; 2] = self.ori[row - 1];
        let undef1: bool = angles[0] == UNDEFINED_ANGLE;
        let undef2: bool = angles[1] == UNDEFINED_ANGLE;
        if undef1 && undef2 {
            return State::Unknown;
        }
        if undef1 || undef2 {
            return state_of(orientations[if undef1 { 1 } else { 0 }]);
        }
        let (st1, st2) = (state_of(orientations[0]), state_of(orientations[1]));
        if st1 != st2 { State::Unknown } else { st1 }
    }
}

/// A face offered to the sectors, with whether its curvature is stored (only the
/// curved Compare overload stores it).
#[derive(Clone, Copy, Debug)]
struct OfferedFace {
    face: SectorFace,
    stores_curvature: bool,
}

/// Whether the tangent follows the maximal and the minimal principal direction.
fn principal_matches(
    tgt: Direction3,
    max_d: Direction3,
    min_d: Direction3,
    tola: f64,
) -> (bool, bool) {
    (max_d.dot(tgt).abs() < tola, min_d.dot(tgt).abs() < tola)
}

/// GetBefore: the state before a boundary of orientation `transition` (OUT for FORWARD
/// and EXTERNAL, IN for REVERSED and INTERNAL);
/// `get_before(Orientation::Forward) == State::Out`.
pub fn get_before(transition: Orientation) -> State {
    match transition {
        Orientation::Forward | Orientation::External => State::Out,
        Orientation::Reversed | Orientation::Internal => State::In,
    }
}

/// GetAfter: the state after a boundary of orientation `transition` (IN for FORWARD and
/// INTERNAL, OUT for REVERSED and EXTERNAL);
/// `get_after(Orientation::Forward) == State::In`.
pub fn get_after(transition: Orientation) -> State {
    match transition {
        Orientation::Forward | Orientation::Internal => State::In,
        Orientation::Reversed | Orientation::External => State::Out,
    }
}
