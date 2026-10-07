//! HLRBRep_Data.hxx/.lxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::rejection_cache::RejectionCache;
use super::{EdgeData, FaceData, FaceIterator, Intersector};
use crate::hlr_algo::{Interference, MinMaxIndices, Projector};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{Array1, ArrayError};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorTopolTool;
use std::sync::{Arc, RwLockWriteGuard};

/// The edge records `myEData` (indices 0 to NbEdges).
pub type EdgeDataArray = Array1<'static, EdgeData>;
/// The face records `myFData` (indices 0 to NbFaces).
pub type FaceDataArray = Array1<'static, FaceData>;

/// HLRBRep_Data: the edges and faces of the hidden line algorithm with their boxes,
/// and the walks of the hiding: the edges a face can hide (`InitEdge`, `MoreEdge`,
/// `NextEdge`), the interferences of an edge with the boundary of the face
/// (`InitInterference` ...), and the classification of edge points against it.
///
/// A transient shared by handles (the Hider, the edge interference tool and the
/// algorithm hold it): the edge records, the face records, the two maps and the walk
/// state each sit behind a lock, so every member takes `&self`; the native members
/// that return references return write guards, and a caller holds no guard while it
/// calls another member (docs/hlr-design.md section 6). The native pointers into the
/// records (`iFaceData`, `myLEData`, `myFEGeom`, ...) are indices.
///
/// ```
/// use rustycad_tkhlr::hlr_brep::Data;
/// let ds = Data::new(0, 0, 0)?;
/// assert_eq!((ds.nb_vertices(), ds.nb_edges(), ds.nb_faces()), (0, 0, 0));
/// assert_eq!(ds.e_data_array().len(), 1);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct Data {
    base: StandardTransient,
    nb_vertices: i32,
    nb_edges: i32,
    nb_faces: i32,
    e_map: SharedState<IndexedMapOfShape>,
    f_map: SharedState<IndexedMapOfShape>,
    e_data: SharedState<EdgeDataArray>,
    f_data: SharedState<FaceDataArray>,
    walk: SharedState<Walk>,
}
impl_transient!(Data, "HLRBRep_Data", StandardTransient, base);

// The scalar members of the native class: the box scaling of Update, the current
// hiding face (`iFace*`), the current hidden edge (`myLE*`), the current boundary edge
// of the face (`myFE*`) and the interference walk. Natively several are
// uninitialized until the member that sets them; the port starts them at 0.
pub(super) struct Walk {
    pub(super) edge_indices: Vec<i32>,
    pub(super) toler: f32,
    pub(super) proj: Arc<Projector>,
    pub(super) big_size: f64,
    pub(super) face_itr1: FaceIterator,
    pub(super) face_itr2: FaceIterator,
    pub(super) i_face: i32,
    // `iFaceGeom`: the face whose surface is loaded (Update leaves the last face).
    pub(super) i_face_geom: Option<i32>,
    pub(super) i_face_type: SurfaceType,
    pub(super) i_face_back: bool,
    pub(super) i_face_simp: bool,
    pub(super) i_face_smpl: bool,
    pub(super) i_face_test: bool,
    pub(super) hide_count: i32,
    pub(super) deca: [f64; 16],
    pub(super) sur_d: [f64; 16],
    pub(super) cur_sort_ed: i32,
    pub(super) nbr_sort_ed: i32,
    pub(super) le: i32,
    pub(super) le_out_line: bool,
    pub(super) le_internal: bool,
    pub(super) le_double: bool,
    pub(super) le_iso_line: bool,
    pub(super) le_type: CurveType,
    pub(super) le_tol: f32,
    // `myLEMinMax`: InitBoundSort points it at other edges than `myLE`.
    pub(super) le_min_max: i32,
    // The edge of `myLLProps.SetCurve` (InitInterference).
    pub(super) ll_curve: i32,
    pub(super) fe: i32,
    pub(super) fe_ori: Orientation,
    pub(super) fe_out_line: bool,
    pub(super) fe_internal: bool,
    pub(super) fe_double: bool,
    // `myFEData` and `myFEGeom`: set by other members than `myFE`, so kept apart.
    pub(super) fe_data: i32,
    pub(super) fe_geom: i32,
    pub(super) fe_type: CurveType,
    pub(super) fe_tol: f32,
    pub(super) intersector: Intersector,
    // The face of `myIntersector.Load(iFaceGeom)` (InitEdge).
    pub(super) loaded_face: Option<i32>,
    pub(super) classifier: Handle<BRepTopAdaptorTopolTool>,
    pub(super) same_vertex: bool,
    pub(super) intersected: bool,
    pub(super) nb_points: i32,
    pub(super) nb_segments: i32,
    pub(super) i_interf: i32,
    pub(super) intf: Interference,
    pub(super) above_intf: bool,
    pub(super) rejection_cache: Option<RejectionCache>,
}

// The locked parts one member works on: the walk, the edges and the faces, locked in
// this order.
pub(super) struct Body<'a> {
    pub(super) w: &'a mut Walk,
    pub(super) e: &'a mut EdgeDataArray,
    pub(super) f: &'a mut FaceDataArray,
}

/// Standard_OutOfRange (or RangeError) of a native array access in HLRBRep_Data.
pub(super) fn data_array(member: &'static str) -> impl Fn(ArrayError) -> Failure {
    move |error: ArrayError| error.into_failure(member)
}

/// The box rejection of HLRBRep_Data: some of the 15 differences `a.Max - b.Min` and
/// `b.Max - a.Min` (the last direction tests only the first) has a negative 16-bit
/// half; the native int subtraction wraps.
pub(super) fn box_rejected(a: &MinMaxIndices, b: &MinMaxIndices) -> bool {
    let negative = |x: i32, y: i32| (x.wrapping_sub(y) as u32 & 0x8000_8000) != 0;
    (0..7).any(|k| negative(a.max[k], b.min[k]) || negative(b.max[k], a.min[k]))
        || negative(a.max[7], b.min[7])
}

impl Data {
    /// HLRBRep_Data(NV, NE, NF): `NE + 1` edge records and `NF + 1` face records
    /// (indices from 0), tolerance 1e-5; `Data::new(8, 12, 6)?`.
    pub fn new(nv: i32, ne: i32, nf: i32) -> Result<Self, Failure> {
        let e_data = EdgeDataArray::with_bounds(0, ne).map_err(data_array("HLRBRep_Data"))?;
        let f_data = FaceDataArray::with_bounds(0, nf).map_err(data_array("HLRBRep_Data"))?;
        let walk = Walk {
            edge_indices: vec![0; ne.max(0) as usize + 1],
            toler: 1e-5,
            proj: Arc::new(Projector::new()?),
            big_size: 0.,
            face_itr1: FaceIterator::new(),
            face_itr2: FaceIterator::new(),
            i_face: 0,
            i_face_geom: None,
            i_face_type: SurfaceType::OtherSurface,
            i_face_back: false,
            i_face_simp: false,
            i_face_smpl: false,
            i_face_test: false,
            hide_count: 0,
            deca: [0.; 16],
            sur_d: [0.; 16],
            cur_sort_ed: 0,
            nbr_sort_ed: 0,
            le: 0,
            le_out_line: false,
            le_internal: false,
            le_double: false,
            le_iso_line: false,
            le_type: CurveType::OtherCurve,
            le_tol: 0.,
            le_min_max: 0,
            ll_curve: 0,
            fe: 0,
            fe_ori: Orientation::Forward,
            fe_out_line: false,
            fe_internal: false,
            fe_double: false,
            fe_data: 0,
            fe_geom: 0,
            fe_type: CurveType::OtherCurve,
            fe_tol: 0.,
            intersector: Intersector::new(),
            loaded_face: None,
            classifier: Handle::default(),
            same_vertex: false,
            intersected: false,
            nb_points: 0,
            nb_segments: 0,
            i_interf: 0,
            intf: Interference::new(),
            above_intf: false,
            rejection_cache: Some(RejectionCache::new(ne.max(0) as usize)),
        };
        Ok(Self {
            base: StandardTransient::default(),
            nb_vertices: nv,
            nb_edges: ne,
            nb_faces: nf,
            e_map: SharedState::new(IndexedMapOfShape::default()),
            f_map: SharedState::new(IndexedMapOfShape::default()),
            e_data: SharedState::new(e_data),
            f_data: SharedState::new(f_data),
            walk: SharedState::new(walk),
        })
    }

    // Locks the walk, the edges and the faces (in this order) for one member.
    pub(super) fn body<R>(&self, f: impl FnOnce(&mut Body<'_>) -> R) -> R {
        let mut w = self.walk.write();
        let mut e = self.e_data.write();
        let mut fd = self.f_data.write();
        f(&mut Body {
            w: &mut w,
            e: &mut e,
            f: &mut fd,
        })
    }

    /// Destroy(): frees the intersection cache (the next member that reads it fails
    /// where the native null pointer crashes); `ds.destroy()`.
    pub fn destroy(&self) {
        self.walk.write().rejection_cache = None;
    }

    /// EDataArray(): the edge records, for edition; `ds.e_data_array()`.
    pub fn e_data_array(&self) -> RwLockWriteGuard<'_, EdgeDataArray> {
        self.e_data.write()
    }

    /// FDataArray(): the face records, for edition; `ds.f_data_array()`.
    pub fn f_data_array(&self) -> RwLockWriteGuard<'_, FaceDataArray> {
        self.f_data.write()
    }

    /// Tolerance(tol); `ds.set_tolerance(1e-4)`.
    pub fn set_tolerance(&self, tol: f32) {
        self.walk.write().toler = tol;
    }

    /// Tolerance(); `ds.tolerance()`.
    pub fn tolerance(&self) -> f32 {
        self.walk.read().toler
    }

    /// Projector(): a copy of the projector (the native reference; see
    /// [`Data::change_projector`] to edit it); `ds.projector()`.
    pub fn projector(&self) -> Projector {
        *self.walk.read().proj
    }

    /// Projector() edited in place: applies `edit` to the projector and shares the
    /// result with every curve and surface that shared the previous one, as the
    /// native pointers to `myProj` see the edit;
    /// `ds.change_projector(|p| p.scaled(true))?`.
    pub fn change_projector(
        &self,
        edit: impl FnOnce(&mut Projector) -> Result<(), Failure>,
    ) -> Result<(), Failure> {
        self.body(|b| {
            let old = b.w.proj.clone();
            let mut p = *old;
            let result = edit(&mut p);
            let new = Arc::new(p);
            b.w.proj = new.clone();
            for ed in b.e.iter_mut() {
                if ed
                    .geometry()
                    .projector_handle()
                    .is_some_and(|h| Arc::ptr_eq(h, &old))
                {
                    ed.change_geometry().set_projector(Some(new.clone()));
                }
            }
            for fd in b.f.iter_mut() {
                if fd
                    .geometry()
                    .projector_handle()
                    .is_some_and(|h| Arc::ptr_eq(h, &old))
                {
                    fd.geometry_mut().set_projector(Some(new.clone()));
                }
            }
            result
        })
    }

    /// NbVertices(); `ds.nb_vertices()`.
    pub fn nb_vertices(&self) -> i32 {
        self.nb_vertices
    }

    /// NbEdges(); `ds.nb_edges()`.
    pub fn nb_edges(&self) -> i32 {
        self.nb_edges
    }

    /// NbFaces(); `ds.nb_faces()`.
    pub fn nb_faces(&self) -> i32 {
        self.nb_faces
    }

    /// EdgeMap(): the edges of the records, for edition; `ds.edge_map()`.
    pub fn edge_map(&self) -> RwLockWriteGuard<'_, IndexedMapOfShape> {
        self.e_map.write()
    }

    /// FaceMap(): the faces of the records, for edition; `ds.face_map()`.
    pub fn face_map(&self) -> RwLockWriteGuard<'_, IndexedMapOfShape> {
        self.f_map.write()
    }

    /// HidingTheFace(): whether the hiding face is tested against itself;
    /// `ds.hiding_the_face()`.
    pub fn hiding_the_face(&self) -> bool {
        self.walk.read().i_face_test
    }

    /// SimpleHidingFace(); `ds.simple_hiding_face()`.
    pub fn simple_hiding_face(&self) -> bool {
        self.walk.read().i_face_simp
    }

    /// MoreInterference(); `ds.more_interference()`.
    pub fn more_interference(&self) -> bool {
        self.walk.read().more_interference()
    }

    /// Interference(): a copy of the current interference (the native reference);
    /// `ds.interference()`.
    pub fn interference(&self) -> Interference {
        self.walk.read().intf
    }

    /// EdgeOfTheHidingFace(E, ED): whether the edge was marked by the hiding face;
    /// `ds.edge_of_the_hiding_face(3, &edge)`.
    pub fn edge_of_the_hiding_face(&self, _e: i32, ed: &EdgeData) -> bool {
        ed.hide_count() == self.walk.read().hide_count - 1
    }

    /// AboveInterference(): whether the last rejected point was above the face;
    /// `ds.above_interference()`.
    pub fn above_interference(&self) -> bool {
        self.walk.read().above_intf
    }

    /// IsBadFace(): whether the loaded surface is periodic with a range of more than
    /// two periods (garbage data); `ds.is_bad_face()?`.
    pub fn is_bad_face(&self) -> Result<bool, Failure> {
        let Some(i) = self.walk.read().i_face_geom else {
            return Ok(false);
        };
        let faces = self.f_data.read();
        let g = faces
            .value(i)
            .map_err(data_array("HLRBRep_Data::IsBadFace"))?
            .geometry();
        if g.is_u_periodic()?
            && g.u_period()? * 2. < g.last_u_parameter()? - g.first_u_parameter()?
        {
            return Ok(true);
        }
        Ok(g.is_v_periodic()?
            && g.v_period()? * 2. < g.last_v_parameter()? - g.first_v_parameter()?)
    }
}

impl Walk {
    pub(super) fn more_interference(&self) -> bool {
        self.i_interf <= self.nb_points + 2 * self.nb_segments
    }

    pub(super) fn cache(&mut self) -> Result<&mut RejectionCache, Failure> {
        self.rejection_cache.as_mut().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "HLRBRep_Data: the intersection cache is destroyed; expected a Data before Destroy",
            )
        })
    }
}

mod classify;
pub use classify::LocalGeometry2D;
mod edges;
mod interference;
mod rejected_point;
mod update;
mod write;
