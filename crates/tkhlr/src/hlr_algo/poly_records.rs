// Rust translation of ModelingAlgorithms/TKHLR/HLRAlgo/HLRAlgo_TriangleData.hxx, HLRAlgo_PolyInternalSegment.hxx, HLRAlgo_PolyHidingData.hxx and HLRAlgo_PolyMask.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The plain records of the polyhedral algorithm: triangles, internal segments, the
//! hiding plane of a triangle and the triangle flag masks.
use rustycad_tkmath::gp::Xyz;

/// HLRAlgo_TriangleData: the three node indices of a triangle and its flags
/// ([`PolyMask`]). Natively the fields are uninitialized; the Rust default is zero.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[allow(missing_docs)] // The native field names, documented on the type.
pub struct TriangleData {
    pub node1: i32,
    pub node2: i32,
    pub node3: i32,
    pub flags: i32,
}

/// HLRAlgo_PolyInternalSegment: a segment of an internal triangulation: its two end
/// nodes, the next segments around each end, and the triangles on its two sides.
/// Natively the fields are uninitialized; the Rust default is zero.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[allow(missing_docs)] // The native field names, documented on the type.
pub struct PolyInternalSegment {
    pub lst_sg1: i32,
    pub lst_sg2: i32,
    pub nxt_sg1: i32,
    pub nxt_sg2: i32,
    pub conex1: i32,
    pub conex2: i32,
}

/// HLRAlgo_PolyHidingData::TriangleIndices: the triangle index and its packed box.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[allow(missing_docs)] // The native field names, documented on the type.
pub struct HidingTriangleIndices {
    pub index: i32,
    pub min: i32,
    pub max: i32,
}

/// HLRAlgo_PolyHidingData::PlaneT: the plane `normal . p = d` of a hiding triangle.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[allow(missing_docs)] // The native field names, documented on the type.
pub struct HidingPlane {
    pub normal: Xyz,
    pub d: f64,
}

/// HLRAlgo_PolyHidingData: a hiding triangle: its indices and its plane. Natively the
/// indices are uninitialized by the default constructor; the Rust default is zero.
///
/// ```
/// use rustycad_tkhlr::hlr_algo::PolyHidingData;
/// let mut hiding = PolyHidingData::default();
/// hiding.set(3, 10, 20, 0.0, 0.0, 1.0, 2.0);
/// assert_eq!(hiding.indices().index, 3);
/// assert_eq!(hiding.plane().d, 2.0);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PolyHidingData {
    indices: HidingTriangleIndices,
    plane: HidingPlane,
}

impl PolyHidingData {
    /// Set(Index, Minim, Maxim, A, B, C, D): the triangle index, its packed box and
    /// its plane `A x + B y + C z = D`; `hiding.set(1, 0, 0, 0.0, 0.0, 1.0, 0.0)`.
    #[allow(clippy::too_many_arguments)] // The native member takes these 7 values.
    pub fn set(&mut self, index: i32, minim: i32, maxim: i32, a: f64, b: f64, c: f64, d: f64) {
        self.indices = HidingTriangleIndices {
            index,
            min: minim,
            max: maxim,
        };
        self.plane = HidingPlane {
            normal: Xyz::new(a, b, c),
            d,
        };
    }

    /// The indices, read only; `hiding.indices().index`.
    pub fn indices(&self) -> &HidingTriangleIndices {
        &self.indices
    }

    /// Indices(): the indices, writable in place; `hiding.indices_mut().min = 0`.
    pub fn indices_mut(&mut self) -> &mut HidingTriangleIndices {
        &mut self.indices
    }

    /// The plane, read only; `hiding.plane().d`.
    pub fn plane(&self) -> &HidingPlane {
        &self.plane
    }

    /// Plane(): the plane, writable in place; `hiding.plane_mut().d = 1.0`.
    pub fn plane_mut(&mut self) -> &mut HidingPlane {
        &mut self.plane
    }
}

/// HLRAlgo_PolyMask: the bits of the triangle flags (outline edges, the "greater along
/// X" choice of each edge, back, side, hiding, flat, on outline, oriented and frontier
/// back faces); the discriminants are the native masks.
///
/// ```
/// use rustycad_tkhlr::hlr_algo::PolyMask;
/// assert_eq!(PolyMask::FMskHiding as i32, 256);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
#[allow(missing_docs)] // The native enumerator names, documented on the type.
pub enum PolyMask {
    EMskOutLin1 = 1,
    EMskOutLin2 = 2,
    EMskOutLin3 = 4,
    EMskGrALin1 = 8,
    EMskGrALin2 = 16,
    EMskGrALin3 = 32,
    FMskBack = 64,
    FMskSide = 128,
    FMskHiding = 256,
    FMskFlat = 512,
    FMskOnOutL = 1024,
    FMskOrBack = 2048,
    FMskFrBack = 4096,
}
