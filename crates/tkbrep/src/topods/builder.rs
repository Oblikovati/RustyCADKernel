// Created on: 1991-02-21
// Created by: Remi Lequette
// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of ModelingData/TKBRep/TopoDS/TopoDS_Builder.hxx, TopoDS_Builder.lxx
// and TopoDS_Builder.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopoDS_Builder`: makes the shapes without geometry and adds or removes
//! sub-shapes (docs/topology-design.md section 5).
use super::errors::{frozen_shape, uncompatible_shapes};
use super::tshape::TShape;
use super::{CompSolid, Compound, Shape, Shell, Solid, TCompSolid, TCompound, TShell, TSolid};
use super::{TWire, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::toploc::Location;

/// `static const unsigned int aTb[9]`: for each component type, the bit set of the
/// parent types that may hold it (indexed by `TopAbs_ShapeEnum`).
const ALLOWED_PARENTS: [u32; 9] = [
    // COMPOUND to:
    1 << ShapeEnum::Compound as u32,
    // COMPSOLID to:
    1 << ShapeEnum::Compound as u32,
    // SOLID to:
    (1 << ShapeEnum::Compound as u32) | (1 << ShapeEnum::CompSolid as u32),
    // SHELL to:
    (1 << ShapeEnum::Compound as u32) | (1 << ShapeEnum::Solid as u32),
    // FACE to:
    (1 << ShapeEnum::Compound as u32) | (1 << ShapeEnum::Shell as u32),
    // WIRE to:
    (1 << ShapeEnum::Compound as u32) | (1 << ShapeEnum::Face as u32),
    // EDGE to:
    (1 << ShapeEnum::Compound as u32)
        | (1 << ShapeEnum::Solid as u32)
        | (1 << ShapeEnum::Wire as u32),
    // VERTEX to:
    (1 << ShapeEnum::Compound as u32)
        | (1 << ShapeEnum::Solid as u32)
        | (1 << ShapeEnum::Face as u32)
        | (1 << ShapeEnum::Edge as u32),
    // SHAPE to:
    0,
];

/// The root builder of topological data structures. It makes the empty wires,
/// shells, solids, composite solids and compounds; the shapes with geometry are made
/// by the builders of the BRep packages.
///
/// ```
/// use rustycad_tkbrep::topods::{Builder, Compound, Wire};
/// let (mut compound, mut wire) = (Compound::default(), Wire::default());
/// Builder.make_compound(&mut compound);
/// Builder.make_wire(&mut wire);
/// Builder.add(&mut compound, &wire).unwrap();
/// assert_eq!(compound.nb_children(), 1);
/// assert!(!wire.free().unwrap());
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct Builder;

impl Builder {
    /// `MakeWire(W)`: an empty wire; `Builder.make_wire(&mut wire)`.
    pub fn make_wire(&self, wire: &mut Wire) {
        self.make_shape(wire, &Handle::new(TWire::new()).into_cast());
    }

    /// `MakeShell(S)`: an empty shell; `Builder.make_shell(&mut shell)`.
    pub fn make_shell(&self, shell: &mut Shell) {
        self.make_shape(shell, &Handle::new(TShell::new()).into_cast());
    }

    /// `MakeSolid(S)`: a solid covering the whole space (no shell); `Builder.make_solid(&mut solid)`.
    pub fn make_solid(&self, solid: &mut Solid) {
        self.make_shape(solid, &Handle::new(TSolid::new()).into_cast());
    }

    /// `MakeCompSolid(C)`: an empty composite solid; `Builder.make_comp_solid(&mut c)`.
    pub fn make_comp_solid(&self, comp_solid: &mut CompSolid) {
        self.make_shape(comp_solid, &Handle::new(TCompSolid::new()).into_cast());
    }

    /// `MakeCompound(C)`: an empty compound; `Builder.make_compound(&mut compound)`.
    pub fn make_compound(&self, compound: &mut Compound) {
        self.make_shape(compound, &Handle::new(TCompound::new()).into_cast());
    }

    /// The protected `MakeShape(S, T)`, public for the derived builders of other
    /// crates: sets the TShape, the identity location and FORWARD;
    /// `Builder.make_shape(&mut shape, &tshape)`.
    pub fn make_shape(&self, shape: &mut Shape, tshape: &Handle<dyn TShape>) {
        *shape = Shape::from_parts(tshape.clone(), Location::default(), Orientation::Forward);
    }

    /// `Add(S, C)`: freezes `component`, then stores it in `shape` relative to the
    /// shape's orientation and location. TopoDS_FrozenShape when `shape` is not free,
    /// TopoDS_UnCompatibleShapes when its type cannot hold the component type,
    /// Standard_NullObject for a null shape (a crash natively);
    /// `Builder.add(&mut compound, &vertex)?`.
    pub fn add(&self, shape: &mut Shape, component: &Shape) -> Result<(), Failure> {
        let component_tshape = component.checked_tshape("TopoDS_Builder::Add")?;
        // Freezing the component before testing the parent prevents self insertion; the
        // component stays frozen when the call then fails.
        component_tshape.set_free(false);
        let parent = shape.checked_tshape("TopoDS_Builder::Add")?;
        if !parent.free() {
            return Err(frozen_shape("TopoDS_Builder::Add: the shape is not free"));
        }
        let (child_kind, parent_kind) = (component_tshape.shape_type(), parent.shape_type());
        if ALLOWED_PARENTS[child_kind as usize] & (1 << parent_kind as u32) == 0 {
            return Err(uncompatible_shapes(&format!(
                "TopoDS_Builder::Add: {child_kind:?} into {parent_kind:?}; expected an allowed parent type"
            )));
        }
        let child = relative_child(shape, component)?;
        parent.tshape_base().append_child(child);
        Ok(())
    }

    /// `Remove(S, C)`: removes the first sub-shape equal to `component` expressed in
    /// the frame of `shape`, and sets Modified when one is removed. TopoDS_FrozenShape
    /// when `shape` is not free; `Builder.remove(&mut compound, &vertex)?`.
    pub fn remove(&self, shape: &mut Shape, component: &Shape) -> Result<(), Failure> {
        let parent = shape.checked_tshape("TopoDS_Builder::Remove")?;
        if !parent.free() {
            return Err(frozen_shape(
                "TopoDS_Builder::Remove: the shape is not free",
            ));
        }
        let mut child = component.clone();
        if shape.orientation() == Orientation::Reversed {
            child.reverse();
        }
        let location = child.location().predivided(shape.location())?;
        child.set_location(&location, false)?;
        parent.tshape_base().remove_child(&child);
        Ok(())
    }
}

/// The child `Add` stores: reversed when the parent is REVERSED, and moved by the
/// inverse of the parent location when it is not the identity.
fn relative_child(shape: &Shape, component: &Shape) -> Result<Shape, Failure> {
    let mut child = component.clone();
    if shape.orientation() == Orientation::Reversed {
        child.reverse();
    }
    let location = shape.location();
    if !location.is_identity() {
        child.move_by(&location.inverted()?, false)?;
    }
    Ok(child)
}
