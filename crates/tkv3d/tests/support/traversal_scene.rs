//! Named real-point scene records visit order, sorting dispatch and partial failure state.
#[path = "traversal_volume.rs"]
pub mod volume;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, WeakHandle};
use rustycad_tkmath::{bnd::BoundingBox3, gp::*};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
use rustycad_tkv3d::{prs_mgr::*, select_basics::*, select_mgr::*, select3d::*};
use std::sync::{
    Arc,
    atomic::{AtomicI32, AtomicUsize, Ordering},
};
use volume::{TraversalConfig, traversal_persistence, traversal_planes, traversal_volume};
pub struct TraversalSelector {
    transient: StandardTransient,
    base: SelectMgrViewerSelectorBase,
    pub sorts: AtomicUsize,
}
rustycad_tkernel::impl_transient!(TraversalSelector,"TraversalSelector",dyn SelectMgrViewerSelectorSource,transient,interfaces:[dyn SelectMgrViewerSelectorSource]);
impl TraversalSelector {
    pub fn new() -> Handle<Self> {
        Handle::new(Self {
            transient: StandardTransient::default(),
            base: SelectMgrViewerSelectorBase::new(),
            sorts: AtomicUsize::new(0),
        })
    }
}
impl SelectMgrViewerSelectorSource for TraversalSelector {
    fn viewer_selector_base(&self) -> &SelectMgrViewerSelectorBase {
        &self.base
    }
    fn sort_result(&self) -> Result<(), Failure> {
        self.sorts.fetch_add(1, Ordering::AcqRel);
        // Reuse the nonvirtual native sorter through another source sharing no client state.
        rustycad_tkv3d::select_mgr::select_mgr_sort_selector_results(&self.base)
    }
}
pub struct TraversalObject {
    transient: StandardTransient,
    presentable: PrsMgrPresentableObjectBase,
    selectable: SelectMgrSelectableObjectBase,
}
rustycad_tkernel::impl_transient!(TraversalObject,"TraversalObject",dyn SelectMgrSelectableObject,transient,interfaces:[dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl TraversalObject {
    pub fn new() -> Handle<Self> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            presentable: PrsMgrPresentableObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
            selectable: SelectMgrSelectableObjectBase::default(),
        });
        prs_mgr_presentable_bind(&object.cast());
        object
    }
}
impl PrsMgrPresentableObject for TraversalObject {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.presentable
    }
    fn compute(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        _: &Handle<dyn Graphic3dStructureSource>,
        _: i32,
    ) -> Result<(), Failure> {
        Ok(())
    }
    fn bounding_box(&self, bounds: &mut BoundingBox3) {
        let mut local = BoundingBox3::default();
        local.update_bounds([-2., -1., 0.], [8., 1., 0.]);
        *bounds = local.transformed(self.transformation());
    }
    rustycad_tkv3d::select_mgr_presentable_overrides!();
}
impl SelectMgrSelectableObject for TraversalObject {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.selectable
    }
    fn compute_selection(&self, _: &Handle<SelectMgrSelection>, _: i32) -> Result<(), Failure> {
        Ok(())
    }
}
pub struct TraversalPoint {
    transient: StandardTransient,
    point: Select3DSensitivePoint,
    pub tag: i32,
    pub fail: AtomicI32,
    pub camera_edit: AtomicI32,
    selector: WeakHandle<dyn SelectMgrViewerSelectorSource>,
    events: Arc<SharedState<Vec<i32>>>,
}
rustycad_tkernel::impl_transient!(TraversalPoint,"TraversalPoint",dyn Select3DSensitiveEntity,transient,interfaces:[dyn Select3DSensitiveEntity]);
impl TraversalPoint {
    pub fn new(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        tag: i32,
        selector: &Handle<TraversalSelector>,
        events: Arc<SharedState<Vec<i32>>>,
    ) -> Handle<Self> {
        let position = Point3::new(
            [-2., 0., 8.][tag as usize % 3],
            [0., 0., -1.][tag as usize % 3],
            0.,
        );
        Handle::new(Self {
            transient: StandardTransient::default(),
            point: Select3DSensitivePoint::new(owner, position),
            tag,
            fail: AtomicI32::new(0),
            camera_edit: AtomicI32::new(0),
            selector: selector
                .cast::<dyn SelectMgrViewerSelectorSource>()
                .downgrade(),
            events,
        })
    }
}
impl Select3DSensitiveEntity for TraversalPoint {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        self.point.sensitive_base()
    }
    fn matches(
        &self,
        volume: &dyn SelectBasicsSelectingVolumeManager,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.events.write().push(self.tag);
        if let Some(selector) = self.selector.upgrade().get() {
            let _ = selector.nb_picked();
            if self.camera_edit.load(Ordering::Acquire) == 1 {
                let original = selector
                    .viewer_selector_base()
                    .selecting_volume
                    .read()
                    .clone();
                let camera = Handle::new(original.camera().copy_members());
                camera.set_scale(8.);
                camera.set_projection_type(
                    rustycad_tkservice::graphic3d::Graphic3dCameraProjection::Perspective,
                );
                original.set_camera(&camera)?;
            }
        }
        if self.fail.load(Ordering::Acquire) != 0 {
            return Err(Failure::with_message(
                "Standard_Failure",
                "fixture entity match failed; expected successful query",
            ));
        }
        self.point.matches(volume, pick)
    }
    fn nb_sub_elements(&self) -> i32 {
        self.point.nb_sub_elements()
    }
    fn bounding_box(&self) -> Select3DBndBox3d {
        self.point.bounding_box()
    }
    fn center_of_geometry(&self) -> Point3 {
        self.point.center_of_geometry()
    }
}
pub struct TraversalScene {
    pub selector: Handle<TraversalSelector>,
    pub objects: Vec<Handle<TraversalObject>>,
    pub entities: Vec<Handle<TraversalPoint>>,
    pub owners: Vec<Handle<dyn SelectMgrEntityOwnerSource>>,
    pub events: Arc<SharedState<Vec<i32>>>,
}
impl TraversalScene {
    pub fn new(config: TraversalConfig) -> Self {
        let mut scene = Self {
            selector: TraversalSelector::new(),
            objects: vec![],
            entities: vec![],
            owners: vec![],
            events: Arc::new(SharedState::default()),
        };
        *scene
            .selector
            .viewer_selector_base()
            .selecting_volume
            .write() = traversal_volume(config);
        for index in 0..config.count {
            scene.add_object(config, index);
        }
        scene
            .selector
            .viewer_selector_base()
            .tolerances
            .write()
            .add(3);
        if config.flags & 8 != 0 {
            scene.selector.set_to_prebuild_bvh(true, 2);
        }
        scene
    }
    fn add_object(&mut self, config: TraversalConfig, index: i32) {
        let object = TraversalObject::new();
        configure_traversal_object(&object, config, index);
        let owner = SelectMgrEntityOwner::with_selectable(&object.cast(), index % 4);
        self.owners.push(owner.cast());
        let selection = Handle::new(SelectMgrSelection::new(0));
        for part in 0..3 {
            self.add_point(&selection, &owner.cast(), config, index * 3 + part);
        }
        object.add_selection(&selection, 0).unwrap();
        self.selector
            .add_selection_to_object(&object.cast(), &selection)
            .unwrap();
        for (part, entity) in selection.entities().iter().enumerate() {
            if config.flags & 2 == 0 || part != 2 {
                entity.set_active_for_selection();
            }
        }
        selection.set_selection_state(SelectMgrStateOfSelection::Activated);
        self.objects.push(object);
    }
    fn add_point(
        &mut self,
        selection: &SelectMgrSelection,
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        config: TraversalConfig,
        tag: i32,
    ) {
        let entity = TraversalPoint::new(owner, tag, &self.selector, self.events.clone());
        entity.set_sensitivity_factor(tag % 3 + 1).unwrap();
        entity
            .camera_edit
            .store(i32::from(config.flags & 16 != 0), Ordering::Release);
        entity
            .fail
            .store(i32::from(config.failing == tag), Ordering::Release);
        if tag % 3 == 1 {
            entity.set_transform_persistence(&traversal_persistence(config.entity_persistence));
        }
        if config.flags & 1 != 0 && tag % 3 == 2 {
            entity.set_flipping_options(
                true,
                &Axis2::new(Point3::default(), Direction3::new(0., 0., -1.).unwrap()).unwrap(),
            );
        }
        selection.add(&entity.cast()).unwrap();
        self.entities.push(entity);
    }
}
fn configure_traversal_object(object: &TraversalObject, config: TraversalConfig, index: i32) {
    let mode = if config.persistence == 6 {
        index % 6
    } else {
        config.persistence
    };
    object.set_transform_persistence(&traversal_persistence(mode));
    let mut transform = Transform3::default();
    if config.transform != 0 {
        transform
            .set_rotation(Axis1::new(Point3::default(), Direction3::Z), 1.8)
            .unwrap();
    }
    transform.set_translation_part(Vector3::new(f64::from(index % 3 - 1) * 10., 0., 0.));
    object.set_local_transformation(&transform).unwrap();
    if config.flags & 4 != 0 && index % 2 == 0 {
        object.view_affinity().set_view_visible(2, false);
    }
    if config.clipping == 2 || config.clipping == 3 {
        let planes = traversal_planes(-1., 4.);
        planes.set_override_global(config.clipping == 3);
        object.set_clip_planes(&planes);
    }
}
