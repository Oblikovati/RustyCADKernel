// Rust translation of BRepFill_PipeShell::Build, MakeSolid and Simulate and of the static
// PerformPlan, IsSameOriented and BuildBoundaries (ModelingAlgorithms/TKBool/BRepFill/
// BRepFill_PipeShell.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The construction of a pipe shell: the sweep between its first and last sections,
//! the closing faces of a solid and the simulated sections.
use super::{BRepFillPipeShell, null};
use crate::brep_fill::shape_table::{ShapeTable, cell};
use crate::brep_fill::sweep::{BRepFillSweep, SweepBuildOptions, SweepHistory, SweepTablesMap};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape, Shell, Solid, Wire};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkgeomalgo::geom_fill::{GeomFillApproxStyle, GeomFillPipeError, GeomFillTrihedron};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPICopy, BRepBuilderAPIMakeShape};
use rustycad_tktopalgo::brep_class3d::BRepClass3dSolidClassifier;
use rustycad_tktopalgo::brep_lib::BRepLibMakeFace;

/// `PerformPlan(S)`: the plane face of the wire (true), nothing for a wire of
/// degenerated edges only (true, the shape nullified), false when no plane is found.
fn perform_plan(s: &mut Shape) -> Result<bool, Failure> {
    let mut is_degen: bool = true;
    for e in Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if !BRepTool::degenerated(&topods::edge(&e)?)? {
            is_degen = false;
        }
    }
    if is_degen {
        *s = Shape::default();
        return Ok(true);
    }
    let w: Wire = topods::wire(s)?;
    if w.is_null() {
        return Ok(false);
    }
    let mkplan = BRepLibMakeFace::from_wire(&w, true)?;
    if !mkplan.is_done() {
        return Ok(false);
    }
    *s = mkplan.face()?.into_shape();
    Ok(true)
}

/// `IsSameOriented(theFace, theShell)`: whether the first edge of the face shared
/// with the shell runs against its use in the first adjacent face (true when none is
/// shared).
fn is_same_oriented(the_face: &Shape, the_shell: &Shape) -> Result<bool, Failure> {
    let mut an_edge_face_map = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(
        the_shell,
        ShapeEnum::Edge,
        ShapeEnum::Face,
        &mut an_edge_face_map,
    )?;
    for a_current_edge in Explorer::new(the_face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let Some(faces) = an_edge_face_map.seek(&a_current_edge) else {
            continue;
        };
        let Some(an_adj_face) = faces.first() else {
            continue;
        };
        for an_adj_edge in Explorer::new(an_adj_face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if an_adj_edge.is_same(&a_current_edge) {
                return Ok(a_current_edge.orientation() != an_adj_edge.orientation());
            }
        }
    }
    Ok(true)
}

/// `BuildBoundaries(theSweep, theSection, theBottom, theTop)`: the wires of the first
/// and last v-edges of the sweep (the top one is the bottom one when all are the same);
/// `None` when neither has an edge.
fn build_boundaries(
    the_sweep: &BRepFillSweep,
    nb_law: i32,
    u_closed: bool,
    (bottom, top): (&Shape, &Shape),
) -> Result<Option<(Shape, Shape)>, Failure> {
    let builder = BRepBuilder::default();
    let (mut a_bottom_wire, mut a_top_wire) = (Wire::default(), Wire::default());
    builder.make_wire(&mut a_bottom_wire);
    builder.make_wire(&mut a_top_wire);
    let (mut bfoundbottom, mut bfoundtop, mut b_all_same) = (false, false, true);
    let handle = the_sweep.sections();
    let shared = handle.get().ok_or_else(|| null("section table"))?;
    let a_v_edges: &ShapeTable = &shared.array2();
    for i in 1..=nb_law {
        let a_bottom_edge: &Shape = cell(a_v_edges, i, a_v_edges.lower_col())?;
        if !a_bottom_edge.is_null() && a_bottom_edge.shape_type()? == ShapeEnum::Edge {
            builder.add(&mut a_bottom_wire, a_bottom_edge)?;
            bfoundbottom = true;
        }
        let a_top_edge: &Shape = cell(a_v_edges, i, a_v_edges.upper_col())?;
        if !a_top_edge.is_null() && a_top_edge.shape_type()? == ShapeEnum::Edge {
            builder.add(&mut a_top_wire, a_top_edge)?;
            bfoundtop = true;
        }
        if !a_bottom_edge.is_null() && !a_top_edge.is_null() && !a_bottom_edge.is_same(a_top_edge) {
            b_all_same = false;
        }
    }
    if u_closed {
        a_bottom_wire.set_closed(true)?;
        a_top_wire.set_closed(true)?;
    }
    if !(bfoundbottom || bfoundtop) {
        return Ok(None);
    }
    let the_bottom: Shape = if bfoundbottom {
        a_bottom_wire.into_shape()
    } else {
        bottom.clone()
    };
    let mut the_top: Shape = if bfoundtop {
        a_top_wire.into_shape()
    } else {
        top.clone()
    };
    if b_all_same && bfoundbottom && bfoundtop {
        the_top = the_bottom.clone();
    }
    Ok(Some((the_bottom, the_top)))
}

/// Whether every edge of the shape is degenerated (true without edge).
fn all_edges_degenerated(s: &Shape) -> Result<bool, Failure> {
    let mut degen: bool = true;
    for e in Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)? {
        degen = degen && BRepTool::degenerated(&topods::edge(&e)?)?;
    }
    Ok(degen)
}

/// A new empty shell (the native nullified result).
fn empty_shell() -> Shape {
    let mut sh = Shell::default();
    BRepBuilder::default().make_shell(&mut sh);
    sh.into_shape()
}

impl BRepFillPipeShell {
    /// `Build()`: the sweep of the sections along the spine between the first and last
    /// sections (their wires then taken from the sweep), closed when both ends are
    /// degenerated, with its history; false (an empty shell) when the laws or the sweep
    /// fail; `pipe.build()?`.
    pub fn build(&mut self) -> Result<bool, Failure> {
        self.prepare()?;
        if self.my_status != GeomFillPipeError::PipeOk {
            self.my_shape = empty_shell();
            return Ok(false);
        }
        self.compute_first_and_last()?;
        let mut mk_sw = BRepFillSweep::new(&self.my_section, &self.my_location, true);
        mk_sw.set_tolerance(
            self.my_tol3d,
            self.my_bound_tol,
            1.0e-5,
            self.my_tol_angular,
        );
        mk_sw.set_angular_control(self.angmin, self.angmax);
        mk_sw.set_force_approx_c1(self.my_force_approx_c1);
        mk_sw.set_bounds(
            &topods::wire(&self.my_first)?,
            &topods::wire(&self.my_last)?,
        )?;
        let continuity = if self.my_trihedron == GeomFillTrihedron::IsDiscreteTrihedron {
            Continuity::C0
        } else {
            Continuity::C2
        };
        let (mut dummy, mut dummy2, mut dummy3) = (
            MapOfShape::default(),
            SweepTablesMap::default(),
            SweepTablesMap::default(),
        );
        let mut hist = SweepHistory {
            reversed_edges: &mut dummy,
            tapes: &mut dummy2,
            rails: &mut dummy3,
        };
        let options = SweepBuildOptions {
            transition: self.my_transition,
            continuity,
            approx: GeomFillApproxStyle::Location,
            degmax: self.my_max_degree,
            segmax: self.my_max_segments,
        };
        mk_sw.build(&mut hist, &options)?;
        self.my_status = self.location()?.get_status()?;
        let ok: bool = mk_sw.is_done() && self.my_status == GeomFillPipeError::PipeOk;
        if !ok {
            self.my_shape = empty_shell();
            if self.my_status == GeomFillPipeError::PipeOk {
                self.my_status = GeomFillPipeError::PipeNotOk;
            }
            return Ok(false);
        }
        self.take_sweep(&mk_sw)?;
        Ok(true)
    }

    /// The first section placed at the start of the spine and the last one at its end
    /// (the same as the first on a closed G1 loop, none on a closed loop with a corner);
    /// a last section sharing its TShapes with the first one is copied (occ332).
    fn compute_first_and_last(&mut self) -> Result<(), Failure> {
        let first_law = self.section()?.law(1)?;
        let (first_s, last_s) = first_law
            .get()
            .ok_or_else(|| null("section law"))?
            .get_domain()?;
        let first_section: Shape = self.section()?.d0(first_s)?;
        self.my_first = self.location()?.d0(0., &first_section)?;
        if self.section()?.is_v_closed() && self.location()?.is_closed()? {
            if self.location()?.is_g1(0, 1.0e-7, 1.0e-4)? >= 0 {
                self.my_last = self.my_first.clone();
            } else {
                self.my_first = Shape::default();
                self.my_last = Shape::default();
            }
            return Ok(());
        }
        let (_, length) = self
            .location()?
            .curvilinear_bounds(self.location()?.nb_law()?)?;
        let last_section: Shape = self.section()?.d0(last_s)?;
        self.my_last = self.location()?.d0(length, &last_section)?;
        if self.my_first.is_partner(&self.my_last) {
            let mut copy = BRepBuilderAPICopy::with_shape(&self.my_last, true, false)?;
            if copy.is_done() {
                self.my_last = copy.shape()?.clone();
            }
        }
        Ok(())
    }

    /// The result of a successful sweep: the shell, its error, the boundary wires, the
    /// closure of a closed section with degenerated ends and the history.
    fn take_sweep(&mut self, mk_sw: &BRepFillSweep) -> Result<(), Failure> {
        self.my_shape = mk_sw.shape();
        self.my_error_on_surf = mk_sw.error_on_surface();
        let (nb_law, u_closed) = (self.section()?.nb_law()?, self.section()?.is_u_closed());
        if let Some((bottom, top)) =
            build_boundaries(mk_sw, nb_law, u_closed, (&self.my_first, &self.my_last))?
        {
            self.my_first = bottom;
            self.my_last = top;
        }
        if u_closed
            && all_edges_degenerated(&self.my_first)?
            && all_edges_degenerated(&self.my_last)?
        {
            self.my_shape.set_closed(true)?;
        }
        if self.my_is_build_history {
            self.build_history(mk_sw)?;
        }
        Ok(())
    }

    /// `MakeSolid()`: the solid of the shell, its open ends closed by plane faces
    /// (oriented as the shell), reversed when its infinite point is inside;
    /// StdFail_NotDone before `build`; false when the ends cannot be closed;
    /// `pipe.make_solid()?`.
    pub fn make_solid(&mut self) -> Result<bool, Failure> {
        if self.my_shape.is_null() {
            return Err(Failure::with_type(
                "StdFail_NotDone",
                "PipeShell is not built",
                "BRepFill_PipeShell",
            ));
        }
        let mut b: bool = self.my_shape.closed()?;
        if !b {
            b = self.close_ends()?;
        }
        if b {
            self.solid_of_shell()?;
        }
        Ok(b)
    }

    /// The ends of an open shell closed by their plane faces; false when an end is not
    /// closed or not planar.
    fn close_ends(&mut self) -> Result<bool, Failure> {
        if self.my_first.is_null()
            || self.my_last.is_null()
            || !(self.my_first.closed()? && self.my_last.closed()?)
        {
            return Ok(false);
        }
        if !perform_plan(&mut self.my_first)? || !perform_plan(&mut self.my_last)? {
            return Ok(false);
        }
        for end in [&mut self.my_first, &mut self.my_last] {
            if !end.is_null() && !is_same_oriented(end, &self.my_shape)? {
                end.reverse();
            }
        }
        let builder = BRepBuilder::default();
        for end in [self.my_first.clone(), self.my_last.clone()] {
            if !end.is_null() {
                let face = topods::face(&end)?;
                builder.add(&mut self.my_shape, &face)?;
            }
        }
        self.my_shape.set_closed(true)?;
        Ok(true)
    }

    /// The solid of the closed shell, reversed when its infinite point is inside.
    fn solid_of_shell(&mut self) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let mut solid = Solid::default();
        builder.make_solid(&mut solid);
        let shell = topods::shell(&self.my_shape)?;
        builder.add(&mut solid, &shell)?;
        let mut sc = BRepClass3dSolidClassifier::with_shape(&solid)?;
        sc.perform_infinite_point(precision::confusion())?;
        if sc.state() == State::In {
            builder.make_solid(&mut solid);
            self.my_shape.reverse();
            let shell = topods::shell(&self.my_shape)?;
            builder.add(&mut solid, &shell)?;
        }
        self.my_shape = solid.into_shape();
        self.my_shape.set_closed(true)
    }

    /// `Simulate(NumberOfSection, Sections)`: `n` sections placed along the spine at
    /// equal abscissas (an edge is not skipped: a step past its end falls back to its
    /// middle); `let sections = pipe.simulate(5)?;`.
    pub fn simulate(&mut self, n: i32) -> Result<List<Shape>, Failure> {
        self.prepare()?;
        let mut list: List<Shape> = List::default();
        let nb_l: i32 = self.location()?.nb_law()?;
        let first_law = self.section()?.law(1)?;
        let (first_s, last) = first_law
            .get()
            .ok_or_else(|| null("section law"))?
            .get_domain()?;
        let delta_s: f64 = last - first_s;
        let (_, length) = self.location()?.curvilinear_bounds(nb_l)?;
        let delta: f64 = if n > 1 {
            length / f64::from(n - 1)
        } else {
            length
        };
        let (mut first, mut last) = self.location()?.curvilinear_bounds(1)?;
        let (mut u, mut ii, mut finis) = (0.0_f64, 1, false);
        while !finis {
            if u >= length {
                u = length;
                finis = true;
            } else {
                if ii < nb_l {
                    (first, last) = self.location()?.curvilinear_bounds(nb_l)?;
                }
                if u > last {
                    u = (last + first) / 2.;
                }
                if u > first {
                    ii += 1;
                }
            }
            let us: f64 = first_s + (u / length) * delta_s;
            let w: Shape = self.section()?.d0(us)?;
            list.append(self.location()?.d0(u, &w)?);
            u += delta;
        }
        Ok(list)
    }
}
