// Rust translation of ModelingAlgorithms/TKBool/BRepFill/BRepFill_Draft.hxx/.cxx (the
// constructor, the options, the Perform variants, Init, BuildShell and the accessors) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFill_Draft`: the draft surface of a wire (or the free border of a face or a
//! shell) swept along a direction at an angle, up to a length, a surface or a shape.
mod fuse;
mod helpers;
mod sewing;

use super::location_law::{BRepFillDraftLaw, BRepFillLocationLaw};
use super::section_law::{BRepFillSectionLaw, BRepFillShapeLaw};
use super::shape_table::{ShapeTable, cell, copy_table};
use super::sweep::{BRepFillSweep, SweepBuildOptions, SweepHistory, SweepTablesMap};
use super::transition_style::TransitionStyle;
use helpers::{compute_trsf, good_orientation, longueur};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_tools::uv_bounds;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Shape, ShapeIterator, Shell, Wire};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{
    GeomCurve, GeomLine, GeomPlane, GeomRectangularTrimmedSurface, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom_fill::GeomFillLocationDraft;
use rustycad_tkgeombase::bnd_lib::AddSurface;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Vector3};
use rustycad_tktopalgo::brep_lib::{
    BRepLibMakeEdge, BRepLibMakeFace, BRepLibMakeWire, BRepLibWireError,
};
use std::f64::consts::PI;

/// The draft surface of a wire swept along a direction (`BRepFill_Draft`);
/// `let mut draft = BRepFillDraft::new(&wire, Direction3::Z, 0.1)?; draft.perform_length(5.0)?;`.
pub struct BRepFillDraft {
    my_dir: Direction3,
    my_angle: f64,
    angmin: f64,
    angmax: f64,
    my_tol: f64,
    my_loc: Handle<BRepFillDraftLaw>,
    my_sec: Handle<dyn BRepFillSectionLaw>,
    my_sections: Option<ShapeTable>,
    my_faces: Option<ShapeTable>,
    my_generated: List<Shape>,
    my_shape: Shape,
    my_top: Shape,
    my_shell: Shell,
    my_wire: Wire,
    my_cont: Continuity,
    my_style: TransitionStyle,
    is_internal: bool,
    my_done: bool,
}

/// Standard_ConstructionError of the constructor.
fn construction_error() -> Failure {
    Failure::with_type(
        "Standard_ConstructionError",
        "BRepFill_Draft",
        "BRepFill_Draft",
    )
}

/// Standard_NullObject for a law or a table that is not built (a crash natively).
pub(super) fn null(what: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("BRepFill_Draft: null {what}; expected a {what}"),
        "BRepFill_Draft",
    )
}

/// The wire of the free (non-degenerated, one face) edges of a shell; Standard_
/// ConstructionError when there is none or they do not make a wire.
fn free_border(s: &Shape) -> Result<Wire, Failure> {
    let mut edgemap = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(s, ShapeEnum::Edge, ShapeEnum::Face, &mut edgemap)?;
    let mut list: List<Shape> = List::default();
    for iedge in 1..=edgemap.extent() {
        let the_edge: Edge = topods::edge(edgemap.find_key(iedge)?)?;
        if !BRepTool::degenerated(&the_edge)? && edgemap.find_from_index(iedge)?.extent() == 1 {
            list.append(the_edge.into_shape());
        }
    }
    if list.is_empty() {
        return Err(construction_error());
    }
    let mut mw = BRepLibMakeWire::new();
    mw.add_list(&list)?;
    if mw.error() != BRepLibWireError::WireDone {
        return Err(construction_error());
    }
    mw.wire()
}

impl BRepFillDraft {
    /// `BRepFill_Draft(Shape, Dir, Angle)`: the wire (the outer wire of a face, the free
    /// border of a shell), marked closed when its ends are the same vertex; the options
    /// right corner, angular control 0.01 and 3, external draft; Standard_
    /// ConstructionError for another shape.
    pub fn new(s: &Shape, dir: Direction3, angle: f64) -> Result<Self, Failure> {
        let my_wire: Wire = match s.shape_type()? {
            ShapeEnum::Wire => topods::wire(s)?,
            ShapeEnum::Face => topods::wire(ShapeIterator::new(s, true, true).value()?)?,
            ShapeEnum::Shell => free_border(s)?,
            _ => return Err(construction_error()),
        };
        if !my_wire.closed()? {
            let (vf, vl) = topexp::wire_vertices(&my_wire);
            if vf.is_same(&vl) {
                my_wire.set_closed(true)?;
            }
        }
        let mut draft = Self {
            my_dir: dir,
            my_angle: angle.abs(),
            angmin: 0.,
            angmax: 0.,
            my_tol: 1.0e-4,
            my_loc: Handle::default(),
            my_sec: Handle::default(),
            my_sections: None,
            my_faces: None,
            my_generated: List::default(),
            my_shape: Shape::default(),
            my_top: s.clone(),
            my_shell: Shell::default(),
            my_wire,
            my_cont: Continuity::C1,
            my_style: TransitionStyle::Right,
            is_internal: false,
            my_done: false,
        };
        draft.set_options(TransitionStyle::Right, 0.01, 3.0);
        draft.set_draft(false);
        Ok(draft)
    }

    /// `SetOptions(Style, AngleMin, AngleMax)` (natively defaulted to a right corner,
    /// 0.01 and 3); `draft.set_options(TransitionStyle::Round, 0.01, 3.0)`.
    pub fn set_options(&mut self, style: TransitionStyle, min: f64, max: f64) {
        self.my_style = style;
        self.angmin = min;
        self.angmax = max;
    }

    /// `SetDraft(IsInternal)` (natively defaulted to false): the draft goes inside the
    /// wire; `draft.set_draft(true)`.
    pub fn set_draft(&mut self, internal: bool) {
        self.is_internal = internal;
    }

    /// `Perform(LengthMax)`: the draft surface of the given length;
    /// `draft.perform_length(5.0)?`.
    pub fn perform_length(&mut self, length_max: f64) -> Result<(), Failure> {
        let (w_box, _) = compute_trsf(&self.my_wire, self.my_dir)?;
        self.init(length_max, &w_box)?;
        self.build_shell(&Handle::default(), false)?;
        self.sewing()?;
        Ok(())
    }

    /// `Perform(Surface, KeepInsideSurface)` (natively defaulted to keep inside): the
    /// draft surface up to the surface, cut by it;
    /// `draft.perform_surface(&surface, true)?`.
    pub fn perform_surface(
        &mut self,
        surface: &Handle<dyn GeomSurface>,
        keep_inside_surface: bool,
    ) -> Result<(), Failure> {
        let (w_box, trsf) = compute_trsf(&self.my_wire, self.my_dir)?;
        let geometry = surface
            .get()
            .ok_or_else(|| null("stop surface"))?
            .transformed(trsf)?;
        let surf: Handle<dyn GeomSurface> = geometry.cast();
        let s1 = GeomAdaptorSurface::with_surface(&surf)?;
        let mut s_box = BoundingBox3::default();
        AddSurface::add(&s1 as &dyn Adaptor3dSurface, 0.1, &mut s_box)?;
        let (l, _) = longueur(&w_box, &s_box, &mut self.my_dir)?;
        let l: f64 = l / self.my_angle.cos().abs();
        self.init(l, &w_box)?;
        self.build_shell(surface, !keep_inside_surface)?;
        self.sewing()?;
        Ok(())
    }

    /// `Perform(StopShape, KeepOutSide)` (natively defaulted to keep outside): the draft
    /// surface up to a plane past the shape, then fused with the shape;
    /// `draft.perform_shape(&stop, true)?`.
    pub fn perform_shape(
        &mut self,
        stop_shape: &Shape,
        keep_out_side: bool,
    ) -> Result<(), Failure> {
        let (w_box, trsf) = compute_trsf(&self.my_wire, self.my_dir)?;
        let mut s_box = BoundingBox3::default();
        let mut b_surf = BoundingBox3::default();
        for face in Explorer::new(stop_shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            let face = topods::face(&face)?;
            let (umin, umax, vmin, vmax) = uv_bounds(&face)?;
            let geometry = BRepTool::surface(&face)?
                .get()
                .ok_or_else(|| null("face surface"))?
                .transformed(trsf)?;
            let surf: Handle<dyn GeomSurface> = geometry.cast();
            let s1 = GeomAdaptorSurface::with_surface(&surf)?;
            AddSurface::add_range(
                &s1 as &dyn Adaptor3dSurface,
                [umin, umax, vmin, vmax],
                0.1,
                &mut b_surf,
            )?;
            s_box.add_box(&b_surf);
        }
        let (l, mut pt) = longueur(&w_box, &s_box, &mut self.my_dir)?;
        let l: f64 = l / self.my_angle.cos().abs();
        pt = pt.transformed(trsf.inverted()?);
        let plan: Handle<dyn GeomSurface> =
            Handle::new(GeomPlane::from_point(pt, self.my_dir)?).cast();
        let surf: Handle<dyn GeomSurface> = Handle::new(GeomRectangularTrimmedSurface::new(
            &plan,
            (-l, l),
            (-l, l),
            (true, true),
        )?)
        .cast();
        self.init(l * 1.01, &w_box)?;
        self.build_shell(&surf, false)?;
        self.fuse(stop_shape, keep_out_side)?;
        self.sewing()?;
        Ok(())
    }

    /// The location law.
    fn loc(&self) -> Result<&BRepFillDraftLaw, Failure> {
        self.my_loc.get().ok_or_else(|| null("draft law"))
    }

    /// `Init(Surf, Length, Box)` (private): the draft law (its angle reversed for an
    /// internal draft of a well oriented wire or the reverse), cleaned of small
    /// discontinuities, and the section law of a straight generating line of the length
    /// along the binormal side of the direction.
    fn init(&mut self, length: f64, bbox: &BoundingBox3) -> Result<(), Failure> {
        let loc = Handle::new(GeomFillLocationDraft::new(self.my_dir, self.my_angle)?);
        self.my_loc = Handle::new(BRepFillDraftLaw::new(&self.my_wire, &loc)?);
        let law_handle: Handle<dyn BRepFillLocationLaw> = self.my_loc.clone().cast();
        let b: bool = good_orientation(bbox, &law_handle, self.my_dir)?;
        if self.is_internal ^ !b {
            self.my_angle = -self.my_angle;
            loc.get()
                .ok_or_else(|| null("draft location"))?
                .set_angle(self.my_angle)?;
            self.my_loc = Handle::new(BRepFillDraftLaw::new(&self.my_wire, &loc)?);
        }
        self.loc()?.clean_law(self.angmin)?;
        let mut d = Vector3::new(0., 1., 0.);
        let first_law = self.loc()?.law(1)?;
        let law1 = first_law.get().ok_or_else(|| null("draft law"))?;
        let (f, l) = law1.get_domain()?;
        let mut placement = rustycad_tkgeomalgo::geom_fill::LocationD0::default();
        law1.d0((f + l) / 2., &mut placement)?;
        let bn = Direction3::from_vector(Vector3::from(placement.m.column(2)?))?;
        if self.my_dir.angle(&bn) > PI / 2. {
            d.reverse();
        }
        let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::new(
            Point3::new(0., 0., 0.),
            Direction3::from_vector(d)?,
        )))
        .cast();
        let tc: Handle<dyn GeomCurve> =
            Handle::new(GeomTrimmedCurve::new(&line, 0., length, true, true)?).cast();
        let eg: Edge = BRepLibMakeEdge::from_curve(&tc)?.edge()?;
        let g: Wire = BRepLibMakeWire::from_edge(&eg)?.wire()?;
        self.my_sec = Handle::new(BRepFillShapeLaw::with_wire(&g, true)?).cast();
        Ok(())
    }

    /// `BuildShell(Surf, KeepOutSide)` (private): the sweep of the generating line along
    /// the draft law (oriented outward, or inward for an internal draft), then fused with
    /// the face of the stop surface when there is one.
    fn build_shell(
        &mut self,
        surf: &Handle<dyn GeomSurface>,
        keep_out_side: bool,
    ) -> Result<(), Failure> {
        let loc: Handle<dyn BRepFillLocationLaw> = self.my_loc.clone().cast();
        let mut sweep = BRepFillSweep::new(&self.my_sec, &loc, true);
        sweep.set_tolerance(self.my_tol, 1.0, 1.0e-5, 1.0e-2);
        sweep.set_angular_control(self.angmin, self.angmax);
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
            transition: self.my_style,
            continuity: self.my_cont,
            ..SweepBuildOptions::default()
        };
        sweep.build(&mut hist, &options)?;
        if !sweep.is_done() {
            self.my_done = false;
            return Ok(());
        }
        self.my_shape = sweep.shape();
        self.my_shell = topods::shell(&self.my_shape)?;
        self.my_faces = Some(copy_table(&sweep.sub_shape(), "BRepFill_Draft")?);
        self.my_sections = Some(copy_table(&sweep.sections(), "BRepFill_Draft")?);
        self.my_done = true;
        self.orient_outward()?;
        if !surf.is_null() {
            let mut mk_f = BRepLibMakeFace::new();
            mk_f.init_surface(surf, true, precision::confusion())?;
            self.fuse(mk_f.face()?.as_ref(), keep_out_side)?;
        }
        Ok(())
    }

    /// The shell reversed when the normal of its first face at its first parameters
    /// goes the way of the draft (outward drafts) or the other way (internal ones).
    fn orient_outward(&mut self) -> Result<(), Failure> {
        let ex = Explorer::new(&self.my_shell, ShapeEnum::Face, ShapeEnum::Shape)?;
        let f = topods::face(ex.current())?;
        let sf = BRepAdaptorSurface::with_face(&f, true)?;
        let (u, v) = (sf.first_u_parameter()?, sf.first_v_parameter()?);
        let d1 = sf.d1(u, v)?;
        let mut normal: Vector3 = d1.d1u.crossed(d1.d1v);
        if f.orientation() == Orientation::Reversed {
            normal.reverse();
        }
        let mut out: bool = true;
        if normal.magnitude() > 1.0e-10 {
            out = self.my_dir.angle(&Direction3::from_vector(normal)?) > PI / 2.;
        }
        if out == self.is_internal {
            self.my_shell.reverse();
            self.my_shape.reverse();
        }
        Ok(())
    }

    /// `IsDone()`; `draft.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_done
    }

    /// `Shell()`: the draft surface; `draft.shell()`.
    pub fn shell(&self) -> Shell {
        self.my_shell.clone()
    }

    /// `Shape()`: the draft surface (sewn to the input face or shell, a solid when
    /// closed); `draft.shape()`.
    pub fn shape(&self) -> Shape {
        self.my_shape.clone()
    }

    /// `Generated(S)`: the face of an edge of the wire, the section of a vertex (looked
    /// for only when `s` is null, as natively: a vertex is a Standard_TypeMismatch);
    /// `let faces = draft.generated(&edge)?;`.
    pub fn generated(&mut self, s: &Shape) -> Result<&List<Shape>, Failure> {
        self.my_generated.clear();
        let e: Edge = topods::edge(s)?;
        let nb_law: i32 = self.loc()?.nb_law()?;
        if e.is_null() {
            for ii in 0..=nb_law {
                if e.is_same(self.loc()?.vertex(ii)?.as_ref()) {
                    let sections = self
                        .my_sections
                        .as_ref()
                        .ok_or_else(|| null("section table"))?;
                    self.my_generated.append(cell(sections, 1, ii + 1)?.clone());
                    break;
                }
            }
            return Ok(&self.my_generated);
        }
        for ii in 1..=nb_law {
            if e.is_same(self.loc()?.edge(ii)?.as_ref()) {
                let faces = self.my_faces.as_ref().ok_or_else(|| null("face table"))?;
                self.my_generated.append(cell(faces, 1, ii)?.clone());
                break;
            }
        }
        Ok(&self.my_generated)
    }
}
