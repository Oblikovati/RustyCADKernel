//! Native color-scale value and layout accessors; setters deliberately do not schedule redisplay.
use super::AisColorScale;
impl AisColorScale {
    /// Inspect smooth_transition; `scale.is_smooth_transition()`.
    pub fn is_smooth_transition(&self) -> bool {
        self.fields.read().inputs.smooth
    }
    /// Replace smooth_transition; `scale.set_smooth_transition(value)`.
    pub fn set_smooth_transition(&self, value: bool) {
        self.fields.write().inputs.smooth = value;
    }
    /// Inspect label_at_border; `scale.is_label_at_border()`.
    pub fn is_label_at_border(&self) -> bool {
        self.fields.read().inputs.border
    }
    /// Replace label_at_border; `scale.set_label_at_border(value)`.
    pub fn set_label_at_border(&self, value: bool) {
        self.fields.write().inputs.border = value;
    }
    /// Inspect logarithmic; `scale.is_logarithmic()`.
    pub fn is_logarithmic(&self) -> bool {
        self.fields.read().inputs.logarithmic
    }
    /// Replace logarithmic; `scale.set_logarithmic(value)`.
    pub fn set_logarithmic(&self, value: bool) {
        self.fields.write().inputs.logarithmic = value;
    }
    /// Inspect breadth; `scale.get_breadth()`.
    pub fn get_breadth(&self) -> i32 {
        self.fields.read().layout.breadth
    }
    /// Replace breadth; `scale.set_breadth(value)`.
    pub fn set_breadth(&self, value: i32) {
        self.fields.write().layout.breadth = value;
    }
    /// Inspect height; `scale.get_height()`.
    pub fn get_height(&self) -> i32 {
        self.fields.read().layout.height
    }
    /// Replace height; `scale.set_height(value)`.
    pub fn set_height(&self, value: i32) {
        self.fields.write().layout.height = value;
    }
    /// Inspect x_position; `scale.get_x_position()`.
    pub fn get_x_position(&self) -> i32 {
        self.fields.read().layout.x
    }
    /// Replace x_position; `scale.set_x_position(value)`.
    pub fn set_x_position(&self, value: i32) {
        self.fields.write().layout.x = value;
    }
    /// Inspect y_position; `scale.get_y_position()`.
    pub fn get_y_position(&self) -> i32 {
        self.fields.read().layout.y
    }
    /// Replace y_position; `scale.set_y_position(value)`.
    pub fn set_y_position(&self, value: i32) {
        self.fields.write().layout.y = value;
    }
    /// Inspect text_height; `scale.get_text_height()`.
    pub fn get_text_height(&self) -> i32 {
        self.fields.read().layout.text_height
    }
    /// Replace text_height; `scale.set_text_height(value)`.
    pub fn set_text_height(&self, value: i32) {
        self.fields.write().layout.text_height = value;
    }
}
