//! Martensite AIS_Plane state and native OCCT 3d097a0 datum identity.
use super::{AisInteractiveObject, AisInteractiveObjectBase, AisKindOfInteractive};
use crate::{prs_mgr::*, select_mgr::*, select3d::Select3DTypeOfSensitivity};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::geom::{GeomAxis2Placement, GeomPlane};
use rustycad_tkmath::gp::Point3;
/// Coordinate plane category; `AisTypeOfPlane::XYPlane`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(i32)]
pub enum AisTypeOfPlane {
    #[default]
    /// Unspecified coordinate plane.
    Unknown = 0,
    /// XY plane.
    XYPlane = 1,
    /// XZ plane.
    XZPlane = 2,
    /// YZ plane.
    YZPlane = 3,
}
/// Native fields, including cached XYZ corners; `plane.fields.read().center`.
#[derive(Clone)]
pub struct AisPlaneFields {
    /// Retained geometry, canonical global plane for XYZ datums.
    pub component: Handle<GeomPlane>,
    /// Retained XYZ frame, dormant after switching to a geometry plane.
    pub ax2: Handle<GeomAxis2Placement>,
    /// Cached center, projected only by automatic frame computation.
    pub center: Point3,
    /// Explicit minimum or cached first XYZ endpoint.
    pub pmin: Point3,
    /// Explicit maximum or cached second XYZ endpoint.
    pub pmax: Point3,
    /// Native bold edge toggle.
    pub current_mode: bool,
    /// Whether the frame projects the center automatically.
    pub automatic_position: bool,
    /// Coordinate-plane category.
    pub type_of_plane: AisTypeOfPlane,
    /// Whether presentations and selection use an XYZ triangle.
    pub is_xyz_plane: bool,
    /// Explicit size ownership; safely false before the first size assignment.
    pub has_own_size: bool,
    /// Boundary or triangle-interior selection.
    pub type_of_sensitivity: Select3DTypeOfSensitivity,
}
impl Default for AisPlaneFields {
    fn default() -> Self {
        Self {
            component: Handle::default(),
            ax2: Handle::default(),
            center: Point3::default(),
            pmin: Point3::default(),
            pmax: Point3::default(),
            current_mode: false,
            automatic_position: true,
            type_of_plane: AisTypeOfPlane::Unknown,
            is_xyz_plane: false,
            // OCCT 3d097a0 leaves myHasOwnSize uninitialized; Martensite defines the safe state.
            has_own_size: false,
            type_of_sensitivity: Select3DTypeOfSensitivity::Boundary,
        }
    }
}
/// A selectable planar datum; `AisPlane::new(&geometry, false)?`.
pub struct AisPlane {
    transient: StandardTransient,
    pub(super) base: AisInteractiveObjectBase,
    /// Native mutable state and cached geometry.
    pub fields: SharedState<AisPlaneFields>,
}
rustycad_tkernel::impl_transient!(AisPlane,"AIS_Plane",dyn AisInteractiveObject,transient,
    interfaces:[dyn AisInteractiveObject,dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl AisPlane {
    fn from_fields(fields: AisPlaneFields) -> Result<Handle<Self>, Failure> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
            fields: SharedState::new(fields),
        });
        object.init_drawer_attributes()?;
        prs_mgr_presentable_bind(&object.cast());
        Ok(object)
    }
    /// Retain a plane with an automatically projected origin; `AisPlane::new(&plane,false)?`.
    pub fn new(component: &Handle<GeomPlane>, current_mode: bool) -> Result<Handle<Self>, Failure> {
        Self::with_center(component, Point3::default(), current_mode)
    }
    /// Retain the requested center until frame computation projects it; `Self::with_center(&plane,center,false)?`.
    pub fn with_center(
        component: &Handle<GeomPlane>,
        center: Point3,
        current_mode: bool,
    ) -> Result<Handle<Self>, Failure> {
        Self::from_fields(AisPlaneFields {
            component: component.clone(),
            center,
            current_mode,
            ..Default::default()
        })
    }
    /// Keep explicit limits; native frame computation derives size only from pmin. `Self::with_bounds(&plane,c,a,b,false)?`.
    pub fn with_bounds(
        component: &Handle<GeomPlane>,
        center: Point3,
        pmin: Point3,
        pmax: Point3,
        current_mode: bool,
    ) -> Result<Handle<Self>, Failure> {
        Self::from_fields(AisPlaneFields {
            component: component.clone(),
            center,
            pmin,
            pmax,
            current_mode,
            automatic_position: false,
            ..Default::default()
        })
    }
    /// Build a coordinate-plane triangle and snapshot its current lengths; `Self::with_axis2(&frame,AisTypeOfPlane::XYPlane,false)?`.
    pub fn with_axis2(
        ax2: &Handle<GeomAxis2Placement>,
        type_of_plane: AisTypeOfPlane,
        current_mode: bool,
    ) -> Result<Handle<Self>, Failure> {
        let object = Self::from_fields(AisPlaneFields {
            ax2: ax2.clone(),
            type_of_plane,
            current_mode,
            is_xyz_plane: true,
            ..Default::default()
        })?;
        // OCCT 3d097a0 dereferences an unset DatumAspect in this constructor.
        // Supply its intended default safely; defined native paths use SetAxis2Placement
        // after linking a drawer, and are compared separately by the oracle.
        object
            .attributes()
            .set_datum_aspect(Handle::new(crate::prs3d::Prs3dDatumAspect::new()?));
        object.compute_fields()?;
        Ok(object)
    }
    /// Return retained geometry; `plane.component()`.
    pub fn component(&self) -> Handle<GeomPlane> {
        self.fields.read().component.clone()
    }
    /// Switch to automatic geometry, preserving the center; `plane.set_component(&geometry)`.
    pub fn set_component(&self, component: &Handle<GeomPlane>) {
        let mut f = self.fields.write();
        f.component = component.clone();
        f.type_of_plane = AisTypeOfPlane::Unknown;
        f.is_xyz_plane = false;
        f.automatic_position = true;
    }
    /// Return the frame only for XYZ planes; `plane.axis2_placement()`.
    pub fn axis2_placement(&self) -> Handle<GeomAxis2Placement> {
        if self.is_xyz_plane() {
            self.fields.read().ax2.clone()
        } else {
            Handle::default()
        }
    }
    /// Switch to XYZ geometry and refresh cached corners; `plane.set_axis2_placement(&frame,AisTypeOfPlane::YZPlane)?`.
    pub fn set_axis2_placement(
        &self,
        ax2: &Handle<GeomAxis2Placement>,
        kind: AisTypeOfPlane,
    ) -> Result<(), Failure> {
        {
            let mut f = self.fields.write();
            f.ax2 = ax2.clone();
            f.type_of_plane = kind;
            f.is_xyz_plane = true;
            f.automatic_position = true;
        }
        self.compute_fields()
    }
    /// Assign outputs only for explicitly positioned planes; `plane.plane_attributes(&mut component,&mut center,&mut min,&mut max)`.
    pub fn plane_attributes(
        &self,
        component: &mut Handle<GeomPlane>,
        center: &mut Point3,
        pmin: &mut Point3,
        pmax: &mut Point3,
    ) -> bool {
        let f = self.fields.read();
        if f.automatic_position {
            return false;
        }
        *component = f.component.clone();
        *center = f.center;
        *pmin = f.pmin;
        *pmax = f.pmax;
        true
    }
    /// Replace geometry and explicit limits without scheduling updates; `plane.set_plane_attributes(&geometry,c,a,b)`.
    pub fn set_plane_attributes(
        &self,
        component: &Handle<GeomPlane>,
        center: Point3,
        pmin: Point3,
        pmax: Point3,
    ) {
        let mut f = self.fields.write();
        f.component = component.clone();
        f.center = center;
        f.pmin = pmin;
        f.pmax = pmax;
        f.automatic_position = false;
        f.type_of_plane = AisTypeOfPlane::Unknown;
        f.is_xyz_plane = false;
    }
    /// Return the cached center; `plane.center()`.
    pub fn center(&self) -> Point3 {
        self.fields.read().center
    }
    /// Replace the cached center without updates; `plane.set_center(point)`.
    pub fn set_center(&self, point: Point3) {
        self.fields.write().center = point;
    }
    /// Return the coordinate-plane category; `plane.type_of_plane()`.
    pub fn type_of_plane(&self) -> AisTypeOfPlane {
        self.fields.read().type_of_plane
    }
    /// Inspect whether selection uses three cached points; `plane.is_xyz_plane()`.
    pub fn is_xyz_plane(&self) -> bool {
        self.fields.read().is_xyz_plane
    }
    /// Inspect the bold edge toggle; `plane.current_mode()`.
    pub fn current_mode(&self) -> bool {
        self.fields.read().current_mode
    }
    /// Change the bold toggle without updates; `plane.set_current_mode(true)`.
    pub fn set_current_mode(&self, flag: bool) {
        self.fields.write().current_mode = flag;
    }
    /// Return boundary or interior sensitivity; `plane.type_of_sensitivity()`.
    pub fn type_of_sensitivity(&self) -> Select3DTypeOfSensitivity {
        self.fields.read().type_of_sensitivity
    }
    /// Change sensitivity without rebuilding selection; `plane.set_type_of_sensitivity(Select3DTypeOfSensitivity::Interior)`.
    pub fn set_type_of_sensitivity(&self, kind: Select3DTypeOfSensitivity) {
        self.fields.write().type_of_sensitivity = kind;
    }
}
impl AisInteractiveObject for AisPlane {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base
    }
    fn kind_of_interactive(&self) -> AisKindOfInteractive {
        AisKindOfInteractive::Datum
    }
    fn signature(&self) -> i32 {
        7
    }
    fn set_context(&self, context: &Handle<super::AisInteractiveContext>) {
        let previous = self.base.fields.read().context.object_address();
        if previous != context.object_address() {
            self.base.fields.write().context = context.downgrade();
            if let Some(ctx) = context.get() {
                self.attributes().link_to(&ctx.fields.read().default_drawer);
            }
        }
        // The native virtual is infallible; a valid retained Axis2 always normalizes successfully.
        self.compute_fields()
            .expect("AIS_Plane frame: expected valid retained Axis2 placement");
    }
}
