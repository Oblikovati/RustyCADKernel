// Created on: 1995-03-28
// Created by: Yves FRICAUD
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
// Rust translation of ModelingData/TKBRep/BRepTools/BRepTools_Substitution.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTools_Substitution`: rebuilds a shape with some sub-shapes substituted by
//! lists of shapes.
use crate::brep::{BRepBuilder, BRepTool};
use crate::topods::{self, Shape, ShapeIterator};
use crate::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};

/// Substitutes sub-shapes by lists of shapes and rebuilds the shapes that contain them
/// (`BRepTools_Substitution`).
///
/// ```
/// use rustycad_tkbrep::brep_tools::Substitution;
/// use rustycad_tkbrep::topods::Shape;
/// let substitution = Substitution::new();
/// assert!(!substitution.is_copied(&Shape::default()));
/// ```
#[derive(Default)]
pub struct Substitution {
    map: DataMap<Shape, Vec<Shape>, ShapeMapHasher>,
}

impl Substitution {
    /// `BRepTools_Substitution()`; `Substitution::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Clear()`; `substitution.clear()`.
    pub fn clear(&mut self) {
        self.map.clear(true);
    }

    /// `Substitute(OS, NS)`: `old` will be replaced by the shapes of `new` (none removes
    /// it); Standard_ConstructionError when `old` is already copied;
    /// `substitution.substitute(&edge, &[e1, e2])?`.
    pub fn substitute(&mut self, old: &Shape, new: &[Shape]) -> Result<(), Failure> {
        if self.is_copied(old) {
            return Err(Failure::with_type(
                "Standard_ConstructionError",
                "BRepTools_CutClue::Substitute: the shape is already substituted; expected a shape without substitution",
                "",
            ));
        }
        self.map.bind(old.clone(), new.to_vec());
        Ok(())
    }

    /// `Build(S)`: rebuilds `shape` and its sub-shapes with the substitutions: a shape
    /// with a substituted sub-shape becomes an empty copy (the range of an edge kept)
    /// holding the substitutes of its sub-shapes with the composed orientations; a
    /// wire, shell, solid or compound left empty disappears. Each built shape is
    /// recorded FORWARD; `substitution.build(&shape)?`.
    pub fn build(&mut self, shape: &Shape) -> Result<(), Failure> {
        if self.is_copied(shape) {
            return Ok(());
        }
        let forward = shape.oriented(Orientation::Forward);
        let children: Vec<Shape> = ShapeIterator::new(&forward, true, true).collect();
        let mut modified = false;
        for child in &children {
            self.build(child)?;
            if self.is_copied(child) {
                modified = true;
            }
        }
        let mut new_shape = forward;
        if modified {
            new_shape = self.rebuilt(shape, &children, new_shape)?;
        }
        let list = if new_shape.is_null() {
            Vec::new()
        } else {
            vec![new_shape.oriented(Orientation::Forward)]
        };
        self.substitute(shape, &list)
    }

    /// The empty copy of the shape filled with the substitutes of its sub-shapes.
    fn rebuilt(
        &mut self,
        shape: &Shape,
        children: &[Shape],
        mut new_shape: Shape,
    ) -> Result<Shape, Failure> {
        let builder = BRepBuilder::default();
        new_shape.empty_copy()?;
        if new_shape.shape_type()? == ShapeEnum::Edge {
            let (first, last) = BRepTool::range(&topods::edge(shape)?)?;
            builder.range(&topods::edge(&new_shape)?, first, last, false)?;
        }
        let mut has_sub_shape = false;
        for child in children {
            let orientation = child.orientation();
            let substitutes = self.substitutes(child)?;
            for substitute in substitutes {
                self.build(&substitute)?;
                let new_orientation = top_abs::compose(orientation, substitute.orientation());
                for item in self.substitutes(&substitute)? {
                    builder.add(&mut new_shape, &item.oriented(new_orientation))?;
                    has_sub_shape = true;
                }
            }
        }
        let container = matches!(
            new_shape.shape_type()?,
            ShapeEnum::Wire | ShapeEnum::Shell | ShapeEnum::Solid | ShapeEnum::Compound
        );
        if !has_sub_shape && container {
            // A wire, a solid, a shell or a compound must have sub-shapes.
            new_shape.nullify();
        }
        Ok(new_shape)
    }

    /// `myMap(S)`: Standard_NoSuchObject for a shape without substitution.
    fn substitutes(&self, shape: &Shape) -> Result<Vec<Shape>, Failure> {
        self.map.find(shape).cloned().map_err(|_| {
            Failure::with_type(
                "Standard_NoSuchObject",
                "BRepTools_Substitution: the shape has no substitution; expected a built shape",
                "",
            )
        })
    }

    /// `IsCopied(S)`: `shape` is substituted by nothing or by other shapes;
    /// `substitution.is_copied(&shape)`.
    pub fn is_copied(&self, shape: &Shape) -> bool {
        match self.map.seek(shape) {
            None => false,
            Some(list) => list.first().is_none_or(|first| !shape.is_same(first)),
        }
    }

    /// `Copy(S)`: the substitutes of a copied shape; Standard_NoSuchObject otherwise;
    /// `substitution.copy(&shape)?`.
    pub fn copy(&self, shape: &Shape) -> Result<Vec<Shape>, Failure> {
        if !self.is_copied(shape) {
            return Err(Failure::with_type(
                "Standard_NoSuchObject",
                "BRepTools_Substitution::Copy: the shape is not copied; expected a substituted shape",
                "",
            ));
        }
        self.substitutes(shape)
    }
}
