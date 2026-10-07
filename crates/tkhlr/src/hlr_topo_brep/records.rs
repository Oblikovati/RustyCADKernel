//! HLRTopoBRep_FaceData.hxx/.lxx/.cxx and HLRTopoBRep_VData.hxx/.lxx/.cxx, OCCT
//! 3d097a0328e71b826377d4814ab05ec3c3d23871.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;

/// HLRTopoBRep_FaceData: the internal outlines, the outlines on restrictions and the
/// isolines of one face.
///
/// ```
/// use rustycad_tkhlr::hlr_topo_brep::FaceData;
/// let data = FaceData::new();
/// assert!(data.face_int_l().is_empty());
/// ```
#[derive(Clone, Default)]
pub struct FaceData {
    int_l: List<Shape>,
    out_l: List<Shape>,
    iso_l: List<Shape>,
}

impl FaceData {
    /// HLRTopoBRep_FaceData(): three empty lists; `FaceData::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// FaceIntL(); `data.face_int_l()`.
    pub fn face_int_l(&self) -> &List<Shape> {
        &self.int_l
    }
    /// FaceOutL(); `data.face_out_l()`.
    pub fn face_out_l(&self) -> &List<Shape> {
        &self.out_l
    }
    /// FaceIsoL(); `data.face_iso_l()`.
    pub fn face_iso_l(&self) -> &List<Shape> {
        &self.iso_l
    }
    /// AddIntL(): the internal outlines, for edition; `data.add_int_l()`.
    pub fn add_int_l(&mut self) -> &mut List<Shape> {
        &mut self.int_l
    }
    /// AddOutL(): the outlines on restrictions, for edition; `data.add_out_l()`.
    pub fn add_out_l(&mut self) -> &mut List<Shape> {
        &mut self.out_l
    }
    /// AddIsoL(): the isolines, for edition; `data.add_iso_l()`.
    pub fn add_iso_l(&mut self) -> &mut List<Shape> {
        &mut self.iso_l
    }
}

/// HLRTopoBRep_VData: a vertex inserted on an edge, with its parameter.
///
/// ```
/// use rustycad_tkhlr::hlr_topo_brep::VData;
/// let v = VData::new();
/// assert_eq!(v.parameter(), 0.0);
/// ```
#[derive(Clone, Default)]
pub struct VData {
    parameter: f64,
    vertex: Shape,
}

impl VData {
    /// HLRTopoBRep_VData(): parameter 0 and a null vertex; `VData::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// HLRTopoBRep_VData(P, V); `VData::with_vertex(0.5, &vertex)`.
    pub fn with_vertex(p: f64, v: &Shape) -> Self {
        Self {
            parameter: p,
            vertex: v.clone(),
        }
    }
    /// Parameter(); `v.parameter()`.
    pub fn parameter(&self) -> f64 {
        self.parameter
    }
    /// Vertex(); `v.vertex()`.
    pub fn vertex(&self) -> &Shape {
        &self.vertex
    }
}
