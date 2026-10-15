// SPDX-License-Identifier: GPL-2.0-only
//! The proof request of a profile: proof edges (one per Bézier piece), vertex ids, loops of
//! piece uses and one seeded region per loop.
use super::admitted::{AdmittedCurve, AdmittedProfile};
use super::bezier::{BezierPiece, use_pieces};
use super::loops::{LoopUse, ProfileLoop};
use super::profile::{ProfileCurveId, ProfileVertexId};
use crate::sketch_region_prism::{ProfileProofInput, ProofEdge};
use rustycad_tkmath::gp::Plane3;
use std::collections::BTreeMap;

/// Proof edge ids of each caller curve's forward pieces, in caller order.
#[derive(Default)]
pub(crate) struct PieceTable {
    edges: Vec<ProofEdge>,
    curve_pieces: BTreeMap<ProfileCurveId, Vec<u64>>,
    piece_curve: BTreeMap<u64, ProfileCurveId>,
    vertex_ids: BTreeMap<ProfileVertexId, u64>,
    next_vertex: u64,
}

impl PieceTable {
    /// Caller vertices keep a dense id in caller id order; split points follow.
    pub(crate) fn new(profile: &AdmittedProfile) -> Self {
        let vertex_ids: BTreeMap<_, _> = profile
            .vertices
            .keys()
            .zip(1..)
            .map(|(id, n)| (*id, n))
            .collect();
        let next_vertex = vertex_ids.len() as u64 + 1;
        let mut table = Self {
            vertex_ids,
            next_vertex,
            ..Self::default()
        };
        profile
            .curves
            .iter()
            .for_each(|curve| table.add_curve(profile, curve));
        table
    }

    /// The caller curve of proof edge `piece`.
    pub(crate) fn curve_of(&self, piece: u64) -> ProfileCurveId {
        self.piece_curve[&piece]
    }

    fn add_curve(&mut self, profile: &AdmittedProfile, curve: &AdmittedCurve) {
        let forward = LoopUse {
            curve: curve.id(),
            reversed: false,
        };
        let pieces = use_pieces(profile, forward);
        let ends = curve
            .ends()
            .map(|ends| ends.map(|vertex| self.vertex_ids[&vertex]));
        let chain = self.vertex_chain(ends, pieces.len());
        for (piece, ends) in pieces.into_iter().zip(chain.windows(2)) {
            self.push(profile, curve.id(), piece, [ends[0], ends[1]]);
        }
    }

    /// Vertex ids along `count` pieces; a closed curve starts and ends at one new id.
    fn vertex_chain(&mut self, ends: Option<[u64; 2]>, count: usize) -> Vec<u64> {
        let [start, end] = ends.unwrap_or_else(|| [self.fresh(); 2]);
        let mut chain = vec![start];
        chain.extend((1..count).map(|_| self.fresh()));
        chain.push(end);
        chain
    }

    fn fresh(&mut self) -> u64 {
        self.next_vertex += 1;
        self.next_vertex - 1
    }

    fn push(
        &mut self,
        profile: &AdmittedProfile,
        curve: ProfileCurveId,
        piece: BezierPiece,
        ends: [u64; 2],
    ) {
        let id = self.edges.len() as u64 + 1;
        let world = |point: &[f64; 2]| profile.frame.world(*point);
        let world_poles = piece.controls.iter().map(world).collect();
        let weights = piece.weights;
        self.edges.push(ProofEdge {
            id,
            world_poles,
            weights,
            ends,
        });
        self.curve_pieces.entry(curve).or_default().push(id);
        self.piece_curve.insert(id, curve);
    }

    /// Proof uses of one directed caller use: its pieces, reversed in order and sense.
    fn uses(&self, used: LoopUse) -> Vec<(u64, bool)> {
        let pieces = &self.curve_pieces[&used.curve];
        let mut uses: Vec<_> = pieces.iter().map(|piece| (*piece, used.reversed)).collect();
        if used.reversed {
            uses.reverse();
        }
        uses
    }

    /// The complete request: loops named `1..=n`, their seeded regions `n+1..=2n`.
    pub(crate) fn request(
        &self,
        loops: &[ProfileLoop],
        seeds: &[[f64; 2]],
        plane: Plane3,
        limit: f64,
    ) -> ProfileProofInput {
        let named = loops
            .iter()
            .zip(1..)
            .map(|(closed, name)| (name, self.loop_uses(closed)));
        let regions = seeds.iter().enumerate();
        let regions = regions.map(|(index, seed)| (region_name(loops.len(), index), *seed));
        let (edges, loops, regions) = (self.edges.clone(), named.collect(), regions.collect());
        let spatial_limit = limit;
        ProfileProofInput {
            edges,
            loops,
            regions,
            plane,
            spatial_limit,
        }
    }

    fn loop_uses(&self, closed: &ProfileLoop) -> Vec<(u64, bool)> {
        closed
            .uses
            .iter()
            .flat_map(|used| self.uses(*used))
            .collect()
    }
}

/// Region names follow the loop names: the proof requires distinct nonzero names.
pub(crate) fn region_name(loop_count: usize, index: usize) -> u64 {
    (loop_count + index + 1) as u64
}
