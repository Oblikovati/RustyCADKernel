//! Native aggregate styles expand into topology leaves while explicit styles win.
use super::shape_drawer_cursor::ShapeDrawerNativeCursor;
use super::{AisColoredDrawer, AisColoredShapeSource, AisShapeDrawerMap};
use rustycad_tkbrep::{topexp::Explorer, topods::Shape};
use rustycad_tkernel::standard::{Failure, Handle, HandleReference};
use rustycad_tkg3d::top_abs::ShapeEnum;
/// Nonvirtual protected expansion bodies; `object.bind_sub_shapes(&output,&wire,&drawer)?`.
pub trait AisColoredShapeExpansion: AisColoredShapeSource {
    /// Bind explicit leaves, or missing shell/solid faces and wire edges; `object.bind_sub_shapes(&output,&shape,&drawer)?`.
    fn bind_sub_shapes(
        &self,
        output: &AisShapeDrawerMap,
        key: &Shape,
        drawer: &Handle<AisColoredDrawer>,
    ) -> Result<(), Failure> {
        // Native helper is nonvirtual; subclasses cannot replace recursive style precedence.
        bind_colored_shape_leaves(output, key, drawer)
    }
    /// Expand external grouping compounds then explicit leaves into a live map; `object.fill_subshape_drawer_map(&output)?`.
    fn fill_subshape_drawer_map(&self, output: &AisShapeDrawerMap) -> Result<(), Failure> {
        // Nonvirtual native protected body must also support valid aliasing with its own map.
        super::ais_colored_shape_fill_subshape_drawer_map(self, output)
    }
    /// Any hidden mapped drawer hides the whole object, including outside keys; `object.is_shape_entirely_visible()?`.
    fn is_shape_entirely_visible(&self) -> Result<bool, Failure> {
        let mut cursor = ShapeDrawerNativeCursor::new(&self.colored_shape_base().shape_colors)?;
        while let Some(node) = cursor.current()? {
            if node.read_fields(|entry| entry.drawer.is_hidden())? {
                return Ok(false);
            }
            cursor.next()?;
        }
        Ok(true)
    }
}
impl<S: AisColoredShapeSource + ?Sized> AisColoredShapeExpansion for S {}
pub(super) fn bind_colored_shape_leaves(
    output: &AisShapeDrawerMap,
    key: &Shape,
    drawer: &impl HandleReference<AisColoredDrawer>,
) -> Result<(), Failure> {
    let target = match key.shape_type()? {
        ShapeEnum::Solid | ShapeEnum::Shell => ShapeEnum::Face,
        ShapeEnum::Wire => ShapeEnum::Edge,
        _ => {
            output.bind_reference(key, drawer)?;
            return Ok(());
        }
    };
    bind_missing_colored_leaves(output, key, target, drawer)
}
fn bind_missing_colored_leaves(
    output: &AisShapeDrawerMap,
    key: &Shape,
    target: ShapeEnum,
    drawer: &impl HandleReference<AisColoredDrawer>,
) -> Result<(), Failure> {
    let mut explorer = Explorer::new(key, target, ShapeEnum::Shape)?;
    while explorer.more() {
        let leaf = explorer.current();
        if !output.is_bound(leaf) {
            output.bind_reference(leaf, drawer)?;
        }
        explorer.next()?;
    }
    Ok(())
}
