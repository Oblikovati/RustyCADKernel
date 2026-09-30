// Created on: 1993-07-23
// Created by: Remi LEQUETTE
// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of BRepLib_MakeSolid.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib_MakeSolid`: a solid from shells, a solid or a compsolid.
use super::errors::BRepLibShapeModification;
use super::make_shape::BRepLibMakeShape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, CompSolid, Face, Shape, Shell, Solid};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::ops::{Deref, DerefMut};

/// `BRepLib_MakeSolid` (docs/topalgo-design.md section 7): a solid of the given
/// shells, always done. From a compsolid, the faces shared by two of its solids are
/// deleted (reported by [`Self::face_status`]) and the others form one shell, in
/// the iteration order of a hashed map (TShape addresses), as natively.
///
/// ```
/// use rustycad_tktopalgo::brep_lib::BRepLibMakeSolid;
/// let maker = BRepLibMakeSolid::new()?;
/// assert!(maker.is_done());
/// assert_eq!(maker.solid()?.nb_children(), 0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepLibMakeSolid {
    base: BRepLibMakeShape,
    my_deleted_faces: List<Shape>,
}

impl Deref for BRepLibMakeSolid {
    type Target = BRepLibMakeShape;
    fn deref(&self) -> &BRepLibMakeShape {
        &self.base
    }
}

impl DerefMut for BRepLibMakeSolid {
    fn deref_mut(&mut self) -> &mut BRepLibMakeShape {
        &mut self.base
    }
}

/// The shapes of type `kind` under `shape`, in explorer order.
fn explored(shape: &Shape, kind: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    let mut shapes: Vec<Shape> = Vec::new();
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape)?;
    while explorer.more() {
        shapes.push(explorer.current().clone());
        explorer.next()?;
    }
    Ok(shapes)
}

impl BRepLibMakeSolid {
    /// The maker on an empty solid with `shells` added; done.
    fn with_shells(shells: &[&Shape]) -> Result<Self, Failure> {
        let mut solid = Solid::default();
        BRepBuilder::default().make_solid(&mut solid);
        let mut base = BRepLibMakeShape::new();
        base.my_shape = solid.into_shape();
        let mut maker = Self {
            base,
            my_deleted_faces: List::default(),
        };
        for shell in shells {
            maker.add_shape(shell)?;
        }
        maker.set_done();
        Ok(maker)
    }

    /// `BRepLib_MakeSolid()`: an empty solid, done; `BRepLibMakeSolid::new()?`.
    pub fn new() -> Result<Self, Failure> {
        Self::with_shells(&[])
    }

    /// `BRepLib_MakeSolid(S)` on a compsolid: one shell of the faces met an odd number
    /// of times in the shells of `S` (a face met twice is deleted);
    /// `BRepLibMakeSolid::from_comp_solid(&cs)?`.
    pub fn from_comp_solid(comp_solid: &CompSolid) -> Result<Self, Failure> {
        let mut maker = Self::with_shells(&[])?;
        let mut faces = MapOfShape::default();
        for shell in explored(comp_solid, ShapeEnum::Shell)? {
            for face in explored(&shell, ShapeEnum::Face)? {
                if !faces.add(face.clone()) {
                    maker.my_deleted_faces.append(face.clone());
                    faces.remove(&face);
                }
            }
        }
        let mut shell = Shell::default();
        BRepBuilder::default().make_shell(&mut shell);
        for face in faces.iter() {
            BRepBuilder::default().add(&mut shell, face)?;
        }
        shell.set_closed(BRepTool::is_closed(&shell)?)?;
        maker.add_shape(&shell)?;
        Ok(maker)
    }

    /// `BRepLib_MakeSolid(S)` on a shell; `BRepLibMakeSolid::from_shell(&s)?`.
    pub fn from_shell(shell: &Shell) -> Result<Self, Failure> {
        Self::with_shells(&[shell])
    }

    /// `BRepLib_MakeSolid(S1, S2)`; `BRepLibMakeSolid::from_two_shells(&s1, &s2)?`.
    pub fn from_two_shells(s1: &Shell, s2: &Shell) -> Result<Self, Failure> {
        Self::with_shells(&[s1, s2])
    }

    /// `BRepLib_MakeSolid(S1, S2, S3)`;
    /// `BRepLibMakeSolid::from_three_shells(&s1, &s2, &s3)?`.
    pub fn from_three_shells(s1: &Shell, s2: &Shell, s3: &Shell) -> Result<Self, Failure> {
        Self::with_shells(&[s1, s2, s3])
    }

    /// `BRepLib_MakeSolid(So)`: a new solid of the shells of `So`;
    /// `BRepLibMakeSolid::from_solid(&so)?`.
    pub fn from_solid(solid: &Solid) -> Result<Self, Failure> {
        let shells: Vec<Shape> = explored(solid, ShapeEnum::Shell)?;
        Self::with_shells(&shells.iter().collect::<Vec<&Shape>>())
    }

    /// `BRepLib_MakeSolid(So, S)`: the shells of `So`, then `S`;
    /// `BRepLibMakeSolid::from_solid_shell(&so, &s)?`.
    pub fn from_solid_shell(solid: &Solid, shell: &Shell) -> Result<Self, Failure> {
        let mut maker = Self::from_solid(solid)?;
        maker.add_shape(shell)?;
        Ok(maker)
    }

    /// `Add(S)`: adds a shell (the done flag is unchanged); `maker.add(&s)?`.
    pub fn add(&mut self, shell: &Shell) -> Result<(), Failure> {
        self.add_shape(shell)
    }

    /// `BRep_Builder::Add(myShape, S)`.
    fn add_shape(&mut self, shape: &Shape) -> Result<(), Failure> {
        BRepBuilder::default().add(&mut self.base.my_shape, shape)
    }

    /// `Solid()` and `operator TopoDS_Solid()`: the solid; `maker.solid()?`.
    pub fn solid(&self) -> Result<Solid, Failure> {
        topods::solid(self.shape()?)
    }

    /// `FaceStatus(F)`, the override of `BRepLib_MakeShape::FaceStatus`:
    /// `BRepLib_Deleted` for a face deleted from the compsolid, else
    /// `BRepLib_Preserved`; `maker.face_status(&face)`.
    pub fn face_status(&self, face: &Face) -> BRepLibShapeModification {
        if self
            .my_deleted_faces
            .iter()
            .any(|deleted| face.is_same(deleted))
        {
            return BRepLibShapeModification::Deleted;
        }
        BRepLibShapeModification::Preserved
    }
}
