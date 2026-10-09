// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_WireEdgeClassifier.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepBuild_WireEdgeClassifier`: the classification of wires and edges on a
//! face, by a 2d point of an edge against the p-curves of the other loop.
use super::block_builder::TopOpeBRepBuildBlockBuilder;
use super::composite_classifier::TopOpeBRepBuildCompositeClassifier;
use super::loops::{TopOpeBRepBuildLoop, TopOpeBRepBuildLoopClassifier};
use crate::top_ope_brep_tool::{
    fc2d_curve_on_surface, fc2d_has_curve_on_surface, fsc_get_psc, fun_tool_bounds, tggeom_e,
};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve2d;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::IndexedMap;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::{Direction2, Line2, Point2, Vector2, Vector3};
use rustycad_tktopalgo::brep_class::{
    BRepClassEdge, BRepClassFaceClassifier, FacePassiveClassifier,
};

/// The classifier of wires and edges on a face (`TopOpeBRepBuild_WireEdgeClassifier`);
/// natively the first compare flag and the 2d point are uninitialized until
/// ResetElement (false and the origin here);
/// `let mut wec = TopOpeBRepBuildWireEdgeClassifier::new(&face, &bb)?;`.
pub struct TopOpeBRepBuildWireEdgeClassifier<'a> {
    my_block_builder: &'a TopOpeBRepBuildBlockBuilder,
    my_first_compare: bool,
    my_point2d: Point2,
    my_bc_edge: BRepClassEdge,
    my_fpc: FacePassiveClassifier,
    my_shape: Shape,
}

/// Standard_ProgramError of the classifier.
fn program_error(message: &str) -> Failure {
    Failure::with_type(
        "Standard_ProgramError",
        message,
        "TopOpeBRepBuild_WireEdgeClassifier",
    )
}

/// The edges of a shape, in exploration order.
fn edges_of(s: &Shape) -> Result<Vec<Edge>, Failure> {
    Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)?
        .map(|e| topods::edge(&e))
        .collect()
}

/// `FUN_tgINE(v, vl, e)` (static): the tangent of `e` at `v` oriented inside the edge
/// (reversed at its last vertex `vl`); the null vector without tangent.
fn fun_tg_ine(v: &Vertex, vl: &Vertex, e: &Edge) -> Result<Vector3, Failure> {
    let par = BRepTool::parameter(v, e)?;
    let Some(tg) = tggeom_e(par, e)? else {
        return Ok(Vector3::new(0., 0., 0.)); // NYIRAISE
    };
    Ok(if v.as_ref().is_same(vl.as_ref()) {
        tg.reversed()
    } else {
        tg
    })
}

/// The p-curve of `e` on `f` stored on the edge when missing (`jyl980406+`): the made
/// one with the larger of its tolerance and the edge one.
fn store_pcurve(e: &Edge, f: &Face, trim3d: bool) -> Result<(), Failure> {
    if fc2d_has_curve_on_surface(e, f)? {
        return Ok(());
    }
    let (c2d, _, _, tolpc) = fc2d_curve_on_surface(e, f, trim3d)?;
    if c2d.is_null() {
        // A null p-curve is not stored (LoopToShape); ResetElement then raises.
        return Ok(());
    }
    let tol = BRepTool::edge_tolerance(e)?.max(tolpc);
    BRepBuilder::default().update_edge_pcurve_on_face(e, &c2d, f, tol)
}

/// The 2d point at the fraction `t` of the p-curve range of `e` on `f`.
fn point_at_fraction(e: &Edge, f: &Face, t: f64, what: &str) -> Result<Point2, Failure> {
    let (c2d, f2, l2, _) = fc2d_curve_on_surface(e, f, false)?;
    if c2d.is_null() {
        return Err(program_error(what));
    }
    c2d.value((1. - t) * f2 + t * l2)
}

impl<'a> TopOpeBRepBuildWireEdgeClassifier<'a> {
    /// `TopOpeBRepBuild_WireEdgeClassifier(F, BB)`: a classifier on face `f`;
    /// `TopOpeBRepBuildWireEdgeClassifier::new(&face, &bb)?`.
    pub fn new(f: &Shape, bb: &'a TopOpeBRepBuildBlockBuilder) -> Result<Self, Failure> {
        let mut my_bc_edge = BRepClassEdge::new();
        my_bc_edge.set_face(topods::face(f)?);
        Ok(Self {
            my_block_builder: bb,
            my_first_compare: false,
            my_point2d: Point2::default(),
            my_bc_edge,
            my_fpc: FacePassiveClassifier::new(),
            my_shape: Shape::default(),
        })
    }

    /// The face of the classifier (`myBCEdge.Face()`).
    fn face(&self) -> Face {
        self.my_bc_edge.face().clone()
    }

    /// `Compare(L1, L2)`: the composite comparison, and for two blocks still UNKNOWN
    /// the state of the face of `l1` relative to the face of `l2` by the per-thread
    /// shape classifier in same domain mode; `wec.compare(&l1, &l2)?`.
    pub fn compare(
        &mut self,
        l1: &Handle<dyn TopOpeBRepBuildLoop>,
        l2: &Handle<dyn TopOpeBRepBuildLoop>,
    ) -> Result<State, Failure> {
        let state = self.composite_compare(l1, l2)?;
        if l1.is_shape() || l2.is_shape() || state != State::Unknown {
            return Ok(state);
        }
        let s1 = self.loop_to_shape(l1)?;
        if s1.is_null() {
            return Ok(state);
        }
        let s2 = self.loop_to_shape(l2)?;
        if s2.is_null() {
            return Ok(state);
        }
        fsc_get_psc(|sc| {
            let samedomain = sc.same_domain();
            sc.set_same_domain(1);
            sc.set_reference(&s2)?;
            let state = sc.state_shape_reference(&s1, &s2);
            sc.set_same_domain(samedomain);
            state
        })
    }

    /// `LoopToShape(L)`: an empty copy of the face bounded by the wire of the edges of
    /// the block (their missing p-curves stored); null for an empty block;
    /// `let face = wec.loop_to_shape(&l)?;`.
    pub fn loop_to_shape(&mut self, l: &Handle<dyn TopOpeBRepBuildLoop>) -> Result<Shape, Failure> {
        self.my_shape = Shape::default();
        let mut bit = l.block_iterator();
        bit.initialize();
        if !bit.more() {
            return Ok(self.my_shape.clone());
        }
        let f = topods::face(&self.face().as_ref().empty_copied()?)?;
        let builder = BRepBuilder::default();
        let mut w = Wire::default();
        builder.make_wire(&mut w);
        let mut w = w.into_shape();
        while bit.more() {
            let e = topods::edge(self.my_block_builder.element_at(&bit)?)?;
            store_pcurve(&e, &f, false)?;
            builder.add(&mut w, e.as_ref())?;
            bit.next();
        }
        let mut face = f.into_shape();
        builder.add(&mut face, &w)?;
        self.my_shape = face;
        Ok(self.my_shape.clone())
    }

    /// Whether the edge `e` of `b2`, sharing a vertex of `mapv1` with the open edge
    /// `e1`, leaves that vertex opposite to `e1` (undecided tangency, xpu101198).
    fn undecided(
        &self,
        e1: &Edge,
        vl1: &Vertex,
        mapv1: &IndexedMap<Shape, ShapeMapHasher>,
        e: &Edge,
    ) -> Result<bool, Failure> {
        let (vf, vl) = topexp::vertices(e, false);
        let eclo = vf.as_ref().is_same(vl.as_ref());
        let (hasf, hasl) = (mapv1.contains(vf.as_ref()), mapv1.contains(vl.as_ref()));
        if !(hasf || hasl) || eclo {
            return Ok(false);
        }
        let vshared = if hasl { vl.clone() } else { vf };
        let tg1 = fun_tg_ine(&vshared, vl1, e1)?;
        let tg = fun_tg_ine(&vshared, &vl, e)?;
        let tol = precision::angular() * 1.0e4; // nyixpu
        Ok((1. + tg1.dot(tg)).abs() < tol)
    }

    /// The state of a point of `e1` relative to the edges of `b2`, UNKNOWN when `b2`
    /// holds `e1` (eap occ416) or an edge of undecided tangency.
    fn compare_edge_to_shape(
        &mut self,
        e1: &Edge,
        b2: &Shape,
        mut state: State,
    ) -> Result<State, Failure> {
        let (vf1, vl1) = topexp::vertices(e1, false);
        let e1clo = vf1.as_ref().is_same(vl1.as_ref());
        let mut mapv1: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::default();
        mapv1.add(vf1.into_shape());
        mapv1.add(vl1.clone().into_shape());
        self.reset_shape(e1.as_ref())?;
        for e in edges_of(b2)? {
            if e.as_ref().is_same(e1.as_ref()) {
                return Ok(State::Unknown);
            }
            if !e1clo && self.undecided(e1, &vl1, &mapv1, &e)? {
                return Ok(State::Unknown);
            }
            self.compare_element(e.as_ref())?;
            state = self.state();
        }
        Ok(state)
    }

    /// The fallback of two disjoint wires: the state of the point at 0.45678 of the
    /// first edge of `b2` not in `b1` in the face bounded by `b1`.
    fn compare_disjoint_wires(&self, b1: &Shape, b2: &Shape) -> Result<Option<State>, Failure> {
        let mut mape1: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::default();
        topexp::map_shapes_of_type(b1, ShapeEnum::Edge, &mut mape1)?;
        let Some(e2) = edges_of(b2)?
            .into_iter()
            .find(|e| !mape1.contains(e.as_ref()))
        else {
            return Ok(None);
        };
        let ftmp = self.face().as_ref().oriented(Orientation::Forward);
        let f2 = bounded_copy(&ftmp, b2)?;
        let bc2d = BRepAdaptorCurve2d::with_edge_on_face(&e2, &f2)?;
        let (f, l) = fun_tool_bounds(&e2)?;
        let x = 0.45678;
        let p2d = bc2d.value((1. - x) * l + x * f)?;
        let f1 = bounded_copy(&ftmp, b1)?;
        let tol_f1 = BRepTool::face_tolerance(&f1)?;
        Ok(Some(
            BRepClassFaceClassifier::with_face_point2d(&f1, &p2d, tol_f1, false, 0.1)?.state(),
        ))
    }
}

/// An empty copy of the face `ftmp` with the wire `w` added.
fn bounded_copy(ftmp: &Shape, w: &Shape) -> Result<Face, Failure> {
    let mut f = ftmp.empty_copied()?;
    BRepBuilder::default().add(&mut f, topods::wire(w)?.as_ref())?;
    topods::face(&f)
}

impl TopOpeBRepBuildCompositeClassifier for TopOpeBRepBuildWireEdgeClassifier<'_> {
    fn block_builder(&self) -> &TopOpeBRepBuildBlockBuilder {
        self.my_block_builder
    }

    /// `CompareShapes(B1, B2)`: the first decided state of an edge of `b1` against
    /// the edges of `b2`, else for two wires the disjoint wires fallback.
    fn compare_shapes(&mut self, b1: &Shape, b2: &Shape) -> Result<State, Failure> {
        let mut state = State::Unknown;
        let edges1 = edges_of(b1)?;
        if edges1.is_empty() {
            return Ok(state);
        }
        for e1 in &edges1 {
            state = self.compare_edge_to_shape(e1, b2, state)?;
            if state != State::Unknown {
                break;
            }
        }
        let resta = state == State::Unknown
            && b2.shape_type()? == ShapeEnum::Wire
            && b1.shape_type()? == ShapeEnum::Wire;
        if !resta {
            return Ok(state);
        }
        Ok(self.compare_disjoint_wires(b1, b2)?.unwrap_or(state))
    }

    /// `CompareElementToShape(E, B)`: the point of `e` against the edges of `b`.
    fn compare_element_to_shape(&mut self, e: &Shape, b: &Shape) -> Result<State, Failure> {
        self.reset_element(e)?;
        for edge in edges_of(b)? {
            self.compare_element(edge.as_ref())?;
        }
        Ok(self.state())
    }

    /// `ResetShape(B)`: ResetElement on `b` (an edge) or on its first edge.
    fn reset_shape(&mut self, b: &Shape) -> Result<(), Failure> {
        if b.shape_type()? == ShapeEnum::Edge {
            return self.reset_element(b);
        }
        match edges_of(b)?.first() {
            Some(e) => self.reset_element(e.as_ref()),
            None => Ok(()),
        }
    }

    /// `ResetElement(E)`: the 2d point at 0.397891143689 of the p-curve range of `e`
    /// (Standard_ProgramError without p-curve).
    fn reset_element(&mut self, ee: &Shape) -> Result<(), Failure> {
        let e = topods::edge(ee)?;
        let f = self.face();
        store_pcurve(&e, &f, true)?;
        self.my_point2d = point_at_fraction(&e, &f, 0.397891143689, "WEC : ResetElement")?;
        self.my_first_compare = true;
        Ok(())
    }

    /// `CompareElement(E)`: on the first compare, the classification line from the
    /// 2d point towards the point at 0.33334567 of `e`; then `e` compared; always true.
    fn compare_element(&mut self, ee: &Shape) -> Result<bool, Failure> {
        let e = topods::edge(ee)?;
        let f = self.face();
        store_pcurve(&e, &f, true)?;
        if self.my_first_compare {
            let p2d = point_at_fraction(&e, &f, 0.33334567, "WEC : CompareElement")?;
            let v2d = Vector2::between(self.my_point2d, p2d);
            let l2d = Line2::new(self.my_point2d, Direction2::from_vector(v2d)?);
            let dist = self.my_point2d.distance(p2d);
            self.my_fpc.reset(l2d, dist, precision::p_confusion()); // NYI : a voir
            self.my_first_compare = false;
        }
        let ori = e.as_ref().orientation();
        self.my_bc_edge.set_edge(e);
        self.my_fpc.compare(&self.my_bc_edge, ori)?;
        Ok(true)
    }

    /// `State()`: the state of the passive classifier.
    fn state(&mut self) -> State {
        self.my_fpc.state()
    }
}

impl TopOpeBRepBuildLoopClassifier for TopOpeBRepBuildWireEdgeClassifier<'_> {
    fn compare(
        &mut self,
        l1: &Handle<dyn TopOpeBRepBuildLoop>,
        l2: &Handle<dyn TopOpeBRepBuildLoop>,
    ) -> Result<State, Failure> {
        TopOpeBRepBuildWireEdgeClassifier::compare(self, l1, l2)
    }
}
