// Rust translation of BRepFill_Pipe::FindEdge and FindVertex
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Pipe.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The indices of the profile edges and vertices in the tables of the pipe.
use super::BRepFillPipe;
use crate::brep_fill::section_law::{BRepFillSectionLaw, BRepFillShapeLaw};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, ShapeIterator, Vertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

impl BRepFillPipe {
    /// `FindEdge(S, E, Init)` (private): the index of the edge among the edges of the
    /// profile (counted through the walks of its wires), 0 when absent.
    pub(super) fn find_edge(
        &self,
        s: &Shape,
        e: &Edge,
        initial_length: &mut i32,
    ) -> Result<i32, Failure> {
        match s.shape_type()? {
            ShapeEnum::Edge => {
                *initial_length += 1;
                Ok(if s.is_same(e) { *initial_length } else { 0 })
            }
            ShapeEnum::Wire => {
                let section = BRepFillShapeLaw::with_wire(&topods::wire(s)?, false)?;
                let nb_law: i32 = section.nb_law()?;
                let mut result: i32 = 0;
                for ii in 1..=nb_law {
                    if e.is_same(section.edge(ii)?.as_ref()) {
                        result = *initial_length + ii;
                        break;
                    }
                }
                *initial_length += nb_law;
                Ok(result)
            }
            ShapeEnum::Face | ShapeEnum::Shell | ShapeEnum::Compound => {
                self.find_in_children(s, initial_length, &mut |pipe, child, init| {
                    pipe.find_edge(child, e, init)
                })
            }
            ShapeEnum::Solid | ShapeEnum::CompSolid => Err(Failure::with_type(
                "Standard_DomainError",
                "BRepFill_Pipe::SOLID or COMPSOLID",
                "BRepFill_Pipe",
            )),
            _ => Ok(0),
        }
    }

    /// `FindVertex(S, V, Init)` (private): the index of the vertex among the vertices of
    /// the profile (two per edge, one more than the edges per wire), 0 when absent.
    pub(super) fn find_vertex(
        &self,
        s: &Shape,
        v: &Vertex,
        initial_length: &mut i32,
    ) -> Result<i32, Failure> {
        match s.shape_type()? {
            ShapeEnum::Vertex => {
                *initial_length += 1;
                Ok(if s.is_same(v) { *initial_length } else { 0 })
            }
            ShapeEnum::Edge => {
                let (mut vf, mut vl) = topexp::vertices(&topods::edge(s)?, false);
                if s.orientation() == Orientation::Reversed {
                    std::mem::swap(&mut vf, &mut vl);
                }
                let result: i32 = if vf.is_same(v) {
                    *initial_length + 1
                } else if vl.is_same(v) {
                    *initial_length + 2
                } else {
                    0
                };
                *initial_length += 2;
                Ok(result)
            }
            ShapeEnum::Wire => {
                let first: i32 = *initial_length + 1;
                let section = BRepFillShapeLaw::with_wire(&topods::wire(s)?, false)?;
                *initial_length += section.nb_law()? + 1;
                for ii in first..=*initial_length {
                    if v.is_same(section.vertex(ii, 0.)?.as_ref()) {
                        return Ok(ii);
                    }
                }
                Ok(0)
            }
            ShapeEnum::Face | ShapeEnum::Shell | ShapeEnum::Compound => {
                self.find_in_children(s, initial_length, &mut |pipe, child, init| {
                    pipe.find_vertex(child, v, init)
                })
            }
            ShapeEnum::Solid | ShapeEnum::CompSolid => Err(Failure::with_type(
                "Standard_DomainError",
                "BRepFill_Pipe::SOLID or COMPSOLID",
                "BRepFill_Pipe",
            )),
            ShapeEnum::Shape => Ok(0),
        }
    }

    /// The first non-zero search result among the children.
    fn find_in_children(
        &self,
        s: &Shape,
        initial_length: &mut i32,
        find: &mut dyn FnMut(&Self, &Shape, &mut i32) -> Result<i32, Failure>,
    ) -> Result<i32, Failure> {
        let mut it = ShapeIterator::new(s, true, true);
        while it.more() {
            let result: i32 = find(self, it.value()?, initial_length)?;
            if result != 0 {
                return Ok(result);
            }
            it.next();
        }
        Ok(0)
    }
}
