// Rust translation of CheckCoupleAndGetAngle, CheckCoupleAndGetAngle2,
// CheckNextStartPoint and GetNextChainStartPoint of
// ModelingAlgorithms/TKGeomAlgo/IntPolyh/IntPolyh_MaillageAffinage.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! One step of the chaining of start points: the next couple across the edge the point
//! lies on, its other intersection point, and where a point is stored (a section line or
//! the tangent zones).
use super::{IntPolyhMaillageAffinage, MY_CONFUSION_PRECISION as MCP};
use crate::int_polyh::array::IntPolyhArrayOfTangentZones;
use crate::int_polyh::edge::IntPolyhCouple;
use crate::int_polyh::section_line::IntPolyhSectionLine;
use crate::int_polyh::start_point::IntPolyhStartPoint;
use rustycad_tkernel::standard::Failure;

/// Where the chain stores its points: the section line and the tangent zones.
pub(crate) struct ChainStore<'s> {
    pub(crate) line: &'s mut IntPolyhSectionLine,
    pub(crate) zones: &'s mut IntPolyhArrayOfTangentZones,
}

/// `CheckCoupleAndGetAngle(T1, T2, Angle, TTrianglesContacts)`: marks the first couple
/// `(t1, t2)` not analyzed yet and returns its angle.
pub(crate) fn check_couple_and_get_angle(
    t1: i32,
    t2: i32,
    couples: &mut [IntPolyhCouple],
) -> Option<f64> {
    let couple = couples
        .iter_mut()
        .find(|c| !c.is_analyzed() && c.first_value() == t1 && c.second_value() == t2)?;
    couple.set_analyzed(true);
    Some(couple.angle())
}

/// The result of `CheckCoupleAndGetAngle2`: whether the couple `(T1, T2)` was found, the
/// positions of the couples `(T1, T22)` and `(T11, T2)`, and the angle of the last couple
/// read.
struct Couple2 {
    found: bool,
    it_ct11: Option<usize>,
    it_ct22: Option<usize>,
    angle: f64,
}

/// `CheckCoupleAndGetAngle2(T1, T2, T11, T22, theItCT11, theItCT22, Angle, ...)`: the
/// couple `(t1, t2)` (marked) and the couples `(t1, t22)` and `(t11, t2)` of an edge-edge
/// contact, among the couples not analyzed yet; the angle is the one of the last couple
/// met, as natively.
fn check_couple_and_get_angle2(
    (t1, t2, t11, t22): (i32, i32, i32, i32),
    couples: &mut [IntPolyhCouple],
    angle: f64,
) -> Couple2 {
    let mut r = Couple2 {
        found: false,
        it_ct11: None,
        it_ct22: None,
        angle,
    };
    for (pos, couple) in couples.iter_mut().enumerate() {
        if couple.is_analyzed() {
            continue;
        }
        if couple.first_value() == t1 {
            if couple.second_value() == t2 {
                r.found = true;
                couple.set_analyzed(true);
                r.angle = couple.angle();
            } else if couple.second_value() == t22 {
                r.it_ct11 = Some(pos);
                r.angle = couple.angle();
            }
        } else if couple.first_value() == t11 && couple.second_value() == t2 {
            r.it_ct22 = Some(pos);
            r.angle = couple.angle();
        }
        if r.found && r.it_ct11.is_some() && r.it_ct22.is_some() {
            break;
        }
    }
    r
}

/// Whether two points have the same parameters on both surfaces.
fn same_parameters(a: &IntPolyhStartPoint, b: &IntPolyhStartPoint) -> bool {
    (a.u1() - b.u1()).abs() < MCP
        && (a.v1() - b.v1()).abs() < MCP
        && (a.u2() - b.u2()).abs() < MCP
        && (a.v2() - b.v2()).abs() < MCP
}

/// `CheckNextStartPoint(SectionLine, TTangentZones, SP, Prepend)`: a vertex (an edge -1)
/// goes to the tangent zones unless already there (list number -1) and ends the chain
/// (0); another point goes to the section line (after it, or before it when `prepend`)
/// and the chain goes on (1).
pub(crate) fn check_next_start_point(
    store: &mut ChainStore<'_>,
    sp: &mut IntPolyhStartPoint,
    prepend: bool,
) -> Result<i32, Failure> {
    if sp.e1() == -1 || sp.e2() == -1 {
        // The vertices of the triangles: is it already a tangent zone?
        let fin_ttz: i32 = store.zones.nb_items();
        let known: bool = (0..fin_ttz).any(|i| same_parameters(sp, &store.zones[i]));
        if !known {
            sp.set_chain_list(-1);
            store.zones[fin_ttz] = *sp;
            store.zones.increment_nb_items();
        }
        return Ok(0);
    }
    if prepend {
        store.line.prepend(sp);
    } else {
        let last: i32 = store.line.nb_start_points();
        *store.line.change_value(last)? = *sp;
        store.line.increment_nb_start_points();
    }
    Ok(1)
}

impl IntPolyhMaillageAffinage {
    /// `GetNextChainStartPoint(SP, SPNext, MySectionLine, TTangentZones, Prepend)`: the
    /// next point of the chain from `sp` across the edge it lies on (1, with the angle of
    /// the couple), 0 at the end; several points are stored at once and end the chain.
    /// `m.get_next_chain_start_point(&sp, &mut next, &mut line, &mut zones, false)?`.
    pub fn get_next_chain_start_point(
        &mut self,
        sp: &IntPolyhStartPoint,
        sp_next: &mut IntPolyhStartPoint,
        line: &mut IntPolyhSectionLine,
        zones: &mut IntPolyhArrayOfTangentZones,
        prepend: bool,
    ) -> Result<i32, Failure> {
        let mut couples: Vec<IntPolyhCouple> = self.t_triangles_contacts.iter().copied().collect();
        let mut store = ChainStore { line, zones };
        let result = self.next_chain_point(sp, sp_next, (&mut couples, &mut store), prepend);
        self.put_back_couples(&couples);
        result
    }

    /// Writes the analyzed flags of `couples` back into the list of contacts.
    pub(crate) fn put_back_couples(&mut self, couples: &[IntPolyhCouple]) {
        for (slot, couple) in self.t_triangles_contacts.iter_mut().zip(couples) {
            *slot = *couple;
        }
    }

    /// The body of `GetNextChainStartPoint` over the couples taken out of the list.
    pub(crate) fn next_chain_point(
        &self,
        sp: &IntPolyhStartPoint,
        sp_next: &mut IntPolyhStartPoint,
        (couples, store): (&mut [IntPolyhCouple], &mut ChainStore<'_>),
        prepend: bool,
    ) -> Result<i32, Failure> {
        let (e1, e2) = (sp.e1(), sp.e2());
        if e1 >= 0 && e2 == -2 {
            // The point is on an edge of T1.
            let next1: i32 = self.across_edge(1, e1, sp.t1());
            return self.next_on_one_edge((next1, sp.t2()), sp, sp_next, (couples, store), prepend);
        }
        if e1 == -2 && e2 >= 0 {
            // The point is on an edge of T2.
            let next2: i32 = self.across_edge(2, e2, sp.t2());
            return self.next_on_one_edge((sp.t1(), next2), sp, sp_next, (couples, store), prepend);
        }
        if e1 >= 0 && e2 >= 0 {
            // The point is on two edges.
            return self.next_on_two_edges(sp, sp_next, (couples, store), prepend);
        }
        // No edge is touched or cut, or the point is a vertex of a triangle.
        Ok(0)
    }

    fn next_on_one_edge(
        &self,
        (t1, t2): (i32, i32),
        sp: &IntPolyhStartPoint,
        sp_next: &mut IntPolyhStartPoint,
        (couples, store): (&mut [IntPolyhCouple], &mut ChainStore<'_>),
        prepend: bool,
    ) -> Result<i32, Failure> {
        let Some(angle) = check_couple_and_get_angle(t1, t2, couples) else {
            // This couple does not intersect.
            return Ok(0);
        };
        let nb_points: i32 = self.next_starting_points_research(t1, t2, sp, sp_next)?;
        if nb_points == 1 {
            sp_next.set_angle(angle);
            return Ok(1);
        }
        if nb_points > 1 {
            check_next_start_point(store, sp_next, prepend)?;
            return Ok(nb_points);
        }
        Ok(0)
    }

    fn next_on_two_edges(
        &self,
        sp: &IntPolyhStartPoint,
        sp_next: &mut IntPolyhStartPoint,
        (couples, store): (&mut [IntPolyhCouple], &mut ChainStore<'_>),
        prepend: bool,
    ) -> Result<i32, Failure> {
        let next1: i32 = self.across_edge(1, sp.e1(), sp.t1());
        let next2: i32 = self.across_edge(2, sp.e2(), sp.t2());
        let r: Couple2 =
            check_couple_and_get_angle2((next1, next2, sp.t1(), sp.t2()), couples, -2.0);
        if !r.found {
            return Ok(0);
        }
        let nb_points: i32 = self.next_starting_points_research(next1, next2, sp, sp_next)?;
        if nb_points != 1 {
            if nb_points > 1 {
                // The new point is checked.
                check_next_start_point(store, sp_next, prepend)?;
            }
            return Ok(0);
        }
        sp_next.set_angle(r.angle);
        // The couples (T1, T22) and (T11, T2) of the edge-edge contact are done too.
        for pos in [r.it_ct11, r.it_ct22].into_iter().flatten() {
            couples[pos].set_analyzed(true);
        }
        Ok(1)
    }
}
