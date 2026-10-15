// SPDX-License-Identifier: GPL-2.0-only
//! The plane graph of a profile: one proof edge per Bézier piece, caller vertices as graph
//! vertices and fresh vertices at split points; and the way back from piece walks to loops
//! of caller curves.
use super::admitted::{AdmittedCurve, AdmittedProfile};
use super::bezier::{BezierPiece, use_pieces};
use super::loops::{LoopUse, ProfileLoop};
use super::profile::{ProfileCurveId, ProfileVertexId};
use crate::sketch_region_prism::{ProfileGraphInput, ProofEdge};
use rustycad_tkmath::gp::Plane3;
use std::collections::BTreeMap;

/// The caller curve of a piece, the piece's index along it and the curve's piece count.
#[derive(Clone, Copy)]
struct PieceOf {
    curve: ProfileCurveId,
    index: usize,
    count: usize,
}

/// Proof edges of every caller curve's forward pieces, in caller order.
#[derive(Default)]
pub(crate) struct PieceTable {
    edges: Vec<ProofEdge>,
    pieces: BTreeMap<u64, PieceOf>,
    vertex_ids: BTreeMap<ProfileVertexId, u64>,
    caller_order: BTreeMap<ProfileCurveId, usize>,
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
        let caller_order = profile.curves.iter().enumerate();
        let caller_order = caller_order.map(|(i, c)| (c.id(), i)).collect();
        let mut table = Self {
            vertex_ids,
            next_vertex,
            caller_order,
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
        self.pieces[&piece].curve
    }

    /// The caller vertex of proof vertex `id`, if it is not a split point.
    pub(crate) fn caller_vertex(&self, id: u64) -> Option<ProfileVertexId> {
        let found = self.vertex_ids.iter().find(|(_, proof)| **proof == id);
        found.map(|(caller, _)| *caller)
    }

    /// Caller index of `curve`.
    pub(crate) fn caller_index(&self, curve: ProfileCurveId) -> usize {
        self.caller_order[&curve]
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
        let count = pieces.len();
        for (index, (piece, ends)) in pieces.into_iter().zip(chain.windows(2)).enumerate() {
            let of = PieceOf {
                curve: curve.id(),
                index,
                count,
            };
            self.push(profile, of, piece, [ends[0], ends[1]]);
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

    fn push(&mut self, profile: &AdmittedProfile, of: PieceOf, piece: BezierPiece, ends: [u64; 2]) {
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
        self.pieces.insert(id, of);
    }

    /// The plane graph request of every piece.
    pub(crate) fn graph_input(&self, plane: Plane3, spatial_limit: f64) -> ProfileGraphInput {
        ProfileGraphInput {
            edges: self.edges.clone(),
            plane,
            spatial_limit,
        }
    }

    /// The loop of caller curves traced by a closed walk of `(piece, reversed)` uses. The
    /// pieces of one curve are consecutive in any walk (their split points have degree two),
    /// so each curve use is emitted at the first piece it traverses.
    pub(crate) fn caller_loop(&self, walk: &[(u64, bool)]) -> ProfileLoop {
        let entry = |(piece, reversed): (u64, bool)| {
            let of = self.pieces[&piece];
            let first = if reversed { of.count - 1 } else { 0 };
            (of.index == first).then_some(LoopUse {
                curve: of.curve,
                reversed,
            })
        };
        let uses = walk.iter().copied().filter_map(entry).collect();
        ProfileLoop { uses }.starting_first(|curve| self.caller_index(curve))
    }
}
