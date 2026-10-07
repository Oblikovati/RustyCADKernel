// Rust translation of the classification members of TopOpeBRepTool_ShapeClassifier (FindEdge,
// FindFace, Perform, StateEdgeReference, StateP2DReference, StateP3DReference in
// ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_ShapeClassifier.cxx) and the FSC_*
// functions of TopOpeBRepTool_SC.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The classification of a point of the shape: in 2d on a same domain face, else in 3d
//! in the solids of the reference.
use super::{TopOpeBRepToolShapeClassifier, program_error};
use crate::top_ope_brep_tool::project::{fun_tool_bounds, fun_tool_proj_pons};
use crate::top_ope_brep_tool::two_d::fc2d_curve_on_surface;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag};
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tktopalgo::brep_class3d::BRepClass3dSolidExplorer;
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;
use std::cell::RefCell;

/// The parameter of the classified edge point (between the ends of its range).
const EDGE_POINT_T: f64 = 0.127956477;

/// Whether a shape type is a solid or a container of solids (`<= TopAbs_SOLID`).
fn up_to_solid(t: ShapeEnum) -> bool {
    matches!(
        t,
        ShapeEnum::Compound | ShapeEnum::CompSolid | ShapeEnum::Solid
    )
}

impl TopOpeBRepToolShapeClassifier {
    /// `FindEdge()` (private): the edge of the shape (of its first face for a shell or
    /// more).
    fn find_edge(&mut self) -> Result<(), Failure> {
        (self.my_edge, self.my_face) = (Edge::default(), Face::default());
        let s = self.my_s.clone();
        if !matches!(
            s.shape_type()?,
            ShapeEnum::Face
                | ShapeEnum::Wire
                | ShapeEnum::Edge
                | ShapeEnum::Vertex
                | ShapeEnum::Shape
        ) {
            self.find_face(&s)?;
            let f = self.my_face.as_ref().clone();
            return self.find_edge_of(&f);
        }
        self.find_edge_of(&s)
    }

    /// Whether a sub-shape is avoided: in the avoided map or the same as the avoided
    /// shape.
    fn is_avoided(&self, x: &Shape) -> bool {
        self.my_map_av_s.contains(x) || (!self.my_av_s.is_null() && x.is_same(&self.my_av_s))
    }

    /// `FindEdge(S)` (private): the first edge of the face (or of `s`) not avoided (not
    /// degenerated when nothing is avoided, unless `s` is an edge).
    fn find_edge_of(&mut self, s: &Shape) -> Result<(), Failure> {
        self.my_edge = Edge::default();
        let avoiding = self.has_av_ls() || !self.my_av_s.is_null();
        if s.is_null() {
            return Ok(());
        }
        let t_s = s.shape_type()?;
        let root = if self.my_face.as_ref().is_null() {
            s.clone()
        } else {
            self.my_face.as_ref().clone()
        };
        for e in Explorer::new(&root, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let toavoid = if avoiding {
                self.is_avoided(&e)
            } else {
                BRepTool::degenerated(&topods::edge(&e)?)? && t_s != ShapeEnum::Edge
            };
            if !toavoid {
                self.my_edge = topods::edge(&e)?;
                break;
            }
        }
        Ok(())
    }

    /// `FindFace(S)` (private): the first face of `s` not avoided.
    fn find_face(&mut self, s: &Shape) -> Result<(), Failure> {
        self.my_face = Face::default();
        let avoiding = self.has_av_ls() || !self.my_av_s.is_null();
        for f in Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)? {
            if avoiding && self.is_avoided(&f) {
                continue;
            }
            self.my_face = topods::face(&f)?;
            break;
        }
        Ok(())
    }

    /// The avoided map of the avoided shape or list (faces with their edges, edges).
    fn map_avoided(&mut self) -> Result<(), Failure> {
        let shapes: Vec<Shape> = if !self.my_av_s.is_null() {
            vec![self.my_av_s.clone()]
        } else {
            self.my_p_av_ls
                .as_ref()
                .map(|l| l.iter().cloned().collect())
                .unwrap_or_default()
        };
        self.my_map_av_s.clear(false);
        let t_av_s = shapes[0].shape_type()?;
        for s in &shapes {
            if t_av_s == ShapeEnum::Face {
                self.my_map_av_s.add(s.clone());
            }
            if matches!(t_av_s, ShapeEnum::Face | ShapeEnum::Edge) {
                topexp::map_shapes_of_type(s, ShapeEnum::Edge, &mut self.my_map_av_s)?;
            }
        }
        Ok(())
    }

    /// A point inside the face shape (FindAPointInTheFace).
    fn point_in_face(&mut self) -> Result<(), Failure> {
        let mut p = Point3::default();
        self.my_p3d_def = BRepClass3dSolidExplorer::find_a_point_in_the_face_point(
            &topods::face(&self.my_s)?,
            &mut p,
        )?;
        if self.my_p3d_def {
            self.my_p3d = p;
        }
        Ok(())
    }

    /// `Perform()` (private): the state of the shape relative to the reference, read as
    /// OUT for an IN state in an INTERNAL or EXTERNAL reference.
    pub(super) fn perform(&mut self) -> Result<(), Failure> {
        self.my_state = State::Unknown;
        if self.my_s.is_null() || self.my_ref.is_null() {
            return Ok(());
        }
        if !self.mymredone {
            self.map_ref()?;
        }
        if !self.my_av_s.is_null() || self.has_av_ls() {
            self.map_avoided()?;
        } else if self.my_s.shape_type()? == ShapeEnum::Face {
            self.point_in_face()?;
        }
        self.classify_by_types()?;
        let ori_ref = self.my_ref.orientation();
        if matches!(ori_ref, Orientation::External | Orientation::Internal)
            && self.my_state == State::In
        {
            self.my_state = State::Out;
        }
        Ok(())
    }

    /// The classification of the shape by its type and the reference type.
    fn classify_by_types(&mut self) -> Result<(), Failure> {
        let (t_s, t_r) = (self.my_s.shape_type()?, self.my_ref.shape_type()?);
        let r_face_or_solid = t_r == ShapeEnum::Face || up_to_solid(t_r);
        match t_s {
            ShapeEnum::Vertex if up_to_solid(t_r) => {
                let p = BRepTool::pnt(&topods::vertex(&self.my_s)?)?;
                self.state_p3d_reference(p)?;
            }
            ShapeEnum::Edge | ShapeEnum::Wire if r_face_or_solid => {
                self.find_edge()?;
                self.state_edge_reference()?;
            }
            ShapeEnum::Face if t_r == ShapeEnum::Face => self.classify_face_on_face()?,
            ShapeEnum::Face if up_to_solid(t_r) => {
                self.find_edge()?;
                if self.my_p3d_def {
                    self.state_p3d_reference(self.my_p3d)?;
                } else {
                    self.state_edge_reference()?;
                }
            }
            ShapeEnum::Shell | ShapeEnum::Solid if up_to_solid(t_r) => {
                self.find_edge()?;
                self.state_edge_reference()?;
            }
            ShapeEnum::Vertex
            | ShapeEnum::Edge
            | ShapeEnum::Wire
            | ShapeEnum::Face
            | ShapeEnum::Shell
            | ShapeEnum::Solid => {}
            _ => return Err(program_error("StateShapeShape : bad operands")),
        }
        Ok(())
    }

    /// A face relative to a face: by an edge point in 2d for same domain faces, else by
    /// a point inside the face in 3d (Standard_ProgramError without one).
    fn classify_face_on_face(&mut self) -> Result<(), Failure> {
        self.find_edge()?;
        if self.my_same_domain == 1 {
            return self.state_edge_reference();
        }
        if !self.my_p3d_def {
            self.point_in_face()?;
        }
        if !self.my_p3d_def {
            self.my_state = State::Unknown;
            return Err(program_error("TopOpeBRepTool_ShapeClassifier !P3Ddef"));
        }
        self.state_p3d_reference(self.my_p3d)
    }

    /// The 3d point of the edge curve at the classification parameter;
    /// Standard_ProgramError without 3d curve.
    fn edge_point_3d(&self) -> Result<Point3, Failure> {
        let (c3d, f3d, l3d) = BRepTool::curve(&self.my_edge)?;
        let c = c3d
            .get()
            .ok_or_else(|| program_error("StateShapeShape : no 3d curve"))?;
        c.value((1. - EDGE_POINT_T) * f3d + EDGE_POINT_T * l3d)
    }

    /// `StateEdgeReference()` (private): the state of a point of the edge: in 2d on a
    /// same domain face, else in 3d (the first vertex of a degenerated edge).
    fn state_edge_reference(&mut self) -> Result<(), Failure> {
        self.my_state = State::Unknown;
        if self.my_edge.as_ref().is_null() || self.my_ref.is_null() {
            return Ok(());
        }
        let t_r = self.my_ref.shape_type()?;
        if t_r == ShapeEnum::Face {
            if self.my_same_domain != 0 {
                let f = topods::face(&self.my_ref)?;
                let (c2d, f2d, l2d, _) = fc2d_curve_on_surface(&self.my_edge, &f, true)?;
                let c = c2d
                    .get()
                    .ok_or_else(|| program_error("StateShapeShape : no 2d curve"))?;
                let p2d = c.value((1. - EDGE_POINT_T) * f2d + EDGE_POINT_T * l2d)?;
                return self.state_p2d_reference(p2d);
            }
            let p3d = self.edge_point_3d()?;
            return self.state_p3d_reference(p3d);
        }
        if !up_to_solid(t_r) {
            return Err(program_error("StateShapeShape : bad operands"));
        }
        if BRepTool::degenerated(&self.my_edge)? {
            let v = topexp::first_vertex(&self.my_edge, false);
            return self.state_p3d_reference(BRepTool::pnt(&v)?);
        }
        let p3d = self.edge_point_3d()?;
        self.state_p3d_reference(p3d)
    }

    /// `StateP2DReference(P2D)`: the 2d state of the point in the reference face (from
    /// the orientations of its edges for a one-edge same domain face, printing
    /// "StateP2DReference o<>E,I" for an oriented edge as natively);
    /// Standard_ProgramError for another reference; `sc.state_p2d_reference(uv)?`.
    pub fn state_p2d_reference(&mut self, p2d: Point2) -> Result<(), Failure> {
        self.my_state = State::Unknown;
        if self.my_ref.is_null() {
            return Ok(());
        }
        if self.my_ref.shape_type()? != ShapeEnum::Face {
            return Err(program_error("StateShapeShape : bad operands"));
        }
        if self.mymren == 1 {
            for x in Explorer::new(&self.my_ref, ShapeEnum::Edge, ShapeEnum::Shape)? {
                match x.orientation() {
                    Orientation::External => self.my_state = State::Out,
                    Orientation::Internal => self.my_state = State::In,
                    _ => {
                        println!("StateP2DReference o<>E,I");
                        break;
                    }
                }
            }
            return Ok(());
        }
        (self.my_p2d, self.my_p2d_def) = (p2d, true);
        let f = topods::face(&self.my_ref.oriented(Orientation::Forward))?;
        let fclass2d = BRepTopAdaptorFClass2d::new(&f, 1e-8)?;
        self.my_state = fclass2d.perform(p2d, true)?;
        Ok(())
    }

    /// `StateP3DReference(P3D)`: the state of the point in the reference solid, or in
    /// the first solid of the reference containing it (IN or ON);
    /// Standard_ProgramError for another reference; `sc.state_p3d_reference(p)?`.
    pub fn state_p3d_reference(&mut self, p3d: Point3) -> Result<(), Failure> {
        self.my_state = State::Unknown;
        if self.my_ref.is_null() {
            return Ok(());
        }
        let t_r = self.my_ref.shape_type()?;
        if !up_to_solid(t_r) {
            return Err(program_error("StateShapeShape : bad operands"));
        }
        (self.my_p3d, self.my_p3d_def) = (p3d, true);
        let tol3d = precision::confusion();
        if t_r == ShapeEnum::Solid {
            self.my_state = self.my_solid_classifier.classify_solid(
                &topods::solid(&self.my_ref)?,
                p3d,
                tol3d,
            )?;
            return Ok(());
        }
        for so in Explorer::new(&self.my_ref, ShapeEnum::Solid, ShapeEnum::Shape)? {
            self.my_state =
                self.my_solid_classifier
                    .classify_solid(&topods::solid(&so)?, p3d, tol3d)?;
            if matches!(self.my_state, State::In | State::On) {
                break;
            }
        }
        Ok(())
    }
}

thread_local! {
    /// `TopOpeBRepTool_PSC` (a thread_local pointer natively, made on first use).
    static TOPOPEBREPTOOL_PSC: RefCell<Option<TopOpeBRepToolShapeClassifier>> = const { RefCell::new(None) };
}

/// `FSC_GetPSC()`: the per-thread classifier, given to `f` (a reference natively);
/// `fsc_get_psc(|psc| psc.state())`.
pub fn fsc_get_psc<R>(f: impl FnOnce(&mut TopOpeBRepToolShapeClassifier) -> R) -> R {
    TOPOPEBREPTOOL_PSC.with(|cell| {
        let mut psc = cell.borrow_mut();
        f(psc.get_or_insert_with(TopOpeBRepToolShapeClassifier::new))
    })
}

/// `FSC_GetPSC(S)`: the per-thread classifier with `s` as reference;
/// `fsc_get_psc_with_reference(&face, |psc| psc.state())?`.
pub fn fsc_get_psc_with_reference<R>(
    s: &Shape,
    f: impl FnOnce(&mut TopOpeBRepToolShapeClassifier) -> R,
) -> Result<R, Failure> {
    fsc_get_psc(|psc| {
        psc.set_reference(s)?;
        Ok(f(psc))
    })
}

/// `FSC_StatePonFace(P, F, PSC)`: the 2d state in the face of the projection of the
/// point on its surface; UNKNOWN when the projection fails;
/// `fsc_state_ponface(p, &face, &mut psc)?`.
pub fn fsc_state_ponface(
    p: Point3,
    f: &Shape,
    psc: &mut TopOpeBRepToolShapeClassifier,
) -> Result<State, Failure> {
    let s = BRepTool::surface(&topods::face(f)?)?;
    let (mut uv, mut dist) = (Point2::default(), 0.);
    if !fun_tool_proj_pons(p, &s, &mut uv, &mut dist, ExtFlag::MinMax, ExtAlgo::Grad)? {
        return Ok(State::Unknown);
    }
    psc.set_reference(f)?;
    psc.state_p2d_reference(uv)?;
    Ok(psc.state())
}

/// `FSC_StateEonFace(E, t, F, PSC)`: the state of the edge point at the fraction `t`
/// of its range; `fsc_state_eonface(&e, 0.5, &face, &mut psc)?`.
pub fn fsc_state_eonface(
    e: &Shape,
    t: f64,
    f: &Shape,
    psc: &mut TopOpeBRepToolShapeClassifier,
) -> Result<State, Failure> {
    let edge = topods::edge(e)?;
    let bac = BRepAdaptorCurve::with_edge(&edge)?;
    let (first, last) = fun_tool_bounds(&edge)?;
    let p = bac.value((1. - t) * first + t * last)?;
    fsc_state_ponface(p, f, psc)
}
