// Created on: 1995-03-06
// Created by: Laurent PAINNOT
// Copyright (c) 1995-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of FoundationClasses/TKMath/Poly/Poly_Connect.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::Triangulation;
use rustycad_tkernel::{
    ncollection::{Array1, PackedMap},
    standard::{Failure, Handle},
};
/// One edge from a node to a higher (or equal) node, with up to two incident triangles and
/// their opposite nodes (the native `polyedge`).
#[derive(Clone, Copy)]
struct PolyEdge {
    second_node: i32,
    triangles: [i32; 2],
    opposite_nodes: [i32; 2],
}
/// `Poly_Connect`: node-to-triangle and triangle-to-neighbour adjacency of a triangulation;
/// `Connect::new(&mesh)?`.
#[derive(Default)]
pub struct Connect {
    triangulation: Handle<Triangulation>,
    triangles: Array1<'static, i32>,
    adjacents: Array1<'static, i32>,
    current: i32,
    first: i32,
    node: i32,
    other_node: i32,
    sense: bool,
    more: bool,
    passed: PackedMap<i32>,
}
impl Connect {
    /// Builds the adjacency of a triangulation; `Connect::new(&mesh)?`.
    pub fn new(triangulation: &Handle<Triangulation>) -> Result<Self, Failure> {
        let mut connect = Self::default();
        connect.load(triangulation)?;
        Ok(connect)
    }
    /// Rebuilds the adjacency for another triangulation; `connect.load(&mesh)?`.
    pub fn load(&mut self, triangulation: &Handle<Triangulation>) -> Result<(), Failure> {
        self.triangulation = triangulation.clone();
        (self.current, self.first, self.node, self.other_node) = (0, 0, 0, 0);
        (self.sense, self.more) = (false, false);
        let mesh = self.mesh()?;
        let (nb_nodes, nb_triangles) = (mesh.nb_nodes(), mesh.nb_triangles());
        self.triangles =
            Array1::with_bounds(1, nb_nodes).map_err(|e| e.into_failure("Poly_Connect::Load"))?;
        self.adjacents = Array1::with_bounds(1, 6 * nb_triangles)
            .map_err(|e| e.into_failure("Poly_Connect::Load"))?;
        let edges = self.collect_edges(nb_nodes, nb_triangles)?;
        self.fill_adjacents(&edges, nb_triangles)
    }
    fn mesh(&self) -> Result<Handle<Triangulation>, Failure> {
        if self.triangulation.is_null() {
            return Err(Failure::with_type(
                "Standard_NullObject",
                "Poly_Connect::Load: null triangulation; expected a triangulation handle",
                "",
            ));
        }
        Ok(self.triangulation.clone())
    }
    fn triangle_nodes(&self, triangle: i32) -> Result<[i32; 3], Failure> {
        let (a, b, c) = self.mesh()?.triangle(triangle)?.get();
        Ok([a, b, c])
    }
    fn collect_edges(
        &mut self,
        nb_nodes: i32,
        nb_triangles: i32,
    ) -> Result<Vec<Vec<PolyEdge>>, Failure> {
        let mut edges: Vec<Vec<PolyEdge>> =
            vec![Vec::new(); usize::try_from(nb_nodes).unwrap_or(0)];
        for triangle in 1..=nb_triangles {
            let nodes = self.triangle_nodes(triangle)?;
            for node in nodes {
                self.triangles
                    .set_value(node, triangle)
                    .map_err(|e| e.into_failure("Poly_Connect::Load"))?;
            }
            for side in 0..3 {
                record_edge(&mut edges, nodes, side, triangle)?;
            }
        }
        Ok(edges)
    }
    fn fill_adjacents(
        &mut self,
        edges: &[Vec<PolyEdge>],
        nb_triangles: i32,
    ) -> Result<(), Failure> {
        let mut slot = 1;
        for triangle in 1..=nb_triangles {
            let nodes = self.triangle_nodes(triangle)?;
            for side in 0..3 {
                let (low, high) = ordered_side(nodes, side);
                let edge = edge_list(edges, low)?
                    .iter()
                    .find(|edge| edge.second_node == high)
                    .copied()
                    .ok_or_else(|| Failure::new("Poly_Connect::Load: edge lost between passes"))?;
                let other = usize::from(edge.triangles[0] == triangle);
                self.set_adjacent(slot, edge.triangles[other])?;
                self.set_adjacent(slot + 3, edge.opposite_nodes[other])?;
                slot += 1;
            }
            slot += 3;
        }
        Ok(())
    }
    fn set_adjacent(&mut self, slot: i32, value: i32) -> Result<(), Failure> {
        self.adjacents
            .set_value(slot, value)
            .map_err(|e| e.into_failure("Poly_Connect::Load"))
    }
    /// The triangulation analysed by this tool; `connect.triangulation()`.
    pub const fn triangulation(&self) -> &Handle<Triangulation> {
        &self.triangulation
    }
    /// One triangle containing a node, 0 for a free node; `connect.triangle(1)?`.
    pub fn triangle(&self, node: i32) -> Result<i32, Failure> {
        self.triangles
            .value(node)
            .copied()
            .map_err(|e| e.into_failure("Poly_Connect::Triangle"))
    }
    /// The triangles across the sides (n1,n2), (n2,n3) and (n3,n1), 0 on a boundary;
    /// `connect.triangles(1)?`.
    pub fn triangles(&self, triangle: i32) -> Result<(i32, i32, i32), Failure> {
        self.adjacent_triple(triangle, 1)
    }
    /// The opposite nodes of those neighbouring triangles, 0 on a boundary; `connect.nodes(1)?`.
    pub fn nodes(&self, triangle: i32) -> Result<(i32, i32, i32), Failure> {
        self.adjacent_triple(triangle, 4)
    }
    fn adjacent_triple(&self, triangle: i32, first: i32) -> Result<(i32, i32, i32), Failure> {
        let base = 6 * (triangle - 1);
        let value = |offset: i32| {
            self.adjacents
                .value(base + first + offset)
                .copied()
                .map_err(|e| e.into_failure("Poly_Connect::Triangles"))
        };
        Ok((value(0)?, value(1)?, value(2)?))
    }
    /// Starts the iteration over the triangles around a node; `connect.initialize(3)?`.
    pub fn initialize(&mut self, node: i32) -> Result<(), Failure> {
        self.node = node;
        self.first = self.triangle(node)?;
        self.current = self.first;
        self.sense = true;
        self.more = self.first != 0;
        self.passed.clear();
        self.passed.add(self.current);
        if self.more {
            let nodes = self.triangle_nodes(self.first)?;
            // A node missing from its triangle leaves the search index at 3, as natively.
            let position = nodes.iter().position(|n| *n == node).unwrap_or(3);
            self.other_node = nodes[(position + 2) % 3];
        }
        Ok(())
    }
    /// True while a triangle remains; `connect.more()`.
    pub const fn more(&self) -> bool {
        self.more
    }
    /// The current triangle; `connect.value()`.
    pub const fn value(&self) -> i32 {
        self.current
    }
    /// Turns around the node: first in the direct sense, then, from the first triangle,
    /// in the other sense; a triangle seen before ends the walk; `connect.next()?`.
    // Keeps the native Initialize/More/Next/Value protocol of Poly_Connect.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<(), Failure> {
        let (t1, t2, t3) = self.triangles(self.current)?;
        let mut neighbours = [t1, t2, t3];
        if self.sense {
            if self.step_around(neighbours, 1, 2)? {
                return Ok(());
            }
            // Otherwise leave to the left: restart from the first triangle the other way.
            let nodes = self.triangle_nodes(self.first)?;
            let position = nodes.iter().position(|n| *n == self.node).unwrap_or(3);
            self.other_node = nodes[(position + 1) % 3];
            self.sense = false;
            self.current = self.first;
            let (t1, t2, t3) = self.triangles(self.current)?;
            neighbours = [t1, t2, t3];
        }
        if self.step_around(neighbours, 2, 1)? {
            return Ok(());
        }
        self.more = false;
        Ok(())
    }
    // Moves to the neighbour holding the edge (node, other node) with the other node at
    // offset `other_at` after the node; the new other node sits at offset `next_at`.
    fn step_around(
        &mut self,
        neighbours: [i32; 3],
        other_at: usize,
        next_at: usize,
    ) -> Result<bool, Failure> {
        for neighbour in neighbours.into_iter().filter(|t| *t != 0) {
            let nodes = self.triangle_nodes(neighbour)?;
            for j in 0..3 {
                if nodes[j] == self.node && nodes[(j + other_at) % 3] == self.other_node {
                    self.current = neighbour;
                    self.other_node = nodes[(j + next_at) % 3];
                    self.more = !self.passed.contains(self.current);
                    self.passed.add(self.current);
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}
fn ordered_side(nodes: [i32; 3], side: usize) -> (i32, i32) {
    let (this, next) = (nodes[side], nodes[(side + 1) % 3]);
    if this < next {
        (this, next)
    } else {
        (next, this)
    }
}
fn edge_list(edges: &[Vec<PolyEdge>], node: i32) -> Result<&Vec<PolyEdge>, Failure> {
    usize::try_from(node - 1)
        .ok()
        .and_then(|offset| edges.get(offset))
        .ok_or_else(|| {
            Failure::out_of_range(&format!(
                "Poly_Connect::Load: node {node}; expected [1, {}]",
                edges.len()
            ))
        })
}
fn record_edge(
    edges: &mut [Vec<PolyEdge>],
    nodes: [i32; 3],
    side: usize,
    triangle: i32,
) -> Result<(), Failure> {
    let (low, high) = ordered_side(nodes, side);
    let third = nodes[3 - side - (side + 1) % 3];
    let count = edges.len();
    let list = usize::try_from(low - 1)
        .ok()
        .and_then(|offset| edges.get_mut(offset))
        .ok_or_else(|| {
            Failure::out_of_range(&format!(
                "Poly_Connect::Load: node {low}; expected [1, {count}]"
            ))
        })?;
    if let Some(edge) = list.iter_mut().find(|edge| edge.second_node == high) {
        edge.triangles[1] = triangle;
        edge.opposite_nodes[1] = third;
        return Ok(());
    }
    list.push(PolyEdge {
        second_node: high,
        triangles: [triangle, 0],
        opposite_nodes: [third, 0],
    });
    Ok(())
}
