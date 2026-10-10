//! `Select3D_SensitiveEntity`: the abstract entity a selecting volume tests (v3d-design.md
//! section 6). The class is abstract with virtual members, so it is the
//! [`Select3DSensitiveEntity`] trait over the embedded [`Select3DSensitiveEntityBase`]; the
//! non-virtual members are [`Select3DSensitiveEntityMembers`].
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::SelectMgrEntityOwnerSource;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::exceptions::program_error;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, Transient};
use rustycad_tkmath::bvh::{BvhBox, BvhVec3d};
use rustycad_tkmath::gp::{Axis2, GeneralTransform3, Point3, Transform3};
use rustycad_tkservice::graphic3d::{Graphic3dFlipper, Graphic3dTransformPersSource};
use rustycad_tkservice::json_dump::{class_begin, field_dumped, field_integer, field_pointer};
use std::io;

/// `Select3D_BndBox3d`: the axis-aligned box of a sensitive entity (`BVH_Box<double, 3>`).
pub type Select3DBndBox3d = BvhBox<BvhVec3d>;

/// The protected fields of `Select3D_SensitiveEntity`.
#[derive(Clone)]
pub struct Select3DSensitiveEntityFields {
    /// `myOwnerId`.
    pub owner_id: Handle<dyn SelectMgrEntityOwnerSource>,
    /// `myTrsfPers`.
    pub trsf_pers: Handle<dyn Graphic3dTransformPersSource>,
    /// `myFlipper`.
    pub flipper: Handle<Graphic3dFlipper>,
    /// `mySFactor`.
    pub sensitivity_factor: i32,
}

/// The state every sensitive entity embeds.
pub struct Select3DSensitiveEntityBase {
    /// The protected fields.
    pub fields: SharedState<Select3DSensitiveEntityFields>,
}

impl Select3DSensitiveEntityBase {
    /// The constructor state: the owner and the sensitivity factor 2.
    /// Example: `Select3DSensitiveEntityBase::new(&owner)`.
    pub fn new(owner: &Handle<dyn SelectMgrEntityOwnerSource>) -> Self {
        let fields = Select3DSensitiveEntityFields {
            owner_id: owner.clone(),
            trsf_pers: Handle::default(),
            flipper: Handle::default(),
            sensitivity_factor: 2,
        };
        Self {
            fields: SharedState::new(fields),
        }
    }
    /// A copy of the fields; `base.read()`.
    pub fn read(&self) -> Select3DSensitiveEntityFields {
        self.fields.read().clone()
    }
}

/// An entity that a selecting volume tests; the default bodies are the native ones.
pub trait Select3DSensitiveEntity: Transient {
    /// The embedded base; `entity.sensitive_base()`.
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase;
    /// `Matches` (pure virtual): whether the volume detects the entity, filling the pick
    /// result; a failure to build the entity BVH is returned. Example: `entity.matches(&manager, &mut pick)?`.
    fn matches(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure>;
    /// `NbSubElements` (pure virtual); `entity.nb_sub_elements()`.
    fn nb_sub_elements(&self) -> i32;
    /// `BoundingBox` (pure virtual); `entity.bounding_box()`.
    fn bounding_box(&self) -> Select3DBndBox3d;
    /// `CenterOfGeometry` (pure virtual); `entity.center_of_geometry()`.
    fn center_of_geometry(&self) -> Point3;
    /// `Set`: the owner; `entity.set(&owner)`.
    fn set(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) {
        self.sensitive_base().fields.write().owner_id = owner.clone();
    }
    /// `GetConnected`: null by default; `entity.get_connected()`.
    fn get_connected(&self) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        Ok(Handle::default())
    }
    /// `BVH`: builds the internal BVH (nothing by default); `entity.bvh()?`.
    fn bvh(&self) -> Result<(), Failure> {
        Ok(())
    }
    /// `ToBuildBVH`: true by default; `entity.to_build_bvh()`.
    fn to_build_bvh(&self) -> bool {
        true
    }
    /// `Clear`: forgets the owner; `entity.clear()`.
    fn clear(&self) {
        self.set(&Handle::default());
    }
    /// `HasInitLocation`: false by default; `entity.has_init_location()`.
    fn has_init_location(&self) -> bool {
        false
    }
    /// `InvInitLocation`: the identity by default; `entity.inv_init_location()`.
    fn inv_init_location(&self) -> GeneralTransform3 {
        GeneralTransform3::from(Transform3::default())
    }
    /// `SetTransformPersistence`; `entity.set_transform_persistence(&pers)`.
    fn set_transform_persistence(&self, trsf_pers: &Handle<dyn Graphic3dTransformPersSource>) {
        self.sensitive_base().fields.write().trsf_pers = trsf_pers.clone();
    }
    /// `SetFlippingOptions`: a flipper about the plane when enabled, none otherwise.
    /// Example: `entity.set_flipping_options(true, &plane)`.
    fn set_flipping_options(&self, is_enabled: bool, ref_plane: &Axis2) {
        let flipper = if is_enabled {
            Handle::new(Graphic3dFlipper::new(*ref_plane))
        } else {
            Handle::default()
        };
        self.sensitive_base().fields.write().flipper = flipper;
    }
    /// `DumpJson`; `entity.dump_json(&mut stream, -1)?`.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        select3d_sensitive_entity_dump_json(self, stream, depth)
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn Select3DSensitiveEntity,
    "Select3D_SensitiveEntity",
    StandardTransient
);

/// The non-virtual members of every sensitive entity.
pub trait Select3DSensitiveEntityMembers: Select3DSensitiveEntity {
    /// `OwnerId`; `entity.owner_id()`.
    fn owner_id(&self) -> Handle<dyn SelectMgrEntityOwnerSource> {
        self.sensitive_base().read().owner_id
    }
    /// `SensitivityFactor`; `entity.sensitivity_factor()`.
    fn sensitivity_factor(&self) -> i32 {
        self.sensitive_base().read().sensitivity_factor
    }
    /// `SetSensitivityFactor`: a negative factor raises `Standard_ProgramError`.
    /// Example: `entity.set_sensitivity_factor(4)?`.
    fn set_sensitivity_factor(&self, new_sens: i32) -> Result<(), Failure> {
        if new_sens < 0 {
            return Err(program_error(&format!(
                "Error! Selection sensitivity should not be negative value: {new_sens}; expected >= 0"
            )));
        }
        self.sensitive_base().fields.write().sensitivity_factor = new_sens;
        Ok(())
    }
    /// `TransformPersistence`; `entity.transform_persistence()`.
    fn transform_persistence(&self) -> Handle<dyn Graphic3dTransformPersSource> {
        self.sensitive_base().read().trsf_pers
    }
    /// `Flipper`; `entity.flipper()`.
    fn flipper(&self) -> Handle<Graphic3dFlipper> {
        self.sensitive_base().read().flipper
    }
}
impl<T: Select3DSensitiveEntity + ?Sized> Select3DSensitiveEntityMembers for T {}

/// The native `Select3D_SensitiveEntity::DumpJson` body, for the overrides that dump it.
/// Example: `select3d_sensitive_entity_dump_json(&*entity, &mut stream, -1)?`.
pub fn select3d_sensitive_entity_dump_json<E: Select3DSensitiveEntity + ?Sized>(
    entity: &E,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    class_begin(stream, "Select3D_SensitiveEntity")?;
    let fields = entity.sensitive_base().read();
    field_pointer(
        stream,
        "myOwnerId.get()",
        fields.owner_id.object_address() as usize,
    )?;
    if let Some(pers) = fields.trsf_pers.get() {
        field_dumped(stream, depth, "myTrsfPers.get()", &|s, d| {
            pers.dump_json(s, d)
        })?;
    }
    if let Some(flipper) = fields.flipper.get() {
        field_dumped(stream, depth, "myFlipper.get()", &|s, d| {
            flipper.dump_json(s, d)
        })?;
    }
    field_integer(stream, "mySFactor", fields.sensitivity_factor)?;
    field_integer(stream, "NbSubElements()", entity.nb_sub_elements())?;
    let center = entity.center_of_geometry();
    field_dumped(stream, depth, "&aCenterOfGeometry", &|s, _| {
        center.dump_json(s)
    })?;
    field_integer(
        stream,
        "aHasInitLocation",
        i32::from(entity.has_init_location()),
    )?;
    let inverse = entity.inv_init_location();
    field_dumped(stream, depth, "&anInvInitLocation", &|s, d| {
        inverse.dump_json(s, d)
    })
}
