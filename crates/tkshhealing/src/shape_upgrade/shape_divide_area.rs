//! Whole-shape area division settings from OCCT ShapeUpgrade_ShapeDivideArea.
use super::{
    ShapeDivider, ShapeUpgradeFaceDivide, ShapeUpgradeFaceDivideArea, ShapeUpgradeShapeDivide,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle, SharedState},
};

#[derive(Clone, Copy)]
struct AreaDivisionSettings {
    max_area: f64,
    parts: i32,
    u_splits: i32,
    v_splits: i32,
    by_number: bool,
}

/// Divide faces of a shape by maximum area; `ShapeUpgradeShapeDivideArea::new(shape)`.
pub struct ShapeUpgradeShapeDivideArea {
    parent: ShapeUpgradeShapeDivide,
    settings: SharedState<AreaDivisionSettings>,
}
impl std::ops::Deref for ShapeUpgradeShapeDivideArea {
    type Target = ShapeUpgradeShapeDivide;
    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}
impl ShapeDivider for ShapeUpgradeShapeDivideArea {
    fn shape_divide_base(&self) -> &ShapeUpgradeShapeDivide {
        &self.parent
    }
    fn split_face_tool(&self) -> Handle<ShapeUpgradeFaceDivide> {
        self.new_face_tool()
    }
}
impl Default for ShapeUpgradeShapeDivideArea {
    fn default() -> Self {
        Self::from_parent(ShapeUpgradeShapeDivide::empty())
    }
}
impl ShapeUpgradeShapeDivideArea {
    /// Retain the area policy for direct calls; `divider.perform(true)?`.
    pub fn perform(&self, new_context: bool) -> Result<bool, Failure> {
        ShapeDivider::perform(self, new_context)
    }

    /// Construct without a source shape; `ShapeUpgradeShapeDivideArea::empty()`.
    pub fn empty() -> Self {
        Self::default()
    }
    /// Construct for a source shape; `ShapeUpgradeShapeDivideArea::new(shape)`.
    pub fn new(shape: Shape) -> Self {
        Self::from_parent(ShapeUpgradeShapeDivide::new(shape))
    }
    /// Read the maximum accepted face area; `divider.max_area()`.
    pub fn max_area(&self) -> f64 {
        self.settings.read().max_area
    }
    /// Set the area threshold; `divider.set_max_area(4.)`.
    pub fn set_max_area(&self, area: f64) {
        self.settings.write().max_area = area;
    }
    /// Read the requested part count; `divider.nb_parts()`.
    pub fn nb_parts(&self) -> i32 {
        self.settings.read().parts
    }
    /// Set the requested part count; `divider.set_nb_parts(4)`.
    pub fn set_nb_parts(&self, parts: i32) {
        self.settings.write().parts = parts;
    }
    /// Read whether fixed-count subdivision is selected; `divider.splitting_by_number()`.
    pub fn splitting_by_number(&self) -> bool {
        self.settings.read().by_number
    }
    /// Select fixed-count mode; true sets MaxArea to -1, while false preserves it.
    /// `divider.set_splitting_by_number(true);`
    pub fn set_splitting_by_number(&self, enabled: bool) {
        let mut settings = self.settings.write();
        settings.by_number = enabled;
        if enabled {
            settings.max_area = -1.;
        }
    }
    /// Select fixed U/V counts; `divider.set_numbers_uv_splits(2, 3)?`.
    pub fn set_numbers_uv_splits(&self, u: i32, v: i32) -> Result<(), Failure> {
        let parts = checked_parts(u, v)?;
        self.store_split_counts(u, v, parts);
        Ok(())
    }
    fn from_parent(parent: ShapeUpgradeShapeDivide) -> Self {
        Self {
            parent,
            settings: SharedState::new(default_settings()),
        }
    }
    fn store_split_counts(&self, u: i32, v: i32, parts: Option<i32>) {
        let mut settings = self.settings.write();
        settings.by_number = true;
        settings.max_area = -1.;
        settings.u_splits = u;
        settings.v_splits = v;
        if let Some(parts) = parts {
            settings.parts = parts;
        }
    }
    fn new_face_tool(&self) -> Handle<ShapeUpgradeFaceDivide> {
        let settings = *self.settings.read();
        let tool = ShapeUpgradeFaceDivideArea::empty();
        tool.set_max_area(settings.max_area);
        tool.set_nb_parts(settings.parts);
        tool.set_numbers_uv_splits(settings.u_splits, settings.v_splits)
            .expect("wrapper settings keep positive U/V products in range");
        tool.set_splitting_by_number(settings.by_number);
        Handle::new(tool).cast()
    }
}
fn default_settings() -> AreaDivisionSettings {
    AreaDivisionSettings {
        max_area: precision::infinite(),
        parts: 0,
        u_splits: -1,
        v_splits: -1,
        by_number: false,
    }
}
fn checked_parts(u: i32, v: i32) -> Result<Option<i32>, Failure> {
    if u <= 0 || v <= 0 {
        return Ok(None);
    }
    u.checked_mul(v).map(Some).ok_or_else(|| {
        Failure::with_type(
            "Standard_OutOfRange",
            &format!("U/V split counts {u} and {v}; expected product within signed 32-bit range"),
            "",
        )
    })
}
