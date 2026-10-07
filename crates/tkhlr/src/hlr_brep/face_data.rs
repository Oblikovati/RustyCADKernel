//! HLRBRep_FaceData.hxx/.lxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::Surface;
use crate::hlr_algo::{EdgesBlock, WiresBlock};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::Orientation;

// The bits of `myFlags` (`EMaskFlags`); the low four bits hold the orientation.
/// HLRBRep_FaceData::EMaskFlags: the bits of `myFlags` (orientation in the low four bits); protected natively, public here so that the packed
/// flags can be read; `FaceDataMask::EMaskOrient as i32 == 15`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum FaceDataMask {
    /// `EMaskOrient`.
    EMaskOrient = 15,
    /// `FMaskSelected`.
    FMaskSelected = 16,
    /// `FMaskBack`.
    FMaskBack = 32,
    /// `FMaskSide`.
    FMaskSide = 64,
    /// `FMaskClosed`.
    FMaskClosed = 128,
    /// `FMaskHiding`.
    FMaskHiding = 256,
    /// `FMaskSimple`.
    FMaskSimple = 512,
    /// `FMaskCut`.
    FMaskCut = 1024,
    /// `FMaskWithOutL`.
    FMaskWithOutL = 2048,
    /// `FMaskPlane`.
    FMaskPlane = 4096,
    /// `FMaskCylinder`.
    FMaskCylinder = 8192,
    /// `FMaskCone`.
    FMaskCone = 16384,
    /// `FMaskSphere`.
    FMaskSphere = 32768,
    /// `FMaskTorus`.
    FMaskTorus = 65536,
}

const EMASK_ORIENT: i32 = FaceDataMask::EMaskOrient as i32;
const FMASK_SELECTED: i32 = FaceDataMask::FMaskSelected as i32;
const FMASK_BACK: i32 = FaceDataMask::FMaskBack as i32;
const FMASK_SIDE: i32 = FaceDataMask::FMaskSide as i32;
const FMASK_CLOSED: i32 = FaceDataMask::FMaskClosed as i32;
const FMASK_HIDING: i32 = FaceDataMask::FMaskHiding as i32;
const FMASK_SIMPLE: i32 = FaceDataMask::FMaskSimple as i32;
const FMASK_CUT: i32 = FaceDataMask::FMaskCut as i32;
const FMASK_WITH_OUT_L: i32 = FaceDataMask::FMaskWithOutL as i32;
const FMASK_PLANE: i32 = FaceDataMask::FMaskPlane as i32;
const FMASK_CYLINDER: i32 = FaceDataMask::FMaskCylinder as i32;
const FMASK_CONE: i32 = FaceDataMask::FMaskCone as i32;
const FMASK_SPHERE: i32 = FaceDataMask::FMaskSphere as i32;
const FMASK_TORUS: i32 = FaceDataMask::FMaskTorus as i32;

/// One flag of `myFlags`: the getter and the setter of the native overload pair.
macro_rules! face_flag {
    ($get:ident, $set:ident, $mask:expr, $name:literal) => {
        #[doc = concat!("`", $name, "()`; `face.", stringify!($get), "()`.")]
        pub fn $get(&self) -> bool {
            (self.flags & $mask) != 0
        }
        #[doc = concat!("`", $name, "(B)`; `face.", stringify!($set), "(true)`.")]
        pub fn $set(&mut self, b: bool) {
            if b {
                self.flags |= $mask;
            } else {
                self.flags &= !$mask;
            }
        }
    };
}

/// HLRBRep_FaceData: the hidden line data of one face: its flags (selected, back,
/// side, closed, hiding, the elementary type flags...), orientation, size, wire
/// blocks and HLR surface.
///
/// ```
/// use rustycad_tkhlr::hlr_brep::FaceData;
/// let mut face = FaceData::new();
/// assert!(face.selected() && face.wires().is_null());
/// face.set_size(2.5);
/// assert_eq!(face.size(), 2.5);
/// ```
pub struct FaceData {
    flags: i32,
    wires: Handle<WiresBlock>,
    geometry: Surface,
    size: f64,
    // Uninitialized natively until Set; 0 here.
    tolerance: f32,
}

impl Default for FaceData {
    fn default() -> Self {
        Self::new()
    }
}

impl FaceData {
    /// HLRBRep_FaceData(): selected, size 0, no wires; `FaceData::new()`.
    pub fn new() -> Self {
        let mut data = Self {
            flags: 0,
            wires: Handle::default(),
            geometry: Surface::new(),
            size: 0.,
            tolerance: 0.,
        };
        data.set_selected(true);
        data
    }

    /// The native copy assignment (`*fd = *f1` in HLRBRep_Data::Write): the flags,
    /// size and tolerance, the wire blocks shared (the native handle copy) and the
    /// surface on a new adaptor of the same face; `let copy = face.try_clone()?;`.
    pub fn try_clone(&self) -> Result<Self, Failure> {
        Ok(Self {
            flags: self.flags,
            wires: self.wires.clone(),
            geometry: self.geometry.try_clone()?,
            size: self.size,
            tolerance: self.tolerance,
        })
    }

    /// Set(FG, Or, Cl, NW): loads the face, its tolerance and orientation, and `NW`
    /// empty wire slots; `face.set(&f, Orientation::Forward, false, 1)?`.
    pub fn set(&mut self, fg: &Face, or: Orientation, cl: bool, nw: i32) -> Result<(), Failure> {
        self.set_closed(cl);
        self.geometry.set_surface(fg)?;
        self.tolerance = BRepTool::face_tolerance(fg)? as f32;
        self.set_orientation(or);
        self.wires = Handle::new(WiresBlock::new(nw)?);
        Ok(())
    }

    fn wire_block(&self, wi: i32) -> Result<Handle<EdgesBlock>, Failure> {
        let wires = self.wires.get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                &format!("HLRBRep_FaceData: no wires for wire {wi}; expected Set first"),
                "",
            )
        })?;
        wires.wire(wi)
    }

    /// SetWire(WI, NE): a block of `NE` edges for wire `WI`; `face.set_wire(1, 4)?`.
    pub fn set_wire(&mut self, wi: i32, ne: i32) -> Result<(), Failure> {
        let block = Handle::new(EdgesBlock::new(ne)?);
        match self.wires.get() {
            Some(wires) => wires.set(wi, block),
            None => Err(Failure::with_type(
                "Standard_NullObject",
                &format!("HLRBRep_FaceData::SetWire: no wires for wire {wi}; expected Set first"),
                "",
            )),
        }
    }

    /// SetWEdge(WI, EWI, EI, Or, OutL, Inte, Dble, IsoL): the edge `EWI` of wire
    /// `WI` is the data edge `EI` with its flags;
    /// `face.set_w_edge(1, 1, 3, Orientation::Forward, false, false, false, false)?`.
    #[allow(clippy::too_many_arguments)] // The native signature.
    pub fn set_w_edge(
        &mut self,
        wi: i32,
        ewi: i32,
        ei: i32,
        or: Orientation,
        out_l: bool,
        inte: bool,
        dble: bool,
        iso_l: bool,
    ) -> Result<(), Failure> {
        let wire = self.wire_block(wi)?;
        let block = wire.get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                &format!(
                    "HLRBRep_FaceData::SetWEdge: wire {wi} has no edge block; expected SetWire"
                ),
                "",
            )
        })?;
        block.set_edge(ewi, ei)?;
        block.set_orientation(ewi, or)?;
        block.set_out_line(ewi, out_l)?;
        block.set_internal(ewi, inte)?;
        block.set_double(ewi, dble)?;
        block.set_iso_line(ewi, iso_l)
    }

    face_flag!(selected, set_selected, FMASK_SELECTED, "Selected");
    face_flag!(back, set_back, FMASK_BACK, "Back");
    face_flag!(side, set_side, FMASK_SIDE, "Side");
    face_flag!(closed, set_closed, FMASK_CLOSED, "Closed");
    face_flag!(hiding, set_hiding, FMASK_HIDING, "Hiding");
    face_flag!(simple, set_simple, FMASK_SIMPLE, "Simple");
    face_flag!(cut, set_cut, FMASK_CUT, "Cut");
    face_flag!(with_out_l, set_with_out_l, FMASK_WITH_OUT_L, "WithOutL");
    face_flag!(plane, set_plane, FMASK_PLANE, "Plane");
    face_flag!(cylinder, set_cylinder, FMASK_CYLINDER, "Cylinder");
    face_flag!(cone, set_cone, FMASK_CONE, "Cone");
    face_flag!(sphere, set_sphere, FMASK_SPHERE, "Sphere");
    face_flag!(torus, set_torus, FMASK_TORUS, "Torus");

    /// Size(); `face.size()`.
    pub fn size(&self) -> f64 {
        self.size
    }
    /// Size(S); `face.set_size(2.0)`.
    pub fn set_size(&mut self, s: f64) {
        self.size = s;
    }
    /// Orientation(): the low four bits of the flags; `face.orientation()`.
    pub fn orientation(&self) -> Orientation {
        Orientation::ALL[(self.flags & EMASK_ORIENT) as usize & 3]
    }
    /// Orientation(O); `face.set_orientation(Orientation::Reversed)`.
    pub fn set_orientation(&mut self, o: Orientation) {
        self.flags &= !EMASK_ORIENT;
        self.flags |= (o as i32) & EMASK_ORIENT;
    }
    /// Wires(): the wire blocks (a null handle before Set), for edition;
    /// `face.wires_mut()`.
    pub fn wires_mut(&mut self) -> &mut Handle<WiresBlock> {
        &mut self.wires
    }
    /// The wire blocks, read only; `face.wires()`.
    pub fn wires(&self) -> &Handle<WiresBlock> {
        &self.wires
    }
    /// Geometry(): the HLR surface, for edition; `face.geometry_mut()`.
    pub fn geometry_mut(&mut self) -> &mut Surface {
        &mut self.geometry
    }
    /// The HLR surface, read only; `face.geometry()`.
    pub fn geometry(&self) -> &Surface {
        &self.geometry
    }
    /// Tolerance(): the face tolerance as a `float`; `face.tolerance()`.
    pub fn tolerance(&self) -> f32 {
        self.tolerance
    }
}
