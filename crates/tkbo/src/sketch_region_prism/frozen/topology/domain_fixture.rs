// SPDX-License-Identifier: GPL-2.0-only
//! Authored annular cell words, not certified production face domains.
use super::super::*;
use super::fixtures::NamedCellComplex;

pub(super) fn annular_complex() -> FrozenPrismProjection {
    let complex = NamedCellComplex {
        edges: annular_edges(),
        faces: Vec::new(),
    };
    complex.domain_projection(&[
        vec![vec![1], vec![-4]],
        vec![vec![-2], vec![5]],
        vec![vec![-1, 3, 2, -3]],
        vec![vec![4, 6, -5, -6]],
    ])
}

pub(super) fn cut_annular_complex() -> FrozenPrismProjection {
    let mut edges = annular_edges();
    edges.extend([(0, 2), (1, 3)]);
    NamedCellComplex {
        edges,
        faces: vec![
            vec![1, 7, -4, -7],
            vec![-2, 8, 5, -8],
            vec![-1, 3, 2, -3],
            vec![4, 6, -5, -6],
        ],
    }
    .projection()
}

pub(super) fn authored_face(source: &FrozenPrismProjection, identity: u64) -> &FrozenPrismUse {
    source
        .uses
        .iter()
        .find(|row| {
            row.key.0.len() == 3 && row.key.0[2].0.source == AuthoredPrismId(900 + identity)
        })
        .expect("authored fixture face identity; expected one declared face")
}

fn annular_edges() -> Vec<(u64, u64)> {
    // Endpoint roles are authored in the finite model, not discovered by rank or proximity.
    vec![(0, 0), (1, 1), (0, 1), (2, 2), (3, 3), (2, 3)]
}
