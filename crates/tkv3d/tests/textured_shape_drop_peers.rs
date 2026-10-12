//! Actual descendant pixmap destructor observes native texture peers before inherited AIS state retires.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{
    standard::{Handle, SharedState},
    tcollection::AsciiString,
};
use rustycad_tkservice::{graphic3d::*, image::ImagePixMap};
use rustycad_tkv3d::{ais::*, prs_mgr::PrsMgrPresentableObjectMembers, prs3d::Prs3dDrawer};
use std::sync::Arc;

struct DropPeerPixels {
    base: ImagePixMap,
    drawer: Handle<Prs3dDrawer>,
    aspect: Handle<Graphic3dAspectFillArea3d>,
    texture: Handle<dyn Graphic3dTexture2DSource>,
    trace: Arc<SharedState<Vec<String>>>,
}
rustycad_tkernel::impl_transient!(DropPeerPixels, "NativeDropPeerPixels", ImagePixMap, base);
impl Drop for DropPeerPixels {
    fn drop(&mut self) {
        let row = format!(
            "MARTENSITE_TEXTURED_DROP {} {} {}",
            self.drawer.ref_count(),
            self.aspect.ref_count(),
            self.texture.ref_count()
        );
        self.trace.write().push(row);
    }
}

#[test]
fn textured_shape_drop_replays_actual_native_derived_peer_lifetimes() {
    let trace = Arc::new(SharedState::new(Vec::new()));
    let object = AisTexturedShape::new(&Shape::default());
    let drawer = object.attributes();
    let aspect = Handle::new(Graphic3dAspectFillArea3d::default());
    let texture = Handle::new(Graphic3dTexture2D::from_file(&AsciiString::default())).cast();
    install_textured_drop_peers(&object, &drawer, &aspect, &texture, &trace);
    drop(object);
    let actual = trace.read().join("\n") + "\n";
    let native = include_str!("fixtures/ais_textured_shape/native-drop-peers.txt");
    assert_eq!(actual, native);
}
fn install_textured_drop_peers(
    object: &AisTexturedShape,
    drawer: &Handle<Prs3dDrawer>,
    aspect: &Handle<Graphic3dAspectFillArea3d>,
    texture: &Handle<dyn Graphic3dTexture2DSource>,
    trace: &Arc<SharedState<Vec<String>>>,
) {
    let pixels = Handle::new(DropPeerPixels {
        base: ImagePixMap::default(),
        drawer: drawer.clone(),
        aspect: aspect.clone(),
        texture: texture.clone(),
        trace: trace.clone(),
    });
    object.set_texture_pix_map(&pixels.cast());
    let mut fields = object.textured_shape_base().fields.write();
    fields.aspect = aspect.clone();
    fields.texture = texture.clone();
}
