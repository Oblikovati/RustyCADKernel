// Rust translation of ModelingAlgorithms/TKBool/BRepFill/BRepFill_Pipe.hxx/.cxx (the
// constructors, Perform, the accessors and the queries) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFill_Pipe`: the pipe of a profile (vertex, edge, wire, face, shell or compound)
//! along a spine wire, with the shapes generated from both.
mod find;
mod history;
mod make_shape;

use super::location_law::{BRepFillEdge3DLaw, BRepFillLocationLaw};
use super::section_law::{BRepFillSectionLaw, BRepFillShapeLaw};
use super::section_placement::BRepFillSectionPlacement;
use super::shape_table::{ShapeTable, cell};
use super::sweep::{BRepFillSweep, SweepBuildOptions, SweepHistory, SweepTablesMap};
use super::transition_style::TransitionStyle;
use super::trim_shell_corner::HistoryMap;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{self, Compound, Edge, Face, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve, GeomOffsetCurve, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeomalgo::geom_fill::{
    GeomFillApproxStyle, GeomFillCorrectedFrenet, GeomFillCurveAndTrihedron,
    GeomFillDiscreteTrihedron, GeomFillFrenet, GeomFillLocationLaw, GeomFillTrihedron,
    GeomFillTrihedronLaw, LocationD0,
};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Point3, Transform3};
use rustycad_tkmath::toploc::Location;
use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeRemoveLocations;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPITransform};
use rustycad_tktopalgo::brep_lib::BRepLibMakeVertex;

/// The pipe of a profile along a spine (`BRepFill_Pipe`);
/// `let pipe = BRepFillPipe::with_profile(&spine, &profile, GeomFillTrihedron::IsCorrectedFrenet, false, false)?;`.
pub struct BRepFillPipe {
    my_spine: Wire,
    my_profile: Shape,
    my_shape: Shape,
    my_trsf: Transform3,
    my_loc: Handle<dyn BRepFillLocationLaw>,
    my_sections: Option<ShapeTable>,
    my_faces: Option<ShapeTable>,
    my_edges: Option<ShapeTable>,
    my_reversed_edges: MapOfShape,
    my_tapes: SweepTablesMap,
    my_rails: SweepTablesMap,
    my_cur_index_of_section_edge: i32,
    my_first: Shape,
    my_last: Shape,
    my_gen_map: HistoryMap,
    my_degmax: i32,
    my_segmax: i32,
    my_continuity: Continuity,
    my_mode: GeomFillTrihedron,
    my_force_approx_c1: bool,
    my_error_on_surf: f64,
}

/// Standard_DomainError of the queries.
fn domain_error(message: &str) -> Failure {
    Failure::with_type("Standard_DomainError", message, "BRepFill_Pipe")
}

/// Standard_NullObject for a table or a law that is not built (a crash natively).
pub(super) fn null(what: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("BRepFill_Pipe: null {what}; expected a {what}"),
        "BRepFill_Pipe",
    )
}

/// The placement of a location law at a parameter as a transformation.
fn placement_trsf(law: &dyn GeomFillLocationLaw, param: f64) -> Result<Transform3, Failure> {
    let mut d0 = LocationD0::default();
    law.d0(param, &mut d0)?;
    let (m, v) = (d0.m, d0.v);
    let mut fila = Transform3::default();
    fila.set_values([
        [m.value(1, 1)?, m.value(1, 2)?, m.value(1, 3)?, v.x()],
        [m.value(2, 1)?, m.value(2, 2)?, m.value(2, 3)?, v.y()],
        [m.value(3, 1)?, m.value(3, 2)?, m.value(3, 3)?, v.z()],
    ])?;
    Ok(fila)
}

impl Default for BRepFillPipe {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepFillPipe {
    /// `BRepFill_Pipe()`: degree 11, 100 segments, C2, the corrected Frenet trihedron;
    /// `BRepFillPipe::new()`.
    pub fn new() -> Self {
        Self {
            my_spine: Wire::default(),
            my_profile: Shape::default(),
            my_shape: Shape::default(),
            my_trsf: Transform3::default(),
            my_loc: Handle::default(),
            my_sections: None,
            my_faces: None,
            my_edges: None,
            my_reversed_edges: MapOfShape::default(),
            my_tapes: SweepTablesMap::default(),
            my_rails: SweepTablesMap::default(),
            my_cur_index_of_section_edge: 1,
            my_first: Shape::default(),
            my_last: Shape::default(),
            my_gen_map: HistoryMap::default(),
            my_degmax: 11,
            my_segmax: 100,
            my_continuity: Continuity::C2,
            my_mode: GeomFillTrihedron::IsCorrectedFrenet,
            my_force_approx_c1: false,
            my_error_on_surf: 0.,
        }
    }

    /// `BRepFill_Pipe(Spine, Profile, aMode, ForceApproxC1, GeneratePartCase)`: the
    /// Frenet, corrected Frenet (any other mode) or discrete (in C0) trihedron, then
    /// `perform`.
    pub fn with_profile(
        spine: &Wire,
        profile: &Shape,
        a_mode: GeomFillTrihedron,
        force_approx_c1: bool,
        k_part: bool,
    ) -> Result<Self, Failure> {
        let mut pipe = Self::new();
        if matches!(
            a_mode,
            GeomFillTrihedron::IsFrenet
                | GeomFillTrihedron::IsCorrectedFrenet
                | GeomFillTrihedron::IsDiscreteTrihedron
        ) {
            pipe.my_mode = a_mode;
        }
        if pipe.my_mode == GeomFillTrihedron::IsDiscreteTrihedron {
            pipe.my_continuity = Continuity::C0;
        }
        pipe.my_force_approx_c1 = force_approx_c1;
        pipe.perform(spine, profile, k_part)?;
        Ok(pipe)
    }

    /// The location law; Standard_NullObject before `perform`.
    fn loc(&self) -> Result<&dyn BRepFillLocationLaw, Failure> {
        self.my_loc.get().ok_or_else(|| null("location law"))
    }

    /// `Perform(Spine, Profile, GeneratePartCase)`: the location law of the spine (none
    /// for a degenerated spine), the profile placed at its start, the first and last
    /// sections, and the swept shape; `pipe.perform(&spine, &profile, false)?`.
    pub fn perform(&mut self, spine: &Wire, profile: &Shape, _k_part: bool) -> Result<(), Failure> {
        (self.my_sections, self.my_faces, self.my_edges) = (None, None, None);
        self.my_spine = spine.clone();
        self.my_profile = profile.clone();
        self.define_real_segmax()?;
        let tlaw: Handle<dyn GeomFillTrihedronLaw> = match self.my_mode {
            GeomFillTrihedron::IsFrenet => Handle::new(GeomFillFrenet::new()).cast(),
            GeomFillTrihedron::IsCorrectedFrenet => {
                Handle::new(GeomFillCorrectedFrenet::new()).cast()
            }
            GeomFillTrihedron::IsDiscreteTrihedron => {
                Handle::new(GeomFillDiscreteTrihedron::new()).cast()
            }
            _ => Handle::default(),
        };
        let loc: Handle<dyn GeomFillLocationLaw> =
            Handle::new(GeomFillCurveAndTrihedron::new(&tlaw)).cast();
        self.my_loc = Handle::new(BRepFillEdge3DLaw::new(&self.my_spine, &loc)?).cast();
        if self.loc()?.nb_law()? == 0 {
            return Ok(());
        }
        self.loc()?.transform_in_g0_law()?;
        let place = BRepFillSectionPlacement::new(&self.my_loc, profile, false, false)?;
        self.my_trsf = *place.transformation();
        let mut the_prof: Shape = self.my_profile.clone();
        the_prof.set_location(
            &Location::new(self.my_trsf).multiplied(profile.location())?,
            false,
        )?;
        self.place_ends()?;
        let (first, last) = (self.my_first.clone(), self.my_last.clone());
        self.my_shape = self.make_shape(&the_prof, &profile.clone(), &first, &last)?;
        Ok(())
    }

    /// The profile placed at the start and at the end of the spine (copies, without
    /// locations; the last one is the first one on a closed spine with the same
    /// placement).
    fn place_ends(&mut self) -> Result<(), Failure> {
        let first_law = self.loc()?.law(1)?;
        let law1 = first_law.get().ok_or_else(|| null("location law"))?;
        let mut fila: Transform3 = placement_trsf(law1, law1.get_domain()?.0)?;
        fila.multiply(self.my_trsf);
        let loc_first = Location::new(fila);
        self.my_first = self.transformed_profile(&loc_first, fila)?;
        let rem_loc = ShapeUpgradeRemoveLocations::new();
        rem_loc.set_remove_level(ShapeEnum::Compound);
        rem_loc.remove(&self.my_first)?;
        self.my_first = rem_loc.get_result();
        let last_law = self.loc()?.law(self.loc()?.nb_law()?)?;
        let law_n = last_law.get().ok_or_else(|| null("location law"))?;
        let mut fila: Transform3 = placement_trsf(law_n, law_n.get_domain()?.1)?;
        fila.multiply(self.my_trsf);
        let loc_last = Location::new(fila);
        self.my_last = if !self.loc()?.is_closed()? || loc_first != loc_last {
            self.transformed_profile(&loc_last, fila)?
        } else {
            self.my_first.clone()
        };
        rem_loc.remove(&self.my_last)?;
        self.my_last = rem_loc.get_result();
        Ok(())
    }

    /// The profile, or its transformed copy when the location is not the identity.
    fn transformed_profile(&self, loc: &Location, fila: Transform3) -> Result<Shape, Failure> {
        if loc.is_identity() {
            return Ok(self.my_profile.clone());
        }
        let mut transform =
            BRepBuilderAPITransform::with_shape(&self.my_profile, fila, true, false)?;
        Ok(transform.shape()?.clone())
    }

    /// `Spine()`; `pipe.spine()`.
    pub fn spine(&self) -> &Wire {
        &self.my_spine
    }

    /// `Profile()`; `pipe.profile()`.
    pub fn profile(&self) -> &Shape {
        &self.my_profile
    }

    /// `Shape()`; `pipe.shape()`.
    pub fn shape(&self) -> &Shape {
        &self.my_shape
    }

    /// `ErrorOnSurface()`: the error of the last sweep (0 before one);
    /// `pipe.error_on_surface()`.
    pub fn error_on_surface(&self) -> f64 {
        self.my_error_on_surf
    }

    /// `FirstShape()`; `pipe.first_shape()`.
    pub fn first_shape(&self) -> &Shape {
        &self.my_first
    }

    /// `LastShape()`; `pipe.last_shape()`.
    pub fn last_shape(&self) -> &Shape {
        &self.my_last
    }

    /// `Generated(S, L)`: the shapes generated from a shape of the profile or of the
    /// spine (empty when none); `let faces = pipe.generated(&edge);`.
    pub fn generated(&self, s: &Shape) -> List<Shape> {
        self.my_gen_map.seek(s).cloned().unwrap_or_default()
    }

    /// The index of the spine edge in the location law; Standard_DomainError when it is
    /// not in the spine.
    fn spine_index(&self, e_spine: &Edge) -> Result<i32, Failure> {
        for ii in 1..=self.loc()?.nb_law()? {
            if e_spine.is_same(self.loc()?.edge(ii)?.as_ref()) {
                return Ok(ii);
            }
        }
        Err(domain_error("BRepFill_Pipe::Edge  : Edge not in the Spine"))
    }

    /// `Face(ESpine, EProfile)`: the face swept by the profile edge along the spine edge
    /// (null for a degenerated profile edge); Standard_DomainError when an edge is not
    /// in the profile or the spine; `pipe.face(&spine_edge, &profile_edge)?`.
    pub fn face(&self, e_spine: &Edge, e_profile: &Edge) -> Result<Face, Failure> {
        if BRepTool::degenerated(e_profile)? {
            return Ok(Face::default());
        }
        let mut count: i32 = 0;
        let iprof: i32 = self.find_edge(&self.my_profile, e_profile, &mut count)?;
        if iprof == 0 {
            return Err(domain_error(
                "BRepFill_Pipe::Face : Edge not in the Profile",
            ));
        }
        let ispin: i32 = self.spine_index(e_spine)?;
        topods::face(cell(
            self.my_faces.as_ref().ok_or_else(|| null("face table"))?,
            iprof,
            ispin,
        )?)
    }

    /// `Edge(ESpine, VProfile)`: the edge swept by the profile vertex along the spine
    /// edge; Standard_DomainError when the vertex is not in the profile or the edge not
    /// in the spine; `pipe.edge(&spine_edge, &profile_vertex)?`.
    pub fn edge(&self, e_spine: &Edge, v_profile: &Vertex) -> Result<Edge, Failure> {
        let mut count: i32 = 0;
        let iprof: i32 = self.find_vertex(&self.my_profile, v_profile, &mut count)?;
        if iprof == 0 {
            return Err(domain_error(
                "BRepFill_Pipe::Edge : Vertex not in the Profile",
            ));
        }
        let ispin: i32 = self.spine_index(e_spine)?;
        topods::edge(cell(
            self.my_edges.as_ref().ok_or_else(|| null("edge table"))?,
            iprof,
            ispin,
        )?)
    }

    /// `Section(VSpine)`: the compound of the section shapes at the spine vertex;
    /// Standard_DomainError when the vertex is not in the spine;
    /// `pipe.section(&spine_vertex)?`.
    pub fn section(&self, v_spine: &Vertex) -> Result<Shape, Failure> {
        let mut ispin: i32 = 0;
        for ii in 1..=self.loc()?.nb_law()? + 1 {
            if v_spine.is_same(self.loc()?.vertex(ii)?.as_ref()) {
                ispin = ii;
                break;
            }
        }
        if ispin == 0 {
            return Err(domain_error(
                "BRepFill_Pipe::Section  : Vertex not in the Spine",
            ));
        }
        let sections = self
            .my_sections
            .as_ref()
            .ok_or_else(|| null("section table"))?;
        let builder = BRepBuilder::default();
        let mut comp = Compound::default();
        builder.make_compound(&mut comp);
        for ii in 1..=sections.col_length() {
            builder.add(&mut comp, cell(sections, ii, ispin)?)?;
        }
        Ok(comp.into_shape())
    }

    /// `PipeLine(Point)`: the wire swept by the point (placed with the profile);
    /// `pipe.pipe_line(Point3::new(0.1, 0.0, 0.0))?`.
    pub fn pipe_line(&mut self, point: Point3) -> Result<Wire, Failure> {
        let p: Point3 = point.transformed(self.my_trsf);
        let vertex_section: Vertex = BRepLibMakeVertex::new(p)?.vertex()?;
        let section: Handle<dyn BRepFillSectionLaw> =
            Handle::new(BRepFillShapeLaw::with_vertex(&vertex_section, true)?).cast();
        let mk_sw = self.swept(&section, None)?;
        let a_local_shape: Shape = mk_sw.shape();
        self.my_error_on_surf = mk_sw.error_on_surface();
        self.build_history(&mk_sw, vertex_section.as_ref())?;
        topods::wire(&a_local_shape)
    }

    /// The sweep of a section law along the spine (bounded by the first and last
    /// wires when given), with the pipe settings and history.
    fn swept(
        &mut self,
        section: &Handle<dyn BRepFillSectionLaw>,
        bounds: Option<(&Shape, &Shape)>,
    ) -> Result<BRepFillSweep, Failure> {
        let mut mk_sw = BRepFillSweep::new(section, &self.my_loc, true);
        if let Some((first, last)) = bounds {
            mk_sw.set_bounds(&topods::wire(first)?, &topods::wire(last)?)?;
        }
        mk_sw.set_force_approx_c1(self.my_force_approx_c1);
        let mut hist = SweepHistory {
            reversed_edges: &mut self.my_reversed_edges,
            tapes: &mut self.my_tapes,
            rails: &mut self.my_rails,
        };
        let options = SweepBuildOptions {
            transition: TransitionStyle::Modified,
            continuity: self.my_continuity,
            approx: GeomFillApproxStyle::Location,
            degmax: self.my_degmax,
            segmax: self.my_segmax,
        };
        mk_sw.build(&mut hist, &options)?;
        Ok(mk_sw)
    }

    /// `DefineRealSegmax()` (private): the segment count raised to the knot spans of
    /// the B-spline spine edges (through trimmed and offset curves).
    fn define_real_segmax(&mut self) -> Result<(), Failure> {
        let mut real_segmax: i32 = 0;
        let mut iter = ShapeIterator::new(&self.my_spine, true, true);
        while iter.more() {
            let e: Edge = topods::edge(iter.value()?)?;
            iter.next();
            let (mut c, first, last) = BRepTool::curve(&e)?;
            if c.is_null() {
                continue;
            }
            c = basis_of(c);
            let bc: Handle<GeomBSplineCurve> = c.cast();
            let Some(bc) = bc.get() else {
                continue;
            };
            real_segmax += real_knot_spans(bc, first, last)?;
        }
        if self.my_segmax < real_segmax {
            self.my_segmax = real_segmax;
        }
        Ok(())
    }
}

/// The basis of a curve through its trimmed and offset layers.
fn basis_of(mut c: Handle<dyn GeomCurve>) -> Handle<dyn GeomCurve> {
    loop {
        let trimmed: Handle<GeomTrimmedCurve> = c.cast();
        if let Some(t) = trimmed.get() {
            c = t.basis_curve();
            continue;
        }
        let offset: Handle<GeomOffsetCurve> = c.cast();
        if let Some(o) = offset.get() {
            c = o.basis_curve();
            continue;
        }
        return c;
    }
}

/// The knot spans of the B-spline between the parameters.
fn real_knot_spans(bc: &GeomBSplineCurve, first: f64, last: f64) -> Result<i32, Failure> {
    let nb_knots: i32 = bc.nb_knots();
    let mut real_nb_knots: i32 = nb_knots;
    if first > bc.first_parameter() {
        let (i1, _) = bc.locate_u(first, precision::p_confusion(), false)?;
        real_nb_knots -= i1 - 1;
    }
    if last < bc.last_parameter() {
        let (_, i2) = bc.locate_u(last, precision::p_confusion(), false)?;
        real_nb_knots -= nb_knots - i2;
    }
    Ok(real_nb_knots - 1)
}
