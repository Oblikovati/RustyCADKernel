// Rust translation of StartPointsChain of
// ModelingAlgorithms/TKGeomAlgo/IntPolyh/IntPolyh_MaillageAffinage.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The chaining of the start points of the couples into section lines: from the points
//! of each couple not analyzed yet, the chain follows the neighbour couples in both
//! directions (appending one way, prepending the other, eap).
use super::IntPolyhMaillageAffinage;
use super::chain_next::{ChainStore, check_couple_and_get_angle, check_next_start_point};
use crate::int_polyh::array::{IntPolyhArrayOfSectionLines, IntPolyhArrayOfTangentZones};
use crate::int_polyh::edge::IntPolyhCouple;
use crate::int_polyh::start_point::IntPolyhStartPoint;
use rustycad_tkernel::standard::Failure;

/// The state of the chaining of one couple: the section line index and the couples.
struct Chain<'c, 's> {
    index: i32,
    couples: &'c mut [IntPolyhCouple],
    store: ChainStore<'s>,
}

impl IntPolyhMaillageAffinage {
    /// `StartPointsChain(TSectionLines, TTangentZones)`: the start points of every couple
    /// not analyzed yet, chained into a section line (a new one unless the last is
    /// empty) or put in the tangent zones; returns 1.
    /// `m.start_points_chain(&mut lines, &mut zones)?`.
    pub fn start_points_chain(
        &mut self,
        section_lines: &mut IntPolyhArrayOfSectionLines,
        tangent_zones: &mut IntPolyhArrayOfTangentZones,
    ) -> Result<i32, Failure> {
        let mut couples: Vec<IntPolyhCouple> = self.t_triangles_contacts.iter().copied().collect();
        let result = self.chain_couples(&mut couples, section_lines, tangent_zones);
        self.put_back_couples(&couples);
        result.map(|()| 1)
    }

    fn chain_couples(
        &self,
        couples: &mut [IntPolyhCouple],
        section_lines: &mut IntPolyhArrayOfSectionLines,
        tangent_zones: &mut IntPolyhArrayOfTangentZones,
    ) -> Result<(), Failure> {
        for k in 0..couples.len() {
            if couples[k].is_analyzed() {
                continue;
            }
            let mut index: i32 = section_lines.nb_items();
            // Fill the last section line if it is still empty (eap).
            if index > 0 && section_lines[index - 1].nb_start_points() == 0 {
                index -= 1;
            } else {
                section_lines.increment_nb_items();
            }
            let line = &mut section_lines[index];
            if line.get_n() == 0 {
                line.init(10000);
            }
            let mut chain = Chain {
                index,
                couples: &mut *couples,
                store: ChainStore {
                    line,
                    zones: &mut *tangent_zones,
                },
            };
            self.chain_couple(&mut chain, k)?;
        }
        Ok(())
    }

    /// The start points of the couple `k` and their chains.
    fn chain_couple(&self, chain: &mut Chain<'_, '_>, k: usize) -> Result<(), Failure> {
        let (t1i, t2i) = (
            chain.couples[k].first_value(),
            chain.couples[k].second_value(),
        );
        let (mut sp1, mut sp2) = (IntPolyhStartPoint::default(), IntPolyhStartPoint::default());
        let nb_points: i32 = self.starting_points_research(t1i, t2i, &mut sp1, &mut sp2)?;
        chain.couples[k].set_analyzed(true);
        let angle: f64 = chain.couples[k].angle();
        if nb_points == 1 {
            // A vertex on a triangle or an edge-edge contact.
            sp1.set_chain_list(chain.index);
            sp1.set_angle(angle);
            if check_next_start_point(&mut chain.store, &mut sp1, false)? != 0 {
                let mut sp_next = IntPolyhStartPoint::default();
                self.chain_side_1(chain, (t1i, t2i), &sp1, &mut sp_next)?;
                self.chain_side_2(chain, (t1i, t2i), &sp1, &mut sp_next)?;
            }
        } else if nb_points == 2 {
            let mut sp_next = IntPolyhStartPoint::default();
            for (mut sp, prepend) in [(sp1, false), (sp2, true)] {
                sp.set_chain_list(chain.index);
                sp.set_angle(angle);
                if check_next_start_point(&mut chain.store, &mut sp, prepend)? != 0 {
                    self.follow(chain, sp, &mut sp_next, prepend)?;
                }
            }
        } else if nb_points > 2 && nb_points < 7 {
            // More than two points: the first one is stored.
            sp1.set_chain_list(chain.index);
            check_next_start_point(&mut chain.store, &mut sp1, false)?;
        }
        Ok(())
    }

    /// Follows the chain from `sp` while the next point is new and goes on.
    fn follow(
        &self,
        chain: &mut Chain<'_, '_>,
        mut sp: IntPolyhStartPoint,
        sp_next: &mut IntPolyhStartPoint,
        prepend: bool,
    ) -> Result<(), Failure> {
        loop {
            let test: i32 = self.next_chain_point(
                &sp,
                sp_next,
                (&mut *chain.couples, &mut chain.store),
                prepend,
            )?;
            if test != 1 {
                // There is no next point.
                return Ok(());
            }
            sp_next.set_chain_list(chain.index);
            if check_next_start_point(&mut chain.store, sp_next, prepend)? == 0 {
                return Ok(());
            }
            sp = *sp_next;
        }
    }

    /// The chain across the edge of T1 the single point lies on: the next couple's other
    /// point, then its chain; several points go to the tangent zones.
    fn chain_side_1(
        &self,
        chain: &mut Chain<'_, '_>,
        (t1i, t2i): (i32, i32),
        sp1: &IntPolyhStartPoint,
        sp_next: &mut IntPolyhStartPoint,
    ) -> Result<(), Failure> {
        if sp1.e1() < 0 {
            return Ok(());
        }
        let next1: i32 = self.across_edge(1, sp1.e1(), t1i);
        let Some(angle) = check_couple_and_get_angle(next1, t2i, chain.couples) else {
            return Ok(());
        };
        let mut sp11 = IntPolyhStartPoint::default();
        let nb_points11: i32 = self.next_starting_points_research(next1, t2i, sp1, &mut sp11)?;
        if nb_points11 == 1 {
            sp11.set_chain_list(chain.index);
            sp11.set_angle(angle);
            if check_next_start_point(&mut chain.store, &mut sp11, false)? != 0 {
                self.follow(chain, sp11, sp_next, false)?;
            }
        } else if nb_points11 > 1 {
            // The point goes to the tangent zones (list number -1 by default).
            let zones = &mut *chain.store.zones;
            let fin: i32 = zones.nb_items();
            zones[fin] = sp11;
            zones.increment_nb_items();
        }
        Ok(())
    }

    /// The chain across the edge of T2, prepended. As natively, several points of the
    /// next couple are not stored (the test sits in the `else` of the insertion of a
    /// single point).
    fn chain_side_2(
        &self,
        chain: &mut Chain<'_, '_>,
        (t1i, t2i): (i32, i32),
        sp1: &IntPolyhStartPoint,
        sp_next: &mut IntPolyhStartPoint,
    ) -> Result<(), Failure> {
        if sp1.e2() < 0 {
            return Ok(());
        }
        let next2: i32 = self.across_edge(2, sp1.e2(), t2i);
        let Some(angle) = check_couple_and_get_angle(t1i, next2, chain.couples) else {
            return Ok(());
        };
        let mut sp12 = IntPolyhStartPoint::default();
        let nb_points12: i32 = self.next_starting_points_research(t1i, next2, sp1, &mut sp12)?;
        if nb_points12 != 1 {
            return Ok(());
        }
        sp12.set_chain_list(chain.index);
        sp12.set_angle(angle);
        if check_next_start_point(&mut chain.store, &mut sp12, true)? != 0 {
            self.follow(chain, sp12, sp_next, true)?;
        }
        Ok(())
    }

    /// The other triangle of the edge `edge` of the surface `surf_id` than `triangle`.
    pub(crate) fn across_edge(&self, surf_id: i32, edge: i32, triangle: i32) -> i32 {
        let e = &self.sides[Self::rank(surf_id)].edges[edge];
        if e.first_triangle() != triangle {
            e.first_triangle()
        } else {
            e.second_triangle()
        }
    }
}
