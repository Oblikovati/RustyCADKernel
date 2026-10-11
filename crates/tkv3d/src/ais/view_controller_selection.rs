//! OCCT 3d097a0 selection accumulation retains pending-tool flags across gesture edits.
use super::{AisSelectionScheme, AisViewController, AisViewSelectionTool};
use rustycad_tkernel::ncollection::{Sequence, Vec2};
impl AisViewController {
    /// Queue one pick, clearing points only on a tool transition; `controller.select_in_viewer(Vec2::new(2, 3), AisSelectionScheme::Replace)`.
    pub fn select_in_viewer(&mut self, point: Vec2<i32>, scheme: AisSelectionScheme) {
        let selection = &mut self.ui_input.selection;
        if selection.tool != AisViewSelectionTool::Picking {
            selection.tool = AisViewSelectionTool::Picking;
            selection.points.clear(None);
        }
        selection.scheme = scheme;
        selection.points.append(point);
    }
    /// Queue an owned snapshot, choosing the tool by length; `controller.select_points_in_viewer(&points, AisSelectionScheme::Add)`.
    pub fn select_points_in_viewer(
        &mut self,
        points: &Sequence<Vec2<i32>>,
        scheme: AisSelectionScheme,
    ) {
        let selection = &mut self.ui_input.selection;
        selection.scheme = scheme;
        selection.points = points.clone();
        selection.to_apply_tool = true;
        selection.tool = match points.len() {
            1 => AisViewSelectionTool::Picking,
            2 => AisViewSelectionTool::RubberBand,
            _ => AisViewSelectionTool::Polygon,
        };
    }
    /// Replace rectangle corners without applying selection; `controller.update_rubber_band(Vec2::new(1, 2), Vec2::new(3, 4))`.
    pub fn update_rubber_band(&mut self, first: Vec2<i32>, second: Vec2<i32>) {
        let selection = &mut self.ui_input.selection;
        selection.tool = AisViewSelectionTool::RubberBand;
        selection.points.clear(None);
        selection.points.append(first);
        selection.points.append(second);
    }
    /// Append a distinct vertex or replace the last one; `controller.update_poly_selection(Vec2::new(2, 3), true)`.
    pub fn update_poly_selection(&mut self, point: Vec2<i32>, append: bool) {
        let selection = &mut self.ui_input.selection;
        if selection.tool != AisViewSelectionTool::Polygon {
            selection.tool = AisViewSelectionTool::Polygon;
            selection.points.clear(None);
        }
        if selection.points.is_empty() || (append && selection.points.last() != Some(&point)) {
            selection.points.append(point);
            return;
        }
        *selection
            .points
            .last_mut()
            .expect("nonempty native polygon") = point;
    }
}
