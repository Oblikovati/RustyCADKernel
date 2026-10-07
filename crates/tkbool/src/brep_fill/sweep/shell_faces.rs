// Rust translation of BRepFill_Sweep::BuildShell, steps 3 to 5 (the faces, the history
// and continuities, the tapes and rails) (ModelingAlgorithms/TKBool/BRepFill/
// BRepFill_Sweep.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The faces of the shell of a sweep, their continuities and the tapes and rails kept
//! for the next sweep.
use super::build_face::{FaceBorders, build_face};
use super::shell::{ShellGrid, at};
use super::{BRepFillSweep, SweepHistory};
use crate::brep_fill::shape_table::{ShapeTable, cell, cell_mut, new_table};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Compound, Face, Shape};
use rustycad_tkernel::ncollection::HArray2;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tktopalgo::brep_lib;

/// Whether a slot of the face table holds a face.
fn is_face(shape: &Shape) -> Result<bool, Failure> {
    Ok(!shape.is_null() && shape.shape_type()? == ShapeEnum::Face)
}

impl BRepFillSweep {
    /// (3) The faces of the grid: an edge of a degenerated face (its u-edge when both
    /// are the same, else its first v-edge), a new face elsewhere when none is set;
    /// (3.1) the faces taken from tapes are reversed.
    pub(super) fn shell_faces(&mut self, g: &mut ShellGrid) -> Result<(), Failure> {
        for (ipath, i_path) in (1..=g.nb_path).zip(g.ifirst..) {
            for isec in 1..=g.nb_law {
                self.grid_face(g, isec, ipath, i_path)?;
            }
        }
        let faces: &mut ShapeTable = self.faces_mut()?;
        for ipath in 1..=g.nb_path {
            for isec in 1..=g.nb_law {
                if *at(&g.is_built, isec, "IsBuilt")? {
                    cell_mut(faces, isec, ipath)?.reverse();
                }
            }
        }
        Ok(())
    }

    /// The face (or the degenerated edge) of slot `(isec, i_path)`.
    fn grid_face(
        &mut self,
        g: &ShellGrid,
        isec: i32,
        ipath: i32,
        i_path: i32,
    ) -> Result<(), Failure> {
        if *cell(&g.degenerated, isec, ipath)? {
            let (u1, u2) = (
                cell(&g.u_edge, isec, ipath)?,
                cell(&g.u_edge, isec + 1, ipath)?,
            );
            let e: Shape = if u1.is_same(u2) {
                u1.clone()
            } else {
                cell(&g.v_edge, isec, ipath)?.clone()
            };
            *cell_mut(self.faces_mut()?, isec, i_path)? = e;
            return Ok(());
        }
        if !cell(self.faces_mut()?, isec, i_path)?.is_null() {
            return Ok(());
        }
        let edges = [
            cell(&g.u_edge, isec, ipath)?,
            cell(&g.v_edge, isec, ipath)?,
            cell(&g.u_edge, isec + 1, ipath)?,
            cell(&g.v_edge, isec, ipath + 1)?,
        ]
        .map(topods::edge);
        let [e1, e2, e3, e4] = edges;
        let (e1, e2, e3, e4) = (e1?, e2?, e3?, e4?);
        let borders = FaceBorders {
            e1: &e1,
            e2: &e2,
            e3: &e3,
            e4: &e4,
        };
        let mut face = Face::default();
        build_face(
            g.surface(isec, ipath)?,
            &borders,
            &mut self.my_v_edges_modified,
            *cell(&g.exch_uv, isec, ipath)?,
            *cell(&g.u_reverse, isec, ipath)?,
            &mut face,
        )?;
        *cell_mut(self.faces_mut()?, isec, i_path)? = face.into_shape();
        Ok(())
    }

    /// (4) The edges of the grid stored in the tables, with the regularity of all the
    /// faces in a degenerated case, else the G1 continuities between the faces of the
    /// G1 sections and paths.
    pub(super) fn shell_continuities(&mut self, g: &ShellGrid) -> Result<(), Failure> {
        if g.hasdegen {
            return self.degenerated_history(g);
        }
        self.u_edge_continuities(g)?;
        self.v_edge_continuities(g)
    }

    /// (4.1) Degenerated case: the tables filled and the regularity of the faces
    /// encoded at once.
    fn degenerated_history(&mut self, g: &ShellGrid) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let mut comp = Compound::default();
        builder.make_compound(&mut comp);
        for isec in 1..=g.nb_law + 1 {
            for (ipath, i_path) in (1..=g.nb_path + 1).zip(g.ifirst..) {
                if ipath <= g.nb_path {
                    *cell_mut(self.u_edges_mut()?, isec, i_path)? =
                        cell(&g.u_edge, isec, ipath)?.clone();
                }
                if isec <= g.nb_law {
                    *cell_mut(self.v_edges_mut()?, isec, i_path)? =
                        cell(&g.v_edge, isec, ipath)?.clone();
                }
                if ipath <= g.nb_path && isec <= g.nb_law {
                    let face: Shape = cell(self.faces_mut()?, isec, i_path)?.clone();
                    if is_face(&face)? {
                        builder.add(&mut comp, &face)?;
                    }
                }
            }
        }
        brep_lib::encode_regularity(&comp, self.my_tol_angular)
    }

    /// (4.2) The u-edges stored, G1 between the faces of the G1 vertices of the section.
    fn u_edge_continuities(&mut self, g: &ShellGrid) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        for isec in 1..=g.nb_law + 1 {
            let is_g1: bool = isec > 1
                && self.sec()?.continuity(isec - 1, self.my_tol_angular)? >= Continuity::G1;
            for (ipath, i_path) in (1..=g.nb_path).zip(g.ifirst..) {
                *cell_mut(self.u_edges_mut()?, isec, i_path)? =
                    cell(&g.u_edge, isec, ipath)?.clone();
                if !is_g1 {
                    continue;
                }
                let faces: &mut ShapeTable = self.faces_mut()?;
                let ff: Face = topods::face(cell(
                    faces,
                    if isec == g.nb_law + 1 { 1 } else { isec },
                    i_path,
                )?)?;
                let previous: Face = topods::face(cell(faces, isec - 1, i_path)?)?;
                let edge = topods::edge(cell(self.u_edges_mut()?, isec, i_path)?)?;
                builder.continuity_on_faces(&edge, &previous, &ff, Continuity::G1)?;
            }
        }
        Ok(())
    }

    /// (4.2) The v-edges stored, their regularity encoded between the faces of the G1
    /// vertices of the path (also at the closing vertex of a closed sweep).
    fn v_edge_continuities(&mut self, g: &ShellGrid) -> Result<(), Failure> {
        let nbpath: i32 = if g.vclose { g.nb_path + 1 } else { g.nb_path };
        for (ipath, i_path) in (1..=g.nb_path + 1).zip(g.ifirst..) {
            let is_g1: bool = ipath > 1
                && ipath <= nbpath
                && self
                    .loc()?
                    .is_g1(i_path - 1, self.my_tol3d, self.my_tol_angular)?
                    >= 0;
            for isec in 1..=g.nb_law {
                *cell_mut(self.v_edges_mut()?, isec, i_path)? =
                    cell(&g.v_edge, isec, ipath)?.clone();
                if !is_g1 {
                    continue;
                }
                let faces: &mut ShapeTable = self.faces_mut()?;
                let ff: Face = topods::face(cell(
                    faces,
                    isec,
                    if ipath == g.nb_path + 1 { 1 } else { i_path },
                )?)?;
                let previous: Face = topods::face(cell(faces, isec, i_path - 1)?)?;
                let an_edge = topods::edge(cell(self.v_edges_mut()?, isec, i_path)?)?;
                brep_lib::encode_regularity_of_edge(&an_edge, &ff, &previous, self.my_tol_angular)?;
            }
        }
        Ok(())
    }

    /// (5) The tapes of the edges of the first section and the rails of their vertices
    /// updated (only for a single shell starting at the first path).
    pub(super) fn shell_tapes_and_rails(
        &mut self,
        g: &ShellGrid,
        hist: &mut SweepHistory,
    ) -> Result<(), Failure> {
        if g.ifirst != 1 || hist.tapes.is_empty() {
            return Ok(());
        }
        for isec in 1..=g.nb_law {
            let start: &Shape = at(&g.start_edges, isec, "StartEdges")?;
            self.fill_tape(g, isec, hist.tapes.find(start)?)?;
            let (vfirst, vlast) = topexp::vertices(&topods::edge(start)?, true);
            for (v, row) in [(vfirst.into_shape(), isec), (vlast.into_shape(), isec + 1)] {
                if !hist.rails.is_bound(&v) {
                    hist.rails
                        .bind(v, Handle::new(HArray2::from_owned(self.rail_of(g, row)?)));
                }
            }
        }
        Ok(())
    }

    /// The tape of section `isec`: its v-edges, u-edges, vertices and faces.
    fn fill_tape(
        &mut self,
        g: &ShellGrid,
        isec: i32,
        tape: &Handle<HArray2<Shape>>,
    ) -> Result<(), Failure> {
        let shared = tape.get().ok_or_else(|| super::null("tape table"))?;
        let mut t = shared.change_array2();
        let np: i32 = g.nb_path;
        for j in 1..=np + 1 {
            *cell_mut(&mut t, 1, j)? = cell(self.v_edges_mut()?, isec, j)?.clone();
        }
        for (row, from) in [(2, isec), (3, isec + 1)] {
            for j in 1..=np {
                *cell_mut(&mut t, row, j)? = cell(self.u_edges_mut()?, from, j)?.clone();
            }
        }
        for (row, from) in [(4, isec), (5, isec + 1)] {
            for j in 1..=np + 1 {
                *cell_mut(&mut t, row, j)? = g.vertex_at(from, j)?.clone();
            }
        }
        for j in 1..=np {
            *cell_mut(&mut t, 6, j)? = cell(self.faces_mut()?, isec, j)?.clone();
        }
        Ok(())
    }

    /// The rail of row `row` of the grid: its u-edges and vertices.
    fn rail_of(&mut self, g: &ShellGrid, row: i32) -> Result<ShapeTable, Failure> {
        let mut an_array: ShapeTable = new_table((1, 2), (1, g.nb_path + 1))?;
        for j in 1..=g.nb_path {
            *cell_mut(&mut an_array, 1, j)? = cell(self.u_edges_mut()?, row, j)?.clone();
        }
        for j in 1..=g.nb_path + 1 {
            *cell_mut(&mut an_array, 2, j)? = g.vertex_at(row, j)?.clone();
        }
        Ok(an_array)
    }
}
