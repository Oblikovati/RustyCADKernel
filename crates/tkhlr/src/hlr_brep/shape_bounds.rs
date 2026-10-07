//! HLRBRep_ShapeBounds.hxx/.lxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::hlr_algo::MinMaxIndices;
use crate::hlr_topo_brep::OutLiner;
use rustycad_tkernel::standard::{Handle, StandardTransient};

/// HLRBRep_ShapeBounds: one shape of the hidden line algorithm: its outliner, its
/// user data, its number of isolines, the ranges of its vertices, edges and faces
/// in the data structure, and its box (natively uninitialized until UpdateMinMax;
/// zero here).
///
/// ```
/// use rustycad_tkhlr::hlr_brep::ShapeBounds;
/// let mut sb = ShapeBounds::new();
/// sb.translate(1, 2, 3);
/// assert_eq!(sb.bounds(), (1, 1, 2, 2, 3, 3));
/// assert_eq!(sb.sizes(), (1, 1, 1));
/// ```
#[derive(Clone, Default)]
pub struct ShapeBounds {
    shape: Handle<OutLiner>,
    shape_data: Handle<StandardTransient>,
    nb_iso: i32,
    vert_start: i32,
    vert_end: i32,
    edge_start: i32,
    edge_end: i32,
    face_start: i32,
    face_end: i32,
    min_max: MinMaxIndices,
}

/// The vertex, edge and face ranges `(V1, V2, E1, E2, F1, F2)` of a shape.
pub type ShapeRanges = (i32, i32, i32, i32, i32, i32);

impl ShapeBounds {
    /// HLRBRep_ShapeBounds(): no shape, every range 0; `ShapeBounds::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// HLRBRep_ShapeBounds(S, SData, nbIso, V1, V2, E1, E2, F1, F2);
    /// `ShapeBounds::with_data(&s, &data, 0, (1, 8, 1, 12, 1, 6))`.
    pub fn with_data(
        s: &Handle<OutLiner>,
        s_data: &Handle<StandardTransient>,
        nb_iso: i32,
        r: ShapeRanges,
    ) -> Self {
        Self {
            shape: s.clone(),
            shape_data: s_data.clone(),
            nb_iso,
            vert_start: r.0,
            vert_end: r.1,
            edge_start: r.2,
            edge_end: r.3,
            face_start: r.4,
            face_end: r.5,
            min_max: MinMaxIndices::default(),
        }
    }

    /// HLRBRep_ShapeBounds(S, nbIso, V1, V2, E1, E2, F1, F2): without user data;
    /// `ShapeBounds::with_shape(&s, 0, (1, 8, 1, 12, 1, 6))`.
    pub fn with_shape(s: &Handle<OutLiner>, nb_iso: i32, r: ShapeRanges) -> Self {
        Self::with_data(s, &Handle::default(), nb_iso, r)
    }

    /// Translate(NV, NE, NF): shifts the ranges; `sb.translate(8, 12, 6)`.
    pub fn translate(&mut self, nv: i32, ne: i32, nf: i32) {
        self.vert_start += nv;
        self.vert_end += nv;
        self.edge_start += ne;
        self.edge_end += ne;
        self.face_start += nf;
        self.face_end += nf;
    }

    /// Shape(S); `sb.set_shape(&outliner)`.
    pub fn set_shape(&mut self, s: &Handle<OutLiner>) {
        self.shape = s.clone();
    }

    /// Shape(); `sb.shape()`.
    pub fn shape(&self) -> &Handle<OutLiner> {
        &self.shape
    }

    /// ShapeData(SD); `sb.set_shape_data(&data)`.
    pub fn set_shape_data(&mut self, sd: &Handle<StandardTransient>) {
        self.shape_data = sd.clone();
    }

    /// ShapeData(); `sb.shape_data()`.
    pub fn shape_data(&self) -> &Handle<StandardTransient> {
        &self.shape_data
    }

    /// NbOfIso(nbIso); `sb.set_nb_of_iso(2)`.
    pub fn set_nb_of_iso(&mut self, nb_iso: i32) {
        self.nb_iso = nb_iso;
    }

    /// NbOfIso(); `sb.nb_of_iso()`.
    pub fn nb_of_iso(&self) -> i32 {
        self.nb_iso
    }

    /// Sizes(NV, NE, NF): the numbers of vertices, edges and faces; `sb.sizes()`.
    pub fn sizes(&self) -> (i32, i32, i32) {
        (
            self.vert_end + 1 - self.vert_start,
            self.edge_end + 1 - self.edge_start,
            self.face_end + 1 - self.face_start,
        )
    }

    /// Bounds(V1, V2, E1, E2, F1, F2); `sb.bounds()`.
    pub fn bounds(&self) -> ShapeRanges {
        (
            self.vert_start,
            self.vert_end,
            self.edge_start,
            self.edge_end,
            self.face_start,
            self.face_end,
        )
    }

    /// UpdateMinMax(theTotMinMax); `sb.update_min_max(&total)`.
    pub fn update_min_max(&mut self, tot_min_max: &MinMaxIndices) {
        self.min_max = *tot_min_max;
    }

    /// MinMax(); `sb.min_max()`.
    pub fn min_max(&self) -> &MinMaxIndices {
        &self.min_max
    }

    /// MinMax() for edition; `sb.min_max_mut()`.
    pub fn min_max_mut(&mut self) -> &mut MinMaxIndices {
        &mut self.min_max
    }
}
