//! Martensite camera navigation policies retain pinned OCCT enum identities and values.
/// Navigation policy; `AisNavigationMode::FirstPersonWalk` uses ground-plane walking.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AisNavigationMode {
    /// Orbit around a pivot.
    Orbit = 0,
    /// First-person free flight.
    FirstPersonFlight = 1,
    /// First-person ground walking.
    FirstPersonWalk = 2,
}
/// The upstream anonymous navigation range enum; `AisNavigationModeBounds::Upper as i32 == 2`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AisNavigationModeBounds {
    /// Lowest native policy value.
    Lower = 0,
    /// Highest native policy value.
    Upper = 2,
}
/// Orbit pivot policy; `AisRotationMode::CameraAt` rotates around the camera center.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AisRotationMode {
    /// Bounding box of active objects.
    BndBoxActive = 0,
    /// Last picked point.
    PickLast = 1,
    /// Point picked at window center.
    PickCenter = 2,
    /// Camera center.
    CameraAt = 3,
    /// Bounding box of the whole scene.
    BndBoxScene = 4,
}
/// The upstream anonymous rotation range enum; `AisRotationModeBounds::Upper as i32 == 4`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AisRotationModeBounds {
    /// Lowest native pivot value.
    Lower = 0,
    /// Highest native pivot value.
    Upper = 4,
}
/// Mutually exclusive mouse gesture; `AisMouseGesture::Pan` translates the camera.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AisMouseGesture {
    /// No current gesture.
    None = 0,
    /// Rectangular selection.
    SelectRectangle = 1,
    /// Polygonal selection.
    SelectLasso = 2,
    /// Horizontal zoom gesture.
    Zoom = 3,
    /// Vertical zoom gesture.
    ZoomVertical = 4,
    /// Zoom into a window rectangle.
    ZoomWindow = 5,
    /// Camera panning.
    Pan = 6,
    /// Orbit rotation around a pivot.
    RotateOrbit = 7,
    /// View rotation.
    RotateView = 8,
    /// Object dragging.
    Drag = 9,
}
