// Rust translation of BRepFill_Sweep::BuildShell, steps 2.1 to 2.3 (the vertices and
// the iso edges of the grid) (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Sweep.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The vertices and edges of the shell of a sweep.
use super::BRepFillSweep;
use super::iso_edge::{build_iso_edge, is_degen, update_iso_edge};
use super::shell::{ShellGrid, at, at_mut};
use super::topology::{null_edge, reverse_edge_in_first_or_last_wire};
use crate::brep_fill::shape_table::{cell, cell_mut};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;

/// The parameter bounds of a face of the grid in the frame of its isos.
#[derive(Clone, Copy)]
struct IsoBounds {
    ufirst: f64,
    ulast: f64,
    vfirst: f64,
    vlast: f64,
}

impl BRepFillSweep {
    /// The vertex the laws give for vertex `index` of the section (at the section
    /// parameter `Vi(vi_index)` for an evolutive section) on vertex `i_path` of the
    /// path, with the tolerance `extra_err` plus the section vertex tolerance.
    fn swept_vertex(
        &self,
        g: &ShellGrid,
        i_path: i32,
        index: i32,
        vi_index: i32,
        extra_err: f64,
    ) -> Result<Shape, Failure> {
        let loc = self.loc()?;
        if g.const_section {
            let input: Vertex = topods::vertex(at(&g.sec_vertex, index, "SecVertex")?)?;
            return Ok(loc
                .perform_vertex(
                    i_path,
                    &input,
                    extra_err + *at(&g.v_error, index, "VError")?,
                    0,
                )?
                .into_shape());
        }
        let vi: f64 = *at(&g.vi, vi_index, "Vi")?;
        let sec = self.sec()?;
        Ok(loc
            .perform_vertex(
                i_path,
                &sec.vertex(index, vi)?,
                extra_err + sec.vertex_tol(index - 1, vi)?,
                0,
            )?
            .into_shape())
    }

    /// `MergeVertex` on two vertices of the grid: the second becomes the first when
    /// they are close.
    fn merge_in_grid(
        &self,
        g: &mut ShellGrid,
        from: (i32, i32),
        to: (i32, i32),
    ) -> Result<bool, Failure> {
        let v1: Shape = g.vertex_at(from.0, from.1)?.clone();
        let mut v2: Shape = g.vertex_at(to.0, to.1)?.clone();
        let merged: bool = self.merge_vertex(&v1, &mut v2)?;
        *g.vertex_mut(to.0, to.1)? = v2;
        Ok(merged)
    }

    /// A degenerated edge on the vertex of the grid (made FORWARD there).
    fn null_edge_at(g: &mut ShellGrid, at_vertex: (i32, i32)) -> Result<Shape, Failure> {
        let mut v: Shape = g.vertex_at(at_vertex.0, at_vertex.1)?.clone();
        let e: Edge = null_edge(&mut v)?;
        *g.vertex_mut(at_vertex.0, at_vertex.1)? = v;
        Ok(e.into_shape())
    }

    /// (2.1) The vertices of the first row (the first one, then the next one, merged
    /// into the previous one with a degenerated u-edge when they are close).
    fn first_row_vertices(
        &self,
        g: &mut ShellGrid,
        ipath: i32,
        i_path: i32,
    ) -> Result<(), Failure> {
        if ipath == 1 && g.vertex_at(1, 1)?.is_null() {
            *g.vertex_mut(1, 1)? = self.swept_vertex(g, i_path - 1, 1, 1, 0.)?;
        }
        if g.vclose && ipath == g.nb_path {
            *g.vertex_mut(1, ipath + 1)? = g.vertex_at(1, 1)?.clone();
        } else if g.vertex_at(1, ipath + 1)?.is_null() {
            let extra: f64 = *cell(&g.tab_err, 1, ipath)?;
            *g.vertex_mut(1, ipath + 1)? = self.swept_vertex(g, i_path, 1, ipath + 1, extra)?;
            if self.merge_in_grid(g, (1, ipath), (1, ipath + 1))? {
                *cell_mut(&mut g.u_edge, 1, ipath)? = Self::null_edge_at(g, (1, ipath))?;
            }
        }
        Ok(())
    }

    /// (2.1) The vertex of the first column after section `isec` (the first one for a
    /// closed section), merged with a degenerated v-edge when it is close.
    fn first_column_vertex(
        &self,
        g: &mut ShellGrid,
        isec: i32,
        i_path: i32,
    ) -> Result<(), Failure> {
        if g.uclose && isec == g.nb_law {
            *g.vertex_mut(isec + 1, 1)? = g.vertex_at(1, 1)?.clone();
        } else if g.vertex_at(isec + 1, 1)?.is_null() {
            let extra: f64 = *cell(&g.tab_err, isec, 1)?;
            *g.vertex_mut(isec + 1, 1)? = self.swept_vertex(g, i_path - 1, isec + 1, 1, extra)?;
            if self.merge_in_grid(g, (isec, 1), (isec + 1, 1))? {
                *cell_mut(&mut g.v_edge, isec, 1)? = Self::null_edge_at(g, (isec, 1))?;
            }
        }
        Ok(())
    }

    /// (2.1) The vertex at the far corner of face `(isec, ipath)` (shared with the
    /// first row or column when the sweep is closed).
    fn far_corner_vertex(
        &self,
        g: &mut ShellGrid,
        isec: i32,
        ipath: i32,
        i_path: i32,
    ) -> Result<(), Failure> {
        if g.uclose && isec == g.nb_law {
            *g.vertex_mut(isec + 1, ipath + 1)? = g.vertex_at(1, ipath + 1)?.clone();
        } else if g.vclose && ipath == g.nb_path {
            *g.vertex_mut(isec + 1, ipath + 1)? = g.vertex_at(isec + 1, 1)?.clone();
        } else if g.vertex_at(isec + 1, ipath + 1)?.is_null() {
            let extra: f64 = *cell(&g.tab_err, isec, ipath)?;
            *g.vertex_mut(isec + 1, ipath + 1)? =
                self.swept_vertex(g, i_path, isec + 1, ipath + 1, extra)?;
        }
        Ok(())
    }

    /// (2) The vertices and edges of every face of the grid not taken from a tape;
    /// false when a u-edge to update is missing (the sweep failed).
    pub(super) fn shell_edges(
        &mut self,
        g: &mut ShellGrid,
        reversed_edges: &mut MapOfShape,
    ) -> Result<bool, Failure> {
        for (ipath, i_path) in (1..=g.nb_path).zip(g.ifirst..) {
            for isec in 1..=g.nb_law {
                if *at(&g.is_built, isec, "IsBuilt")? {
                    continue;
                }
                if !self.face_edges(g, (isec, ipath, i_path), reversed_edges)? {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    /// The vertices and edges of face `(isec, ipath)`: degenerated or copied edges at a
    /// singular corner of a degenerated surface, else the iso edges.
    fn face_edges(
        &mut self,
        g: &mut ShellGrid,
        (isec, ipath, i_path): (i32, i32, i32),
        reversed_edges: &mut MapOfShape,
    ) -> Result<bool, Failure> {
        let (ufirst, ulast, vfirst, vlast) = g.oriented_bounds(isec, ipath)?;
        if isec == 1 {
            self.first_row_vertices(g, ipath, i_path)?;
        }
        if ipath == 1 {
            self.first_column_vertex(g, isec, i_path)?;
        }
        self.far_corner_vertex(g, isec, ipath, i_path)?;
        let singv: bool = self.merge_in_grid(g, (isec, ipath + 1), (isec + 1, ipath + 1))?;
        let singu: bool = self.merge_in_grid(g, (isec + 1, ipath), (isec + 1, ipath + 1))?;
        if singu || singv {
            let tol: f64 = self.my_tol3d.max(*cell(&g.tab_err, isec, ipath)?);
            *cell_mut(&mut g.degenerated, isec, ipath)? = is_degen(g.surface(isec, ipath)?, tol)?;
        }
        if *cell(&g.degenerated, isec, ipath)? {
            g.hasdegen = true;
            return Self::degenerated_face_edges(g, (isec, ipath), singu, singv).map(|()| true);
        }
        let exuv: bool = *cell(&g.exch_uv, isec, ipath)?;
        let b = if exuv {
            IsoBounds {
                ufirst: vfirst,
                ulast: vlast,
                vfirst: ufirst,
                vlast: ulast,
            }
        } else {
            IsoBounds {
                ufirst,
                ulast,
                vfirst,
                vlast,
            }
        };
        let s: Handle<dyn GeomSurface> = g.surface(isec, ipath)?.clone();
        if !self.u_iso_edges(g, (isec, ipath), &s, exuv, b)? {
            return Ok(false);
        }
        self.v_iso_edges(g, (isec, ipath), &s, exuv, b, reversed_edges)?;
        Ok(true)
    }

    /// The edges of a degenerated face: a degenerated edge at a singular corner, else a
    /// copy of the opposite edge.
    fn degenerated_face_edges(
        g: &mut ShellGrid,
        (isec, ipath): (i32, i32),
        singu: bool,
        singv: bool,
    ) -> Result<(), Failure> {
        if cell(&g.u_edge, isec + 1, ipath)?.is_null() {
            let e: Shape = if singu {
                Self::null_edge_at(g, (isec + 1, ipath))?
            } else {
                cell(&g.u_edge, isec, ipath)?.clone()
            };
            *cell_mut(&mut g.u_edge, isec + 1, ipath)? = e;
        }
        if cell(&g.v_edge, isec, ipath + 1)?.is_null() {
            let e: Shape = if singv {
                Self::null_edge_at(g, (isec, ipath + 1))?
            } else {
                cell(&g.v_edge, isec, ipath)?.clone()
            };
            *cell_mut(&mut g.v_edge, isec, ipath + 1)? = e;
        }
        Ok(())
    }

    /// The edge on the iso of the surface between two vertices of the grid.
    fn iso_edge_between(
        &self,
        g: &ShellGrid,
        s: &Handle<dyn GeomSurface>,
        (is_uiso, val): (bool, f64),
        v1: (i32, i32),
        v2: (i32, i32),
    ) -> Result<Shape, Failure> {
        Ok(build_iso_edge(
            s,
            is_uiso,
            val,
            g.vertex_at(v1.0, v1.1)?,
            g.vertex_at(v2.0, v2.1)?,
            self.my_tol3d,
        )?
        .into_shape())
    }

    /// (2.2) The u-iso edges of face `(isec, ipath)`: built or updated on the first and
    /// last u-isos (the first vertex of the first row merged into the previous one when
    /// close); false when a u-edge to update is missing.
    fn u_iso_edges(
        &self,
        g: &mut ShellGrid,
        (isec, ipath): (i32, i32),
        s: &Handle<dyn GeomSurface>,
        exuv: bool,
        b: IsoBounds,
    ) -> Result<bool, Failure> {
        if isec == 1 && cell(&g.u_edge, 1, ipath)?.is_null() {
            let (v1, v2) = (
                g.vertex_at(1, ipath)?.clone(),
                g.vertex_at(1, ipath + 1)?.clone(),
            );
            if !v1.is_same(&v2)
                && BRepTool::pnt(&topods::vertex(&v1)?)?
                    .distance(&BRepTool::pnt(&topods::vertex(&v2)?)?)
                    <= self.my_tol3d
            {
                *g.vertex_mut(1, ipath + 1)? = v1;
            }
            *cell_mut(&mut g.u_edge, 1, ipath)? =
                self.iso_edge_between(g, s, (!exuv, b.ufirst), (1, ipath), (1, ipath + 1))?;
        } else {
            let e: &Shape = cell(&g.u_edge, isec, ipath)?;
            if e.is_null() {
                return Ok(false);
            }
            update_iso_edge(&topods::edge(e)?, s, !exuv, b.ufirst)?;
        }
        let last_row: i32 = if g.uclose && isec == g.nb_law {
            1
        } else {
            isec + 1
        };
        if cell(&g.u_edge, last_row, ipath)?.is_null() {
            *cell_mut(&mut g.u_edge, isec + 1, ipath)? = self.iso_edge_between(
                g,
                s,
                (!exuv, b.ulast),
                (isec + 1, ipath),
                (isec + 1, ipath + 1),
            )?;
            return Ok(true);
        }
        update_iso_edge(
            &topods::edge(cell(&g.u_edge, last_row, ipath)?)?,
            s,
            !exuv,
            b.ulast,
        )?;
        if last_row == 1 {
            *cell_mut(&mut g.u_edge, isec + 1, ipath)? = cell(&g.u_edge, 1, ipath)?.clone();
        }
        Ok(true)
    }

    /// (2.3) The v-iso edges of face `(isec, ipath)`: the first one built (rebuilding a
    /// bound edge in place, reversed in its bound when needed) or updated, the last one
    /// shared with the first column of a closed sweep, built, updated or (at the last
    /// bound) rebuilt.
    fn v_iso_edges(
        &mut self,
        g: &mut ShellGrid,
        (isec, ipath): (i32, i32),
        s: &Handle<dyn GeomSurface>,
        exuv: bool,
        b: IsoBounds,
        reversed_edges: &mut MapOfShape,
    ) -> Result<(), Failure> {
        if ipath == 1 {
            let new_first: Edge = topods::edge(&self.iso_edge_between(
                g,
                s,
                (exuv, b.vfirst),
                (isec, 1),
                (isec + 1, 1),
            )?)?;
            if cell(&g.v_edge, isec, ipath)?.is_null() {
                *cell_mut(&mut g.v_edge, isec, ipath)? = new_first.into_shape();
            } else if self.rebuild_in_grid(g, (isec, ipath), &new_first, reversed_edges)? {
                reverse_edge_in_first_or_last_wire(
                    &mut self.first_shape,
                    cell(&g.v_edge, isec, ipath)?,
                )?;
                at_mut(&mut g.start_edges, isec, "StartEdges")?.reverse();
            }
        } else {
            update_iso_edge(
                &topods::edge(cell(&g.v_edge, isec, ipath)?)?,
                s,
                exuv,
                b.vfirst,
            )?;
        }
        self.last_v_iso_edge(g, (isec, ipath), s, exuv, b.vlast, reversed_edges)
    }

    /// The last v-iso edge of face `(isec, ipath)`.
    fn last_v_iso_edge(
        &mut self,
        g: &mut ShellGrid,
        (isec, ipath): (i32, i32),
        s: &Handle<dyn GeomSurface>,
        exuv: bool,
        vlast: f64,
        reversed_edges: &mut MapOfShape,
    ) -> Result<(), Failure> {
        let ends = ((isec, ipath + 1), (isec + 1, ipath + 1));
        if g.vclose && ipath == g.nb_path {
            let e: Shape = if cell(&g.v_edge, isec, 1)?.is_null() {
                self.iso_edge_between(g, s, (exuv, vlast), ends.0, ends.1)?
            } else {
                update_iso_edge(&topods::edge(cell(&g.v_edge, isec, 1)?)?, s, exuv, vlast)?;
                cell(&g.v_edge, isec, 1)?.clone()
            };
            *cell_mut(&mut g.v_edge, isec, ipath + 1)? = e;
            return Ok(());
        }
        if cell(&g.v_edge, isec, ipath + 1)?.is_null() {
            *cell_mut(&mut g.v_edge, isec, ipath + 1)? =
                self.iso_edge_between(g, s, (exuv, vlast), ends.0, ends.1)?;
            return Ok(());
        }
        if ipath != g.nb_path
            || g.vclose
            || (g.global_vclose && g.ilast == self.loc()?.nb_law()? + 1)
        {
            return update_iso_edge(
                &topods::edge(cell(&g.v_edge, isec, ipath + 1)?)?,
                s,
                exuv,
                vlast,
            );
        }
        let new_last: Edge =
            topods::edge(&self.iso_edge_between(g, s, (exuv, vlast), ends.0, ends.1)?)?;
        if self.rebuild_in_grid(g, (isec, ipath + 1), &new_last, reversed_edges)? {
            reverse_edge_in_first_or_last_wire(
                &mut self.last_shape,
                cell(&g.v_edge, isec, ipath + 1)?,
            )?;
        }
        Ok(())
    }

    /// `RebuildTopOrBottomEdge` on the v-edge of the grid (updated in place); whether
    /// it is then among the reversed edges.
    fn rebuild_in_grid(
        &self,
        g: &mut ShellGrid,
        (isec, ipath): (i32, i32),
        new_edge: &Edge,
        reversed_edges: &mut MapOfShape,
    ) -> Result<bool, Failure> {
        let mut an_edge: Edge = topods::edge(cell(&g.v_edge, isec, ipath)?)?;
        self.rebuild_top_or_bottom_edge(new_edge, &mut an_edge, reversed_edges)?;
        let contained: bool = reversed_edges.contains(&an_edge);
        *cell_mut(&mut g.v_edge, isec, ipath)? = an_edge.into_shape();
        Ok(contained)
    }
}
