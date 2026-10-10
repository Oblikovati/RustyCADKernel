//! A real selectable scene with duplicate memberships, custom modes and two entity types.
use super::traversal_scene::{TraversalScene, volume::TraversalConfig};
use rustycad_tkernel::{
    ncollection::Vec3,
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::{
    graphic3d::Graphic3dPresentationAttributesSource,
    image::{ImageFormat, ImagePixMap},
};
use rustycad_tkv3d::{prs_mgr::PrsMgrPresentableObjectMembers, select_mgr::*, select3d::*};
pub struct SelectionImageScene {
    pub scene: TraversalScene,
    _retained: Vec<Handle<SelectMgrSelection>>,
    pub entities: Vec<Handle<dyn Select3DSensitiveEntity>>,
}
impl SelectionImageScene {
    pub fn new(mode: i32, failing: i32) -> Self {
        let scene = TraversalScene::new(TraversalConfig {
            failing,
            ..Default::default()
        });
        let segment = Handle::new(Select3DSensitiveSegment::new(
            &scene.owners[0],
            Point3::new(-1., 0., 0.),
            Point3::new(1., 0., 0.),
        ))
        .cast();
        let mut entities: Vec<Handle<dyn Select3DSensitiveEntity>> =
            scene.entities.iter().map(Handle::cast).collect();
        entities.push(segment);
        let retained = scene
            .objects
            .iter()
            .flat_map(|object| object.selections())
            .collect();
        let fixture = Self {
            scene,
            entities,
            _retained: retained,
        };
        fixture.configure_modes(mode);
        fixture
    }
    fn configure_modes(&self, mode: i32) {
        for (index, object) in self.scene.objects.iter().enumerate() {
            object.attributes().set_color(Color::named(
                [NamedColor::RED, NamedColor::GREEN, NamedColor::BLUE1][index],
            ));
            let selection = Handle::new(SelectMgrSelection::new(mode));
            selection.add(&self.entities[index * 3]).unwrap();
            if index == 0 {
                selection.add(&self.entities[9]).unwrap();
            }
            object.add_selection(&selection, mode).unwrap();
        }
    }
    pub fn seed_pick(&self, tag: i32, depth: f64, normal: Vec3<f32>) {
        let mut fields = self.scene.selector.viewer_selector_base().fields.write();
        fields.results.stored.clear(true);
        fields.results.is_sorted = false;
        if tag < 0 {
            return;
        }
        let entity = self.entities[tag as usize].clone();
        let criterion = SelectMgrSortCriterion {
            entity: entity.clone(),
            depth,
            normal,
            ..Default::default()
        };
        fields.results.stored.add(entity.owner_id(), criterion);
    }
}
pub fn selection_test_image(
    format: i32,
    width: usize,
    height: usize,
    top_down: bool,
    padding: usize,
) -> ImagePixMap {
    let image = ImagePixMap::default();
    let format = ImageFormat::from_value(format).unwrap();
    let stride = width * ImagePixMap::size_pixel_bytes_of(format) + padding;
    image.init_zero(format, width, height, stride, 17);
    image.set_top_down(top_down);
    image
}
pub fn image_bytes(image: &ImagePixMap) -> Vec<u8> {
    image.data().data.data().to_vec()
}
pub fn image_pixel(image: &ImagePixMap, x: i32, y: i32) -> Result<[f32; 4], Failure> {
    let rgba = image.pixel_color(x, y, false)?.to_vec4();
    Ok([rgba.x, rgba.y, rgba.z, rgba.w])
}
