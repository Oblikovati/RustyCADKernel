// Rust translation of BRepFill_Sweep::BuildShell, step 2.0 (the edges and vertices of
// the bounding sections, from the bounds, the tapes and the rails) and the vertices of
// the first and last columns (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Sweep.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The ends of the shell of a sweep: the first and last sections.
use super::shell::{ShellGrid, at, at_mut};
use super::{BRepFillSweep, SweepHistory, SweepTablesMap, null};
use crate::brep_fill::shape_table::{ShapeTable, cell, cell_mut, clone_table, new_table};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkernel::ncollection::HArray2;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::toploc::Location;

/// The table of a tape or rail; Standard_NoSuchObject when the key is not bound.
fn bound_table(map: &SweepTablesMap, key: &Shape) -> Result<ShapeTable, Failure> {
    let handle = map.find(key)?;
    let shared = handle.get().ok_or_else(|| null("tape table"))?;
    Ok(clone_table(&shared.array2()))
}

/// The vertex at the start (or, `at_end`, the end) of an edge as oriented.
fn oriented_end(e: &Shape, at_end: bool) -> Result<Vertex, Failure> {
    let edge: Edge = topods::edge(e)?;
    let reversed: bool = edge.orientation() == Orientation::Reversed;
    Ok(if reversed == at_end {
        topexp::first_vertex(&edge, false)
    } else {
        topexp::last_vertex(&edge, false)
    })
}

impl ShellGrid {
    /// Row `row` of a tape copied into columns of a grid table.
    fn copy_row(
        tape: &ShapeTable,
        row: i32,
        target: &mut ShapeTable,
        target_row: i32,
        count: i32,
    ) -> Result<(), Failure> {
        for j in 1..=count {
            *cell_mut(target, target_row, j)? = cell(tape, row, j)?.clone();
        }
        Ok(())
    }

    /// The section `isec` taken from its tape: its v-edges reversed (the round goes the
    /// other way), u-edges, vertices and faces, copied to the closing row when the
    /// section is closed.
    fn take_tape(
        &mut self,
        isec: i32,
        tape: &ShapeTable,
        faces: &mut ShapeTable,
    ) -> Result<(), Failure> {
        *at_mut(&mut self.is_built, isec, "IsBuilt")? = true;
        for j in 1..=self.nb_path + 1 {
            let mut e: Shape = cell(tape, 1, j)?.clone();
            e.reverse();
            *cell_mut(&mut self.v_edge, isec, j)? = e;
        }
        let (ifirst, ilast) = (isec + 1, isec);
        Self::copy_row(tape, 2, &mut self.u_edge, ifirst, self.nb_path)?;
        Self::copy_row(tape, 3, &mut self.u_edge, ilast, self.nb_path)?;
        Self::copy_row(tape, 4, &mut self.vertex, ifirst, self.nb_path + 1)?;
        Self::copy_row(tape, 5, &mut self.vertex, ilast, self.nb_path + 1)?;
        Self::copy_row(tape, 6, faces, isec, self.nb_path)?;
        if self.uclose && isec == 1 {
            self.copy_closing_row(1, self.nb_law + 1)?;
        }
        if self.uclose && isec == self.nb_law {
            self.copy_closing_row(self.nb_law + 1, 1)?;
        }
        Ok(())
    }

    /// The u-edges and vertices of row `from` copied to row `to`.
    fn copy_closing_row(&mut self, from: i32, to: i32) -> Result<(), Failure> {
        for j in 1..=self.nb_path {
            *cell_mut(&mut self.u_edge, to, j)? = cell(&self.u_edge, from, j)?.clone();
        }
        for j in 1..=self.nb_path + 1 {
            *cell_mut(&mut self.vertex, to, j)? = cell(&self.vertex, from, j)?.clone();
        }
        Ok(())
    }

    /// The u-edges and vertices of a rail put in row `ind`.
    fn take_rail(&mut self, rail: &ShapeTable, ind: i32) -> Result<(), Failure> {
        Self::copy_row(rail, 1, &mut self.u_edge, ind, self.nb_path)?;
        Self::copy_row(rail, 2, &mut self.vertex, ind, self.nb_path + 1)
    }

    /// The vertex of section bound at `(isec, ipath)` placed on the surface point (made
    /// when null, else moved and its location removed) with the section tolerance.
    fn place_vertex(
        &mut self,
        (isec, ipath): (i32, i32),
        point: rustycad_tkmath::gp::Point3,
        tol: f64,
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let slot: &mut Shape = self.vertex_mut(isec, ipath)?;
        if slot.is_null() {
            let mut v = Vertex::default();
            builder.make_vertex_at(&mut v, point, tol)?;
            *slot = v.into_shape();
            return Ok(());
        }
        slot.set_location(&Location::default(), false)?;
        builder.update_vertex_point(&topods::vertex(slot)?, point, tol)
    }
}

impl BRepFillSweep {
    /// (2.0) The first section from the first bound: its edges, their end vertices
    /// (tolerances updated), and for each edge the grid of its tape when it has one,
    /// else a new empty tape and the rails of its vertices.
    pub(super) fn shell_first_bound(
        &mut self,
        g: &mut ShellGrid,
        hist: &mut SweepHistory,
    ) -> Result<(), Failure> {
        if self.first_shape.is_null() || g.ifirst != 1 {
            return Ok(());
        }
        self.sec()?.init(&self.first_shape)?;
        for isec in 1..=g.nb_law {
            let e: Edge = self.sec()?.current_edge()?;
            let (vfirst, vlast) = topexp::vertices(&e, false);
            *cell_mut(&mut g.v_edge, isec, 1)? = e.as_ref().clone();
            *g.vertex_mut(isec + 1, 1)? = if e.orientation() == Orientation::Reversed {
                vfirst.as_ref().clone()
            } else {
                vlast.as_ref().clone()
            };
            self.update_vertex(
                g.ifirst - 1,
                isec + 1,
                *cell(&g.tab_err, isec, 1)?,
                *at(&g.vi, 1, "Vi")?,
                g.vertex_at(isec + 1, 1)?,
            )?;
            *at_mut(&mut g.start_edges, isec, "StartEdges")? = e.as_ref().clone();
            if hist.tapes.is_bound(&e) {
                let tape = bound_table(hist.tapes, &e)?;
                let faces = self.faces_mut()?;
                g.take_tape(isec, &tape, faces)?;
                continue;
            }
            let empty: ShapeTable = new_table((1, 6), (1, g.nb_path + 1))?;
            hist.tapes
                .bind(e.as_ref().clone(), Handle::new(HArray2::from_owned(empty)));
            for (v, forward_end) in [(&vfirst, false), (&vlast, true)] {
                if hist.rails.is_bound(v) {
                    let toward: Orientation = if forward_end {
                        Orientation::Forward
                    } else {
                        Orientation::Reversed
                    };
                    let ind: i32 = if e.orientation() == toward {
                        isec + 1
                    } else {
                        isec
                    };
                    g.take_rail(&bound_table(hist.rails, v)?, ind)?;
                }
            }
        }
        *g.vertex_mut(1, 1)? = oriented_end(cell(&g.v_edge, 1, 1)?, false)?.into_shape();
        self.update_vertex(
            g.ifirst - 1,
            1,
            *cell(&g.tab_err, 1, 1)?,
            *at(&g.vi, 1, "Vi")?,
            g.vertex_at(1, 1)?,
        )
    }

    /// The vertices of the first column put on the corners of the first swept surfaces.
    pub(super) fn shell_first_column(&self, g: &mut ShellGrid) -> Result<(), Failure> {
        let vi1: f64 = *at(&g.vi, 1, "Vi")?;
        for isec in 1..=g.nb_law + 1 {
            let row: i32 = isec.min(g.nb_law);
            let (ufirst, ulast, vfirst, vlast) = g.oriented_bounds(row, 1)?;
            let exuv: bool = *cell(&g.exch_uv, row, 1)?;
            let (u, v) = if isec != g.nb_law + 1 {
                (ufirst, vfirst)
            } else if exuv {
                (ufirst, vlast)
            } else {
                (ulast, vfirst)
            };
            let point = g
                .surface(row, 1)?
                .get()
                .ok_or_else(|| null("swept surface"))?
                .value(u, v)?;
            g.place_vertex((isec, 1), point, self.sec()?.vertex_tol(isec - 1, vi1)?)?;
        }
        Ok(())
    }

    /// The last section from the last bound: its edges where none is set, their end
    /// vertices where none is set, and the tolerances of those vertices updated.
    pub(super) fn shell_last_bound(&mut self, g: &mut ShellGrid) -> Result<(), Failure> {
        if self.last_shape.is_null() || g.ilast != self.loc()?.nb_law()? + 1 {
            return Ok(());
        }
        let np1: i32 = g.nb_path + 1;
        let vi_last: f64 = *at(&g.vi, np1, "Vi")?;
        self.sec()?.init(&self.last_shape)?;
        for isec in 1..=g.nb_law {
            let e: Edge = self.sec()?.current_edge()?;
            if cell(&g.v_edge, isec, np1)?.is_null() {
                *cell_mut(&mut g.v_edge, isec, np1)? = e.into_shape();
            }
            if g.vertex_at(isec + 1, np1)?.is_null() {
                *g.vertex_mut(isec + 1, np1)? =
                    oriented_end(cell(&g.v_edge, isec, np1)?, true)?.into_shape();
            }
            self.update_vertex(
                g.ilast - 1,
                isec + 1,
                *cell(&g.tab_err, isec, g.nb_path)?,
                vi_last,
                g.vertex_at(isec + 1, np1)?,
            )?;
        }
        if g.vertex_at(1, np1)?.is_null() {
            *g.vertex_mut(1, np1)? = oriented_end(cell(&g.v_edge, 1, np1)?, false)?.into_shape();
        }
        self.update_vertex(
            g.ilast - 1,
            1,
            *cell(&g.tab_err, 1, g.nb_path)?,
            vi_last,
            g.vertex_at(1, np1)?,
        )
    }

    /// The vertices of the last column put on the corners of the last swept surfaces.
    pub(super) fn shell_last_column(&self, g: &mut ShellGrid) -> Result<(), Failure> {
        let vi_last: f64 = *at(&g.vi, g.nb_path + 1, "Vi")?;
        for isec in 1..=g.nb_law + 1 {
            let row: i32 = isec.min(g.nb_law);
            let (ufirst, ulast, vfirst, vlast) = g.oriented_bounds(row, g.nb_path)?;
            let exuv: bool = *cell(&g.exch_uv, row, g.nb_path)?;
            let (u, v) = if isec == g.nb_law + 1 {
                (ulast, vlast)
            } else if exuv {
                (ulast, vfirst)
            } else {
                (ufirst, vlast)
            };
            let point = g
                .surface(row, g.nb_path)?
                .get()
                .ok_or_else(|| null("swept surface"))?
                .value(u, v)?;
            g.place_vertex(
                (isec, g.nb_path + 1),
                point,
                self.sec()?.vertex_tol(isec - 1, vi_last)?,
            )?;
        }
        Ok(())
    }
}
