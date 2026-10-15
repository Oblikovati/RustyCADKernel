// Rust port of ApplicationFramework/TKTObj/TObj/TObj_Persistence.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TObj_Persistence`: the process-wide registry that creates correctly typed objects
//! from their type names (tobj-design.md section 4).
use super::{TObjHiddenPartition, TObjObject, TObjStandardPartition};
use rustycad_tkernel::{
    ncollection::DataMap,
    standard::{Failure, Handle},
    tcollection::AsciiString,
};
use rustycad_tklcaf::tdf::TdfLabel;
use std::{
    io,
    sync::{Arc, LazyLock, Mutex, MutexGuard, PoisonError},
};

/// `TObj_Persistence::New` (pure virtual): creates an object of the registered type on
/// a label with the persistence constructor (not attached, `initFields` called).
pub trait TObjPersistenceFactory: Send + Sync {
    /// `New(theLabel)`.
    fn new_object(&self, label: &TdfLabel) -> Result<Handle<dyn TObjObject>, Failure>;
}
impl<F> TObjPersistenceFactory for F
where
    F: Fn(&TdfLabel) -> Result<Handle<dyn TObjObject>, Failure> + Send + Sync,
{
    fn new_object(&self, label: &TdfLabel) -> Result<Handle<dyn TObjObject>, Failure> {
        self(label)
    }
}

/// The registry map (`getMapOfTypes`), keyed by type name.
type TypeMap = DataMap<AsciiString, Arc<dyn TObjPersistenceFactory>>;

/// `getMapOfTypes`: the native static map, with the types of this toolkit registered
/// when it is first used (their native static `myPersistence_` members).
static MAP_OF_TYPES: LazyLock<Mutex<TypeMap>> = LazyLock::new(|| {
    let mut map = TypeMap::default();
    let partition: Arc<dyn TObjPersistenceFactory> =
        Arc::new(TObjStandardPartition::persistence_new);
    let hidden: Arc<dyn TObjPersistenceFactory> = Arc::new(TObjHiddenPartition::persistence_new);
    map.bind(AsciiString::new("TObj_Partition"), partition);
    map.bind(AsciiString::new("TObj_HiddenPartition"), hidden);
    Mutex::new(map)
});

fn map_of_types() -> MutexGuard<'static, TypeMap> {
    MAP_OF_TYPES.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The registry map type of `getMapOfTypes`: the tools by type name.
pub type TObjPersistenceMap = TypeMap;

/// `TObj_Persistence`: a registration of one type; the constructor registers the tool
/// and `Drop` (the native destructor) unregisters the type name.
pub struct TObjPersistence {
    type_name: AsciiString,
}

impl TObjPersistence {
    /// `TObj_Persistence(theType)`: registers the tool under the type name.
    ///
    /// ```
    /// use rustycad_tklcaf::tdf::TdfLabel;
    /// use rustycad_tktobj::tobj::{TObjObject, TObjPersistence};
    /// use rustycad_tkernel::standard::{Failure, Handle};
    /// fn none(_: &TdfLabel) -> Result<Handle<dyn TObjObject>, Failure> { Ok(Handle::default()) }
    /// let registration = TObjPersistence::new("Doc_None", std::sync::Arc::new(none));
    /// let mut names = Vec::new();
    /// TObjPersistence::dump_types(&mut names).unwrap();
    /// assert!(String::from_utf8(names).unwrap().contains("Doc_None\n"));
    /// drop(registration);
    /// ```
    pub fn new(type_name: &str, tool: Arc<dyn TObjPersistenceFactory>) -> Self {
        let type_name = AsciiString::new(type_name);
        map_of_types().bind(type_name.clone(), tool);
        Self { type_name }
    }
    /// A registration that lives as long as the process, as the native static
    /// `myPersistence_` members of `IMPLEMENT_TOBJOCAF_PERSISTENCE`.
    pub fn register_static(type_name: &str, tool: Arc<dyn TObjPersistenceFactory>) {
        map_of_types().bind(AsciiString::new(type_name), tool);
    }
    /// `getMapOfTypes` (protected): the locked registry map; the guard must be dropped
    /// before a tool is called.
    pub fn map_of_types() -> MutexGuard<'static, TObjPersistenceMap> {
        map_of_types()
    }
    /// `CreateNewObject(theType, theLabel)`: an object of the registered type, null when
    /// the type is not registered.
    pub fn create_new_object(
        type_name: &str,
        label: &TdfLabel,
    ) -> Result<Handle<dyn TObjObject>, Failure> {
        let tool = map_of_types().seek(&AsciiString::new(type_name)).cloned();
        match tool {
            Some(tool) => tool.new_object(label),
            None => Ok(Handle::default()),
        }
    }
    /// `DumpTypes(theOs)`: each registered type name and a newline, in map order.
    pub fn dump_types(stream: &mut dyn io::Write) -> io::Result<()> {
        let names: Vec<AsciiString> = map_of_types()
            .items()
            .map(|(name, _)| name.clone())
            .collect();
        for name in names {
            stream.write_all(name.as_bytes())?;
            stream.write_all(b"\n")?;
        }
        Ok(())
    }
}

impl Drop for TObjPersistence {
    /// `~TObj_Persistence`: unregisters the type name.
    fn drop(&mut self) {
        map_of_types().unbind(&self.type_name);
    }
}
