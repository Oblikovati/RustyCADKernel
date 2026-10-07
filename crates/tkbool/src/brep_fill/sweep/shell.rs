// Rust translation of BRepFill_Sweep::BuildShell (the tables and the swept surfaces,
// steps 1.1 and 1.2, and the sequence of the steps)
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Sweep.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The shell of a sweep between two vertices of the path: the grid of swept surfaces,
//! vertices, edges and faces.
use super::topology::reverse_modified_edges;
use super::{BRepFillSweep, SweepHistory, null};
use crate::brep_fill::shape_table::{ShapeTable, cell, cell_mut, new_table};
use crate::brep_fill::transition_style::TransitionStyle;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBoundedSurface, GeomSurface};
use rustycad_tkgeomalgo::geom_fill::GeomFillSweep;
use rustycad_tkgeombase::geom_lib;

/// The grid of BuildShell: the swept surfaces of the sections (rows) by the paths
/// (columns) and the vertices and edges between them.
pub(super) struct ShellGrid {
    pub(super) ifirst: i32,
    pub(super) ilast: i32,
    pub(super) nb_path: i32,
    pub(super) nb_law: i32,
    pub(super) uclose: bool,
    pub(super) vclose: bool,
    pub(super) global_vclose: bool,
    pub(super) const_section: bool,
    pub(super) hasdegen: bool,
    pub(super) exch_uv: Array2<'static, bool>,
    pub(super) u_reverse: Array2<'static, bool>,
    pub(super) degenerated: Array2<'static, bool>,
    pub(super) tab_err: Array2<'static, f64>,
    pub(super) tab_s: Array2<'static, Handle<dyn GeomSurface>>,
    pub(super) u_edge: ShapeTable,
    pub(super) v_edge: ShapeTable,
    pub(super) vertex: ShapeTable,
    pub(super) sec_vertex: Vec<Shape>,
    pub(super) v_error: Vec<f64>,
    pub(super) vi: Vec<f64>,
    pub(super) is_built: Vec<bool>,
    pub(super) start_edges: Vec<Shape>,
}

/// Standard_OutOfRange for a 1-based row access.
fn row_error(what: &str, index: i32, len: usize) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("BRepFill_Sweep BuildShell: {what} index {index}; expected 1..={len}"),
        "NCollection_Array1",
    )
}

/// The item `index` (1-based) of a row.
pub(super) fn at<'v, T>(row: &'v [T], index: i32, what: &str) -> Result<&'v T, Failure> {
    usize::try_from(index - 1)
        .ok()
        .and_then(|i| row.get(i))
        .ok_or_else(|| row_error(what, index, row.len()))
}

/// The item `index` (1-based) of a row, for writing.
pub(super) fn at_mut<'v, T>(
    row: &'v mut [T],
    index: i32,
    what: &str,
) -> Result<&'v mut T, Failure> {
    let len: usize = row.len();
    usize::try_from(index - 1)
        .ok()
        .and_then(|i| row.get_mut(i))
        .ok_or_else(|| row_error(what, index, len))
}

impl ShellGrid {
    /// The surface swept by section `isec` along path `ipath`.
    pub(super) fn surface(
        &self,
        isec: i32,
        ipath: i32,
    ) -> Result<&Handle<dyn GeomSurface>, Failure> {
        cell(&self.tab_s, isec, ipath)
    }

    /// The parameter bounds of a swept surface (U and V first and last), the U ones
    /// exchanged (or the V ones when the parameters are exchanged) when the U direction
    /// is reversed.
    pub(super) fn oriented_bounds(
        &self,
        isec: i32,
        ipath: i32,
    ) -> Result<(f64, f64, f64, f64), Failure> {
        let surface = self
            .surface(isec, ipath)?
            .get()
            .ok_or_else(|| null("swept surface"))?;
        let (ufirst, ulast, vfirst, vlast) = surface.bounds();
        if !*cell(&self.u_reverse, isec, ipath)? {
            return Ok((ufirst, ulast, vfirst, vlast));
        }
        if *cell(&self.exch_uv, isec, ipath)? {
            return Ok((ufirst, ulast, vlast, vfirst));
        }
        Ok((ulast, ufirst, vfirst, vlast))
    }

    /// The vertex at `(isec, ipath)`.
    pub(super) fn vertex_at(&self, isec: i32, ipath: i32) -> Result<&Shape, Failure> {
        cell(&self.vertex, isec, ipath)
    }

    /// The vertex at `(isec, ipath)`, for writing.
    pub(super) fn vertex_mut(&mut self, isec: i32, ipath: i32) -> Result<&mut Shape, Failure> {
        cell_mut(&mut self.vertex, isec, ipath)
    }
}

impl BRepFillSweep {
    /// `BuildShell(Transition, IFirst, ILast, ReversedEdges, Tapes, Rails, ExtendFirst,
    /// ExtendLast)` (protected): the faces of the paths `ifirst..ilast` (the surfaces
    /// extended at the ends by the given lengths) with their vertices, edges, history
    /// and continuities; false when a sweep fails.
    /// `sweep.build_shell(TransitionStyle::Right, (1, 3), &mut history, (0.0, 0.0))?`.
    pub fn build_shell(
        &mut self,
        _transition: TransitionStyle,
        (ifirst, ilast): (i32, i32),
        hist: &mut SweepHistory,
        (extend_first, extend_last): (f64, f64),
    ) -> Result<bool, Failure> {
        let mut g: ShellGrid = self.shell_grid(ifirst, ilast)?;
        if !self.shell_surfaces(&mut g, extend_first, extend_last)? {
            return Ok(false);
        }
        reverse_modified_edges(&mut self.first_shape, hist.reversed_edges)?;
        reverse_modified_edges(&mut self.last_shape, hist.reversed_edges)?;
        self.shell_first_bound(&mut g, hist)?;
        self.shell_first_column(&mut g)?;
        self.shell_last_bound(&mut g)?;
        self.shell_last_column(&mut g)?;
        if !self.shell_edges(&mut g, hist.reversed_edges)? {
            return Ok(false);
        }
        self.shell_faces(&mut g)?;
        self.shell_continuities(&g)?;
        self.shell_tapes_and_rails(&g, hist)?;
        Ok(true)
    }

    /// (1.1) The tables of the grid, the first section parameter and (for a constant
    /// section) the a priori vertex errors.
    fn shell_grid(&mut self, ifirst: i32, ilast: i32) -> Result<ShellGrid, Failure> {
        self.error = 0.;
        let (nb_path, nb_law) = (ilast - ifirst, self.sec()?.nb_law()?);
        let global_vclose: bool =
            self.loc()?.is_closed()? && self.loc()?.is_g1(0, self.my_tol3d, 1.0e-4)? >= 0;
        let sec = self.sec()?;
        let vclose: bool = global_vclose && sec.is_v_closed() && nb_path == self.loc()?.nb_law()?;
        let mut g = ShellGrid {
            ifirst,
            ilast,
            nb_path,
            nb_law,
            uclose: sec.is_u_closed(),
            vclose,
            global_vclose,
            const_section: sec.is_constant(),
            hasdegen: false,
            exch_uv: new_table((1, nb_law), (1, nb_path))?,
            u_reverse: new_table((1, nb_law), (1, nb_path))?,
            degenerated: new_table((1, nb_law), (1, nb_path))?,
            tab_err: new_table((1, nb_law), (1, nb_path))?,
            tab_s: new_table((1, nb_law), (1, nb_path))?,
            u_edge: new_table((1, nb_law + 1), (1, nb_path))?,
            v_edge: new_table((1, nb_law), (1, nb_path + 1))?,
            vertex: new_table((1, nb_law + 1), (1, nb_path + 1))?,
            sec_vertex: vec![Shape::default(); (nb_law + 1).max(0) as usize],
            v_error: vec![0.; (nb_law + 1).max(0) as usize],
            vi: vec![0.; (nb_path + 1).max(0) as usize],
            is_built: vec![false; nb_law.max(0) as usize],
            start_edges: vec![Shape::default(); nb_law.max(0) as usize],
        };
        let (sec_deb, sec_dom) = self.section_domain()?;
        *at_mut(&mut g.vi, 1, "Vi")? = if ifirst > 1 {
            self.section_parameter(ifirst - 1, (sec_deb, sec_dom))?
        } else {
            sec_deb
        };
        if g.const_section {
            for isec in 1..=nb_law + 1 {
                *at_mut(&mut g.v_error, isec, "VError")? = sec.vertex_tol(isec - 1, 0.)?;
                *at_mut(&mut g.sec_vertex, isec, "SecVertex")? = sec.vertex(isec, 0.)?.into_shape();
            }
        }
        Ok(g)
    }

    /// The start and the length of the domain of the first section law.
    pub(super) fn section_domain(&self) -> Result<(f64, f64), Failure> {
        let first_law = self.sec()?.law(1)?;
        let (sec_deb, sec_end) = first_law
            .get()
            .ok_or_else(|| null("section law"))?
            .get_domain()?;
        Ok((sec_deb, sec_end - sec_deb))
    }

    /// The section parameter at the end of path `index`: proportional to the
    /// curvilinear abscissa of its end on the whole path.
    pub(super) fn section_parameter(
        &self,
        index: i32,
        (sec_deb, sec_dom): (f64, f64),
    ) -> Result<f64, Failure> {
        let (_, length) = self.loc()?.curvilinear_bounds(self.loc()?.nb_law()?)?;
        let (_, ll) = self.loc()?.curvilinear_bounds(index)?;
        Ok(sec_deb + (ll / length) * sec_dom)
    }

    /// (1.2) The surfaces swept by every section law along every path (with the section
    /// domain of an evolutive section), extended at the first and last paths; false
    /// when a sweep fails.
    fn shell_surfaces(
        &mut self,
        g: &mut ShellGrid,
        extend_first: f64,
        extend_last: f64,
    ) -> Result<bool, Failure> {
        let domain = self.section_domain()?;
        for (ipath, i_path) in (1..=g.nb_path).zip(g.ifirst..) {
            let law = self.loc()?.law(i_path)?;
            let mut sweep = GeomFillSweep::new(&law, self.k_part)?;
            sweep.set_tolerance(
                self.my_tol3d,
                self.my_bound_tol,
                self.my_tol2d,
                self.my_tol_angular,
            );
            sweep.set_force_approx_c1(self.my_force_approx_c1);
            let (lf, ll) = law
                .get()
                .ok_or_else(|| null("location law"))?
                .get_domain()?;
            *at_mut(&mut g.vi, ipath + 1, "Vi")? = self.section_parameter(i_path, domain)?;
            if !g.const_section {
                sweep.set_domain(
                    lf,
                    ll,
                    *at(&g.vi, ipath, "Vi")?,
                    *at(&g.vi, ipath + 1, "Vi")?,
                );
            }
            for isec in 1..=g.nb_law {
                if !self.sweep_one(g, &mut sweep, (isec, ipath), (extend_first, extend_last))? {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    /// The surface of section `isec` along path `ipath` stored in the grid (extended
    /// before the first path or after the last one); false when the sweep fails.
    fn sweep_one(
        &mut self,
        g: &mut ShellGrid,
        sweep: &mut GeomFillSweep,
        (isec, ipath): (i32, i32),
        (extend_first, extend_last): (f64, f64),
    ) -> Result<bool, Failure> {
        sweep.build(
            &self.sec()?.law(isec)?,
            self.my_approx_style,
            self.my_continuity,
            self.my_degmax,
            self.my_segmax,
        )?;
        if !sweep.is_done() {
            return Ok(false);
        }
        *cell_mut(&mut g.tab_s, isec, ipath)? = sweep.surface();
        *cell_mut(&mut g.tab_err, isec, ipath)? = sweep.error_on_surface();
        *cell_mut(&mut g.exch_uv, isec, ipath)? = sweep.exchange_uv();
        *cell_mut(&mut g.u_reverse, isec, ipath)? = sweep.u_reversed();
        if sweep.error_on_surface() > self.error {
            self.error = sweep.error_on_surface();
        }
        if ipath == 1 && extend_first > 0. {
            extend_surface(
                cell_mut(&mut g.tab_s, isec, ipath)?,
                extend_first,
                sweep.exchange_uv(),
                false,
            )?;
        }
        if ipath == g.nb_path && extend_last > 0. {
            extend_surface(
                cell_mut(&mut g.tab_s, isec, ipath)?,
                extend_last,
                sweep.exchange_uv(),
                true,
            )?;
        }
        Ok(true)
    }
}

/// The surface (cast to a bounded surface, null when it is not one) extended by a
/// length in U or V, before its start or after its end, in C1.
fn extend_surface(
    surface: &mut Handle<dyn GeomSurface>,
    length: f64,
    in_u: bool,
    after: bool,
) -> Result<(), Failure> {
    let mut bnd_s: Handle<dyn GeomBoundedSurface> = surface.cast();
    geom_lib::extend_surf_by_length(&mut bnd_s, length, 1, in_u, after)?;
    *surface = bnd_s.cast();
    Ok(())
}
