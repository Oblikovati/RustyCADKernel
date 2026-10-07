//! HLRBRep_InternalAlgo.hxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{Data, FaceIterator, ShapeBounds, ShapeToHLR};
use crate::hlr_algo::{
    MinMaxIndices, Projector, add_min_max, copy_min_max, decode_min_max, encode_min_max,
};
use crate::hlr_topo_brep::{MapOfShapeTool, OutLiner};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::top_abs::Orientation;
use std::sync::RwLockWriteGuard;

/// HLRBRep_InternalAlgo: the exact hidden line algorithm on a sequence of
/// outlined shapes: `Update` builds one data structure of all of them, the `Hide`
/// members hide the edges of each shape by its own faces and by the faces of the
/// others, the `Select` and `Show` members restrict and reset the result.
///
/// A transient shared by handles: the data structure, the projector, the shapes and
/// the surface tools sit behind locks and every member takes `&self`; `Projector()`
/// returns a copy, `SeqOfShapeBounds()` a write guard and `ShapeBounds(I)` a copy.
///
/// ```
/// use rustycad_tkhlr::hlr_brep::InternalAlgo;
/// let algo = InternalAlgo::new()?;
/// assert_eq!(algo.nb_shapes(), 0);
/// algo.update()?;
/// assert!(algo.data_structure().is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct InternalAlgo {
    base: StandardTransient,
    ds: SharedState<Handle<Data>>,
    proj: SharedState<Projector>,
    shapes: SharedState<Vec<ShapeBounds>>,
    mst: SharedState<MapOfShapeTool>,
}
impl_transient!(
    InternalAlgo,
    "HLRBRep_InternalAlgo",
    StandardTransient,
    base
);

pub(super) fn unknown_shape(member: &str, i: i32) -> Failure {
    Failure::with_message(
        "Standard_OutOfRange",
        &format!("HLRBRep_InternalAlgo::{member} : unknown Shape {i}; expected 1 to NbShapes"),
    )
}

impl InternalAlgo {
    /// HLRBRep_InternalAlgo(): no shape, the default projector; `InternalAlgo::new()?`.
    pub fn new() -> Result<Self, Failure> {
        Ok(Self {
            base: StandardTransient::default(),
            ds: SharedState::new(Handle::default()),
            proj: SharedState::new(Projector::new()?),
            shapes: SharedState::new(Vec::new()),
            mst: SharedState::new(MapOfShapeTool::default()),
        })
    }

    /// HLRBRep_InternalAlgo(A): the data structure, projector and shapes of `a` (the
    /// surface tools are not copied); `InternalAlgo::from_algo(&a)?`.
    pub fn from_algo(a: &InternalAlgo) -> Result<Self, Failure> {
        let algo = Self::new()?;
        *algo.ds.write() = a.data_structure();
        *algo.proj.write() = a.projector();
        *algo.shapes.write() = a.seq_of_shape_bounds().clone();
        Ok(algo)
    }

    /// Projector(P); `algo.set_projector(&p)`.
    pub fn set_projector(&self, p: &Projector) {
        *self.proj.write() = *p;
    }

    /// Projector(): a copy of the projector; `algo.projector()`.
    pub fn projector(&self) -> Projector {
        *self.proj.read()
    }

    /// Update(): the data structure of the shapes (each loaded by ShapeToHLR, one
    /// that fails is empty; several are written one after the other), updated by the
    /// projector, and the box of each shape; `algo.update()?`.
    pub fn update(&self) -> Result<(), Failure> {
        let mut shapes = self.shapes.read().clone();
        if shapes.is_empty() {
            return Ok(());
        }
        let proj = self.projector();
        let parts = self.load_shapes(&mut shapes, &proj)?;
        let ds = if parts.len() == 1 {
            parts[0].clone()
        } else {
            Self::write_parts(&mut shapes, &parts)?
        };
        let data = ds
            .get()
            .ok_or_else(|| Failure::new("HLRBRep_InternalAlgo::Update: no data structure"))?;
        data.update(&proj)?;
        shape_boxes(data, &mut shapes)?;
        *self.ds.write() = ds;
        *self.shapes.write() = shapes;
        Ok(())
    }

    // The data structure of each shape, and its ranges from 1.
    fn load_shapes(
        &self,
        shapes: &mut [ShapeBounds],
        proj: &Projector,
    ) -> Result<Vec<Handle<Data>>, Failure> {
        let mut mst = self.mst.write();
        let mut parts = Vec::with_capacity(shapes.len());
        for sb in shapes.iter_mut() {
            let (ds, dv, de, df) =
                match ShapeToHLR::load(sb.shape(), proj, &mut mst, sb.nb_of_iso()) {
                    Ok(ds) => {
                        let d = ds.get().ok_or_else(|| {
                            Failure::new("HLRBRep_InternalAlgo::Update: no data structure")
                        })?;
                        let sizes = (d.nb_vertices(), d.nb_edges(), d.nb_faces());
                        (ds, sizes.0, sizes.1, sizes.2)
                    }
                    Err(_) => (Handle::new(Data::new(0, 0, 0)?), 0, 0, 0),
                };
            *sb = ShapeBounds::with_data(
                sb.shape(),
                sb.shape_data(),
                sb.nb_of_iso(),
                (1, dv, 1, de, 1, df),
            );
            parts.push(ds);
        }
        Ok(parts)
    }

    // One data structure of all the parts, each written after the previous ones.
    fn write_parts(
        shapes: &mut [ShapeBounds],
        parts: &[Handle<Data>],
    ) -> Result<Handle<Data>, Failure> {
        let (nv, ne, nf) = shapes.iter().fold((0, 0, 0), |(v, e, f), sb| {
            let (dv, de, df) = sb.sizes();
            (v + dv, e + de, f + df)
        });
        let ds = Handle::new(Data::new(nv, ne, nf)?);
        let data = ds
            .get()
            .ok_or_else(|| Failure::new("HLRBRep_InternalAlgo::Update: no data structure"))?;
        let (mut nv, mut ne, mut nf) = (0, 0, 0);
        for (sb, part) in shapes.iter_mut().zip(parts) {
            let (dv, de, df) = sb.sizes();
            sb.translate(nv, ne, nf);
            let part = part
                .get()
                .ok_or_else(|| Failure::new("HLRBRep_InternalAlgo::Update: no part"))?;
            data.write(part, nv, ne, nf)?;
            nv += dv;
            ne += de;
            nf += df;
        }
        Ok(ds)
    }

    /// Load(S, SData, nbIso): appends a shape with user data and forgets the data
    /// structure; `algo.load_with_data(&outliner, &data, 0)`.
    pub fn load_with_data(
        &self,
        s: &Handle<OutLiner>,
        s_data: &Handle<StandardTransient>,
        nb_iso: i32,
    ) {
        self.shapes.write().push(ShapeBounds::with_data(
            s,
            s_data,
            nb_iso,
            (0, 0, 0, 0, 0, 0),
        ));
        *self.ds.write() = Handle::default();
    }

    /// Load(S, nbIso); `algo.load(&outliner, 0)`.
    pub fn load(&self, s: &Handle<OutLiner>, nb_iso: i32) {
        self.shapes
            .write()
            .push(ShapeBounds::with_shape(s, nb_iso, (0, 0, 0, 0, 0, 0)));
        *self.ds.write() = Handle::default();
    }

    /// Index(S): the index of the outliner, 0 when absent; `algo.index(&outliner)`.
    pub fn index(&self, s: &Handle<OutLiner>) -> i32 {
        let shapes = self.shapes.read();
        shapes
            .iter()
            .position(|sb| sb.shape().object_address() == s.object_address())
            .map_or(0, |i| i as i32 + 1)
    }

    fn check(&self, member: &str, i: i32) -> Result<usize, Failure> {
        let n = self.shapes.read().len() as i32;
        if i <= 0 || i > n {
            return Err(unknown_shape(member, i));
        }
        Ok(i as usize - 1)
    }

    /// Remove(I): removes the shape `i`, the surface tools and the data structure;
    /// `algo.remove(1)?`.
    pub fn remove(&self, i: i32) -> Result<(), Failure> {
        let k = self.check("Remove", i)?;
        self.shapes.write().remove(k);
        self.mst.write().clear(false);
        *self.ds.write() = Handle::default();
        Ok(())
    }

    /// ShapeData(I, SData); `algo.set_shape_data(1, &data)?`.
    pub fn set_shape_data(
        &self,
        i: i32,
        s_data: &Handle<StandardTransient>,
    ) -> Result<(), Failure> {
        let k = self.check("ShapeData", i)?;
        self.shapes.write()[k].set_shape_data(s_data);
        Ok(())
    }

    /// SeqOfShapeBounds(): the shapes, for edition; `algo.seq_of_shape_bounds()`.
    pub fn seq_of_shape_bounds(&self) -> RwLockWriteGuard<'_, Vec<ShapeBounds>> {
        self.shapes.write()
    }

    /// NbShapes(); `algo.nb_shapes()`.
    pub fn nb_shapes(&self) -> i32 {
        self.shapes.read().len() as i32
    }

    /// ShapeBounds(I): a copy of the bounds of the shape `i`; `algo.shape_bounds(1)?`.
    pub fn shape_bounds(&self, i: i32) -> Result<ShapeBounds, Failure> {
        let k = self.check("ShapeBounds", i)?;
        Ok(self.shapes.read()[k].clone())
    }

    /// DataStructure(); `algo.data_structure()`.
    pub fn data_structure(&self) -> Handle<Data> {
        self.ds.read().clone()
    }

    // The data structure, when there is one.
    pub(super) fn data(&self) -> Option<Handle<Data>> {
        let ds = self.data_structure();
        (!ds.is_null()).then_some(ds)
    }

    /// InitEdgeStatus(): shows the selected edges, hides the edges of the selected
    /// faces, then shows again the edges of the faces that can be seen;
    /// `algo.init_edge_status()?`.
    pub fn init_edge_status(&self) -> Result<(), Failure> {
        let ds = self.data_structure();
        let ds = ds.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "HLRBRep_InternalAlgo::InitEdgeStatus: no data structure; expected Update",
            )
        })?;
        let mut ed = ds.e_data_array();
        let fd = ds.f_data_array();
        for e in 1..=ds.nb_edges() {
            let edge = ed
                .change_value(e)
                .map_err(|x| x.into_failure("HLRBRep_InternalAlgo::InitEdgeStatus"))?;
            if edge.selected() {
                edge.status_mut().show_all();
            }
        }
        let mut face_it = FaceIterator::new();
        for f in 1..=ds.nb_faces() {
            let face = fd
                .value(f)
                .map_err(|x| x.into_failure("HLRBRep_InternalAlgo::InitEdgeStatus"))?;
            if !face.selected() {
                continue;
            }
            face_it.init_edge(face)?;
            while face_it.more_edge() {
                let edf = ed
                    .change_value(face_it.edge()?)
                    .map_err(|x| x.into_failure("HLRBRep_InternalAlgo::InitEdgeStatus"))?;
                if edf.selected() {
                    edf.status_mut().hide_all();
                }
                face_it.next_edge()?;
            }
        }
        for f in 1..=ds.nb_faces() {
            let face = fd
                .value(f)
                .map_err(|x| x.into_failure("HLRBRep_InternalAlgo::InitEdgeStatus"))?;
            if !face_visible(face) {
                continue;
            }
            face_it.init_edge(face)?;
            while face_it.more_edge() {
                let edf = ed
                    .change_value(face_it.edge()?)
                    .map_err(|x| x.into_failure("HLRBRep_InternalAlgo::InitEdgeStatus"))?;
                if edf.selected() && !edf.vertical() {
                    edf.status_mut().show_all();
                }
                face_it.next_edge()?;
            }
        }
        Ok(())
    }
}

// Whether the edges of a face can be seen: not a selected closed face seen from
// the side, nor turned away.
fn face_visible(fd: &super::FaceData) -> bool {
    if !(fd.selected() && fd.closed()) {
        return true;
    }
    if fd.side() {
        return false;
    }
    if fd.with_out_l() {
        return true;
    }
    match fd.orientation() {
        Orientation::Reversed => fd.back(),
        Orientation::Forward => !fd.back(),
        Orientation::External | Orientation::Internal => true,
    }
}

// The box of each shape: the union of the boxes of its edges and of its faces
// (the native box variables keep the previous shape's box for a shape without
// edges).
fn shape_boxes(data: &Data, shapes: &mut [ShapeBounds]) -> Result<(), Failure> {
    let ed = data.e_data_array();
    let fd = data.f_data_array();
    let (mut shap_min, mut shap_max) = (MinMaxIndices::default(), MinMaxIndices::default());
    let (mut the_min, mut the_max) = (MinMaxIndices::default(), MinMaxIndices::default());
    for sb in shapes.iter_mut() {
        let mut first_time = true;
        let (_, _, e1, e2, f1, f2) = sb.bounds();
        for e in e1..=e2 {
            let edge = ed
                .value(e)
                .map_err(|x| x.into_failure("HLRBRep_InternalAlgo::Update"))?;
            decode_min_max(edge.min_max(), &mut the_min, &mut the_max);
            if first_time {
                first_time = false;
                copy_min_max(&the_min, &the_max, &mut shap_min, &mut shap_max);
            } else {
                add_min_max(&the_min, &the_max, &mut shap_min, &mut shap_max);
            }
        }
        for f in f1..=f2 {
            let face = fd
                .value(f)
                .map_err(|x| x.into_failure("HLRBRep_InternalAlgo::Update"))?;
            let wires = face.wires().get().ok_or_else(|| {
                Failure::with_message(
                    "Standard_NullObject",
                    "HLRBRep_InternalAlgo::Update: a face without wire blocks",
                )
            })?;
            decode_min_max(&wires.min_max(), &mut the_min, &mut the_max);
            add_min_max(&the_min, &the_max, &mut shap_min, &mut shap_max);
        }
        let mut min_max_shap = MinMaxIndices::default();
        encode_min_max(&shap_min, &shap_max, &mut min_max_shap);
        sb.update_min_max(&min_max_shap);
    }
    Ok(())
}

mod hide;
