//! `Graphic3d_TransformPers`: the transformation persistence of a presentation (zoom, rotate,
//! axial, trihedron, 2D, camera and ortho persistence), with its anchor point or 2D corner.
//!
//! The native parameters are a union of the 3D anchor and the 2D corner/offset; both views
//! are kept over the same 24 bytes, so the dump of the inactive view reads the active one's
//! bytes as natively (bytes never written natively read as zero here).
use crate::aspect::AspectTypeOfTriedronPosition;
use crate::graphic3d::Graphic3dTransModeFlags;
use crate::json_dump::{class_begin, field_dumped, field_integer};
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::exceptions::program_error;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient, Transient};
use rustycad_tkmath::gp::Point3;
use std::io;

type Flags = Graphic3dTransModeFlags;

/// The mode and the union parameters of a transform persistence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Graphic3dTransformPersState {
    /// `myMode`.
    pub mode: Flags,
    /// `myParams`: `PntX PntY PntZ` (3 doubles) over `OffsetX OffsetY Corner` (3 ints).
    pub params: [u8; 24],
}

impl Graphic3dTransformPersState {
    /// `Params3d`: the anchor coordinates.
    pub fn params_3d(&self) -> [f64; 3] {
        std::array::from_fn(|k| f64::from_ne_bytes(self.chunk::<8>(8 * k)))
    }
    /// `Params2d`: `(OffsetX, OffsetY, Corner)`.
    pub fn params_2d(&self) -> [i32; 3] {
        std::array::from_fn(|k| i32::from_ne_bytes(self.chunk::<4>(4 * k)))
    }
    fn chunk<const N: usize>(&self, start: usize) -> [u8; N] {
        std::array::from_fn(|k| self.params[start + k])
    }
    fn set_params_3d(&mut self, point: &Point3) {
        for (k, value) in [point.x(), point.y(), point.z()].into_iter().enumerate() {
            self.params[8 * k..8 * k + 8].copy_from_slice(&value.to_ne_bytes());
        }
    }
    fn set_params_2d(&mut self, values: [i32; 3]) {
        for (k, value) in values.into_iter().enumerate() {
            self.params[4 * k..4 * k + 4].copy_from_slice(&value.to_ne_bytes());
        }
    }
}

/// The embedded state of every transform persistence class.
pub struct Graphic3dTransformPersBase {
    /// The mode and parameters, shared by every handle.
    pub state: SharedState<Graphic3dTransformPersState>,
}

/// `IsZoomOrRotate(mode)`.
fn zoom_or_rotate(mode: Flags) -> bool {
    mode.0 & (Flags::ZOOM_PERS.0 | Flags::ROTATE_PERS.0) != 0
}
/// `IsTrihedronOr2d(mode)`.
fn trihedron_or_2d(mode: Flags) -> bool {
    mode.0 & (Flags::TRIEDRON_PERS.0 | Flags::TMF_2D.0) != 0
}
/// `IsAxial(mode)`.
fn axial(mode: Flags) -> bool {
    mode.0 & Flags::AXIAL_SCALE_PERS.0 != 0
}

/// The `Standard_ProgramError` of a member called in a wrong persistence mode.
fn wrong_mode(member: &str, mode: Flags) -> Failure {
    program_error(&format!(
        "Graphic3d_TransformPers::{member}(), wrong persistence mode {}",
        mode.0
    ))
}

impl Graphic3dTransformPersBase {
    fn with_mode(mode: Flags) -> Self {
        Self {
            state: SharedState::new(Graphic3dTransformPersState {
                mode,
                params: [0; 24],
            }),
        }
    }
    /// `Graphic3d_TransformPers(theMode)`: zoom/rotate/axial modes anchor at the origin,
    /// trihedron/2D modes at the lower-left corner; another mode except `CameraPers` raises
    /// `Standard_ProgramError`. Example: `Graphic3dTransformPersBase::new(Graphic3dTransModeFlags::ZOOM_PERS)?`.
    pub fn new(mode: Flags) -> Result<Self, Failure> {
        let base = Self::with_mode(mode);
        if zoom_or_rotate(mode) || axial(mode) {
            base.set_persistence_point(mode, &Point3::new(0.0, 0.0, 0.0))?;
        } else if trihedron_or_2d(mode) {
            base.set_persistence_corner(
                mode,
                AspectTypeOfTriedronPosition::LEFT_LOWER,
                Vec2::new(0, 0),
            )?;
        } else if mode != Flags::CAMERA_PERS {
            return Err(wrong_mode("SetPersistence", mode));
        }
        Ok(base)
    }
    /// `Graphic3d_TransformPers(theMode, thePnt)`; `Graphic3dTransformPersBase::with_point(mode, &p)?`.
    pub fn with_point(mode: Flags, point: &Point3) -> Result<Self, Failure> {
        let base = Self::with_mode(Flags::NONE);
        base.set_persistence_point(mode, point)?;
        Ok(base)
    }
    /// `Graphic3d_TransformPers(theMode, theCorner, theOffset)`.
    /// Example: `Graphic3dTransformPersBase::with_corner(mode, corner, Vec2::new(0, 0))?`.
    pub fn with_corner(
        mode: Flags,
        corner: AspectTypeOfTriedronPosition,
        offset: Vec2<i32>,
    ) -> Result<Self, Failure> {
        let base = Self::with_mode(Flags::NONE);
        base.set_persistence_corner(mode, corner, offset)?;
        Ok(base)
    }
    /// `SetPersistence(theMode, thePnt)`: zoom, rotate or axial modes only.
    /// Example: `base.set_persistence_point(Graphic3dTransModeFlags::ZOOM_PERS, &p)?`.
    pub fn set_persistence_point(&self, mode: Flags, point: &Point3) -> Result<(), Failure> {
        if !zoom_or_rotate(mode) && !axial(mode) {
            return Err(wrong_mode("SetPersistence", mode));
        }
        let mut state = self.state.write();
        state.mode = mode;
        state.set_params_3d(point);
        Ok(())
    }
    /// `SetPersistence(theMode, theCorner, theOffset)`: trihedron or 2D modes only.
    /// Example: `base.set_persistence_corner(Graphic3dTransModeFlags::TMF_2D, corner, offset)?`.
    pub fn set_persistence_corner(
        &self,
        mode: Flags,
        corner: AspectTypeOfTriedronPosition,
        offset: Vec2<i32>,
    ) -> Result<(), Failure> {
        if !trihedron_or_2d(mode) {
            return Err(wrong_mode("SetPersistence", mode));
        }
        let mut state = self.state.write();
        state.mode = mode;
        state.set_params_2d([offset.x, offset.y, corner.0]);
        Ok(())
    }
}

/// A transform persistence class; implementers return their embedded base and may override
/// the virtual persistent scale and rotation (`Graphic3d_TransformPersScaledAbove` does).
pub trait Graphic3dTransformPersSource: Transient {
    /// The embedded base; `pers.transform_pers_base().state.read().mode`.
    fn transform_pers_base(&self) -> &Graphic3dTransformPersBase;
    /// `persistentScale`; see [`super::graphic3d_transform_pers_persistent_scale`].
    fn persistent_scale(
        &self,
        camera: &super::Graphic3dCamera,
        viewport_width: i32,
        viewport_height: i32,
    ) -> f64 {
        super::graphic3d_transform_pers_persistent_scale(
            self.transform_pers_base(),
            camera,
            viewport_width,
            viewport_height,
        )
    }
    /// `persistentRotationMatrix`: the identity by default.
    /// Example: `pers.persistent_rotation_matrix(&camera, 800, 600)`.
    fn persistent_rotation_matrix(
        &self,
        _camera: &super::Graphic3dCamera,
        _viewport_width: i32,
        _viewport_height: i32,
    ) -> rustycad_tkernel::ncollection::Mat3<f64> {
        rustycad_tkernel::ncollection::Mat3::identity()
    }
    /// The virtual `ComputeApply`; see [`super::graphic3d_transform_pers_compute_apply`].
    fn compute_apply(
        &self,
        camera: &mut rustycad_tkernel::standard::Handle<super::Graphic3dCamera>,
        viewport_width: i32,
        viewport_height: i32,
        anchor: Option<&Point3>,
    ) -> Result<rustycad_tkernel::ncollection::Mat4<f64>, Failure> {
        super::graphic3d_transform_pers_compute_apply(
            self,
            camera,
            viewport_width,
            viewport_height,
            anchor,
        )
    }
    /// `DumpJson`; `pers.dump_json(&mut stream, -1)?`.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        graphic3d_transform_pers_dump_json(self.transform_pers_base(), stream, depth)
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn Graphic3dTransformPersSource,
    "Graphic3d_TransformPers",
    StandardTransient
);

/// The body of `Graphic3d_TransformPers::DumpJson`, both union views included.
/// Example: `graphic3d_transform_pers_dump_json(pers.transform_pers_base(), &mut s, -1)?`.
pub fn graphic3d_transform_pers_dump_json(
    base: &Graphic3dTransformPersBase,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    let state = *base.state.read();
    class_begin(stream, "Graphic3d_TransformPers")?;
    field_integer(stream, "myMode", state.mode.0)?;
    let [x, y, z] = state.params_3d();
    field_dumped(stream, depth, "&myParams.Params3d", &|o, d| {
        field_dumped(o, d, "&anAttachPoint", &|p, _| {
            Point3::new(x, y, z).dump_json(p)
        })
    })?;
    let [offset_x, offset_y, corner] = state.params_2d();
    field_dumped(stream, depth, "&myParams.Params2d", &|o, _| {
        field_integer(o, "OffsetX", offset_x)?;
        field_integer(o, "OffsetY", offset_y)?;
        field_integer(o, "Corner", corner)
    })
}

/// The concrete `Graphic3d_TransformPers`; `Graphic3dTransformPers::new(Graphic3dTransModeFlags::ZOOM_PERS)?`.
pub struct Graphic3dTransformPers {
    transient: StandardTransient,
    base: Graphic3dTransformPersBase,
}
impl Graphic3dTransformPersSource for Graphic3dTransformPers {
    fn transform_pers_base(&self) -> &Graphic3dTransformPersBase {
        &self.base
    }
}
rustycad_tkernel::impl_transient!(Graphic3dTransformPers, "Graphic3d_TransformPers",
    StandardTransient, transient, interfaces: [dyn Graphic3dTransformPersSource]);

impl Graphic3dTransformPers {
    /// See [`Graphic3dTransformPersBase::new`]; `Graphic3dTransformPers::new(mode)?`.
    pub fn new(mode: Flags) -> Result<Self, Failure> {
        Ok(Self::from_base(Graphic3dTransformPersBase::new(mode)?))
    }
    /// See [`Graphic3dTransformPersBase::with_point`]; `Graphic3dTransformPers::with_point(mode, &p)?`.
    pub fn with_point(mode: Flags, point: &Point3) -> Result<Self, Failure> {
        Ok(Self::from_base(Graphic3dTransformPersBase::with_point(
            mode, point,
        )?))
    }
    /// See [`Graphic3dTransformPersBase::with_corner`].
    /// Example: `Graphic3dTransformPers::with_corner(mode, corner, Vec2::new(0, 0))?`.
    pub fn with_corner(
        mode: Flags,
        corner: AspectTypeOfTriedronPosition,
        offset: Vec2<i32>,
    ) -> Result<Self, Failure> {
        Ok(Self::from_base(Graphic3dTransformPersBase::with_corner(
            mode, corner, offset,
        )?))
    }
    fn from_base(base: Graphic3dTransformPersBase) -> Self {
        Self {
            transient: StandardTransient::default(),
            base,
        }
    }
    /// Static `IsZoomOrRotate(theMode)`; `Graphic3dTransformPers::is_zoom_or_rotate_mode(mode)`.
    pub fn is_zoom_or_rotate_mode(mode: Flags) -> bool {
        zoom_or_rotate(mode)
    }
    /// Static `IsTrihedronOr2d(theMode)`; `Graphic3dTransformPers::is_trihedron_or_2d_mode(mode)`.
    pub fn is_trihedron_or_2d_mode(mode: Flags) -> bool {
        trihedron_or_2d(mode)
    }
    /// Static `IsOrthoPers(theMode)`; `Graphic3dTransformPers::is_ortho_pers_mode(mode)`.
    pub fn is_ortho_pers_mode(mode: Flags) -> bool {
        mode.0 & Flags::ORTHO_PERS.0 != 0
    }
    /// Static `IsAxial(theMode)`; `Graphic3dTransformPers::is_axial_mode(mode)`.
    pub fn is_axial_mode(mode: Flags) -> bool {
        axial(mode)
    }
}

/// The non-virtual members of `Graphic3d_TransformPers`, for every implementer.
pub trait Graphic3dTransformPersMembers: Graphic3dTransformPersSource {
    /// The mode; `pers.mode()`.
    fn mode(&self) -> Flags {
        self.transform_pers_base().state.read().mode
    }
    /// Alias of `Mode`; `pers.flags()`.
    fn flags(&self) -> Flags {
        self.mode()
    }
    /// Whether zoom or rotate persistence is on; `pers.is_zoom_or_rotate()`.
    fn is_zoom_or_rotate(&self) -> bool {
        zoom_or_rotate(self.mode())
    }
    /// Whether trihedron or 2D persistence is on; `pers.is_trihedron_or_2d()`.
    fn is_trihedron_or_2d(&self) -> bool {
        trihedron_or_2d(self.mode())
    }
    /// Whether ortho persistence is on; `pers.is_ortho_pers()`.
    fn is_ortho_pers(&self) -> bool {
        Graphic3dTransformPers::is_ortho_pers_mode(self.mode())
    }
    /// Whether axial scale persistence is on; `pers.is_axial()`.
    fn is_axial(&self) -> bool {
        axial(self.mode())
    }
    /// See [`Graphic3dTransformPersBase::set_persistence_point`].
    fn set_persistence_point(&self, mode: Flags, point: &Point3) -> Result<(), Failure> {
        self.transform_pers_base()
            .set_persistence_point(mode, point)
    }
    /// See [`Graphic3dTransformPersBase::set_persistence_corner`].
    fn set_persistence_corner(
        &self,
        mode: Flags,
        corner: AspectTypeOfTriedronPosition,
        offset: Vec2<i32>,
    ) -> Result<(), Failure> {
        self.transform_pers_base()
            .set_persistence_corner(mode, corner, offset)
    }
    /// The anchor point of a zoom, rotate or axial persistence; `pers.anchor_point()?`.
    fn anchor_point(&self) -> Result<Point3, Failure> {
        let state = *self.transform_pers_base().state.read();
        if !zoom_or_rotate(state.mode) && !axial(state.mode) {
            return Err(wrong_mode("AnchorPoint", state.mode));
        }
        let [x, y, z] = state.params_3d();
        Ok(Point3::new(x, y, z))
    }
    /// Moves the anchor point; `pers.set_anchor_point(&p)?`.
    fn set_anchor_point(&self, point: &Point3) -> Result<(), Failure> {
        let mut state = self.transform_pers_base().state.write();
        if !zoom_or_rotate(state.mode) && !axial(state.mode) {
            return Err(wrong_mode("SetAnchorPoint", state.mode));
        }
        state.set_params_3d(point);
        Ok(())
    }
    /// The corner of a trihedron or 2D persistence; `pers.corner_2d()?`.
    fn corner_2d(&self) -> Result<AspectTypeOfTriedronPosition, Failure> {
        let state = *self.transform_pers_base().state.read();
        if !trihedron_or_2d(state.mode) {
            return Err(wrong_mode("Corner2d", state.mode));
        }
        Ok(AspectTypeOfTriedronPosition(state.params_2d()[2]))
    }
    /// Moves the corner; `pers.set_corner_2d(corner)?`.
    fn set_corner_2d(&self, corner: AspectTypeOfTriedronPosition) -> Result<(), Failure> {
        let mut state = self.transform_pers_base().state.write();
        if !trihedron_or_2d(state.mode) {
            return Err(wrong_mode("SetCorner2d", state.mode));
        }
        let [x, y, _] = state.params_2d();
        state.set_params_2d([x, y, corner.0]);
        Ok(())
    }
    /// The pixel offset of a trihedron or 2D persistence; `pers.offset_2d()?`.
    fn offset_2d(&self) -> Result<Vec2<i32>, Failure> {
        let state = *self.transform_pers_base().state.read();
        if !trihedron_or_2d(state.mode) {
            return Err(wrong_mode("Offset2d", state.mode));
        }
        let [x, y, _] = state.params_2d();
        Ok(Vec2::new(x, y))
    }
    /// Moves the offset; `pers.set_offset_2d(Vec2::new(10, 10))?`.
    fn set_offset_2d(&self, offset: Vec2<i32>) -> Result<(), Failure> {
        let mut state = self.transform_pers_base().state.write();
        if !trihedron_or_2d(state.mode) {
            return Err(wrong_mode("SetOffset2d", state.mode));
        }
        let corner = state.params_2d()[2];
        state.set_params_2d([offset.x, offset.y, corner]);
        Ok(())
    }
}
impl<T: Graphic3dTransformPersSource + ?Sized> Graphic3dTransformPersMembers for T {}
