// Rust translation of BRepBlend_Walking::Transition, MakeExtremity, MakeSingularExtremity,
// CorrectExtremityOnOneRst and RecadreIfPeriodic (BRepBlend_Walking.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The ends of a walking on the restrictions: their transitions and arcs, and the
//! correction of an end near a vertex of the guide.
use super::recadre::{Arc, Vertex};
use super::{BRepBlendWalking, live, live_guide};
use crate::brep_blend::{BRepBlendBlendTool, BRepBlendExtremity, BRepBlendHCurve2dTool};
use crate::chfi_ds::lock;
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkgeomalgo::int_surf::{IntSurfTransition, make_transition};
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag, ExtPC, ExtPS};
use rustycad_tkgeombase::gce::{MakePln, Root};
use rustycad_tkmath::cslib::{self, NormalStatus, ParametricPosition};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Vector3};

/// A corrected end on the other surface: its parameters and its point (the `NewU`,
/// `NewV`, `NewPoint` outputs).
pub(super) struct CorrectedEnd {
    pub(super) u: f64,
    pub(super) v: f64,
    pub(super) point: Point3,
}

/// The outputs of CorrectExtremityOnOneRst: `NewParam`, written as soon as the guide has
/// vertices (even when the correction fails), and the corrected end.
pub(super) struct Correction {
    pub(super) new_param: Option<f64>,
    pub(super) end: Option<CorrectedEnd>,
}

impl Correction {
    /// The guide parameter and the end of a successful correction.
    pub(super) fn corrected(&self) -> Option<(f64, &CorrectedEnd)> {
        Some((self.new_param?, self.end.as_ref()?))
    }
}

/// A point of a surface and its parameters.
type SurfacePoint = (Point3, (f64, f64));

/// `TolAng` of CorrectExtremityOnOneRst (bug OCC25701).
const TOL_ANG: f64 = 0.001;

impl BRepBlendWalking {
    /// `Transition(OnFirst, A, Param, TLine, TArc)`: the transitions of the line and of
    /// the arc `a` at `param`; the default transitions when the previous point is a
    /// tangency point of a line of less than two points.
    fn transition(
        &self,
        on_first: bool,
        a: &Arc,
        param: f64,
    ) -> Result<(IntSurfTransition, IntSurfTransition), Failure> {
        let (mut tline, mut tarc) = (IntSurfTransition::new(), IntSurfTransition::new());
        let Some(tgline) = self.line_tangent(on_first)? else {
            return Ok((tline, tarc));
        };
        let (p2d, dp2d) = BRepBlendHCurve2dTool::d1(a, param)?;
        let surf = live(self.surface(on_first))?;
        let d1 = surf.d1(p2d.x(), p2d.y())?;
        let mut tgrst = Vector3::default();
        tgrst.set_linear_form(dp2d.x(), d1.d1u, dp2d.y(), d1.d1v);
        let normal = cslib::normal_with_magnitude(d1.d1u, d1.d1v, 1.0e-9)?;
        let thenormal = match normal.status {
            NormalStatus::Defined => normal.normal.unwrap_or_default(),
            _ => singular_normal(surf, p2d.x(), p2d.y())?,
        };
        let normale = Vector3::from(thenormal);
        make_transition(
            tgline,
            tgrst,
            Direction3::from_vector(normale)?,
            &mut tline,
            &mut tarc,
        );
        Ok((tline, tarc))
    }

    /// The tangent of the line at the previous point: its own one, or the chord from the
    /// point before when it is a tangency point; `None` on a line too short for a chord.
    fn line_tangent(&self, on_first: bool) -> Result<Option<Vector3>, Failure> {
        let previous = &self.previous_p;
        if !previous.is_tangency_point() {
            let tangent = if on_first {
                previous.tangent_on_s1()?
            } else {
                previous.tangent_on_s2()?
            };
            return Ok(Some(tangent));
        }
        let line = lock(self.line_handle()?);
        if line.nb_points() < 2 {
            return Ok(None);
        }
        let index = if self.sens < 0.0 {
            2
        } else {
            line.nb_points() - 1
        };
        let prevprev = *line.point(index)?;
        let chord = if on_first {
            Vector3::between(prevprev.point_on_s1(), previous.point_on_s1())
        } else {
            Vector3::between(prevprev.point_on_s2(), previous.point_on_s2())
        };
        Ok(Some(chord))
    }

    /// `MakeExtremity(Extrem, OnFirst, Index, Param, IsVtx, Vtx)`: the end at the
    /// previous point on the arc `index`, at the parameter `param` of the arc.
    pub(super) fn make_extremity(
        &self,
        extrem: &mut BRepBlendExtremity,
        on_first: bool,
        index: i32,
        param: f64,
        vtx: Option<&Vertex>,
    ) -> Result<(), Failure> {
        let previous = &self.previous_p;
        let (point, (u, v)) = if on_first {
            (previous.point_on_s1(), (self.sol[0], self.sol[1]))
        } else {
            (previous.point_on_s2(), (self.sol[2], self.sol[3]))
        };
        extrem.set_value(point, u, v, previous.parameter(), self.tolpoint3d);
        if !previous.is_tangency_point() {
            let tangent = if on_first {
                previous.tangent_on_s1()?
            } else {
                previous.tangent_on_s2()?
            };
            extrem.set_tangent(tangent);
        }
        let arc = self.nth_arc(on_first, index)?;
        let (tline, tarc) = self.transition(on_first, &arc, param)?;
        extrem.add_arc(&arc, param, tline, tarc);
        if let Some(vtx) = vtx {
            extrem.set_vertex(vtx);
        }
        Ok(())
    }

    /// `MakeSingularExtremity(Extrem, OnFirst, Vtx)`: the end at the vertex `vtx`, on
    /// every arc of the domain that holds it.
    pub(super) fn make_singular_extremity(
        &self,
        extrem: &mut BRepBlendExtremity,
        on_first: bool,
        vtx: &Vertex,
    ) -> Result<(), Failure> {
        if !self.previous_p.is_tangency_point() {
            let previous = &self.previous_p;
            let tangent = if on_first {
                previous.tangent_on_s1()?
            } else {
                previous.tangent_on_s2()?
            };
            extrem.set_tangent(tangent);
        }
        let iter = self.rec_domain(on_first);
        iter.init();
        extrem.set_vertex(vtx);
        while iter.more() {
            let arc = iter.value()?;
            iter.initialize_arc(&arc)?;
            iter.init_vertex_iterator()?;
            while iter.more_vertex() {
                if iter.identical(vtx, &iter.vertex()?)? {
                    let prm = BRepBlendBlendTool::parameter(vtx, &arc)?;
                    let (tline, tarc) = self.transition(on_first, &arc, prm)?;
                    extrem.add_arc(&arc, prm, tline, tarc);
                }
                iter.next_vertex()?;
            }
            iter.next();
        }
        Ok(())
    }

    /// `CorrectExtremityOnOneRst(IndexOfRst, theU, theV, theParam, thePntOnRst, ...)`:
    /// when the end on the restriction `index_of_rst` is at a vertex of the guide, the
    /// point of the other surface `(the_u, the_v)` moved into the plane of that vertex.
    pub(super) fn correct_extremity_on_one_rst(
        &self,
        index_of_rst: i32,
        uv: (f64, f64),
        the_param: f64,
        the_pnt_on_rst: Point3,
    ) -> Result<Correction, Failure> {
        let guide = live_guide(&self.hguide)?;
        if guide.nb_vertices() == 0 {
            return Ok(Correction {
                new_param: None,
                end: None,
            });
        }
        let on_first = index_of_rst == 1;
        // Correct point on surface 2. First we find right <param>
        let (param_on_guide, point_on_guide) = self.nearest_guide_point(on_first)?;
        let new_param = self.guide_parameter_near(param_on_guide, the_param)?;
        let end = self.corrected_end(on_first, uv, (the_param, the_pnt_on_rst), point_on_guide)?;
        Ok(Correction {
            new_param: Some(new_param),
            end,
        })
    }

    /// The second part of CorrectExtremityOnOneRst: the point of the other surface in
    /// the plane of the guide vertex nearest to `point_on_guide`, when it is a solution.
    fn corrected_end(
        &self,
        on_first: bool,
        (the_u, the_v): (f64, f64),
        (the_param, the_pnt_on_rst): (f64, Point3),
        point_on_guide: Point3,
    ) -> Result<Option<CorrectedEnd>, Failure> {
        let guide = live_guide(&self.hguide)?;
        // Second we find right point and tangent on guide
        let ax1 = nearest_vertex(guide, point_on_guide)?;
        let another = live(self.surface(!on_first))?;
        // Check new point: is it real solution?
        let old_pon_guide = guide.value(the_param)?;
        let pnt_on_surf2 = another.value(the_u, the_v)?; // old point
        if !in_vertex_plane(&ax1, [the_pnt_on_rst, old_pon_guide, pnt_on_surf2])? {
            return Ok(None);
        }
        // Project the point (theU, theV) on the plane (Pnt0, Dir0)
        let (pnt0, dir0) = (ax1.location(), ax1.direction());
        let a_vec = Vector3::between(pnt0, pnt_on_surf2);
        let a_translation = Vector3::from((*a_vec.xyz() * dir0.xyz()) * dir0.xyz());
        let pnt_on_plane = pnt_on_surf2.translated(-a_translation);
        // Check new point again: does point on restriction belong to the plane?
        if !in_vertex_plane(&ax1, [the_pnt_on_rst, pnt0, pnt_on_plane])? {
            return Ok(None);
        }
        // Project the point <PntOnPlane> on the surface 2
        let Some((point, (u, v))) = nearest_on_surface(pnt_on_plane, another)? else {
            return Ok(None);
        };
        let uperiod = if another.is_u_periodic()? {
            another.u_period()?
        } else {
            0.0
        };
        let vperiod = if another.is_v_periodic()? {
            another.v_period()?
        } else {
            0.0
        };
        let (u, v) = recadre_if_periodic((u, v), (the_u, the_v), (uperiod, vperiod));
        Ok(Some(CorrectedEnd { u, v, point }))
    }

    /// The parameter and the point of the guide nearest to the ends of the current arc
    /// of the restriction domain.
    fn nearest_guide_point(&self, on_first: bool) -> Result<(f64, Point3), Failure> {
        let guide = live_guide(&self.hguide)?;
        let arc = self.rec_domain(on_first).value()?;
        let surf = live(self.surface(on_first))?;
        let ends = [
            BRepBlendHCurve2dTool::first_parameter(&arc)?,
            BRepBlendHCurve2dTool::last_parameter(&arc)?,
        ];
        let mut global_min = precision::infinite();
        let (mut param_on_guide, mut point_on_guide) = (0.0, Point3::default());
        for end in ends {
            let p2d = BRepBlendHCurve2dTool::value(&arc, end)?;
            let pnt_on_end = surf.value(p2d.x(), p2d.y())?;
            let projoncurv = ExtPC::with_curve(pnt_on_end, guide as &dyn Adaptor3dCurve, 1.0e-10)?;
            if !projoncurv.is_done() {
                continue;
            }
            let (mut min_sq, mut imin) = (precision::infinite(), 0);
            for ind in 1..=projoncurv.nb_ext()? {
                let sq = projoncurv.square_distance(ind)?;
                if sq < min_sq {
                    (min_sq, imin) = (sq, ind);
                }
            }
            if min_sq < global_min {
                global_min = min_sq;
                let p = projoncurv.point(imin)?;
                (param_on_guide, point_on_guide) = (p.parameter(), *p.value());
            }
        }
        Ok((param_on_guide, point_on_guide))
    }

    /// The guide parameter `param` moved by periods within half a period of `reference`.
    fn guide_parameter_near(&self, param: f64, reference: f64) -> Result<f64, Failure> {
        let guide = live_guide(&self.hguide)?;
        if !guide.is_periodic()? {
            return Ok(param);
        }
        let period = guide.period()?;
        let sign = if param < reference { 1.0 } else { -1.0 };
        let mut param = param;
        while (param - reference).abs() > period / 2.0 {
            param += sign * period;
        }
        Ok(param)
    }
}

/// The vertex of the guide (with its tangent) nearest to `point`; the Z axis when none
/// is nearer than Precision::Infinite.
fn nearest_vertex(guide: &crate::chfi_ds::ElSpine, point: Point3) -> Result<Axis1, Failure> {
    let mut global_min = precision::infinite();
    let mut the_ax1 = Axis1::default();
    for ind in 1..=guide.nb_vertices() {
        let an_ax1 = guide.vertex_with_tangent(ind)?;
        let sq = point.square_distance(&an_ax1.location());
        if sq < global_min {
            (global_min, the_ax1) = (sq, an_ax1);
        }
    }
    Ok(the_ax1)
}

/// Whether the plane through the three points is normal to the vertex tangent within
/// TolAng; false when the points are colinear.
fn in_vertex_plane(ax1: &Axis1, [p1, p2, p3]: [Point3; 3]) -> Result<bool, Failure> {
    let builder = MakePln::through_points(p1, p2, p3)?;
    if !builder.is_done() {
        return Ok(false);
    }
    let dir = builder.value()?.axis().direction();
    let mut angle = dir.angle(&ax1.direction());
    if angle > std::f64::consts::PI / 2.0 {
        angle = std::f64::consts::PI - angle;
    }
    Ok(angle.partial_cmp(&TOL_ANG) != Some(std::cmp::Ordering::Greater))
}

/// The nearest point of the surface to `point` (minimum extrema at PConfusion).
fn nearest_on_surface(
    point: Point3,
    surface: &dyn Adaptor3dSurface,
) -> Result<Option<SurfacePoint>, Failure> {
    let tol = precision::p_confusion();
    let projonsurf = ExtPS::with_surface(point, surface, tol, tol, ExtFlag::Min, ExtAlgo::Grad)?;
    if !projonsurf.is_done() {
        return Ok(None);
    }
    let (mut min_sq, mut imin) = (precision::infinite(), 0);
    for ind in 1..=projonsurf.nb_ext()? {
        let sq = projonsurf.square_distance(ind)?;
        if sq < min_sq {
            (min_sq, imin) = (sq, ind);
        }
    }
    if imin == 0 {
        return Ok(None);
    }
    let p = projonsurf.point(imin)?;
    Ok(Some((*p.value(), p.parameter())))
}

/// The normal at a singular point from the derivatives of the surface up to order 2;
/// the X direction (the default `gp_Dir`) when it is not defined.
///
/// The native code passes the derivatives of the surface where `CSLib::Normal` expects
/// those of its normal (`DerNUV`); kept as is.
fn singular_normal(surf: &dyn Adaptor3dSurface, u: f64, v: f64) -> Result<Direction3, Failure> {
    let mut der: Array2<'static, Vector3> = Array2::with_bounds(0, 2, 0, 2)
        .map_err(|e| Failure::out_of_range(&format!("BRepBlend_Walking::Transition: {e:?}")))?;
    let d2 = surf.d2(u, v)?;
    let values = [
        ((1, 0), d2.d1u),
        ((0, 1), d2.d1v),
        ((2, 0), d2.d2u),
        ((0, 2), d2.d2v),
        ((1, 1), d2.d2uv),
        ((2, 1), surf.dn(u, v, 2, 1)?),
        ((1, 2), surf.dn(u, v, 1, 2)?),
        ((2, 2), surf.dn(u, v, 2, 2)?),
    ];
    for ((i, j), value) in values {
        der.set_value(i, j, value).map_err(|e| {
            Failure::out_of_range(&format!("BRepBlend_Walking::Transition ({i}, {j}): {e:?}"))
        })?;
    }
    let position = ParametricPosition {
        u,
        v,
        u_min: surf.first_u_parameter()?,
        u_max: surf.last_u_parameter()?,
        v_min: surf.first_v_parameter()?,
        v_max: surf.last_v_parameter()?,
    };
    let normal = cslib::normal_at_singularity(2, &der, 1.0e-9, position)?;
    Ok(normal.result.normal.unwrap_or_default())
}

/// `RecadreIfPeriodic(NewU, NewV, OldU, OldV, UPeriod, VPeriod)`: the new parameters
/// moved by periods within half a period of the old ones (a period of 0 is no period).
/// `recadre_if_periodic((7.0, 0.5), (0.5, 0.5), (2.0 * PI, 0.0))`.
pub(super) fn recadre_if_periodic(
    (mut new_u, mut new_v): (f64, f64),
    (old_u, old_v): (f64, f64),
    (u_period, v_period): (f64, f64),
) -> (f64, f64) {
    if u_period > 0.0 {
        let sign = if new_u < old_u { 1.0 } else { -1.0 };
        while (new_u - old_u).abs() > u_period / 2.0 {
            new_u += sign * u_period;
        }
    }
    if v_period > 0.0 {
        let sign = if new_v < old_v { 1.0 } else { -1.0 };
        while (new_v - old_v).abs() > v_period / 2.0 {
            new_v += sign * v_period;
        }
    }
    (new_u, new_v)
}
