// Rust port of DataExchange/TKXCAF/XCAFView/XCAFView_Object.hxx and .cxx and
// XCAFView_ProjectionType.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The XCAFView package: `XCAFView_Object`, the camera, window and clipping of a saved view.
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::HAsciiString;
use rustycad_tkmath::gp::{Direction3, Point3};

/// `XCAFView_ProjectionType`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum XCAFViewProjectionType {
    /// `XCAFView_ProjectionType_NoCamera`.
    #[default]
    NoCamera = 0,
    /// `XCAFView_ProjectionType_Parallel`.
    Parallel = 1,
    /// `XCAFView_ProjectionType_Central`.
    Central = 2,
}

impl crate::NativeEnum for XCAFViewProjectionType {
    const ALL: &'static [Self] = &[Self::NoCamera, Self::Parallel, Self::Central];
}

/// The fields of `XCAFView_Object`. The native default constructor leaves the type, the
/// camera values and the plane distances uninitialized; the OCCT allocator clears memory,
/// so they read zero, as here (a zeroed `gp_Dir` is not the default `gp_Dir`; the port keeps
/// the default `gp_Dir`, which no code path can tell apart before a setter runs).
#[derive(Clone, Default)]
struct ViewFields {
    name: Handle<HAsciiString>,
    kind: XCAFViewProjectionType,
    projection_point: Point3,
    view_direction: Direction3,
    up_direction: Direction3,
    zoom_factor: f64,
    window_horizontal_size: f64,
    window_vertical_size: f64,
    clipping_expression: Handle<HAsciiString>,
    front_plane_clipping: bool,
    front_plane_distance: f64,
    back_plane_clipping: bool,
    back_plane_distance: f64,
    view_volume_sides_clipping: bool,
    gdt_points: Handle<HArray1<Point3>>,
}

/// `XCAFView_Object`.
///
/// ```
/// use rustycad_tkxcaf::xcaf_view::XCAFViewObject;
/// let view = XCAFViewObject::new();
/// view.set_front_plane_distance(2.0);
/// assert!(view.has_front_plane_clipping());
/// view.create_gdt_points(2);
/// assert_eq!(view.nb_gdt_points(), 2);
/// ```
pub struct XCAFViewObject {
    transient: StandardTransient,
    fields: SharedState<ViewFields>,
}
rustycad_tkernel::impl_transient!(
    XCAFViewObject,
    "XCAFView_Object",
    StandardTransient,
    transient
);

impl Default for XCAFViewObject {
    fn default() -> Self {
        Self::new()
    }
}

impl XCAFViewObject {
    /// `XCAFView_Object()`: an empty clipping expression, no clipping, no GD&T points.
    pub fn new() -> Self {
        let fields = ViewFields {
            clipping_expression: Handle::new(HAsciiString::new("")),
            ..ViewFields::default()
        };
        Self::with_fields(fields)
    }
    fn with_fields(fields: ViewFields) -> Self {
        Self {
            transient: StandardTransient::default(),
            fields: SharedState::new(fields),
        }
    }
    /// `XCAFView_Object(theObj)`: a copy without the name and the GD&T points, as natively
    /// (a null object gives a default object).
    pub fn from_object(object: &Handle<XCAFViewObject>) -> Self {
        let Some(object) = object.get() else {
            return Self::new();
        };
        let mut fields = object.fields.read().clone();
        fields.name = Handle::default();
        fields.gdt_points = Handle::default();
        Self::with_fields(fields)
    }
    /// `SetName`.
    pub fn set_name(&self, name: &Handle<HAsciiString>) {
        self.fields.write().name = name.clone();
    }
    /// `Name`.
    pub fn name(&self) -> Handle<HAsciiString> {
        self.fields.read().name.clone()
    }
    /// `SetType`.
    pub fn set_type(&self, kind: XCAFViewProjectionType) {
        self.fields.write().kind = kind;
    }
    /// `Type`.
    pub fn kind(&self) -> XCAFViewProjectionType {
        self.fields.read().kind
    }
    /// `SetProjectionPoint`.
    pub fn set_projection_point(&self, point: &Point3) {
        self.fields.write().projection_point = *point;
    }
    /// `ProjectionPoint`.
    pub fn projection_point(&self) -> Point3 {
        self.fields.read().projection_point
    }
    /// `SetViewDirection`.
    pub fn set_view_direction(&self, direction: &Direction3) {
        self.fields.write().view_direction = *direction;
    }
    /// `ViewDirection`.
    pub fn view_direction(&self) -> Direction3 {
        self.fields.read().view_direction
    }
    /// `SetUpDirection`.
    pub fn set_up_direction(&self, direction: &Direction3) {
        self.fields.write().up_direction = *direction;
    }
    /// `UpDirection`.
    pub fn up_direction(&self) -> Direction3 {
        self.fields.read().up_direction
    }
    /// `SetZoomFactor`.
    pub fn set_zoom_factor(&self, factor: f64) {
        self.fields.write().zoom_factor = factor;
    }
    /// `ZoomFactor`.
    pub fn zoom_factor(&self) -> f64 {
        self.fields.read().zoom_factor
    }
    /// `SetWindowHorizontalSize`.
    pub fn set_window_horizontal_size(&self, size: f64) {
        self.fields.write().window_horizontal_size = size;
    }
    /// `WindowHorizontalSize`.
    pub fn window_horizontal_size(&self) -> f64 {
        self.fields.read().window_horizontal_size
    }
    /// `SetWindowVerticalSize`.
    pub fn set_window_vertical_size(&self, size: f64) {
        self.fields.write().window_vertical_size = size;
    }
    /// `WindowVerticalSize`.
    pub fn window_vertical_size(&self) -> f64 {
        self.fields.read().window_vertical_size
    }
}

/// Clipping and GD&T points.
impl XCAFViewObject {
    /// `SetClippingExpression`.
    pub fn set_clipping_expression(&self, expression: &Handle<HAsciiString>) {
        self.fields.write().clipping_expression = expression.clone();
    }
    /// `ClippingExpression`.
    pub fn clipping_expression(&self) -> Handle<HAsciiString> {
        self.fields.read().clipping_expression.clone()
    }
    /// `UnsetFrontPlaneClipping`.
    pub fn unset_front_plane_clipping(&self) {
        self.fields.write().front_plane_clipping = false;
    }
    /// `HasFrontPlaneClipping`.
    pub fn has_front_plane_clipping(&self) -> bool {
        self.fields.read().front_plane_clipping
    }
    /// `SetFrontPlaneDistance`: clipping on.
    pub fn set_front_plane_distance(&self, distance: f64) {
        let mut fields = self.fields.write();
        (fields.front_plane_distance, fields.front_plane_clipping) = (distance, true);
    }
    /// `FrontPlaneDistance`.
    pub fn front_plane_distance(&self) -> f64 {
        self.fields.read().front_plane_distance
    }
    /// `UnsetBackPlaneClipping`.
    pub fn unset_back_plane_clipping(&self) {
        self.fields.write().back_plane_clipping = false;
    }
    /// `HasBackPlaneClipping`.
    pub fn has_back_plane_clipping(&self) -> bool {
        self.fields.read().back_plane_clipping
    }
    /// `SetBackPlaneDistance`: clipping on.
    pub fn set_back_plane_distance(&self, distance: f64) {
        let mut fields = self.fields.write();
        (fields.back_plane_distance, fields.back_plane_clipping) = (distance, true);
    }
    /// `BackPlaneDistance`.
    pub fn back_plane_distance(&self) -> f64 {
        self.fields.read().back_plane_distance
    }
    /// `SetViewVolumeSidesClipping`.
    pub fn set_view_volume_sides_clipping(&self, clipping: bool) {
        self.fields.write().view_volume_sides_clipping = clipping;
    }
    /// `HasViewVolumeSidesClipping`.
    pub fn has_view_volume_sides_clipping(&self) -> bool {
        self.fields.read().view_volume_sides_clipping
    }
    /// `CreateGDTPoints(theLenght)`: a new array of that many origins (nothing for a length
    /// below 1).
    pub fn create_gdt_points(&self, length: i32) {
        if length > 0 {
            let points = HArray1::with_bounds(1, length).unwrap_or_default();
            self.fields.write().gdt_points = Handle::new(points);
        }
    }
    /// `HasGDTPoints`.
    pub fn has_gdt_points(&self) -> bool {
        !self.fields.read().gdt_points.is_null()
    }
    /// `NbGDTPoints`.
    pub fn nb_gdt_points(&self) -> i32 {
        let points = self.fields.read().gdt_points.clone();
        points.get().map_or(0, |points| points.array1().length())
    }
    /// `SetGDTPoint(theIndex, thePoint)`: ignored outside `1..=NbGDTPoints`.
    pub fn set_gdt_point(&self, index: i32, point: &Point3) {
        let points = self.fields.read().gdt_points.clone();
        if let Some(points) = points.get() {
            let _ = points.change_array1().set_value(index, *point);
        }
    }
    /// `GDTPoint(theIndex)`: the origin outside `1..=NbGDTPoints`.
    pub fn gdt_point(&self, index: i32) -> Point3 {
        let points = self.fields.read().gdt_points.clone();
        let point = points
            .get()
            .and_then(|p| p.array1().value(index).ok().copied());
        point.unwrap_or_default()
    }
}
