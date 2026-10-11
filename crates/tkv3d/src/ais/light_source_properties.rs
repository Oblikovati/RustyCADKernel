//! Native property mutations preserve their distinct presentation invalidation behavior.
use super::AisLightSourceSource;
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::{aspect::AspectTypeOfMarker, graphic3d::Graphic3dMarkerImage};
/// Nonvirtual light members; `light.set_size(80.0)`.
pub trait AisLightSourceMembers: AisLightSourceSource {
    /// Read the native `display_name` property; `light.to_display_name()`.
    fn to_display_name(&self) -> bool {
        self.light_source_base().parameters.read().display_name
    }
    /// Change `display_name` and invalidate only when unequal; `light.set_display_name(true)`.
    fn set_display_name(&self, value: bool) {
        let mut parameters = self.light_source_base().parameters.write();
        if parameters.display_name == value {
            return;
        }
        parameters.display_name = value;
        drop(parameters);
        self.set_to_update_all();
    }
    /// Read the native `display_range` property; `light.to_display_range()`.
    fn to_display_range(&self) -> bool {
        self.light_source_base().parameters.read().display_range
    }
    /// Change `display_range` and invalidate only when unequal; `light.set_display_range(true)`.
    fn set_display_range(&self, value: bool) {
        let mut parameters = self.light_source_base().parameters.write();
        if parameters.display_range == value {
            return;
        }
        parameters.display_range = value;
        drop(parameters);
        self.set_to_update_all();
    }
    /// Read the native `size` property; `light.size()`.
    fn size(&self) -> f64 {
        self.light_source_base().parameters.read().size
    }
    /// Change `size` and invalidate only when unequal; `light.set_size(50.0)`.
    fn set_size(&self, value: f64) {
        let mut parameters = self.light_source_base().parameters.write();
        if parameters.size == value {
            return;
        }
        parameters.size = value;
        drop(parameters);
        self.set_to_update_all();
    }
    /// Read the native `arc_size` property; `light.arc_size()`.
    fn arc_size(&self) -> i32 {
        self.light_source_base().parameters.read().arc_size
    }
    /// Change `arc_size` and invalidate only when unequal; `light.set_arc_size(5)`.
    fn set_arc_size(&self, value: i32) {
        let mut parameters = self.light_source_base().parameters.write();
        if parameters.arc_size == value {
            return;
        }
        parameters.arc_size = value;
        drop(parameters);
        self.set_to_update_all();
    }
    /// Read the native `zoomable` property; `light.is_zoomable()`.
    fn is_zoomable(&self) -> bool {
        self.light_source_base().parameters.read().zoomable
    }
    /// Change `zoomable` and invalidate only when unequal; `light.set_zoomable(true)`.
    fn set_zoomable(&self, value: bool) {
        let mut parameters = self.light_source_base().parameters.write();
        if parameters.zoomable == value {
            return;
        }
        parameters.zoomable = value;
        drop(parameters);
        self.set_to_update_all();
    }
    /// Read the native `arrows` property; `light.nb_arrows()`.
    fn nb_arrows(&self) -> i32 {
        self.light_source_base().parameters.read().arrows
    }
    /// Change `arrows` and invalidate only when unequal; `light.set_nb_arrows(5)`.
    fn set_nb_arrows(&self, value: i32) {
        let mut parameters = self.light_source_base().parameters.write();
        if parameters.arrows == value {
            return;
        }
        parameters.arrows = value;
        drop(parameters);
        self.set_to_update_all();
    }
    /// Read native tessellation level; `light.nb_splits_quadric()`.
    fn nb_splits_quadric(&self) -> i32 {
        self.light_source_base().parameters.read().quadric_splits
    }
    /// Set tessellation without invalidation or validation; `light.set_nb_splits_quadric(20)`.
    fn set_nb_splits_quadric(&self, value: i32) {
        self.light_source_base().parameters.write().quadric_splits = value;
    }
    /// Read native tessellation level; `light.nb_splits_arrow()`.
    fn nb_splits_arrow(&self) -> i32 {
        self.light_source_base().parameters.read().arrow_splits
    }
    /// Set tessellation without invalidation or validation; `light.set_nb_splits_arrow(20)`.
    fn set_nb_splits_arrow(&self, value: i32) {
        self.light_source_base().parameters.write().arrow_splits = value;
    }
    /// Retain the enabled or disabled icon; `light.marker_image(true)`.
    fn marker_image(&self, enabled: bool) -> Handle<Graphic3dMarkerImage> {
        self.light_source_base().parameters.read().marker_images[usize::from(enabled)].clone()
    }
    /// Read the enabled or disabled standard marker; `light.marker_type(false)`.
    fn marker_type(&self, enabled: bool) -> AspectTypeOfMarker {
        self.light_source_base().parameters.read().marker_types[usize::from(enabled)]
    }
    /// Set an icon and its type without invalidation; `light.set_marker_image(&image, true)`.
    fn set_marker_image(&self, image: &Handle<Graphic3dMarkerImage>, enabled: bool) {
        let mut parameters = self.light_source_base().parameters.write();
        parameters.marker_images[usize::from(enabled)] = image.clone();
        parameters.marker_types[usize::from(enabled)] = if !image.is_null() {
            AspectTypeOfMarker::Userdefined
        } else if enabled {
            AspectTypeOfMarker::OPoint
        } else {
            AspectTypeOfMarker::OX
        };
    }
    /// Change marker type while retaining any custom image; `light.set_marker_type(AspectTypeOfMarker::Star, true)`.
    fn set_marker_type(&self, kind: AspectTypeOfMarker, enabled: bool) {
        self.light_source_base().parameters.write().marker_types[usize::from(enabled)] = kind;
    }
}
impl<T: AisLightSourceSource + ?Sized> AisLightSourceMembers for T {}
