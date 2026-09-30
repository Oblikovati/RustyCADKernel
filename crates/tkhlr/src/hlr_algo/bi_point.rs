// Rust translation of ModelingAlgorithms/TKHLR/HLRAlgo/HLRAlgo_BiPoint.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! HLRAlgo_BiPoint: one segment of a polygonal edge, in the model and projected.
use rustycad_tkmath::gp::{Xy, Xyz};

/// HLRAlgo_BiPoint::IndicesT: the shape index, the two faces around the segment with
/// the indices of the segment ends in their triangulations, the packed box of the
/// segment and its flags.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(missing_docs)] // The native field names, documented on the type.
pub struct BiPointIndices {
    pub shape_index: i32,
    pub face_conex1: i32,
    pub face1_pt1: i32,
    pub face1_pt2: i32,
    pub face_conex2: i32,
    pub face2_pt1: i32,
    pub face2_pt2: i32,
    pub min_seg: i32,
    pub max_seg: i32,
    pub seg_flags: i32,
}

impl Default for BiPointIndices {
    /// The native constructor: shape index -1, every other field 0.
    fn default() -> Self {
        Self {
            shape_index: -1,
            face_conex1: 0,
            face1_pt1: 0,
            face1_pt2: 0,
            face_conex2: 0,
            face2_pt1: 0,
            face2_pt2: 0,
            min_seg: 0,
            max_seg: 0,
            seg_flags: 0,
        }
    }
}

/// HLRAlgo_BiPoint::PointsT: the segment ends in the model (`pnt1`, `pnt2`) and in the
/// projection frame (`pnt_p1`, `pnt_p2`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[allow(missing_docs)] // The native field names, documented on the type.
pub struct BiPointPoints {
    pub pnt1: Xyz,
    pub pnt2: Xyz,
    pub pnt_p1: Xyz,
    pub pnt_p2: Xyz,
}

impl BiPointPoints {
    /// The four ends in native argument order (X1..Z1, X2..Z2, XT1..ZT1, XT2..ZT2);
    /// `BiPointPoints::from_coordinates([0.0; 12])`.
    pub fn from_coordinates(c: [f64; 12]) -> Self {
        Self {
            pnt1: Xyz::new(c[0], c[1], c[2]),
            pnt2: Xyz::new(c[3], c[4], c[5]),
            pnt_p1: Xyz::new(c[6], c[7], c[8]),
            pnt_p2: Xyz::new(c[9], c[10], c[11]),
        }
    }

    /// PntP12D: the projected first end in the view plane; `points.pnt_p1_2d()`.
    pub fn pnt_p1_2d(&self) -> Xy {
        Xy::new(self.pnt_p1.x(), self.pnt_p1.y())
    }

    /// PntP22D: the projected last end in the view plane; `points.pnt_p2_2d()`.
    pub fn pnt_p2_2d(&self) -> Xy {
        Xy::new(self.pnt_p2.x(), self.pnt_p2.y())
    }
}

/// The four regularity flags of the constructors taking booleans: `reg1` (regular
/// once), `regn` (regular n times), `outl` (outline), `intl` (internal line).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[allow(missing_docs)] // The native argument names, documented on the type.
pub struct BiPointRegularity {
    pub reg1: bool,
    pub regn: bool,
    pub outl: bool,
    pub intl: bool,
}

/// A face around the segment: its index and the indices of the segment ends in its
/// triangulation (`i1, i1p1, i1p2` or `i2, i2p1, i2p2`).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[allow(missing_docs)] // The native argument names, documented on the type.
pub struct BiPointFace {
    pub conex: i32,
    pub pt1: i32,
    pub pt2: i32,
}

const MASK_RG1_LINE: i32 = 1;
const MASK_RGN_LINE: i32 = 2;
const MASK_OUT_LINE: i32 = 4;
const MASK_INT_LINE: i32 = 8;
const MASK_HIDDEN: i32 = 16;

/// HLRAlgo_BiPoint: a segment of a polygonal edge with its projection, its faces and
/// its flags (regular once, regular n times, outline, internal line, hidden). The six
/// native constructors differ by the faces they set and by booleans or a raw flag
/// word; each clears the hidden flag.
///
/// ```
/// use rustycad_tkhlr::hlr_algo::{BiPoint, BiPointPoints, BiPointRegularity};
/// let points = BiPointPoints::from_coordinates([0.0; 12]);
/// let regularity = BiPointRegularity { outl: true, ..Default::default() };
/// let segment = BiPoint::with_regularity(points, 3, regularity);
/// assert!(segment.out_line() && !segment.hidden());
/// assert_eq!(segment.indices().shape_index, 3);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BiPoint {
    indices: BiPointIndices,
    points: BiPointPoints,
}

impl BiPoint {
    /// HLRAlgo_BiPoint(X1, ..., ZT2, Index, reg1, regn, outl, intl);
    /// `BiPoint::with_regularity(points, 1, regularity)`.
    pub fn with_regularity(
        points: BiPointPoints,
        index: i32,
        regularity: BiPointRegularity,
    ) -> Self {
        Self::build(points, index, [BiPointFace::default(); 2], 0).regular(regularity)
    }

    /// HLRAlgo_BiPoint(X1, ..., ZT2, Index, flag); `BiPoint::with_flags(points, 1, 5)`.
    pub fn with_flags(points: BiPointPoints, index: i32, flag: i32) -> Self {
        Self::build(points, index, [BiPointFace::default(); 2], flag)
    }

    /// HLRAlgo_BiPoint(X1, ..., ZT2, Index, i1, i1p1, i1p2, reg1, regn, outl, intl);
    /// `BiPoint::with_face_regularity(points, 1, face, regularity)`.
    pub fn with_face_regularity(
        points: BiPointPoints,
        index: i32,
        face1: BiPointFace,
        regularity: BiPointRegularity,
    ) -> Self {
        Self::build(points, index, [face1, BiPointFace::default()], 0).regular(regularity)
    }

    /// HLRAlgo_BiPoint(X1, ..., ZT2, Index, i1, i1p1, i1p2, flag);
    /// `BiPoint::with_face_flags(points, 1, face, 5)`.
    pub fn with_face_flags(
        points: BiPointPoints,
        index: i32,
        face1: BiPointFace,
        flag: i32,
    ) -> Self {
        Self::build(points, index, [face1, BiPointFace::default()], flag)
    }

    /// HLRAlgo_BiPoint(X1, ..., ZT2, Index, i1, i1p1, i1p2, i2, i2p1, i2p2, reg1, regn,
    /// outl, intl); `BiPoint::with_faces_regularity(points, 1, [f1, f2], regularity)`.
    pub fn with_faces_regularity(
        points: BiPointPoints,
        index: i32,
        faces: [BiPointFace; 2],
        regularity: BiPointRegularity,
    ) -> Self {
        Self::build(points, index, faces, 0).regular(regularity)
    }

    /// HLRAlgo_BiPoint(X1, ..., ZT2, Index, i1, i1p1, i1p2, i2, i2p1, i2p2, flag);
    /// `BiPoint::with_faces_flags(points, 1, [f1, f2], 5)`.
    pub fn with_faces_flags(
        points: BiPointPoints,
        index: i32,
        faces: [BiPointFace; 2],
        flag: i32,
    ) -> Self {
        Self::build(points, index, faces, flag)
    }

    /// The common body: the points, the faces and the raw flags, hidden cleared. The
    /// packed box keeps the default 0.
    fn build(points: BiPointPoints, index: i32, faces: [BiPointFace; 2], flag: i32) -> Self {
        let indices = BiPointIndices {
            shape_index: index,
            face_conex1: faces[0].conex,
            face1_pt1: faces[0].pt1,
            face1_pt2: faces[0].pt2,
            face_conex2: faces[1].conex,
            face2_pt1: faces[1].pt1,
            face2_pt2: faces[1].pt2,
            seg_flags: flag,
            ..BiPointIndices::default()
        };
        let mut bi_point = Self { indices, points };
        bi_point.set_hidden(false);
        bi_point
    }

    /// Sets the four regularity flags in the native order.
    fn regular(mut self, regularity: BiPointRegularity) -> Self {
        self.set_rg1_line(regularity.reg1);
        self.set_rgn_line(regularity.regn);
        self.set_out_line(regularity.outl);
        self.set_int_line(regularity.intl);
        self
    }

    fn flag(&self, mask: i32) -> bool {
        (self.indices.seg_flags & mask) != 0
    }

    fn set_flag(&mut self, mask: i32, on: bool) {
        if on {
            self.indices.seg_flags |= mask;
        } else {
            self.indices.seg_flags &= !mask;
        }
    }

    /// Rg1Line(): the segment is on an edge regular once; `segment.rg1_line()`.
    pub fn rg1_line(&self) -> bool {
        self.flag(MASK_RG1_LINE)
    }

    /// Rg1Line(B); `segment.set_rg1_line(true)`.
    pub fn set_rg1_line(&mut self, on: bool) {
        self.set_flag(MASK_RG1_LINE, on);
    }

    /// RgNLine(): the segment is on an edge regular n times; `segment.rgn_line()`.
    pub fn rgn_line(&self) -> bool {
        self.flag(MASK_RGN_LINE)
    }

    /// RgNLine(B); `segment.set_rgn_line(true)`.
    pub fn set_rgn_line(&mut self, on: bool) {
        self.set_flag(MASK_RGN_LINE, on);
    }

    /// OutLine(): the segment is on an outline; `segment.out_line()`.
    pub fn out_line(&self) -> bool {
        self.flag(MASK_OUT_LINE)
    }

    /// OutLine(B); `segment.set_out_line(true)`.
    pub fn set_out_line(&mut self, on: bool) {
        self.set_flag(MASK_OUT_LINE, on);
    }

    /// IntLine(): the segment is on an internal line; `segment.int_line()`.
    pub fn int_line(&self) -> bool {
        self.flag(MASK_INT_LINE)
    }

    /// IntLine(B); `segment.set_int_line(true)`.
    pub fn set_int_line(&mut self, on: bool) {
        self.set_flag(MASK_INT_LINE, on);
    }

    /// Hidden(): the segment is hidden entirely; `segment.hidden()`.
    pub fn hidden(&self) -> bool {
        self.flag(MASK_HIDDEN)
    }

    /// Hidden(B); `segment.set_hidden(true)`.
    pub fn set_hidden(&mut self, on: bool) {
        self.set_flag(MASK_HIDDEN, on);
    }

    /// The indices, read only; `segment.indices().shape_index`.
    pub fn indices(&self) -> &BiPointIndices {
        &self.indices
    }

    /// Indices(): the indices, writable in place; `segment.indices_mut().min_seg = 0`.
    pub fn indices_mut(&mut self) -> &mut BiPointIndices {
        &mut self.indices
    }

    /// The points, read only; `segment.points().pnt1`.
    pub fn points(&self) -> &BiPointPoints {
        &self.points
    }

    /// Points(): the points, writable in place; `segment.points_mut().pnt1 = p`.
    pub fn points_mut(&mut self) -> &mut BiPointPoints {
        &mut self.points
    }
}
