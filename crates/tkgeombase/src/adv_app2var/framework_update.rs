// Copyright (c) 1996-1999 Matra Datavision
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

//! The cutting of the constraint frame: `AdvApp2Var_Framework::UpdateInU` and
//! `UpdateInV`.
use super::framework::{AdvApp2VarFramework, AdvApp2VarIsoStrip};
use super::iso::AdvApp2VarIso;
use super::node::AdvApp2VarNode;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Xy;

fn out_of_range(member: &str, cut: f64) -> Failure {
    Failure::out_of_range(&format!(
        "AdvApp2Var_Framework::{member}: no strip contains the cut {cut}"
    ))
}

/// A fresh, not approximated, inner iso copying the type and orders of `model`.
fn inner_iso(model: &AdvApp2VarIso, constant: f64, domain: [f64; 4]) -> Handle<AdvApp2VarIso> {
    let iso = AdvApp2VarIso::with_domain(
        model.iso_type(),
        constant,
        domain,
        0,
        model.u_order(),
        model.v_order(),
    );
    iso.reset_approx();
    Handle::new(iso)
}

impl AdvApp2VarFramework {
    /// Cuts the frame at `U = cutting_value` (`UpdateInU`): the U strip that holds
    /// the cut is split, a U-iso is inserted in each V strip, and nodes are added
    /// where the cut crosses a row. Standard_OutOfRange when no strip holds the cut.
    /// `frame.update_in_u(0.5)?`.
    pub fn update_in_u(&mut self, cutting_value: f64) -> Result<(), Failure> {
        let (u_strips, v_strips) = self.strips_mut();
        let index: usize = u_strips
            .iter()
            .position(|strip| {
                strip
                    .first()
                    .is_some_and(|iso| iso.u0() <= cutting_value && iso.u1() >= cutting_value)
            })
            .map_or(u_strips.len() + 1, |offset| offset + 1);
        let strip: &AdvApp2VarIsoStrip = u_strips
            .value(index)
            .map_err(|_| out_of_range("UpdateInU", cutting_value))?;
        let first = strip
            .first()
            .ok_or_else(|| out_of_range("UpdateInU", cutting_value))?;
        let (u_first, u_last) = (first.u0(), first.u1());
        for iso in strip.iter() {
            iso.change_domain(u_first, cutting_value);
            iso.reset_approx();
        }
        let new_strip: AdvApp2VarIsoStrip = strip
            .iter()
            .map(|iso| {
                inner_iso(
                    iso,
                    iso.constante(),
                    [cutting_value, u_last, iso.v0(), iso.v1()],
                )
            })
            .collect();
        u_strips.insert_after(index, new_strip)?;
        for strip in v_strips.iter_mut() {
            let iso = strip.value(index)?.clone();
            iso.change_domain_uv(iso.u0(), cutting_value, iso.v0(), iso.v1());
            let new_iso = inner_iso(
                &iso,
                cutting_value,
                [iso.u0(), cutting_value, iso.v0(), iso.v1()],
            );
            strip.insert_after(index, new_iso)?;
            let next = strip.value(index + 2)?;
            next.change_domain_uv(cutting_value, next.u1(), next.v0(), next.v1());
        }
        insert_row_nodes(self.nodes_mut(), cutting_value)
    }

    /// Cuts the frame at `V = cutting_value` (`UpdateInV`): the V strip that holds
    /// the cut is split, a V-iso is inserted in each U strip, and a row of nodes is
    /// added. Standard_OutOfRange when no strip holds the cut.
    /// `frame.update_in_v(0.5)?`.
    pub fn update_in_v(&mut self, cutting_value: f64) -> Result<(), Failure> {
        let (u_strips, v_strips) = self.strips_mut();
        let mut index: usize = 1;
        loop {
            let strip = v_strips
                .value(index)
                .map_err(|_| out_of_range("UpdateInV", cutting_value))?;
            let first = strip
                .first()
                .ok_or_else(|| out_of_range("UpdateInV", cutting_value))?;
            if !(first.v0() > cutting_value || first.v1() < cutting_value) {
                break;
            }
            index += 1;
        }
        let strip: &AdvApp2VarIsoStrip = v_strips.value(index)?;
        let first = strip
            .first()
            .ok_or_else(|| out_of_range("UpdateInV", cutting_value))?;
        let (v_first, v_last) = (first.v0(), first.v1());
        for iso in strip.iter() {
            iso.change_domain(v_first, cutting_value);
            iso.reset_approx();
        }
        let new_strip: AdvApp2VarIsoStrip = strip
            .iter()
            .map(|iso| {
                inner_iso(
                    iso,
                    iso.constante(),
                    [iso.u0(), iso.u1(), cutting_value, v_last],
                )
            })
            .collect();
        v_strips.insert_after(index, new_strip)?;
        for strip in u_strips.iter_mut() {
            let iso = strip.value(index)?.clone();
            iso.change_domain_uv(iso.u0(), iso.u1(), iso.v0(), cutting_value);
            let new_iso = inner_iso(
                &iso,
                cutting_value,
                [iso.u0(), iso.u1(), iso.v0(), cutting_value],
            );
            strip.insert_after(index, new_iso)?;
            let next = strip.value(index + 2)?;
            next.change_domain_uv(next.u0(), next.u1(), cutting_value, next.v1());
        }
        let nb_iso_in_v: usize = u_strips.len() + 1;
        insert_column_nodes(self.nodes_mut(), cutting_value, nb_iso_in_v)
    }
}

/// The node loop of `UpdateInU`: a node `(cut, V)` between two consecutive nodes of
/// one row that straddle the cut; the loop bound follows the growing sequence.
fn insert_row_nodes(
    nodes: &mut rustycad_tkernel::ncollection::Sequence<Handle<AdvApp2VarNode>>,
    cut: f64,
) -> Result<(), Failure> {
    let mut previous: Handle<AdvApp2VarNode> = nodes
        .first()
        .cloned()
        .ok_or_else(|| out_of_range("UpdateInU", cut))?;
    let mut index: usize = 1;
    while index < nodes.len() {
        let next: Handle<AdvApp2VarNode> = nodes.value(index + 1)?.clone();
        let (p, n) = (previous.coord(), next.coord());
        if p.x() < cut && n.x() > cut && p.y() == n.y() {
            let node = AdvApp2VarNode::with_coord(
                Xy::new(cut, p.y()),
                previous.u_order(),
                previous.v_order(),
            )?;
            nodes.insert_after(index, Handle::new(node))?;
        }
        previous = next;
        index += 1;
    }
    Ok(())
}

/// The node loop of `UpdateInV`: after the rows below the cut, a row of nodes at
/// `V = cut` with the U coordinates and orders of the first row.
fn insert_column_nodes(
    nodes: &mut rustycad_tkernel::ncollection::Sequence<Handle<AdvApp2VarNode>>,
    cut: f64,
    nb_iso_in_v: usize,
) -> Result<(), Failure> {
    let mut start: usize = 1;
    while start <= nodes.len() && nodes.value(start)?.coord().y() < cut {
        start += nb_iso_in_v;
    }
    for iso_index in 1..=nb_iso_in_v {
        let model: Handle<AdvApp2VarNode> = nodes.value(iso_index)?.clone();
        let node = AdvApp2VarNode::with_coord(
            Xy::new(model.coord().x(), cut),
            model.u_order(),
            model.v_order(),
        )?;
        nodes.insert_after(start + iso_index - 2, Handle::new(node))?;
    }
    Ok(())
}
