// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_CLASSI.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepTool_CLASSI`: the classification of the wires of a face by their UV
//! boxes and by points inside their faces: one inside the other, the same or apart.
use crate::top_ope_brep_tool::face::TopOpeBRepToolFace;
use crate::top_ope_brep_tool::project::fun_tool_bounds;
use crate::top_ope_brep_tool::tool::{ShapeListMap, TopOpeBRepToolTOOL};
use crate::top_ope_brep_tool::topology::fun_tool_paron_ef_tol;
use crate::top_ope_brep_tool::two_d::fc2d_has_curve_on_surface;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve2d;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, IndexedDataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkgeombase::bnd_lib::Add2dCurve;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tktopalgo::brep_class::BRepClassFaceClassifier;
use rustycad_tktopalgo::brep_class3d::BRepClass3dSolidExplorer;

/// The classification codes: the shapes are the same.
pub const CLASSI_SAME: i32 = -1;
/// The shapes are apart.
pub const CLASSI_DIFF: i32 = -2;
/// Unknown.
pub const CLASSI_UNKNOWN: i32 = 0;
/// The first shape is inside the second.
pub const CLASSI_ONE_IN_TWO: i32 = 1;
/// The second shape is inside the first.
pub const CLASSI_TWO_IN_ONE: i32 = 2;

/// The classifier of the wires of a face; `let mut c = TopOpeBRepToolCLASSI::new(); c.init2d(&f);`.
#[derive(Default)]
pub struct TopOpeBRepToolCLASSI {
    my_fref: Face,
    mymapsbox2d: IndexedDataMap<Shape, BoundingBox2>,
    mymapsface: DataMap<Shape, TopOpeBRepToolFace, ShapeMapHasher>,
}

/// `FUN_thegreatest(F1, class2)` (static): for the first edge of `F1` with a UV point
/// at its parameter `(1 - x) f + x + l` (x = 0.45678, sic), the state of the point
/// already classified by `class2`: IN gives oneINtwo, OUT twoINone (natively the
/// classifier is not run on the new point).
fn fun_thegreatest(f1: &Face, class2: &BRepClassFaceClassifier) -> Result<i32, Failure> {
    let tolf1 = BRepTool::face_tolerance(f1)?;
    for e in Explorer::new(f1, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let e1 = topods::edge(&e)?;
        let (f, l) = fun_tool_bounds(&e1)?;
        let x: f64 = 0.45678;
        let p1 = (1. - x) * f + x + l;
        let mut uv1 = Point2::default();
        if !fun_tool_paron_ef_tol(&e1, p1, f1, &mut uv1, tolf1)? {
            continue;
        }
        match class2.state() {
            State::In => return Ok(CLASSI_ONE_IN_TWO),
            State::Out => return Ok(CLASSI_TWO_IN_ONE),
            _ => {}
        }
    }
    Ok(CLASSI_UNKNOWN)
}

/// `FUN_addOwlw(Ow, lw, lresu)`: the lower wires appended, or the wire itself when
/// there are none; `fun_add_owlw(&w, &lw, &mut l)`.
pub fn fun_add_owlw(ow: &Shape, lw: &List<Shape>, lresu: &mut List<Shape>) {
    if lw.is_empty() {
        lresu.append(ow.clone());
        return;
    }
    for w in lw.iter() {
        lresu.append(w.clone());
    }
}

/// The classification of the finite faces knowing their relation and finiteness.
fn finite_classification(staffi12: i32, finite1: bool, finite2: bool) -> i32 {
    match staffi12 {
        CLASSI_DIFF => match (finite1, finite2) {
            (true, true) => CLASSI_DIFF,
            (true, false) => CLASSI_TWO_IN_ONE,
            _ => CLASSI_ONE_IN_TWO,
        },
        CLASSI_ONE_IN_TWO if !finite1 && !finite2 => CLASSI_TWO_IN_ONE,
        CLASSI_ONE_IN_TWO => CLASSI_ONE_IN_TWO,
        CLASSI_TWO_IN_ONE if !finite1 && !finite2 => CLASSI_ONE_IN_TWO,
        CLASSI_TWO_IN_ONE => CLASSI_TWO_IN_ONE,
        _ => CLASSI_UNKNOWN,
    }
}

/// A point inside a face; `None` when none is found.
fn point_in_face(f: &Face) -> Result<Option<Point2>, Failure> {
    let (mut p, mut u, mut v) = (Point3::default(), 0., 0.);
    if !BRepClass3dSolidExplorer::find_a_point_in_the_face_uv(f, &mut p, &mut u, &mut v)? {
        return Ok(None);
    }
    Ok(Some(Point2::new(u, v)))
}

/// The classifier of a UV point in a face with its UV tolerance.
fn classify(f: &Face, p2d: Point2) -> Result<BRepClassFaceClassifier, Failure> {
    let tol2d = TopOpeBRepToolTOOL::tol_uv(f, BRepTool::face_tolerance(f)?)?;
    BRepClassFaceClassifier::with_face_point2d(f, &p2d, tol2d, false, 0.1)
}

impl TopOpeBRepToolCLASSI {
    /// `TopOpeBRepTool_CLASSI()`; `TopOpeBRepToolCLASSI::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Init2d(Fref)`; `c.init2d(&f)`.
    pub fn init2d(&mut self, fref: &Face) {
        self.my_fref = fref.clone();
    }

    /// `HasInit2d()`; `c.has_init2d()`.
    pub fn has_init2d(&self) -> bool {
        !self.my_fref.is_null()
    }

    /// `Add2d(S)`: the UV box of the edges of the shape on the reference face (with
    /// the resolution of their tolerances) stored once; false without reference face or
    /// for an edge without p-curve; `c.add2d(&w)?`.
    pub fn add2d(&mut self, s: &Shape) -> Result<bool, Failure> {
        if !self.has_init2d() {
            return Ok(false);
        }
        if self.mymapsbox2d.contains(s) {
            return Ok(true);
        }
        let mut b2d = BoundingBox2::default();
        for es in Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let e = topods::edge(&es)?;
            let tol_e = BRepTool::edge_tolerance(&e)?;
            if !fc2d_has_curve_on_surface(&e, &self.my_fref)? {
                return Ok(false);
            }
            let bc2d = BRepAdaptorCurve2d::with_edge_on_face(&e, &self.my_fref)?;
            let tol2d = bc2d.resolution(tol_e)?;
            Add2dCurve::add_adaptor(&bc2d, tol2d, &mut b2d)?;
        }
        self.mymapsbox2d.add(s.clone(), b2d);
        Ok(true)
    }

    /// `GetBox2d(S, B2d)`: the stored box of the shape, added first when needed;
    /// `c.get_box2d(&w)?`.
    pub fn get_box2d(&mut self, s: &Shape) -> Result<Option<BoundingBox2>, Failure> {
        if !self.mymapsbox2d.contains(s) && !self.add2d(s)? {
            return Ok(None);
        }
        Ok(self.mymapsbox2d.seek(s).cloned())
    }

    /// The bounds `(Umin, Umax, Vmin, Vmax)` of the stored box of a shape.
    fn uv_bounds(&mut self, s: &Shape) -> Result<Option<[f64; 4]>, Failure> {
        let Some(b) = self.get_box2d(s)? else {
            return Ok(None);
        };
        let [umin, vmin, umax, vmax] = b.get()?;
        Ok(Some([umin, umax, vmin, vmax]))
    }

    /// `ClassiBnd2d(S1, S2, tol, chklarge)`: DIFF for disjoint boxes, SAME for equal ones
    /// (within `tol`), oneINtwo or twoINone for a box inside the other, UNKNOWN
    /// otherwise (or when a box is missing); `chklarge` widens the tests by `tol`;
    /// `c.classi_bnd2d(&w1, &w2, 1e-9, true)?`.
    pub fn classi_bnd2d(
        &mut self,
        s1: &Shape,
        s2: &Shape,
        tol: f64,
        chklarge: bool,
    ) -> Result<i32, Failure> {
        let Some(b1) = self.uv_bounds(s1)? else {
            return Ok(CLASSI_UNKNOWN);
        };
        let Some(b2) = self.uv_bounds(s2)? else {
            return Ok(CLASSI_UNKNOWN);
        };
        let uv = [b1, b2];
        for k in [0usize, 2] {
            for (ii, jj) in [(0usize, 1usize), (1, 0)] {
                let diff = uv[ii][k] - uv[jj][k + 1];
                if (chklarge && diff >= -tol) || (!chklarge && diff > 0.) {
                    return Ok(CLASSI_DIFF);
                }
            }
        }
        for (ii, jj) in [(0usize, 1usize), (1, 0)] {
            let (mut smaller, mut same) = (true, true);
            for k in [0usize, 2] {
                let diff = uv[ii][k] - uv[jj][k];
                smaller = smaller && if chklarge { diff > -tol } else { diff > 0. };
                same = same && diff.abs() <= tol;
            }
            for k in [1usize, 3] {
                let diff = uv[ii][k] - uv[jj][k];
                smaller = smaller && if chklarge { diff < tol } else { diff < 0. };
                same = same && diff.abs() <= tol;
            }
            if same {
                return Ok(CLASSI_SAME);
            }
            if smaller {
                return Ok(if ii == 0 {
                    CLASSI_ONE_IN_TWO
                } else {
                    CLASSI_TWO_IN_ONE
                });
            }
        }
        Ok(CLASSI_UNKNOWN)
    }

    /// The finite face of a wire, made and stored on first use.
    fn face_of(&mut self, s: &Shape) -> Result<TopOpeBRepToolFace, Failure> {
        if let Some(fa) = self.mymapsface.seek(s) {
            return Ok(fa.clone());
        }
        let mut fa = TopOpeBRepToolFace::new();
        fa.init(&topods::wire(s)?, &self.my_fref)?;
        self.mymapsface.bind(s.clone(), fa.clone());
        Ok(fa)
    }

    /// The relation of the finite faces from the box relation and the state of the
    /// point of the first face in the second; `Err(code)` returns at once.
    fn staffi_from_boxes(stabnd2d12: i32, sta12: State) -> Result<(i32, bool), i32> {
        if stabnd2d12 == CLASSI_UNKNOWN || stabnd2d12 == CLASSI_SAME {
            return Ok((CLASSI_UNKNOWN, true));
        }
        if stabnd2d12 == CLASSI_DIFF {
            return Err(CLASSI_DIFF);
        }
        match sta12 {
            State::In if stabnd2d12 == CLASSI_ONE_IN_TWO => Ok((CLASSI_ONE_IN_TWO, false)),
            State::In if stabnd2d12 == CLASSI_TWO_IN_ONE => Ok((CLASSI_TWO_IN_ONE, false)),
            State::In => Ok((CLASSI_UNKNOWN, true)),
            State::Out if stabnd2d12 == CLASSI_TWO_IN_ONE => Ok((CLASSI_TWO_IN_ONE, false)),
            State::Out => Ok((CLASSI_UNKNOWN, true)),
            _ => Err(CLASSI_UNKNOWN),
        }
    }

    /// The relation of the finite faces from both point states (`try21`).
    fn staffi_from_points(
        bnd2d_unk: bool,
        sta12: State,
        sta21: State,
        staffi12: i32,
        greatest: impl FnOnce() -> Result<i32, Failure>,
    ) -> Result<Result<i32, i32>, Failure> {
        let (in12, out12) = (sta12 == State::In, sta12 == State::Out);
        let (in21, out21) = (sta21 == State::In, sta21 == State::Out);
        if bnd2d_unk {
            return Ok(Ok(match () {
                _ if out12 && out21 => CLASSI_DIFF,
                _ if in12 && out21 => CLASSI_ONE_IN_TWO,
                _ if out12 && in21 => CLASSI_TWO_IN_ONE,
                _ if in12 && in21 => greatest()?,
                _ => staffi12,
            }));
        }
        Ok(match () {
            _ if in12 && out21 => Ok(CLASSI_ONE_IN_TWO),
            _ if in12 => Ok(staffi12),
            _ if out12 && out21 => Ok(CLASSI_DIFF),
            _ if out12 && in21 => Ok(CLASSI_TWO_IN_ONE),
            _ if out12 => Ok(staffi12),
            _ => Err(CLASSI_UNKNOWN),
        })
    }

    /// `Classip2d(S1, S2, stabnd2d12)`: the relation of two wires from points inside
    /// their finite faces, refined by the relation of their boxes, then corrected by the
    /// finiteness of the faces; `c.classip2d(&w1, &w2, CLASSI_UNKNOWN)?`.
    pub fn classip2d(&mut self, s1: &Shape, s2: &Shape, stabnd2d12: i32) -> Result<i32, Failure> {
        if !self.has_init2d() {
            return Ok(CLASSI_UNKNOWN);
        }
        let bnd2d_unk = stabnd2d12 == CLASSI_UNKNOWN || stabnd2d12 == CLASSI_SAME;
        let fa1 = self.face_of(s1)?;
        let (ffi1, finite1) = (fa1.ffinite()?.clone(), fa1.finite()?);
        let fa2 = self.face_of(s2)?;
        let (ffi2, finite2) = (fa2.ffinite()?.clone(), fa2.finite()?);
        let Some(p2d1) = point_in_face(&ffi1)? else {
            return Ok(CLASSI_UNKNOWN);
        };
        let class2 = classify(&ffi2, p2d1)?;
        let sta12 = class2.state();
        let (mut staffi12, try21) = match Self::staffi_from_boxes(stabnd2d12, sta12) {
            Ok(found) => found,
            Err(code) => return Ok(code),
        };
        if try21 {
            let Some(p2d2) = point_in_face(&ffi2)? else {
                return Ok(CLASSI_UNKNOWN);
            };
            let sta21 = classify(&ffi1, p2d2)?.state();
            let greatest = || fun_thegreatest(&ffi1, &class2);
            match Self::staffi_from_points(bnd2d_unk, sta12, sta21, staffi12, greatest)? {
                Ok(code) => staffi12 = code,
                Err(code) => return Ok(code),
            }
        }
        Ok(finite_classification(staffi12, finite1, finite2))
    }

    /// `Getface(S, fa)`: the stored face of the shape; `c.getface(&w)`.
    pub fn getface(&self, s: &Shape) -> Option<TopOpeBRepToolFace> {
        self.mymapsface.seek(s).cloned()
    }

    /// One pass of the comparisons of the first wire still greater with the next ones
    /// (the inner loop of `Classilist`); false when two wires cannot be classified.
    fn classify_first(
        &mut self,
        lw: &[Shape],
        toluv: f64,
        mapgreasma: &mut ShapeListMap,
    ) -> Result<bool, Failure> {
        let Some(start) = lw.iter().position(|w| mapgreasma.is_bound(w)) else {
            return Ok(true);
        };
        let wi1 = lw[start].clone();
        let mut next = start;
        loop {
            if !mapgreasma.is_bound(&wi1) {
                break;
            }
            next += 1;
            if next >= lw.len() {
                break;
            }
            let (mut sta12, mut out_all) = (CLASSI_UNKNOWN, false);
            let mut wi2 = Shape::default();
            while next < lw.len() {
                wi2 = lw[next].clone();
                if !mapgreasma.is_bound(&wi2) {
                    next += 1;
                    continue;
                }
                let stabnd2d12 = self.classi_bnd2d(&wi1, &wi2, toluv, true)?;
                sta12 = self.classip2d(&wi1, &wi2, stabnd2d12)?;
                if sta12 == CLASSI_DIFF {
                    out_all = true;
                    next += 1;
                    continue;
                }
                break;
            }
            match sta12 {
                CLASSI_ONE_IN_TWO => merge_smaller(mapgreasma, &wi2, &wi1)?,
                CLASSI_TWO_IN_ONE => merge_smaller(mapgreasma, &wi1, &wi2)?,
                _ if out_all => {}
                _ => return Ok(false),
            }
        }
        Ok(true)
    }

    /// `Classilist(lS, mapgreasma)`: the wires sorted into the greater ones (keys)
    /// holding the smaller ones inside them; false when two wires cannot be classified;
    /// `c.classilist(&wires, &mut map)?`.
    pub fn classilist(
        &mut self,
        l_s: &List<Shape>,
        mapgreasma: &mut ShapeListMap,
    ) -> Result<bool, Failure> {
        let tolref = BRepTool::face_tolerance(&self.my_fref)?;
        let toluv = TopOpeBRepToolTOOL::tol_uv(&self.my_fref, tolref)?;
        let mut lw: Vec<Shape> = l_s.iter().cloned().collect();
        mapgreasma.clear(false);
        for w in l_s.iter() {
            mapgreasma.bind(w.clone(), List::default());
        }
        let nw = lw.len();
        if nw <= 1 {
            return Ok(true);
        }
        let nitemax = nw * (nw - 1) / 2;
        // Natively `nite` is never incremented: the loop ends when one wire is left.
        for _ in 0..=nitemax.max(nw) {
            if lw.len() <= 1 {
                break;
            }
            if !self.classify_first(&lw, toluv, mapgreasma)? {
                return Ok(false);
            }
            lw.remove(0);
        }
        Ok(true)
    }
}

/// The smaller wire (with its own smaller ones) moved into the list of the greater.
fn merge_smaller(
    mapgreasma: &mut ShapeListMap,
    greater: &Shape,
    smaller: &Shape,
) -> Result<(), Failure> {
    let mut lsma: List<Shape> = List::default();
    let lower = mapgreasma.seek(smaller).cloned().unwrap_or_default();
    fun_add_owlw(smaller, &lower, &mut lsma);
    mapgreasma.unbind(smaller);
    let lwgre = mapgreasma.change_seek(greater).ok_or_else(|| {
        Failure::with_type(
            "Standard_NoSuchObject",
            "TopOpeBRepTool_CLASSI::Classilist: greater wire not bound; expected a bound wire",
            "TopOpeBRepTool_CLASSI",
        )
    })?;
    for w in lsma.iter() {
        lwgre.append(w.clone());
    }
    Ok(())
}
