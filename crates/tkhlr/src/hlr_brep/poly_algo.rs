//! HLRBRep_PolyAlgo.hxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::hlr_algo::{BiPointPoints, EdgeStatus, PolyAlgo as AlgoPolyAlgo, Projector};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorCurve2d, BRepAdaptorSurface};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Transform3, Xyz};
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;

// The node flags of the internal triangulations (the file enum of PolyAlgo.cxx).
pub(super) const NMSK_VERT: i32 = 1;
pub(super) const NMSK_OUT_L: i32 = 2;
pub(super) const NMSK_NORM: i32 = 4;
pub(super) const NMSK_FUCK: i32 = 8;
pub(super) const NMSK_EDGE: i32 = 16;
pub(super) const NMSK_MOVE: i32 = 32;

/// One rigid transformation as the native 3x3 matrix and translation (`TMat`/`TLoc`).
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Frame {
    mat: [[f64; 3]; 3],
    loc: [f64; 3],
}

impl Frame {
    pub(super) fn of(t: &Transform3) -> Result<Self, Failure> {
        let v = t.translation_part();
        let m = t.vectorial_part();
        let mut mat = [[0.; 3]; 3];
        for (r, row) in mat.iter_mut().enumerate() {
            for (c, value) in row.iter_mut().enumerate() {
                *value = m.value(r as i32 + 1, c as i32 + 1)?;
            }
        }
        Ok(Self {
            mat,
            loc: [v.x(), v.y(), v.z()],
        })
    }

    /// The native TMultiply/TTMultiply/TIMultiply: `vpo` adds 0 instead of the
    /// translation.
    pub(super) fn apply(&self, x: &mut f64, y: &mut f64, z: &mut f64, vpo: bool) {
        let m = &self.mat;
        let loc = |k: usize| if vpo { 0. } else { self.loc[k] };
        let xt = m[0][0] * *x + m[0][1] * *y + m[0][2] * *z + loc(0);
        let yt = m[1][0] * *x + m[1][1] * *y + m[1][2] * *z + loc(1);
        *z = m[2][0] * *x + m[2][1] * *y + m[2][2] * *z + loc(2);
        *x = xt;
        *y = yt;
    }

    pub(super) fn apply_xyz(&self, p: &mut Xyz, vpo: bool) {
        let (mut x, mut y, mut z) = (p.x(), p.y(), p.z());
        self.apply(&mut x, &mut y, &mut z, vpo);
        *p = Xyz::new(x, y, z);
    }
}

/// The segment answered by `hide` and `show`: its points (the native returned
/// reference, copied), the edge or face it comes from and its four regularity flags.
#[derive(Clone, Debug, Default)]
#[allow(missing_docs)] // The native output names, documented on the type.
pub struct PolyAlgoSegment {
    pub points: BiPointPoints,
    pub shape: Shape,
    pub reg1: bool,
    pub regn: bool,
    pub outl: bool,
    pub intl: bool,
}

// The members of the native class; the private members of the computation take
// `&mut PolyState`.
pub(super) struct PolyState {
    pub(super) proj: Projector,
    pub(super) t: Frame,
    pub(super) tt: Frame,
    pub(super) ti: Frame,
    pub(super) shapes: Vec<Shape>,
    pub(super) e_map: IndexedMapOfShape,
    pub(super) f_map: IndexedMapOfShape,
    pub(super) algo: Handle<AlgoPolyAlgo>,
    pub(super) debug: bool,
    pub(super) tol_sta: f64,
    pub(super) tol_end: f64,
    pub(super) tol_angular: f64,
    pub(super) g_surf: Handle<dyn GeomSurface>,
    pub(super) b_surf: BRepAdaptorSurface,
    pub(super) b_curv: BRepAdaptorCurve,
    pub(super) pc: BRepAdaptorCurve2d,
}

/// HLRBRep_PolyAlgo: the polyhedral hidden line algorithm on the triangulations of
/// shapes: `update` builds the polyhedral shells (the triangles oriented to the eye,
/// the outlines inserted where the normals turn away, the edges and outlines as
/// segments) and the HLRAlgo polyhedral algorithm, whose walks `init_hide`/`hide`
/// and `init_show`/`show` give the segments with their visible parts.
///
/// A transient shared by handles: its members sit behind one lock and every member
/// takes `&self`; `Shape(I)` and `Projector()` return copies.
///
/// ```
/// use rustycad_tkhlr::hlr_brep::PolyAlgo;
/// let algo = PolyAlgo::new()?;
/// assert_eq!((algo.nb_shapes(), algo.tol_coef(), algo.tol_angular()), (0, 0.1, 0.001));
/// algo.update()?;
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct PolyAlgo {
    base: StandardTransient,
    pub(super) state: SharedState<PolyState>,
}
impl_transient!(PolyAlgo, "HLRBRep_PolyAlgo", StandardTransient, base);

impl PolyAlgo {
    /// HLRBRep_PolyAlgo(): no shape, the tolerances 0.1 and 0.001; `PolyAlgo::new()?`.
    pub fn new() -> Result<Self, Failure> {
        let state = PolyState {
            proj: Projector::new()?,
            t: Frame::default(),
            tt: Frame::default(),
            ti: Frame::default(),
            shapes: Vec::new(),
            e_map: IndexedMapOfShape::default(),
            f_map: IndexedMapOfShape::default(),
            algo: Handle::new(AlgoPolyAlgo::new()),
            debug: false,
            tol_sta: 0.1,
            tol_end: 0.9,
            tol_angular: 0.001,
            g_surf: Handle::default(),
            b_surf: BRepAdaptorSurface::default(),
            b_curv: BRepAdaptorCurve::default(),
            pc: BRepAdaptorCurve2d::default(),
        };
        Ok(Self {
            base: StandardTransient::default(),
            state: SharedState::new(state),
        })
    }

    /// HLRBRep_PolyAlgo(A): the tolerances, polyhedral algorithm (shared), projector
    /// and shapes of `a`; `PolyAlgo::from_algo(&a)?`.
    pub fn from_algo(a: &PolyAlgo) -> Result<Self, Failure> {
        let algo = Self::new()?;
        {
            let other = a.state.read();
            let mut s = algo.state.write();
            s.debug = other.debug;
            s.tol_angular = other.tol_angular;
            s.tol_sta = other.tol_sta;
            s.tol_end = 1.0 - s.tol_sta;
            s.algo = other.algo.clone();
            s.proj = other.proj;
            s.shapes = other.shapes.clone();
        }
        Ok(algo)
    }

    /// HLRBRep_PolyAlgo(S): one shape; `PolyAlgo::with_shape(&shape)?`.
    pub fn with_shape(s: &Shape) -> Result<Self, Failure> {
        let algo = Self::new()?;
        algo.load(s);
        Ok(algo)
    }

    /// NbShapes(); `algo.nb_shapes()`.
    pub fn nb_shapes(&self) -> i32 {
        self.state.read().shapes.len() as i32
    }

    /// Shape(I): a copy of the shape `i`; `algo.shape(1)?`.
    pub fn shape(&self, i: i32) -> Result<Shape, Failure> {
        let s = self.state.read();
        usize::try_from(i - 1)
            .ok()
            .and_then(|k| s.shapes.get(k).cloned())
            .ok_or_else(|| unknown_shape("Shape", i))
    }

    /// Remove(I): removes the shape `i` and the computed data; `algo.remove(1)?`.
    pub fn remove(&self, i: i32) -> Result<(), Failure> {
        let mut s = self.state.write();
        if i <= 0 || i as usize > s.shapes.len() {
            return Err(unknown_shape("Remove", i));
        }
        s.shapes.remove(i as usize - 1);
        s.algo.get().ok_or_else(null_algo)?.clear();
        s.e_map.clear(false);
        s.f_map.clear(false);
        Ok(())
    }

    /// Index(S): the index of the shape equal to `s`, 0 when absent;
    /// `algo.index(&shape)`.
    pub fn index(&self, s: &Shape) -> i32 {
        let st = self.state.read();
        st.shapes
            .iter()
            .position(|x| x == s)
            .map_or(0, |k| k as i32 + 1)
    }

    /// Load(S): appends a shape; `algo.load(&shape)`.
    pub fn load(&self, s: &Shape) {
        self.state.write().shapes.push(s.clone());
    }

    /// Algo(): the HLRAlgo polyhedral algorithm; `algo.algo()`.
    pub fn algo(&self) -> Handle<AlgoPolyAlgo> {
        self.state.read().algo.clone()
    }

    /// Projector(): a copy; `algo.projector()`.
    pub fn projector(&self) -> Projector {
        self.state.read().proj
    }

    /// Projector(P); `algo.set_projector(&p)`.
    pub fn set_projector(&self, p: &Projector) {
        self.state.write().proj = *p;
    }

    /// TolAngular(); `algo.tol_angular()`.
    pub fn tol_angular(&self) -> f64 {
        self.state.read().tol_angular
    }

    /// TolAngular(tol); `algo.set_tol_angular(0.001)`.
    pub fn set_tol_angular(&self, tol: f64) {
        self.state.write().tol_angular = tol;
    }

    /// TolCoef(); `algo.tol_coef()`.
    pub fn tol_coef(&self) -> f64 {
        self.state.read().tol_sta
    }

    /// TolCoef(tol): the start tolerance and `1 - tol` the end one;
    /// `algo.set_tol_coef(0.1)`.
    pub fn set_tol_coef(&self, tol: f64) {
        let mut s = self.state.write();
        s.tol_sta = tol;
        s.tol_end = 1.0 - tol;
    }

    /// Debug(); `algo.debug()`.
    pub fn debug(&self) -> bool {
        self.state.read().debug
    }

    /// Debug(B); `algo.set_debug(false)`.
    pub fn set_debug(&self, b: bool) {
        self.state.write().debug = b;
    }

    /// Update(): the polyhedral shells of the shapes (the shells, the faces outside
    /// shells, then the edges outside faces) and the HLRAlgo algorithm on them;
    /// `algo.update()?`.
    pub fn update(&self) -> Result<(), Failure> {
        self.state.write().update()
    }

    /// InitHide(); `algo.init_hide()?`.
    pub fn init_hide(&self) -> Result<(), Failure> {
        self.algo().get().ok_or_else(null_algo)?.init_hide()
    }

    /// MoreHide(); `algo.more_hide()`.
    pub fn more_hide(&self) -> bool {
        self.algo().get().is_some_and(|a| a.more_hide())
    }

    /// NextHide(); `algo.next_hide()?`.
    pub fn next_hide(&self) -> Result<(), Failure> {
        self.algo().get().ok_or_else(null_algo)?.next_hide()
    }

    /// Hide(status, S, reg1, regn, outl, intl): the visible parts of the current
    /// segment into `status`, with its points and the face (internal line) or edge it
    /// comes from; `let seg = algo.hide(&mut status)?;`.
    pub fn hide(&self, status: &mut EdgeStatus) -> Result<PolyAlgoSegment, Failure> {
        let s = self.state.read();
        let seg = s.algo.get().ok_or_else(null_algo)?.hide(status)?;
        s.segment(
            seg.points,
            seg.index,
            (seg.reg1, seg.regn, seg.outl, seg.intl),
        )
    }

    /// InitShow(); `algo.init_show()?`.
    pub fn init_show(&self) -> Result<(), Failure> {
        self.algo().get().ok_or_else(null_algo)?.init_show()
    }

    /// MoreShow(); `algo.more_show()`.
    pub fn more_show(&self) -> bool {
        self.algo().get().is_some_and(|a| a.more_show())
    }

    /// NextShow(); `algo.next_show()?`.
    pub fn next_show(&self) -> Result<(), Failure> {
        self.algo().get().ok_or_else(null_algo)?.next_show()
    }

    /// Show(S, reg1, regn, outl, intl): the current segment; `algo.show()?`.
    pub fn show(&self) -> Result<PolyAlgoSegment, Failure> {
        let s = self.state.read();
        let seg = s.algo.get().ok_or_else(null_algo)?.show()?;
        s.segment(
            seg.points,
            seg.index,
            (seg.reg1, seg.regn, seg.outl, seg.intl),
        )
    }

    /// OutLinedShape(S): a compound of `s` and of the internal outlines of its faces
    /// as 3D edges; `algo.out_lined_shape(&shape)?`.
    pub fn out_lined_shape(&self, s: &Shape) -> Result<Shape, Failure> {
        self.state.read().out_lined_shape(s)
    }
}

pub(super) fn unknown_shape(member: &str, i: i32) -> Failure {
    Failure::with_message(
        "Standard_OutOfRange",
        &format!("HLRBRep_PolyAlgo::{member} : unknown Shape {i}; expected 1 to NbShapes"),
    )
}

pub(super) fn null_algo() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "HLRBRep_PolyAlgo: no HLRAlgo_PolyAlgo",
    )
}

fn compound() -> Shape {
    let mut c = Compound::default();
    BRepBuilder::default().make_compound(&mut c);
    Shape::from(c)
}

impl PolyState {
    fn segment(
        &self,
        points: BiPointPoints,
        index: i32,
        flags: (bool, bool, bool, bool),
    ) -> Result<PolyAlgoSegment, Failure> {
        let (reg1, regn, outl, intl) = flags;
        let map = if intl { &self.f_map } else { &self.e_map };
        let shape = map.find_key(index as usize)?.clone();
        Ok(PolyAlgoSegment {
            points,
            shape,
            reg1,
            regn,
            outl,
            intl,
        })
    }

    // MakeShape(): a compound of the shapes, null without shape.
    fn make_shape(&self) -> Result<Shape, Failure> {
        if self.shapes.is_empty() {
            return Ok(Shape::default());
        }
        let mut comp = compound();
        let b = BRepBuilder::default();
        for s in &self.shapes {
            b.add(&mut comp, s)?;
        }
        Ok(comp)
    }

    // InitShape(S, IsoledF, IsoledE): the number of shells with triangulated faces,
    // plus one for the triangulated faces outside shells and one for the edges
    // outside faces; initializes the algorithm with it.
    fn init_shape(&mut self, shape: &Shape) -> Result<(i32, bool, bool), Failure> {
        let mut shape_map0 = MapOfShape::default();
        let mut nb_shell = 0;
        let (mut isoled_f, mut isoled_e) = (false, false);
        for shell in Explorer::new(shape, ShapeEnum::Shell, ShapeEnum::Shape)? {
            let mut has_trian = false;
            for face in Explorer::new(&shell, ShapeEnum::Face, ShapeEnum::Shape)? {
                if has_triangulation(&face)? && shape_map0.add(face) {
                    has_trian = true;
                }
            }
            if has_trian {
                nb_shell += 1;
            }
        }
        let mut faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shell)?;
        while faces.more() && !isoled_f {
            if has_triangulation(faces.current())? && shape_map0.add(faces.current().clone()) {
                isoled_f = true;
            }
            faces.next()?;
        }
        if isoled_f {
            nb_shell += 1;
        }
        if Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Face)?.more() {
            isoled_e = true;
            nb_shell += 1;
        }
        if nb_shell > 0 {
            self.algo.get().ok_or_else(null_algo)?.init(nb_shell)?;
        }
        Ok((nb_shell, isoled_f, isoled_e))
    }

    fn update(&mut self) -> Result<(), Failure> {
        self.algo.get().ok_or_else(null_algo)?.clear();
        self.e_map.clear(false);
        self.f_map.clear(false);
        let shape = self.make_shape()?;
        if shape.is_null() {
            return Ok(());
        }
        let (nb_shell, isoled_f, isoled_e) = self.init_shape(&shape)?;
        if nb_shell <= 0 {
            return Ok(());
        }
        topexp::map_shapes_of_type(&shape, ShapeEnum::Edge, &mut self.e_map)?;
        topexp::map_shapes_of_type(&shape, ShapeEnum::Face, &mut self.f_map)?;
        let nb_face = self.f_map.extent();
        let mut store = store::ShellStore {
            i_shell: 0,
            pd: vec![Handle::default(); nb_face + 1],
            pid: vec![Handle::default(); nb_face + 1],
            shape_map1: MapOfShape::default(),
            shape_map2: MapOfShape::default(),
        };
        for shell in Explorer::new(&shape, ShapeEnum::Shell, ShapeEnum::Shape)? {
            self.store_shell(&shell, &mut store, false, false)?;
        }
        if isoled_f {
            self.store_shell(&shape, &mut store, true, false)?;
        }
        if isoled_e {
            self.store_shell(&shape, &mut store, false, true)?;
        }
        self.algo.get().ok_or_else(null_algo)?.update()
    }

    fn out_lined_shape(&self, s: &Shape) -> Result<Shape, Failure> {
        if s.is_null() {
            return Ok(Shape::default());
        }
        let mut result = compound();
        let b = BRepBuilder::default();
        b.add(&mut result, s)?;
        if self.f_map.is_empty() {
            return Ok(result);
        }
        let mut map = MapOfShape::default();
        for e in Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)? {
            map.add(e);
        }
        for f in Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)? {
            map.add(f);
        }
        let algo = self.algo.get().ok_or_else(null_algo)?;
        let shells = algo.poly_shell();
        for i in 1..=shells.upper() {
            let shell = shells
                .value(i)
                .map_err(|x| x.into_failure("HLRBRep_PolyAlgo::OutLinedShape"))?;
            let shell = shell.get().ok_or_else(|| {
                Failure::with_message(
                    "Standard_NullObject",
                    "HLRBRep_PolyAlgo::OutLinedShape: a null shell",
                )
            })?;
            for bp in shell.edges().iter() {
                if bp.int_line()
                    && map.contains(self.f_map.find_key(bp.indices().shape_index as usize)?)
                {
                    let p = bp.points();
                    let e = BRepLibMakeEdge::from_points(p.pnt1.into(), p.pnt2.into())?;
                    b.add(&mut result, e.shape()?)?;
                }
            }
        }
        Ok(result)
    }
}

fn has_triangulation(face: &Shape) -> Result<bool, Failure> {
    let face = rustycad_tkbrep::topods::face(face)?;
    let (tri, _) = rustycad_tkbrep::brep::BRepTool::triangulation(
        &face,
        rustycad_tkmath::poly::mesh_purpose::NONE,
    )?;
    Ok(!tri.is_null())
}

mod bipoints;
mod finish;
mod insert;
mod orient;
mod outline;
mod store;
