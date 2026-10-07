// Rust translation of ModelingAlgorithms/TKBool/BRepFill/BRepFill_PipeShell.hxx/.cxx (the
// constructor, the laws of location, the settings and the accessors) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFill_PipeShell`: the sweep of one or several profiles along a spine, with the
//! trihedron law chosen among the location modes.
mod build;
mod history;
mod prepare;
mod profiles;

use super::location_law::{
    BRepFillACRLaw, BRepFillEdge3DLaw, BRepFillEdgeOnSurfLaw, BRepFillLocationLaw,
};
use super::section::BRepFillSection;
use super::section_law::BRepFillSectionLaw;
use super::transition_style::{TransitionStyle, TypeOfContact};
use super::trim_shell_corner::HistoryMap;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCompCurve;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Shape, Wire};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeomalgo::geom_fill::{
    GeomFillConstantBiNormal, GeomFillCorrectedFrenet, GeomFillCurveAndTrihedron,
    GeomFillDiscreteTrihedron, GeomFillFixed, GeomFillFrenet, GeomFillGuideTrihedronAC,
    GeomFillGuideTrihedronPlan, GeomFillLocationGuide, GeomFillLocationLaw, GeomFillPipeError,
    GeomFillTrihedron, GeomFillTrihedronLaw, GeomFillTrihedronWithGuide,
};
use rustycad_tkgeomalgo::law::LawFunction;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3, Vector3};

/// The sweep of profiles along a spine (`BRepFill_PipeShell`);
/// `let mut pipe = BRepFillPipeShell::new(&spine)?; pipe.add(&profile, false, false)?;`.
pub struct BRepFillPipeShell {
    parent: StandardTransient,
    my_spine: Wire,
    my_first: Shape,
    my_last: Shape,
    my_shape: Shape,
    my_seq: Vec<BRepFillSection>,
    w_seq: Vec<Shape>,
    my_ind_of_sec: Vec<i32>,
    my_edge_new_edges: HistoryMap,
    my_gen_map: HistoryMap,
    my_tol3d: f64,
    my_bound_tol: f64,
    my_tol_angular: f64,
    angmin: f64,
    angmax: f64,
    my_max_degree: i32,
    my_max_segments: i32,
    my_force_approx_c1: bool,
    my_law: Handle<dyn LawFunction>,
    my_is_automatic_law: bool,
    my_location: Handle<dyn BRepFillLocationLaw>,
    my_section: Handle<dyn BRepFillSectionLaw>,
    my_trihedron: GeomFillTrihedron,
    my_transition: TransitionStyle,
    my_status: GeomFillPipeError,
    my_error_on_surf: f64,
    my_is_build_history: bool,
}
impl_transient!(
    BRepFillPipeShell,
    "BRepFill_PipeShell",
    StandardTransient,
    parent
);

/// Standard_NullObject for a law that is not set (a crash natively).
pub(super) fn null(what: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("BRepFill_PipeShell: null {what}; expected a {what}"),
        "BRepFill_PipeShell",
    )
}

/// The Edge3DLaw of the spine on a curve-and-trihedron law.
fn trihedron_law(
    spine: &Wire,
    tlaw: Handle<dyn GeomFillTrihedronLaw>,
) -> Result<Handle<dyn BRepFillLocationLaw>, Failure> {
    let loc: Handle<dyn GeomFillLocationLaw> =
        Handle::new(GeomFillCurveAndTrihedron::new(&tlaw)).cast();
    Ok(Handle::new(BRepFillEdge3DLaw::new(spine, &loc)?).cast())
}

impl BRepFillPipeShell {
    /// `BRepFill_PipeShell(Spine)`: the corrected Frenet trihedron, the modified
    /// transition, the tolerances 1e-4, 1e-4 and 1e-2, degree 11 and 100 segments; a
    /// spine whose ends are the same vertex is marked closed. The angular control is 0
    /// until SetTransition (it is uninitialized natively).
    pub fn new(spine: &Wire) -> Result<Self, Failure> {
        let mut pipe = Self {
            parent: StandardTransient::default(),
            my_spine: spine.clone(),
            my_first: Shape::default(),
            my_last: Shape::default(),
            my_shape: Shape::default(),
            my_seq: Vec::new(),
            w_seq: Vec::new(),
            my_ind_of_sec: Vec::new(),
            my_edge_new_edges: HistoryMap::default(),
            my_gen_map: HistoryMap::default(),
            my_tol3d: 0.,
            my_bound_tol: 0.,
            my_tol_angular: 0.,
            angmin: 0.,
            angmax: 0.,
            my_max_degree: 11,
            my_max_segments: 100,
            my_force_approx_c1: false,
            my_law: Handle::default(),
            my_is_automatic_law: false,
            my_location: Handle::default(),
            my_section: Handle::default(),
            my_trihedron: GeomFillTrihedron::IsCorrectedFrenet,
            my_transition: TransitionStyle::Modified,
            my_status: GeomFillPipeError::PipeOk,
            my_error_on_surf: 0.,
            my_is_build_history: true,
        };
        pipe.set_tolerance(1.0e-4, 1.0e-4, 1.0e-2);
        // Attention to a closed wire that is not declared closed.
        if !pipe.my_spine.closed()? {
            let (vf, vl) = topexp::wire_vertices(&pipe.my_spine);
            if vf.is_same(&vl) {
                pipe.my_spine.set_closed(true)?;
            }
        }
        Ok(pipe)
    }

    /// The location law; Standard_NullObject before one is built.
    pub(super) fn location(&self) -> Result<&dyn BRepFillLocationLaw, Failure> {
        self.my_location.get().ok_or_else(|| null("location law"))
    }

    /// The section law; Standard_NullObject before one is built.
    pub(super) fn section(&self) -> Result<&dyn BRepFillSectionLaw, Failure> {
        self.my_section.get().ok_or_else(|| null("section law"))
    }

    /// `Set(Frenet)` (natively defaulted to false): the Frenet or corrected Frenet
    /// trihedron (both recorded as Frenet, as natively); the sections are relocated.
    pub fn set_frenet(&mut self, is_frenet: bool) -> Result<(), Failure> {
        self.my_trihedron = GeomFillTrihedron::IsFrenet;
        let tlaw: Handle<dyn GeomFillTrihedronLaw> = if is_frenet {
            Handle::new(GeomFillFrenet::new()).cast()
        } else {
            Handle::new(GeomFillCorrectedFrenet::new()).cast()
        };
        self.my_location = trihedron_law(&self.my_spine, tlaw)?;
        self.my_section = Handle::default();
        Ok(())
    }

    /// `SetDiscrete()`: the discrete trihedron; `pipe.set_discrete()?`.
    pub fn set_discrete(&mut self) -> Result<(), Failure> {
        self.my_trihedron = GeomFillTrihedron::IsDiscreteTrihedron;
        let tlaw: Handle<dyn GeomFillTrihedronLaw> =
            Handle::new(GeomFillDiscreteTrihedron::new()).cast();
        self.my_location = trihedron_law(&self.my_spine, tlaw)?;
        self.my_section = Handle::default();
        Ok(())
    }

    /// `Set(Axe)`: the fixed trihedron of the frame (its main and X directions);
    /// `pipe.set_axe(&frame)?`.
    pub fn set_axe(&mut self, axe: &Axis2) -> Result<(), Failure> {
        self.my_trihedron = GeomFillTrihedron::IsFixed;
        let v1 = Vector3::from(axe.direction().xyz());
        let v2 = Vector3::from(axe.x_direction().xyz());
        let tlaw: Handle<dyn GeomFillTrihedronLaw> =
            Handle::new(GeomFillFixed::new(v1, v2)?).cast();
        self.my_location = trihedron_law(&self.my_spine, tlaw)?;
        self.my_section = Handle::default();
        Ok(())
    }

    /// `Set(BiNormal)`: the trihedron of constant binormal;
    /// `pipe.set_bi_normal(Direction3::Z)?`.
    pub fn set_bi_normal(&mut self, bi_normal: Direction3) -> Result<(), Failure> {
        self.my_trihedron = GeomFillTrihedron::IsConstantNormal;
        let tlaw: Handle<dyn GeomFillTrihedronLaw> =
            Handle::new(GeomFillConstantBiNormal::new(bi_normal)).cast();
        self.my_location = trihedron_law(&self.my_spine, tlaw)?;
        self.my_section = Handle::default();
        Ok(())
    }

    /// `Set(SpineSupport)`: the Darboux trihedron of the spine on its support; false
    /// (nothing changed) when the spine is not on it; `pipe.set_spine_support(&face)?`.
    pub fn set_spine_support(&mut self, spine_support: &Shape) -> Result<bool, Failure> {
        let loc = BRepFillEdgeOnSurfLaw::new(&self.my_spine, spine_support)?;
        let b: bool = loc.has_result();
        if b {
            self.my_location = Handle::new(loc).cast();
            self.my_trihedron = GeomFillTrihedron::IsDarboux;
            self.my_section = Handle::default();
        }
        Ok(b)
    }

    /// `Set(AuxiliarySpine, CurvilinearEquivalence, KeepContact)` (natively defaulted
    /// to true and no contact): the trihedron guided by the auxiliary spine (reoriented
    /// for open wires, its origin searched when it is closed), by curvilinear abscissa
    /// or by planes, with or without contact;
    /// `pipe.set_auxiliary_spine(&guide, true, TypeOfContact::NoContact)?`.
    pub fn set_auxiliary_spine(
        &mut self,
        auxiliary_spine: &Wire,
        curvilinear_equivalence: bool,
        keep_contact: TypeOfContact,
    ) -> Result<(), Failure> {
        let the_guide: Wire = self.reorganized_guide(auxiliary_spine, keep_contact)?;
        let guide: Handle<dyn Adaptor3dCurve> =
            Handle::new(BRepAdaptorCompCurve::with_wire(&the_guide, false)?).cast();
        let with_contact: bool = matches!(
            keep_contact,
            TypeOfContact::Contact | TypeOfContact::ContactOnBorder
        );
        if curvilinear_equivalence {
            self.my_trihedron = if with_contact {
                GeomFillTrihedron::IsGuideACWithContact
            } else {
                GeomFillTrihedron::IsGuideAC
            };
            let tlaw: Handle<dyn GeomFillTrihedronWithGuide> =
                Handle::new(GeomFillGuideTrihedronAC::new(&guide)?).cast();
            let loc = Handle::new(GeomFillLocationGuide::new(&tlaw)?);
            self.my_location = Handle::new(BRepFillACRLaw::new(&self.my_spine, &loc)?).cast();
        } else {
            self.my_trihedron = if with_contact {
                GeomFillTrihedron::IsGuidePlanWithContact
            } else {
                GeomFillTrihedron::IsGuidePlan
            };
            let tlaw: Handle<dyn GeomFillTrihedronWithGuide> =
                Handle::new(GeomFillGuideTrihedronPlan::new(&guide)?).cast();
            let loc: Handle<dyn GeomFillLocationLaw> =
                Handle::new(GeomFillLocationGuide::new(&tlaw)?).cast();
            self.my_location = Handle::new(BRepFillEdge3DLaw::new(&self.my_spine, &loc)?).cast();
        }
        self.my_section = Handle::default();
        Ok(())
    }

    /// The guide reoriented as the spine (open wires, through CompatibleWires) or with
    /// its origin near the spine start (closed guide); the automatic law is set for a
    /// contact on the border.
    fn reorganized_guide(
        &mut self,
        auxiliary_spine: &Wire,
        keep_contact: TypeOfContact,
    ) -> Result<Wire, Failure> {
        let mut the_guide: Wire = auxiliary_spine.clone();
        let (sp_close, guide_close) = (self.my_spine.closed()?, auxiliary_spine.closed()?);
        if keep_contact == TypeOfContact::ContactOnBorder {
            self.my_is_automatic_law = true;
        }
        if !sp_close && !guide_close {
            let mut cw = super::compatible_wires::BRepFillCompatibleWires::with_sections(&[
                self.my_spine.as_ref().clone(),
                the_guide.as_ref().clone(),
            ]);
            cw.set_percent(0.1);
            cw.perform(true)?;
            if !cw.is_done() {
                return Err(Failure::with_type(
                    "StdFail_NotDone",
                    "Uncompatible wires",
                    "BRepFill_PipeShell",
                ));
            }
            return rustycad_tkbrep::topods::wire(&cw.shape()[1]);
        }
        if guide_close {
            let (sp_or, dir) = self.spine_origin(sp_close)?;
            super::package::search_origin(&mut the_guide, sp_or, dir, 100. * self.my_tol3d)?;
        }
        Ok(the_guide)
    }

    /// The point and direction at the origin of the spine: the middle of an open spine
    /// with the direction from its last to its first vertex, else the start of the
    /// closed spine with its tangent.
    fn spine_origin(&self, sp_close: bool) -> Result<(Point3, Vector3), Failure> {
        if sp_close {
            let bc = BRepAdaptorCompCurve::with_wire(&self.my_spine, false)?;
            return bc.d1(0.);
        }
        let (vf, vl) = topexp::wire_vertices(&self.my_spine);
        let mut sp_or: Point3 = BRepTool::pnt(&vf)?;
        let p: Point3 = BRepTool::pnt(&vl)?;
        let v = Vector3::between(p, sp_or);
        sp_or.barycenter(0.5, p, 0.5);
        Ok((sp_or, v))
    }

    /// `SetMaxDegree(NewMaxDegree)`; `pipe.set_max_degree(8)`.
    pub fn set_max_degree(&mut self, new_max_degree: i32) {
        self.my_max_degree = new_max_degree;
    }

    /// `SetMaxSegments(NewMaxSegments)`; `pipe.set_max_segments(30)`.
    pub fn set_max_segments(&mut self, new_max_segments: i32) {
        self.my_max_segments = new_max_segments;
    }

    /// `SetForceApproxC1(ForceApproxC1)`; `pipe.set_force_approx_c1(true)`.
    pub fn set_force_approx_c1(&mut self, force_approx_c1: bool) {
        self.my_force_approx_c1 = force_approx_c1;
    }

    /// `SetIsBuildHistory(theIsBuildHistory)`; `pipe.set_is_build_history(false)`.
    pub fn set_is_build_history(&mut self, the_is_build_history: bool) {
        self.my_is_build_history = the_is_build_history;
    }

    /// `IsBuildHistory()`; `pipe.is_build_history()`.
    pub fn is_build_history(&self) -> bool {
        self.my_is_build_history
    }

    /// `IsReady()`: at least one profile; `pipe.is_ready()`.
    pub fn is_ready(&self) -> bool {
        !self.my_seq.is_empty()
    }

    /// `GetStatus()`; `pipe.get_status()`.
    pub fn get_status(&self) -> GeomFillPipeError {
        self.my_status
    }

    /// `SetTolerance(Tol3d, BoundTol, TolAngular)` (natively defaulted to 1e-4, 1e-4
    /// and 1e-2); `pipe.set_tolerance(1e-4, 1e-4, 1e-2)`.
    pub fn set_tolerance(&mut self, tol3d: f64, bound_tol: f64, tol_angular: f64) {
        self.my_tol3d = tol3d;
        self.my_bound_tol = bound_tol;
        self.my_tol_angular = tol_angular;
    }

    /// `SetTransition(Mode, Angmin, Angmax)` (natively defaulted to modified, 1e-2 and
    /// 6): the sections are relocated when the mode changes;
    /// `pipe.set_transition(TransitionStyle::Right, 1e-2, 6.0)`.
    pub fn set_transition(&mut self, mode: TransitionStyle, angmin: f64, angmax: f64) {
        if self.my_transition != mode {
            self.my_section = Handle::default();
        }
        self.my_transition = mode;
        self.angmin = angmin;
        self.angmax = angmax;
    }

    /// `Shape()`: the shell or solid built; `pipe.shape()`.
    pub fn shape(&self) -> &Shape {
        &self.my_shape
    }

    /// `ErrorOnSurface()`; `pipe.error_on_surface()`.
    pub fn error_on_surface(&self) -> f64 {
        self.my_error_on_surf
    }

    /// `FirstShape()`: the first section (or the face closing it after MakeSolid);
    /// `pipe.first_shape()`.
    pub fn first_shape(&self) -> &Shape {
        &self.my_first
    }

    /// `LastShape()`; `pipe.last_shape()`.
    pub fn last_shape(&self) -> &Shape {
        &self.my_last
    }

    /// `Profiles(theProfiles)`: the original profiles, in order; `pipe.profiles()`.
    pub fn profiles(&self) -> List<Shape> {
        let mut the_profiles: List<Shape> = List::default();
        for section in &self.my_seq {
            the_profiles.append(section.original_shape().clone());
        }
        the_profiles
    }

    /// `Spine()`; `pipe.spine()`.
    pub fn spine(&self) -> &Wire {
        &self.my_spine
    }

    /// `Generated(S, L)`: the shapes generated from a shape of a profile or of the
    /// spine (empty when none); `let faces = pipe.generated(&edge);`.
    pub fn generated(&self, s: &Shape) -> List<Shape> {
        self.my_gen_map.seek(s).cloned().unwrap_or_default()
    }
}
