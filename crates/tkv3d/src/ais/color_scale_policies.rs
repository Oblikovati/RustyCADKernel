//! Native color-scale value and layout accessors; setters deliberately do not schedule redisplay.
use super::AisColorScale;
use rustycad_tkservice::aspect::{AspectTypeOfColorScaleData, AspectTypeOfColorScalePosition};
impl AisColorScale {
    /// Inspect min; `scale.get_min()`.
    pub fn get_min(&self) -> f64 {
        self.fields.read().range.min
    }
    /// Replace min; `scale.set_min(value)`.
    pub fn set_min(&self, value: f64) {
        self.set_range(value, self.get_max());
    }
    /// Inspect max; `scale.get_max()`.
    pub fn get_max(&self) -> f64 {
        self.fields.read().range.max
    }
    /// Replace max; `scale.set_max(value)`.
    pub fn set_max(&self, value: f64) {
        self.set_range(self.get_min(), value);
    }
    /// Inspect number_of_intervals; `scale.get_number_of_intervals()`.
    pub fn get_number_of_intervals(&self) -> i32 {
        self.fields.read().range.intervals
    }
    /// Replace number_of_intervals; `scale.set_number_of_intervals(value)`.
    pub fn set_number_of_intervals(&self, value: i32) {
        if value < 1 {
            return;
        }
        self.fields.write().range.intervals = value;
    }
    /// Inspect label_type; `scale.get_label_type()`.
    pub fn get_label_type(&self) -> AspectTypeOfColorScaleData {
        self.fields.read().inputs.label_type
    }
    /// Replace label_type; `scale.set_label_type(value)`.
    pub fn set_label_type(&self, value: AspectTypeOfColorScaleData) {
        self.fields.write().inputs.label_type = value;
    }
    /// Inspect color_type; `scale.get_color_type()`.
    pub fn get_color_type(&self) -> AspectTypeOfColorScaleData {
        self.fields.read().inputs.color_type
    }
    /// Replace color_type; `scale.set_color_type(value)`.
    pub fn set_color_type(&self, value: AspectTypeOfColorScaleData) {
        self.fields.write().inputs.color_type = value;
    }
    /// Inspect label_position; `scale.get_label_position()`.
    pub fn get_label_position(&self) -> AspectTypeOfColorScalePosition {
        self.fields.read().layout.label_pos
    }
    /// Replace label_position; `scale.set_label_position(value)`.
    pub fn set_label_position(&self, value: AspectTypeOfColorScalePosition) {
        self.fields.write().layout.label_pos = value;
    }
    /// Inspect title_position; `scale.get_title_position()`.
    pub fn get_title_position(&self) -> AspectTypeOfColorScalePosition {
        self.fields.read().layout.title_pos
    }
    /// Replace title_position; `scale.set_title_position(value)`.
    pub fn set_title_position(&self, value: AspectTypeOfColorScalePosition) {
        self.fields.write().layout.title_pos = value;
    }
    /// Inspect reversed; `scale.is_reversed()`.
    pub fn is_reversed(&self) -> bool {
        self.fields.read().inputs.reversed
    }
    /// Replace reversed; `scale.set_reversed(value)`.
    pub fn set_reversed(&self, value: bool) {
        self.fields.write().inputs.reversed = value;
    }
}
