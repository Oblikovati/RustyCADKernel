// Rust translation of ModelingAlgorithms/TKFeat/LocOpe/LocOpe_Gluer.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LocOpe_Gluer`: a new shape glued on a basis shape along bound faces (and edges):
//! the basis is split by the edges of the glued faces, then rebuilt with the new shape
//! by the generator; the operation fuses or cuts by the relative orientation of the
//! faces.
mod orientation;
mod perform;

use super::Operation;
use crate::errors::{construction_error, not_done};
use orientation::get_orientation;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, IndexedDataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// `LocOpe_Gluer`: the gluing of a new shape on a basis shape.
///
/// ```
/// use rustycad_tkfeat::loc_ope::{Gluer, Operation};
/// let gluer = Gluer::new();
/// assert_eq!(gluer.ope_type(), Operation::Invalid);
/// ```
#[derive(Clone)]
pub struct Gluer {
    done: bool,
    sb: Shape,
    sn: Shape,
    res: Shape,
    ori: Orientation,
    ope: Operation,
    map_ef: IndexedDataMap<Shape, Shape, ShapeMapHasher>,
    map_ee: DataMap<Shape, Shape, ShapeMapHasher>,
    desc_f: DataMap<Shape, List<Shape>, ShapeMapHasher>,
    edges: List<Shape>,
    tgt_edges: List<Shape>,
}

impl Default for Gluer {
    fn default() -> Self {
        Self::new()
    }
}

/// The face of `s` that is the same as `f`, with its orientation in `s`;
/// Standard_ConstructionError when `s` has no such face.
fn face_in(s: &Shape, f: &Face) -> Result<Face, Failure> {
    let found = Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)?
        .find(|x| x.is_same(f))
        .ok_or_else(|| construction_error("LocOpe_Gluer::Bind", "the face is not in its shape"))?;
    topods::face(&f.oriented(found.orientation()))
}

impl Gluer {
    /// `LocOpe_Gluer()`: no shape, INTERNAL orientation, INVALID operation;
    /// `Gluer::new()`.
    pub fn new() -> Self {
        Self {
            done: false,
            sb: Shape::default(),
            sn: Shape::default(),
            res: Shape::default(),
            ori: Orientation::Internal,
            ope: Operation::Invalid,
            map_ef: IndexedDataMap::default(),
            map_ee: DataMap::default(),
            desc_f: DataMap::default(),
            edges: List::default(),
            tgt_edges: List::default(),
        }
    }

    /// `LocOpe_Gluer(Sbase, Snew)`; `Gluer::with_shapes(&sbase, &snew)`.
    pub fn with_shapes(sbase: &Shape, snew: &Shape) -> Self {
        Self {
            sb: sbase.clone(),
            sn: snew.clone(),
            ..Self::new()
        }
    }

    /// `Init(Sbase, Snew)`: new shapes, the bindings forgotten (the edge lists kept, as
    /// natively); `gluer.init(&sbase, &snew)`.
    pub fn init(&mut self, sbase: &Shape, snew: &Shape) {
        self.sb = sbase.clone();
        self.sn = snew.clone();
        self.map_ef.clear(false);
        self.map_ee.clear(false);
        self.desc_f.clear(false);
        self.done = false;
        self.ori = Orientation::Internal;
        self.ope = Operation::Invalid;
    }

    /// `Bind(Fnew, Fbase)`: the face `fnew` of the new shape is glued on the face
    /// `fbase` of the basis. The first binding sets the operation (the new shape
    /// reversed for a cut); a later one of another orientation invalidates it. The edges
    /// of `fnew` are bound to `fbase`, the edges on two base faces to none;
    /// `gluer.bind_faces(&fnew, &fbase)?`.
    pub fn bind_faces(&mut self, fnew: &Face, fbase: &Face) -> Result<(), Failure> {
        let fnor = face_in(&self.sn, fnew)?;
        let fbor = face_in(&self.sb, fbase)?;
        let ori = get_orientation(&fnor, &fbor)?;
        if self.ori == Orientation::Internal {
            self.ori = ori;
            if self.ori == Orientation::Reversed {
                self.sn.reverse();
                self.ope = Operation::Cut;
            } else {
                self.ope = Operation::Fuse;
            }
        } else if ori != Orientation::Forward {
            self.ope = Operation::Invalid;
        }
        for edg in Explorer::new(&fnor, ShapeEnum::Edge, ShapeEnum::Shape)? {
            match self.map_ef.change_seek(&edg) {
                None => {
                    self.map_ef.add(edg, fbor.as_ref().clone());
                }
                // edg sur 2 face. a binder avec l`edge commun
                Some(face) if !face.is_same(&fbor) => face.nullify(),
                Some(_) => {}
            }
        }
        self.map_ef.add(fnor.into_shape(), fbor.into_shape());
        Ok(())
    }

    /// `Bind(Enew, Ebase)`: the edge `enew` lies on `ebase`; Standard_ConstructionError
    /// when it lies on another one already; `gluer.bind_edges(&enew, &ebase)?`.
    pub fn bind_edges(&mut self, enew: &Edge, ebase: &Edge) -> Result<(), Failure> {
        if self.map_ee.seek(enew).is_some_and(|e| !e.is_same(ebase)) {
            return Err(construction_error(
                "LocOpe_Gluer::Bind",
                "the edge is bound to another base edge",
            ));
        }
        self.map_ee
            .bind(enew.as_ref().clone(), ebase.as_ref().clone());
        Ok(())
    }

    /// `OpeType()`; `gluer.ope_type()`.
    pub fn ope_type(&self) -> Operation {
        self.ope
    }

    /// `IsDone()`; `gluer.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `ResultingShape()`: StdFail_NotDone before a successful Perform;
    /// `gluer.resulting_shape()?`.
    pub fn resulting_shape(&self) -> Result<&Shape, Failure> {
        if !self.done {
            return Err(not_done("LocOpe_Gluer::ResultingShape"));
        }
        Ok(&self.res)
    }

    /// `DescendantFaces(F)`: the faces `f` became (none for an unknown face);
    /// `gluer.descendant_faces(&f)?`.
    pub fn descendant_faces(&self, f: &Face) -> Result<List<Shape>, Failure> {
        if !self.done {
            return Err(not_done("LocOpe_Gluer::DescendantFaces"));
        }
        Ok(self.desc_f.seek(f).cloned().unwrap_or_default())
    }

    /// `BasisShape()`; `gluer.basis_shape()`.
    pub fn basis_shape(&self) -> &Shape {
        &self.sb
    }

    /// `GluedShape()`: the new shape (reversed for a cut); `gluer.glued_shape()`.
    pub fn glued_shape(&self) -> &Shape {
        &self.sn
    }

    /// `Edges()`: the edges of the glued faces kept in the result; `gluer.edges()`.
    pub fn edges(&self) -> &List<Shape> {
        &self.edges
    }

    /// `TgtEdges()`: those of them between tangent faces; `gluer.tgt_edges()`.
    pub fn tgt_edges(&self) -> &List<Shape> {
        &self.tgt_edges
    }
}
