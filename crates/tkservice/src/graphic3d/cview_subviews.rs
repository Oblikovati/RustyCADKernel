//! The subview members of `Graphic3d_CView`: the parent and child views, the subview
//! placement and `SubviewResized`.
use super::Graphic3dCView;
use crate::aspect::{AspectNeutralWindow, AspectTypeOfTriedronPosition, AspectWindowMembers as _};
use crate::cast::int_from_f64;
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::standard::Handle;

/// `getSubViewOffset`: an absolute offset from 1 up, else a fraction of the window size.
fn sub_view_offset(offset: f64, window_size: i32) -> i32 {
    if offset >= 1.0 {
        return int_from_f64(offset);
    }
    int_from_f64(offset * f64::from(window_size))
}

/// The subview members of every view.
pub trait Graphic3dCViewSubviews: Graphic3dCView {
    /// `IsSubview`: a parent view is set; `view.is_subview()`.
    fn is_subview(&self) -> bool {
        !self.cview_base().fields.read().parent_view.is_null()
    }
    /// `ParentView`; `view.parent_view()`.
    fn parent_view(&self) -> Handle<dyn Graphic3dCView> {
        self.cview_base().fields.read().parent_view.upgrade()
    }
    /// `IsSubviewComposer`; `view.is_subview_composer()`.
    fn is_subview_composer(&self) -> bool {
        self.cview_base().fields.read().is_subview_composer
    }
    /// `SetSubviewComposer`; `view.set_subview_composer(true)`.
    fn set_subview_composer(&self, is_composer: bool) {
        self.cview_base().fields.write().is_subview_composer = is_composer;
    }
    /// `Subviews`; `view.subviews().len()`.
    fn subviews(&self) -> Vec<Handle<dyn Graphic3dCView>> {
        self.cview_base().fields.read().subviews.clone()
    }
    /// `AddSubview`: appends a child view (its parent is set by the caller); `view.add_subview(&child)`.
    fn add_subview(&self, view: &Handle<dyn Graphic3dCView>) {
        self.cview_base().fields.write().subviews.push(view.clone());
    }
    /// `RemoveSubview`: removes the child view at `address`; false when absent;
    /// `view.remove_subview(child.object_address())`.
    fn remove_subview(&self, address: *const ()) -> bool {
        let mut fields = self.cview_base().fields.write();
        let Some(index) = fields
            .subviews
            .iter()
            .position(|v| v.object_address() == address)
        else {
            return false;
        };
        fields.subviews.remove(index);
        true
    }
    /// `SubviewCorner`; `view.subview_corner()`.
    fn subview_corner(&self) -> AspectTypeOfTriedronPosition {
        self.cview_base().fields.read().subview_corner
    }
    /// `SetSubviewCorner`; `view.set_subview_corner(AspectTypeOfTriedronPosition::RIGHT_LOWER)`.
    fn set_subview_corner(&self, corner: AspectTypeOfTriedronPosition) {
        self.cview_base().fields.write().subview_corner = corner;
    }
    /// `SubviewTopLeft`; `view.subview_top_left()`.
    fn subview_top_left(&self) -> Vec2<i32> {
        self.cview_base().fields.read().subview_top_left
    }
    /// `IsSubViewRelativeSize`: both size components at most 1; `view.is_sub_view_relative_size()`.
    fn is_sub_view_relative_size(&self) -> bool {
        let size = self.cview_base().fields.read().subview_size;
        size.x <= 1.0 && size.y <= 1.0
    }
    /// `SubviewSize`; `view.subview_size()`.
    fn subview_size(&self) -> Vec2<f64> {
        self.cview_base().fields.read().subview_size
    }
    /// `SetSubviewSize`; `view.set_subview_size(Vec2::new(0.5, 0.5))`.
    fn set_subview_size(&self, size: Vec2<f64>) {
        self.cview_base().fields.write().subview_size = size;
    }
    /// `SubviewOffset`; `view.subview_offset()`.
    fn subview_offset(&self) -> Vec2<f64> {
        self.cview_base().fields.read().subview_offset
    }
    /// `SetSubviewOffset`; `view.set_subview_offset(Vec2::new(10.0, 10.0))`.
    fn set_subview_offset(&self, offset: Vec2<f64>) {
        self.cview_base().fields.write().subview_offset = offset;
    }
    /// `SubviewMargins`; `view.subview_margins()`.
    fn subview_margins(&self) -> Vec2<i32> {
        self.cview_base().fields.read().subview_margins
    }
    /// `SetSubviewMargins`; `view.set_subview_margins(Vec2::new(2, 2))`.
    fn set_subview_margins(&self, margins: Vec2<i32>) {
        self.cview_base().fields.write().subview_margins = margins;
    }
    /// `SubviewResized`: places the subview in its parent window by corner, offset, size
    /// and margins, and resizes `window`; `view.subview_resized(&window)`.
    fn subview_resized(&self, window: &Handle<AspectNeutralWindow>) {
        let parent = self.parent_view();
        let (Some(window), Some(parent)) = (window.get(), parent.get()) else {
            return;
        };
        let parent_window = parent.window();
        let Some(parent_window) = parent_window.get() else {
            return;
        };
        let win_size = parent_window.dimensions();
        let (top_left, view_size) = {
            let fields = self.cview_base().fields.read();
            subview_placement(win_size, &fields)
        };
        self.cview_base().fields.write().subview_top_left = top_left;
        window.set_size(view_size.x, view_size.y);
    }
}
impl<T: Graphic3dCView + ?Sized> Graphic3dCViewSubviews for T {}

/// The top-left corner and size of a subview in a window of `win_size`.
fn subview_placement(
    win_size: Vec2<i32>,
    fields: &super::Graphic3dCViewFields,
) -> (Vec2<i32>, Vec2<i32>) {
    let size = fields.subview_size;
    let mut view_size = Vec2::new(
        int_from_f64(f64::from(win_size.x) * size.x),
        int_from_f64(f64::from(win_size.y) * size.y),
    );
    if size.x > 1.0 {
        view_size.x = int_from_f64(size.x);
    }
    if size.y > 1.0 {
        view_size.y = int_from_f64(size.y);
    }
    let offset = Vec2::new(
        sub_view_offset(fields.subview_offset.x, win_size.x),
        sub_view_offset(fields.subview_offset.y, win_size.y),
    );
    let corner = fields.subview_corner.0;
    let mut top_left = Vec2::new(
        (win_size.x - view_size.x) / 2,
        (win_size.y - view_size.y) / 2,
    );
    if corner & AspectTypeOfTriedronPosition::LEFT.0 != 0 {
        top_left.x = offset.x;
    } else if corner & AspectTypeOfTriedronPosition::RIGHT.0 != 0 {
        top_left.x = (win_size.x - offset.x - view_size.x).max(0);
    }
    if corner & AspectTypeOfTriedronPosition::TOP.0 != 0 {
        top_left.y = offset.y;
    } else if corner & AspectTypeOfTriedronPosition::BOTTOM.0 != 0 {
        top_left.y = (win_size.y - offset.y - view_size.y).max(0);
    }
    let margins = fields.subview_margins;
    top_left = Vec2::new(top_left.x + margins.x, top_left.y + margins.y);
    view_size = Vec2::new(view_size.x - margins.x * 2, view_size.y - margins.y * 2);
    view_size.x = (top_left.x + view_size.x).min(win_size.x) - top_left.x;
    view_size.y = (top_left.y + view_size.y).min(win_size.y) - top_left.y;
    (top_left, view_size)
}
