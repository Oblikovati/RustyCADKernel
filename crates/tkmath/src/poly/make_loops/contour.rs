// Copyright (c) 2009-2014 OPEN CASCADE SAS
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
// Rust translation of findContour, acceptContour and markHangChain of
// FoundationClasses/TKMath/Poly/Poly_MakeLoops.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::{ListOfLink, LoopWay, MakeLoops};
use rustycad_tkernel::standard::Failure;
use std::collections::HashMap;
/// A chain of signed link indices (the native `NCollection_IndexedMap<int>`) and the
/// one-based position where its loop starts; one past the end for an open chain.
pub(super) struct Contour {
    indices: Vec<i32>,
    pub(super) start: usize,
}
impl Contour {
    pub(super) fn len(&self) -> usize {
        self.indices.len()
    }
    pub(super) fn at(&self, position: usize) -> i32 {
        self.indices[position - 1]
    }
    fn position(&self, index: i32) -> Option<usize> {
        self.indices.iter().position(|i| *i == index).map(|p| p + 1)
    }
}
impl<W: LoopWay> MakeLoops<W> {
    /// First node of a signed link, as `getFirstNode`.
    pub(super) fn first_node(&self, segment: i32) -> Result<i32, Failure> {
        let link = self.stored(segment.unsigned_abs() as usize)?;
        Ok(if segment > 0 { link.node1 } else { link.node2 })
    }
    /// Last node of a signed link, as `getLastNode`.
    pub(super) fn last_node(&self, segment: i32) -> Result<i32, Failure> {
        let link = self.stored(segment.unsigned_abs() as usize)?;
        Ok(if segment > 0 { link.node2 } else { link.node1 })
    }
    // A boundary link can be taken once, in the orientation given by the sign.
    fn can_link_be_taken(&self, segment: i32) -> bool {
        self.start_indices.contains(segment)
    }
    /// Follows links from `start` until the chain closes or ends, as `findContour`.
    pub(super) fn find_contour(&self, start: i32) -> Result<Contour, Failure> {
        let mut contour = Contour {
            indices: Vec::new(),
            start: 0,
        };
        let mut node_link: HashMap<i32, i32> = HashMap::new();
        let (mut segment, mut last) = (start, self.last_node(start)?);
        loop {
            if contour.position(segment).is_none() {
                contour.indices.push(segment);
            }
            node_link.insert(self.first_node(segment)?, segment);
            segment = self.next_segment(last, segment)?;
            if let Some(start) = self.stop_position(&mut contour, &node_link, segment, &mut last)? {
                contour.start = start;
                return Ok(contour);
            }
        }
    }
    // The single usable continuation, or the strategy's choice among several; 0 for none.
    fn next_segment(&self, last: i32, segment: i32) -> Result<i32, Failure> {
        let candidates = self.candidates(last, segment.unsigned_abs() as usize)?;
        Ok(match candidates.as_slice() {
            [] => 0,
            [first, ..] if Some(first) == candidates.last() => *first,
            _ => self
                .way
                .choose_left_way(&self.links, last, segment, &candidates),
        })
    }
    // Where the loop starts when `segment` ends the search: an open end, a link met again
    // (in either orientation), or a node met again, which closes the loop.
    fn stop_position(
        &self,
        contour: &mut Contour,
        node_link: &HashMap<i32, i32>,
        segment: i32,
        last: &mut i32,
    ) -> Result<Option<usize>, Failure> {
        if segment == 0 {
            return Ok(Some(contour.len() + 1));
        }
        if let Some(position) = contour.position(segment) {
            return Ok(Some(position));
        }
        if let Some(position) = contour.position(-segment) {
            return Ok(Some(position + 1));
        }
        *last = self.last_node(segment)?;
        let Some(&closing) = node_link.get(last) else {
            return Ok(None);
        };
        contour.indices.push(segment);
        Ok(Some(contour.position(closing).unwrap_or(0)))
    }
    // Links at `node`, other than link `current`, that can continue the chain, signed so
    // that they leave the node.
    fn candidates(&self, node: i32, current: usize) -> Result<Vec<i32>, Failure> {
        let mut result = Vec::new();
        for link in self.way.helper().get_adjacent_links(node).iter() {
            let index = self.links.find_index(link);
            if index == 0 || index == current {
                continue;
            }
            let mut signed = index as i32;
            if self.first_node(signed)? != node {
                signed = -signed;
            }
            if self.can_link_be_taken(signed) {
                result.push(signed);
            }
        }
        Ok(result)
    }
    /// Appends the closed part of a contour as a loop and takes its links out of the
    /// start set, as `acceptContour`.
    pub(super) fn accept_contour(&mut self, contour: &Contour) -> Result<(), Failure> {
        let mut result = ListOfLink::default();
        for position in contour.start..=contour.len() {
            let segment = contour.at(position);
            let mut link = *self.stored(segment.unsigned_abs() as usize)?;
            if segment < 0 {
                link.reverse();
            }
            result.append(link);
            self.start_indices.remove(segment);
        }
        self.loops.push(result);
        Ok(())
    }
    /// Marks the chain of links hanging from `node` (up to a bifurcation) and takes them
    /// out of the start set, as `markHangChain`.
    pub(super) fn mark_hang_chain(&mut self, node: i32, segment: i32) -> Result<(), Failure> {
        let (mut node, mut segment) = (node, segment);
        let outgoing = node == self.first_node(segment)?;
        loop {
            if self.has_other_way(node, segment, outgoing)? {
                return Ok(());
            }
            self.start_indices.remove(segment);
            self.hang_indices.add(segment);
            node = if outgoing {
                self.last_node(segment)?
            } else {
                self.first_node(segment)?
            };
            match self.single_next(node, segment, outgoing)? {
                Some(next) => segment = next,
                None => return Ok(()),
            }
        }
    }
    // True when another usable link at `node` keeps the chain attached there.
    fn has_other_way(&self, node: i32, segment: i32, outgoing: bool) -> Result<bool, Failure> {
        let current = segment.unsigned_abs() as usize;
        for link in self.way.helper().get_adjacent_links(node).iter() {
            let index = self.links.find_index(link);
            if index == 0 || index == current {
                continue;
            }
            let mut signed = index as i32;
            if (outgoing && node == link.node1) || (!outgoing && node == link.node2) {
                signed = -signed;
            }
            if self.can_link_be_taken(signed) {
                return Ok(true);
            }
        }
        Ok(false)
    }
    // The only usable continuation at `node`, or None for zero or several.
    fn single_next(&self, node: i32, segment: i32, outgoing: bool) -> Result<Option<i32>, Failure> {
        let current = segment.unsigned_abs() as usize;
        let mut next = None;
        for link in self.way.helper().get_adjacent_links(node).iter() {
            let index = self.links.find_index(link);
            if index == 0 || index == current {
                continue;
            }
            let mut signed = index as i32;
            if (outgoing && node == link.node2) || (!outgoing && node == link.node1) {
                signed = -signed;
            }
            if self.can_link_be_taken(signed) {
                if next.is_some() {
                    return Ok(None);
                }
                next = Some(signed);
            }
        }
        Ok(next)
    }
}
