// Rust translation of BRepFill_Sweep::Build
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Sweep.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The construction of the result of a sweep: the shells between the holes of the
//! path, their corners, the substitution of the modified edges and the closure.
use super::helpers::translate;
use super::same_param::correct_same_parameter;
use super::topology::substitute;
use super::{BRepFillSweep, SweepHistory, null};
use crate::brep_fill::shape_table::{ShapeTable, cell, cell_mut, new_table};
use crate::brep_fill::transition_style::TransitionStyle;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::Substitution;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Shell};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeomalgo::geom_fill::GeomFillApproxStyle;
use rustycad_tkmath::geom_abs::Shape as Continuity;

/// The options of `Build` (natively defaulted to a modified transition, C2, the
/// approximation by location, degree 11 and 30 segments).
#[derive(Clone, Copy, Debug)]
pub struct SweepBuildOptions {
    /// The treatment of the discontinuities of the path.
    pub transition: TransitionStyle,
    /// The continuity of the approximated surfaces.
    pub continuity: Continuity,
    /// The approximation style.
    pub approx: GeomFillApproxStyle,
    /// The largest degree of the approximation.
    pub degmax: i32,
    /// The largest segment count of the approximation.
    pub segmax: i32,
}

impl Default for SweepBuildOptions {
    fn default() -> Self {
        Self {
            transition: TransitionStyle::Modified,
            continuity: Continuity::C2,
            approx: GeomFillApproxStyle::Location,
            degmax: 11,
            segmax: 30,
        }
    }
}

/// Whether a shape is a (non-null) face.
fn is_face(shape: &Shape) -> Result<bool, Failure> {
    Ok(!shape.is_null() && shape.shape_type()? == ShapeEnum::Face)
}

/// Column `in_col` of a table copied into its column `out_col`.
fn translate_in_place(table: &mut ShapeTable, in_col: i32, out_col: i32) -> Result<(), Failure> {
    for ii in 1..=table.upper_row() {
        let value: Shape = cell(table, ii, in_col)?.clone();
        *cell_mut(table, ii, out_col)? = value;
    }
    Ok(())
}

impl BRepFillSweep {
    /// `Build(ReversedEdges, Tapes, Rails, Transition, Continuity, Approx, Degmax,
    /// Segmax)`: the wire of a vertex section, else the shell of the faces of the
    /// sweep (built between the holes of the path, with their corners);
    /// `sweep.build(&mut history, &SweepBuildOptions::default())?`.
    pub fn build(
        &mut self,
        hist: &mut SweepHistory,
        options: &SweepBuildOptions,
    ) -> Result<(), Failure> {
        self.my_continuity = options.continuity;
        self.my_approx_style = options.approx;
        self.my_degmax = options.degmax;
        self.my_segmax = options.segmax;
        self.correct_approx_parameters()?;
        if self.sec()?.is_vertex() {
            self.is_done = self.build_wire(options.transition)?;
            return Ok(());
        }
        self.build_sweep_shell(hist, options.transition)
    }

    /// The tables and tapes of the shell, the holes of the path and the shells between
    /// them (with their corners), then the shell of the faces.
    fn build_sweep_shell(
        &mut self,
        hist: &mut SweepHistory,
        transition: TransitionStyle,
    ) -> Result<(), Failure> {
        let nb_trous: i32 = self.loc()?.nb_holes(self.my_tol3d)?;
        let (nb_path, nb_law) = (self.loc()?.nb_law()?, self.sec()?.nb_law()?);
        self.my_u_edges = Some(new_table((1, nb_law + 1), (1, nb_path))?);
        self.my_v_edges = Some(new_table((1, nb_law), (1, nb_path + 1))?);
        self.my_faces = Some(new_table((1, nb_law), (1, nb_path))?);
        let builder = BRepBuilder::default();
        let mut tapes: Vec<Shape> = Vec::new();
        for _ in 1..=nb_law {
            let mut a_shell = Shell::default();
            builder.make_shell(&mut a_shell);
            tapes.push(a_shell.into_shape());
        }
        self.my_tapes = Some(tapes);
        let mut bounds: ShapeTable = new_table((1, nb_law), (1, 2))?;
        let mut trous: Vec<i32> = vec![0; nb_trous.max(0) as usize];
        let mut nb_part: i32 = 1;
        if nb_trous > 0 {
            self.loc()?.holes(&mut trous);
            nb_part += nb_trous;
            if trous.last() == Some(&(nb_path + 1)) {
                nb_part -= 1;
            }
        }
        if !self.build_parts(hist, transition, (&trous, nb_part), &mut bounds)? {
            return Ok(());
        }
        if nb_trous > 0 && self.loc()?.is_closed()? && trous.last() == Some(&(nb_path + 1)) {
            let v_edges: &mut ShapeTable = self.v_edges_mut()?;
            translate(v_edges, nb_path + 1, &mut bounds, 1)?;
            translate(v_edges, 1, &mut bounds, 2)?;
            if !self.perform_corner(1, transition, &bounds)? {
                self.is_done = false;
                return Ok(());
            }
            translate_in_place(self.v_edges_mut()?, 1, nb_path + 1)?;
        }
        self.assemble_shell(nb_trous, (nb_path, nb_law))
    }

    /// The shells between the holes (at once when there is a single part, extended at
    /// a single hole); false when a corner fails (the sweep is then not done).
    fn build_parts(
        &mut self,
        hist: &mut SweepHistory,
        transition: TransitionStyle,
        (trous, nb_part): (&[i32], i32),
        bounds: &mut ShapeTable,
    ) -> Result<bool, Failure> {
        let nb_path: i32 = self.loc()?.nb_law()?;
        if nb_part == 1 {
            let extend: f64 = if trous.len() == 1 {
                self.eval_extrapol(1, transition)?
            } else {
                0.
            };
            self.is_done =
                self.build_shell(transition, (1, nb_path + 1), hist, (extend, extend))?;
            return Ok(true);
        }
        let mut ifirst: i32 = 1;
        self.is_done = true;
        let mut ii: i32 = 1;
        while ii <= nb_part && self.is_done {
            let ilast: i32 = if ii as usize > trous.len() {
                nb_path + 1
            } else {
                trous[(ii - 1) as usize]
            };
            let extend = (
                self.eval_extrapol(ifirst, transition)?,
                self.eval_extrapol(ilast, transition)?,
            );
            self.is_done = self.build_shell(transition, (ifirst, ilast), hist, extend)?;
            if ifirst > 1 {
                translate(
                    self.my_v_edges
                        .as_ref()
                        .ok_or_else(|| null("v-edge table"))?,
                    ifirst,
                    bounds,
                    2,
                )?;
                if !self.perform_corner(ifirst, transition, bounds)? {
                    self.is_done = false;
                    return Ok(false);
                }
            }
            ifirst = ilast;
            translate(
                self.my_v_edges
                    .as_ref()
                    .ok_or_else(|| null("v-edge table"))?,
                ifirst,
                bounds,
                1,
            )?;
            ii += 1;
        }
        Ok(true)
    }

    /// The shell of the faces and auxiliary faces with the modified v-edges
    /// substituted, the tables updated, the u-edges made SameParameter, the tapes
    /// filled and the closure checked; the sweep is not done without enough faces.
    fn assemble_shell(
        &mut self,
        nb_trous: i32,
        (nb_path, nb_law): (i32, i32),
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let mut shell = Shell::default();
        builder.make_shell(&mut shell);
        let mut a_nb_faces: i32 = 0;
        for ipath in 1..=nb_path {
            for isec in 1..=nb_law {
                let face: Shape = cell(self.faces_mut()?, isec, ipath)?.clone();
                if is_face(&face)? {
                    builder.add(&mut shell, &face)?;
                    a_nb_faces += 1;
                }
            }
        }
        if if nb_trous > 0 {
            a_nb_faces < nb_law
        } else {
            a_nb_faces == 0
        } {
            self.is_done = false;
            return Ok(());
        }
        for face in self.my_aux_shape.iter() {
            if is_face(face)? {
                builder.add(&mut shell, face)?;
            }
        }
        let shell: Shape = self.substitute_modified_edges(shell.into_shape())?;
        self.ensure_same_parameter()?;
        for ii in 1..=nb_law {
            for jj in 1..=nb_path {
                let a_face: Shape = cell(self.faces_mut()?, ii, jj)?.clone();
                if is_face(&a_face)? {
                    let tapes = self.my_tapes.as_mut().ok_or_else(|| null("tape array"))?;
                    builder.add(&mut tapes[(ii - 1) as usize], &a_face)?;
                }
            }
        }
        if self.loc()?.is_closed()? && self.sec()?.is_u_closed() {
            shell.set_closed(closed_shell(&shell)?)?;
        }
        self.my_shape = shell;
        Ok(())
    }

    /// The v-edges modified by the faces substituted in the shell and in the tables of
    /// faces, v-edges and u-edges.
    fn substitute_modified_edges(&mut self, shell: Shape) -> Result<Shape, Failure> {
        let mut a_substitute = Substitution::new();
        // The native order is the bucket order of TShape addresses: with modified edges
        // sharing vertices, which vertex survives depends on it.
        for (old_edge, new_edge) in self.my_v_edges_modified.items() {
            substitute(
                &mut a_substitute,
                &topods::edge(old_edge)?,
                &topods::edge(new_edge)?,
            )?;
        }
        a_substitute.build(&shell)?;
        let mut shell = shell;
        if a_substitute.is_copied(&shell) {
            shell = topods::shell(&a_substitute.copy(&shell)?[0])?.into_shape();
        }
        substitute_in_table(&a_substitute, self.faces_mut()?)?;
        substitute_in_table(&a_substitute, self.v_edges_mut()?)?;
        substitute_in_table(&a_substitute, self.u_edges_mut()?)?;
        Ok(shell)
    }

    /// (Ensure Same Parameter on U-edges) the tolerance of every u-edge raised to the
    /// distance to its curves on the faces on both sides (the closing row of a closed
    /// section is skipped).
    fn ensure_same_parameter(&mut self) -> Result<(), Failure> {
        let u_closed: bool = self.sec()?.is_u_closed();
        let u_edges: ShapeTable = crate::brep_fill::shape_table::clone_table(self.u_edges_mut()?);
        let faces: &ShapeTable = self.my_faces.as_ref().ok_or_else(|| null("face table"))?;
        for ii in u_edges.lower_row()..=u_edges.upper_row() {
            if u_closed && ii == u_edges.upper_row() {
                continue;
            }
            for jj in u_edges.lower_col()..=u_edges.upper_col() {
                let mut an_edge: Edge = topods::edge(cell(&u_edges, ii, jj)?)?;
                if an_edge.is_null() || BRepTool::degenerated(&an_edge)? {
                    continue;
                }
                let (mut i1, mut i2) = (ii - 1, ii);
                if i1 == 0 && u_closed {
                    i1 = faces.upper_row();
                }
                if i2 > faces.upper_row() {
                    i2 = 0;
                }
                let (face1, face2) = (side_face(faces, i1, jj)?, side_face(faces, i2, jj)?);
                if !face1.is_null() && !face2.is_null() {
                    correct_same_parameter(&mut an_edge, &face1, &face2)?;
                }
            }
        }
        Ok(())
    }
}

/// The face of the table at `(row, col)`, null for row 0 or a slot that is not a face.
fn side_face(faces: &ShapeTable, row: i32, col: i32) -> Result<Face, Failure> {
    if row == 0 {
        return Ok(Face::default());
    }
    let a_shape: &Shape = cell(faces, row, col)?;
    if a_shape.shape_type()? == ShapeEnum::Face {
        return topods::face(a_shape);
    }
    Ok(Face::default())
}

/// Every shape of the table replaced by the first of its copies in the substitution.
fn substitute_in_table(a_substitute: &Substitution, table: &mut ShapeTable) -> Result<(), Failure> {
    for ii in table.lower_row()..=table.upper_row() {
        for jj in table.lower_col()..=table.upper_col() {
            let a_local_shape: Shape = cell(table, ii, jj)?.clone();
            if a_local_shape.is_null() || !a_substitute.is_copied(&a_local_shape) {
                continue;
            }
            if let Some(first) = a_substitute.copy(&a_local_shape)?.first() {
                *cell_mut(table, ii, jj)? = first.clone();
            }
        }
    }
    Ok(())
}

/// Whether every non-degenerated edge of the shell bounds more than one face.
fn closed_shell(shell: &Shape) -> Result<bool, Failure> {
    let mut ef_map = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(shell, ShapeEnum::Edge, ShapeEnum::Face, &mut ef_map)?;
    for iedge in 1..=ef_map.extent() {
        let the_edge: Edge = topods::edge(ef_map.find_key(iedge)?)?;
        if BRepTool::degenerated(&the_edge)? {
            continue;
        }
        if ef_map.find_from_index(iedge)?.extent() <= 1 {
            return Ok(false);
        }
    }
    Ok(true)
}
