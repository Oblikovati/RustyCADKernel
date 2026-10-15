//! 1:1 translation of src/ApplicationFramework/TKTObj/GTests/TObj_Model_Test.cxx.
//!
//! The two stream tests (`CafBug_29901_StreamInteger`, `CafBug_21231_StreamObjectGraph`)
//! register the native `BinTObjDrivers` format, which is out of scope (no OCCT native file
//! formats, user decision 2026-09); they stay native-only in the ledger, and
//! `tobj_model_io.rs` checks the named refusal of `SaveAs` without a storage driver.
#![allow(non_snake_case)]
use rustycad_tkernel::{
    impl_transient,
    message::Messenger,
    ncollection::HArray1,
    standard::{Failure, Handle, StandardTransient},
    tcollection::{AsciiString, ExtendedString, HExtendedString},
};
use rustycad_tklcaf::tdf::TdfLabel;
use rustycad_tktobj::tobj::{
    TObjDeletingMode, TObjModel, TObjModelBase, TObjModelState, TObjObject, TObjObjectArrays,
    TObjObjectBase, TObjObjectChildTag, TObjObjectData, TObjObjectDataTag, TObjObjectIterator,
    TObjObjectRefTag, TObjObjectState, TObjObjectStateFlags, TObjOcafObjectIterator, TObjPartition,
    TObjPersistence, TObjTypeFlags,
};
use std::sync::{Arc, Mutex, MutexGuard, Once, PoisonError};

/// Minimal concrete subclass of `TObj_Model` (abstract because of `NewEmpty`).
pub struct TObjTestModel {
    transient: StandardTransient,
    base: TObjModelBase,
}
impl_transient!(TObjTestModel, "TObj_TestModel", dyn TObjModel, transient,
    interfaces: [dyn TObjModel]);
impl TObjTestModel {
    /// `new TObj_TestModel()`, already in its bound handle.
    pub fn new() -> Handle<TObjTestModel> {
        let model = Handle::new(Self {
            transient: StandardTransient::default(),
            base: TObjModelBase::new().unwrap(),
        });
        model.model_base().bind_self(&model.cast());
        model
    }
}
impl TObjModel for TObjTestModel {
    fn model_base(&self) -> &TObjModelBase {
        &self.base
    }
    fn new_empty(&self) -> Result<Handle<dyn TObjModel>, Failure> {
        Ok(TObjTestModel::new().cast())
    }
}

/// `TObj_TestObject::DataTag`.
const DATA_TAG_INT_VAL: i32 = TObjObjectDataTag::LAST + 1;
const DATA_TAG_REAL_ARR: i32 = TObjObjectDataTag::LAST + 2;
/// `TObj_TestObject::RefTag_Other`.
const REF_TAG_OTHER: i32 = TObjObjectRefTag::LAST + 1;
/// `TObj_TestObject::ChildTag_Child`.
const CHILD_TAG_CHILD: i32 = TObjObjectChildTag::LAST + 1;

/// Minimal concrete subclass of `TObj_Object` (whose constructor is protected).
pub struct TObjTestObject {
    transient: StandardTransient,
    base: TObjObjectBase,
}
impl_transient!(TObjTestObject, "TObj_TestObject", dyn TObjObject, transient,
    interfaces: [dyn TObjObject]);
impl TObjObject for TObjTestObject {
    fn object_base(&self) -> &TObjObjectBase {
        &self.base
    }
}
impl TObjTestObject {
    fn unattached(label: &TdfLabel) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: TObjObjectBase::new(label),
        }
    }
    /// `TObj_TestObject(theLabel, theSetName = true)`.
    pub fn new(label: &TdfLabel) -> Handle<TObjTestObject> {
        let object = Handle::new(Self::unattached(label));
        TObjObjectBase::attach(&object.cast(), true).unwrap();
        object
    }
    /// The persistence constructor of `DECLARE_TOBJOCAF_PERSISTENCE`.
    fn persistence_new(label: &TdfLabel) -> Result<Handle<dyn TObjObject>, Failure> {
        let object: Handle<dyn TObjObject> = Handle::new(Self::unattached(label)).cast();
        object.object_base().bind_self(&object);
        object.init_fields();
        Ok(object)
    }
    /// `SetInt`: `setInteger(theValue, DataTag_IntVal)`.
    pub fn set_int(&self, value: i32) {
        self.set_integer(value, DATA_TAG_INT_VAL, 0).unwrap();
    }
    /// `GetInt`.
    pub fn get_int(&self) -> i32 {
        self.get_integer(DATA_TAG_INT_VAL, 0).unwrap()
    }
    /// `SetRealArr`: `setArray(theArray, DataTag_RealArr)`.
    pub fn set_real_arr(&self, array: &Handle<HArray1<f64>>) {
        self.set_real_array(array, DATA_TAG_REAL_ARR, 0).unwrap();
    }
    /// `GetRealArr`: `getRealArray(0, DataTag_RealArr)`.
    pub fn get_real_arr(&self) -> Handle<HArray1<f64>> {
        self.get_real_array(0, DATA_TAG_REAL_ARR, 0, 0.0).unwrap()
    }
    /// `SetRef`: `setReference(theObject, RefTag_Other)`.
    pub fn set_ref(&self, object: &Handle<dyn TObjObject>) {
        self.set_reference(object, REF_TAG_OTHER, 0).unwrap();
    }
    /// `GetRef`.
    pub fn get_ref(&self) -> Handle<dyn TObjObject> {
        self.get_reference(REF_TAG_OTHER, 0).unwrap()
    }
    /// `AddChild`: a new object on a new child of `ChildTag_Child`.
    pub fn add_child(&self) -> Handle<TObjTestObject> {
        TObjTestObject::new(
            &self
                .get_child_label_at(CHILD_TAG_CHILD)
                .unwrap()
                .new_child()
                .unwrap(),
        )
    }
}

/// `IMPLEMENT_TOBJOCAF_PERSISTENCE(TObj_TestObject)`: the native static registration.
pub fn register_test_object_persistence() {
    static REGISTERED: Once = Once::new();
    REGISTERED.call_once(|| {
        TObjPersistence::register_static(
            "TObj_TestObject",
            Arc::new(TObjTestObject::persistence_new),
        );
    });
}

/// `FindTestObject(theModel, theName)`.
pub fn find_test_object(model: &Handle<dyn TObjModel>, name: &str) -> Handle<TObjTestObject> {
    let name = Handle::new(HExtendedString::from(ExtendedString::new(name)));
    let dictionary = model.get_dictionary().unwrap();
    model.find_object(&name, &dictionary).unwrap().cast()
}

/// GoogleTest runs the cases of a binary one after the other; the TObj statics
/// (`TObj_Assistant`, the application) are process-wide, so the fixtures serialize.
static SERIAL: Mutex<()> = Mutex::new(());

/// Holds the process-wide TObj statics for one test.
pub fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The `TObj_ModelTest` fixture: a model loaded empty, closed on tear-down.
pub struct ModelFixture {
    /// `myModel`.
    pub my_model: Handle<TObjTestModel>,
    _serial: MutexGuard<'static, ()>,
}
impl ModelFixture {
    /// `SetUp`: `Load` with an empty file name creates a new empty model.
    pub fn set_up() -> Self {
        let serial = serial();
        register_test_object_persistence();
        let my_model = TObjTestModel::new();
        assert!(my_model.load(&ExtendedString::default()).unwrap());
        Self {
            my_model,
            _serial: serial,
        }
    }
}
impl Drop for ModelFixture {
    fn drop(&mut self) {
        if !self.my_model.is_null() {
            let _ = self.my_model.close();
        }
    }
}

/// A new `TCollection_HExtendedString`.
pub fn h_name(text: &str) -> Handle<HExtendedString> {
    Handle::new(HExtendedString::from(ExtendedString::new(text)))
}

#[test]
fn TObj_ModelTest__Load_EmptyString_CreatesModel() {
    let fixture = ModelFixture::set_up();
    assert!(!fixture.my_model.get_label().is_null());
    assert!(!fixture.my_model.get_document().unwrap().is_null());
}

#[test]
fn TObj_ModelTest__GetMainPartition_ReturnsNonNull() {
    let fixture = ModelFixture::set_up();
    assert!(!fixture.my_model.get_main_partition().unwrap().is_null());
}

#[test]
fn TObj_ModelTest__GetRoot_ReturnsNonNull() {
    let fixture = ModelFixture::set_up();
    assert!(!fixture.my_model.get_root().unwrap().is_null());
}

#[test]
fn TObj_ModelTest__GetModelName_ReturnsDefault() {
    let fixture = ModelFixture::set_up();
    let name = fixture.my_model.get_model_name();
    assert!(!name.is_null());
    assert!(!name.is_empty());
}

#[test]
fn TObj_ModelTest__GetFormat_ReturnsTObjBin() {
    let fixture = ModelFixture::set_up();
    assert!(
        fixture
            .my_model
            .get_format()
            .is_equal(ExtendedString::new("TObjBin"))
    );
}

#[test]
fn TObj_ModelTest__GetApplication_ReturnsNonNull() {
    let fixture = ModelFixture::set_up();
    assert!(!fixture.my_model.get_application().unwrap().is_null());
}

#[test]
fn TObj_ModelTest__GetDocumentModel_FromLabel_ReturnsSameModel() {
    let fixture = ModelFixture::set_up();
    let label = fixture.my_model.get_label();
    let found = TObjModelBase::get_document_model(&label).unwrap();
    assert!(!found.is_null());
    assert!(found == fixture.my_model);
}

#[test]
fn TObj_ModelTest__Close_NullifiesLabel() {
    let fixture = ModelFixture::set_up();
    assert!(fixture.my_model.close().unwrap());
    assert!(fixture.my_model.get_label().is_null());
}

#[test]
fn TObj_ModelTest__Close_ReturnsFalse_WhenAlreadyClosed() {
    let fixture = ModelFixture::set_up();
    assert!(fixture.my_model.close().unwrap());
    assert!(!fixture.my_model.close().unwrap());
}

#[test]
fn TObj_ModelTest__IsModified_InitialState() {
    let fixture = ModelFixture::set_up();
    // A freshly loaded model reports its modification status; just verify no failure.
    let _modified = fixture.my_model.is_modified().unwrap();
}

#[test]
fn TObj_ModelTest__SetModified_True_MakesModified() {
    let fixture = ModelFixture::set_up();
    fixture.my_model.set_modified(true).unwrap();
    assert!(fixture.my_model.is_modified().unwrap());
}

#[test]
fn TObj_ModelTest__SetModified_False_ClearsModified() {
    let fixture = ModelFixture::set_up();
    fixture.my_model.set_modified(true).unwrap();
    fixture.my_model.set_modified(false).unwrap();
    assert!(!fixture.my_model.is_modified().unwrap());
}

#[test]
fn TObj_ModelTest__CommandTransaction_OpenCommit() {
    let fixture = ModelFixture::set_up();
    let model = &fixture.my_model;
    model.get_document().unwrap().set_undo_limit(10).unwrap();
    assert!(!model.has_open_command().unwrap());
    model.open_command().unwrap();
    assert!(model.has_open_command().unwrap());
    model.commit_command().unwrap();
    assert!(!model.has_open_command().unwrap());
}

#[test]
fn TObj_ModelTest__CommandTransaction_OpenAbort() {
    let fixture = ModelFixture::set_up();
    let model = &fixture.my_model;
    model.get_document().unwrap().set_undo_limit(10).unwrap();
    model.open_command().unwrap();
    assert!(model.has_open_command().unwrap());
    model.abort_command().unwrap();
    assert!(!model.has_open_command().unwrap());
}

#[test]
fn TObj_ModelTest__GetChildren_EmptyModel_NoChildren() {
    let fixture = ModelFixture::set_up();
    let iterator = fixture.my_model.get_children().unwrap();
    if let Some(iterator) = iterator.get() {
        assert!(!iterator.more());
    }
}

#[test]
fn TObj_ModelTest__GetObjects_EmptyModel_HasRootOnly() {
    let fixture = ModelFixture::set_up();
    let iterator = fixture.my_model.get_objects().unwrap();
    assert!(!iterator.is_null());
    // The model iterator should at least exist; root partition is always present.
    let mut count = 0;
    while iterator.more() {
        count += 1;
        iterator.next().unwrap();
    }
    assert!(count >= 0);
}

#[test]
fn TObj_ModelTest__NewEmpty_ReturnsFreshModel() {
    let fixture = ModelFixture::set_up();
    let new_model = fixture.my_model.new_empty().unwrap();
    assert!(!new_model.is_null());
    // The new model should not share the same label.
    assert!(new_model.get_label().is_null());
}

#[test]
fn TObj_ModelTest__GetDictionary_ReturnsNonNull() {
    let fixture = ModelFixture::set_up();
    assert!(!fixture.my_model.get_dictionary().unwrap().is_null());
}

#[test]
fn TObj_ModelTest__GetFormatVersion_FreshModel_ReturnsMinusOne() {
    let fixture = ModelFixture::set_up();
    assert_eq!(fixture.my_model.get_format_version().unwrap(), -1);
}

#[test]
fn TObj_ModelTest__Update_DefaultReturnsTrue() {
    let fixture = ModelFixture::set_up();
    assert!(fixture.my_model.update().unwrap());
}

#[test]
fn TObj_ModelTest__Messenger_SetAndGet() {
    let fixture = ModelFixture::set_up();
    let messenger = Arc::new(Messenger::new());
    fixture.my_model.set_messenger(messenger.clone());
    assert!(Arc::ptr_eq(&messenger, &fixture.my_model.messenger()));
}

/// The `TObj_ObjectTest` fixture: a loaded model and its main partition.
pub struct ObjectFixture {
    /// The model part of the fixture.
    pub model: ModelFixture,
    /// `myPartition`.
    pub my_partition: Handle<dyn TObjPartition>,
}
impl ObjectFixture {
    /// `SetUp`.
    pub fn set_up() -> Self {
        let model = ModelFixture::set_up();
        let my_partition = model.my_model.get_main_partition().unwrap();
        assert!(!my_partition.is_null());
        Self {
            model,
            my_partition,
        }
    }
    /// `myModel`.
    pub fn my_model(&self) -> &Handle<TObjTestModel> {
        &self.model.my_model
    }
    /// `createObject`: a new object on a new label of the partition.
    pub fn create_object(&self) -> Handle<TObjTestObject> {
        TObjTestObject::new(&self.my_partition.new_label().unwrap())
    }
    /// `OpenCommand`, the body, `CommitCommand`.
    pub fn command<R>(&self, body: impl FnOnce() -> R) -> R {
        self.my_model().open_command().unwrap();
        let result = body();
        self.my_model().commit_command().unwrap();
        result
    }
}

#[test]
fn TObj_ObjectTest__CreateObject_IsAlive() {
    let fixture = ObjectFixture::set_up();
    let object = fixture.command(|| fixture.create_object());
    assert!(!object.is_null());
    assert!(object.is_alive().unwrap());
}

#[test]
fn TObj_ObjectTest__CreateObject_HasLabel() {
    let fixture = ObjectFixture::set_up();
    let object = fixture.command(|| fixture.create_object());
    assert!(!object.get_label().is_null());
}

#[test]
fn TObj_ObjectTest__CreateObject_GetModel_ReturnsSameModel() {
    let fixture = ObjectFixture::set_up();
    let object = fixture.command(|| fixture.create_object());
    let found = object.get_model().unwrap();
    assert!(!found.is_null());
    assert!(found == *fixture.my_model());
}

#[test]
fn TObj_ObjectTest__SetName_GetName_Roundtrip() {
    let fixture = ObjectFixture::set_up();
    let object = fixture.command(|| fixture.create_object());
    let new_name = h_name("TestObjectName");
    fixture.command(|| assert!(object.set_name(&new_name).unwrap()));
    let got = object.get_name().unwrap();
    assert!(!got.is_null());
    assert!(got.string().is_equal(&*new_name.string()));
}

#[test]
fn TObj_ObjectTest__SetName_CString_Roundtrip() {
    let fixture = ObjectFixture::set_up();
    let object = fixture.command(|| fixture.create_object());
    fixture.command(|| assert!(object.set_name_c_str("AsciiName").unwrap()));
    let mut name = AsciiString::default();
    assert!(object.get_name_ascii(&mut name).unwrap());
    assert!(name.is_equal(AsciiString::new("AsciiName")));
}

#[test]
fn TObj_ObjectTest__SetName_Duplicate_ReturnsFalse() {
    let fixture = ObjectFixture::set_up();
    let (first, second) = fixture.command(|| (fixture.create_object(), fixture.create_object()));
    let name = h_name("UniqueName");
    fixture.command(|| assert!(first.set_name(&name).unwrap()));
    fixture.command(|| assert!(!second.set_name(&name).unwrap()));
}

#[test]
fn TObj_ObjectTest__GetChildren_NoChildren_Empty() {
    let fixture = ObjectFixture::set_up();
    let object = fixture.command(|| fixture.create_object());
    let iterator = object.get_children(None).unwrap();
    if let Some(iterator) = iterator.get() {
        assert!(!iterator.more());
    }
}

#[test]
fn TObj_ObjectTest__GetDataLabel_ReturnsValidLabel() {
    let fixture = ObjectFixture::set_up();
    let object = fixture.command(|| fixture.create_object());
    assert!(!object.get_data_label().unwrap().is_null());
}

#[test]
fn TObj_ObjectTest__GetReferenceLabel_ReturnsValidLabel() {
    let fixture = ObjectFixture::set_up();
    let object = fixture.command(|| fixture.create_object());
    assert!(!object.get_reference_label().unwrap().is_null());
}

#[test]
fn TObj_ObjectTest__Flags_SetAndTest() {
    let fixture = ObjectFixture::set_up();
    let object = fixture.command(|| fixture.create_object());
    fixture.command(|| object.set_flags(TObjObjectStateFlags::HIDDEN).unwrap());
    assert!(object.test_flags(TObjObjectStateFlags::HIDDEN).unwrap());
    assert!(!object.test_flags(TObjObjectStateFlags::SAVED).unwrap());
}

#[test]
fn TObj_ObjectTest__Flags_ClearFlags() {
    let fixture = ObjectFixture::set_up();
    let object = fixture.command(|| fixture.create_object());
    let both = TObjObjectStateFlags::HIDDEN | TObjObjectStateFlags::SAVED;
    fixture.command(|| object.set_flags(both).unwrap());
    fixture.command(|| object.clear_flags(TObjObjectStateFlags::HIDDEN).unwrap());
    assert!(!object.test_flags(TObjObjectStateFlags::HIDDEN).unwrap());
    assert!(object.test_flags(TObjObjectStateFlags::SAVED).unwrap());
}

#[test]
fn TObj_ObjectTest__Order_SetAndGet() {
    let fixture = ObjectFixture::set_up();
    let object = fixture.command(|| fixture.create_object());
    fixture.command(|| assert!(object.set_order(42).unwrap()));
    assert_eq!(42, object.get_order().unwrap());
}

#[test]
fn TObj_ObjectTest__Detach_RemovesObject() {
    let fixture = ObjectFixture::set_up();
    let object = fixture.command(|| fixture.create_object());
    fixture.command(|| assert!(object.detach(TObjDeletingMode::FreeOnly).unwrap()));
    assert!(!object.is_alive().unwrap());
}

#[test]
fn TObj_ObjectTest__GetObj_StaticLookup() {
    let fixture = ObjectFixture::set_up();
    let object = fixture.command(|| fixture.create_object());
    let label = object.get_label();
    let mut found = Handle::default();
    assert!(TObjObjectBase::get_obj(&label, &mut found, false).unwrap());
    assert!(!found.is_null());
    assert!(found == object);
}

#[test]
fn TObj_ObjectTest__ModelIteration_AfterAddingObjects() {
    let fixture = ObjectFixture::set_up();
    let _objects = fixture.command(|| {
        [
            fixture.create_object(),
            fixture.create_object(),
            fixture.create_object(),
        ]
    });
    let iterator = fixture.my_model().get_children().unwrap();
    assert!(!iterator.is_null());
    let mut count = 0;
    while iterator.more() {
        count += 1;
        iterator.next().unwrap();
    }
    assert_eq!(3, count);
}

/// The names of the objects an iterator visits, in a set (`NCollection_IndexedMap`).
fn visited_names(iterator: &Handle<dyn TObjObjectIterator>) -> Vec<String> {
    let mut names = Vec::new();
    while iterator.more() {
        let mut name = AsciiString::default();
        assert!(iterator.value().get_name_ascii(&mut name).unwrap());
        let name = String::from_utf8_lossy(name.as_bytes()).into_owned();
        if !names.contains(&name) {
            names.push(name);
        }
        iterator.next().unwrap();
    }
    names
}

// caf/bugs/bug31323: typed child iteration remains recursive through all object levels.
#[test]
fn TObj_ObjectTest__CafBug_31323_RecursiveTypedChildren() {
    let fixture = ObjectFixture::set_up();
    let object = fixture.command(|| {
        let object = fixture.create_object();
        let sub1 = object.add_child();
        let sub2 = object.add_child();
        let sub11 = sub1.add_child();
        let sub12 = sub1.add_child();
        let sub13 = sub1.add_child();
        let sub121 = sub12.add_child();
        assert!(object.set_name_c_str("obj").unwrap());
        assert!(sub1.set_name_c_str("sub1").unwrap());
        assert!(sub2.set_name_c_str("sub2").unwrap());
        assert!(sub11.set_name_c_str("sub11").unwrap());
        assert!(sub12.set_name_c_str("sub12").unwrap());
        assert!(sub13.set_name_c_str("sub13").unwrap());
        assert!(sub121.set_name_c_str("sub121").unwrap());
        object
    });
    let flat = object.get_children(None).unwrap();
    assert!(!flat.is_null());
    let flat_names = visited_names(&flat);
    assert_eq!(flat_names.len(), 2);
    assert!(flat_names.contains(&"sub1".to_owned()));
    assert!(flat_names.contains(&"sub2".to_owned()));
    let all: Handle<dyn TObjObjectIterator> =
        TObjOcafObjectIterator::new(&object.get_child_label().unwrap(), None, true, true)
            .unwrap()
            .cast();
    assert!(!all.is_null());
    let all_names = visited_names(&all);
    assert_eq!(all_names.len(), 6);
    for name in ["sub1", "sub2", "sub11", "sub12", "sub13", "sub121"] {
        assert!(all_names.contains(&name.to_owned()));
    }
}

#[test]
fn TObj_ObjectTest__GetTypeFlags_DefaultVisible() {
    let fixture = ObjectFixture::set_up();
    let object = fixture.command(|| fixture.create_object());
    assert!(object.get_type_flags() & TObjTypeFlags::VISIBLE != 0);
}

#[test]
fn TObj_ObjectTest__HasBackReferences_InitiallyFalse() {
    let fixture = ObjectFixture::set_up();
    let object = fixture.command(|| fixture.create_object());
    assert!(!object.has_back_references());
}

#[test]
fn TObj_ObjectTest__NameRegistration_InModel() {
    let fixture = ObjectFixture::set_up();
    let object = fixture.command(|| fixture.create_object());
    let name = h_name("RegisteredName");
    fixture.command(|| object.set_name(&name).unwrap());
    let dictionary = fixture.my_model().get_dictionary().unwrap();
    assert!(
        fixture
            .my_model()
            .is_registered_name(&name, &dictionary)
            .unwrap()
    );
}

#[test]
fn TObj_ObjectTest__FindObject_ByName() {
    let fixture = ObjectFixture::set_up();
    let object = fixture.command(|| fixture.create_object());
    let name = h_name("FindMe");
    fixture.command(|| object.set_name(&name).unwrap());
    let dictionary = fixture.my_model().get_dictionary().unwrap();
    let found = fixture.my_model().find_object(&name, &dictionary).unwrap();
    assert!(!found.is_null());
    assert!(found == object);
}

// OCC31320: TObj_Object::GetFatherObject() is not protected against a deleted object;
// after detaching a parent object, GetFatherObject() on its child must return null.
#[test]
fn TObj_ObjectTest__OCC31320_GetFatherObject_ReturnsNull_AfterParentDetach() {
    let fixture = ObjectFixture::set_up();
    let parent = fixture.command(|| fixture.create_object());
    assert!(!parent.is_null());
    let child = fixture
        .command(|| TObjTestObject::new(&parent.get_child_label().unwrap().new_child().unwrap()));
    assert!(!child.is_null());
    fixture.command(|| assert!(parent.detach(TObjDeletingMode::FreeOnly).unwrap()));
    // After the parent is detached its TObj attributes are gone, so the walk up finds
    // no father.
    assert!(child.get_father_object(None).unwrap().is_null());
}
