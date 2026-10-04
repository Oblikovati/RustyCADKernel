//! OCCT deferred triangulation filesystem forwarding and load state replay.
use rustycad_tkernel::{
    osd::{
        FileSystem, FileSystemHandle, IosOpenMode, SharedIStream, SharedOStream, SharedStreamBuf,
    },
    standard::{Failure, Handle},
    tcollection::AsciiString,
};
use rustycad_tkmath::poly::{DeferredTriangulation, Triangulation, mesh_purpose};
use std::sync::{Arc, Mutex};

struct FakeFileSystem;
impl FileSystem for FakeFileSystem {
    fn is_supported_path(&self, _url: &AsciiString) -> bool {
        false
    }
    fn is_open_istream(&self, _stream: &SharedIStream) -> bool {
        false
    }
    fn is_open_ostream(&self, _stream: &SharedOStream) -> bool {
        false
    }
    fn open_stream_buffer(
        &self,
        _url: &AsciiString,
        _mode: IosOpenMode,
        _offset: i64,
        _size: Option<&mut i64>,
    ) -> Result<Option<SharedStreamBuf>, Failure> {
        Ok(None)
    }
}

struct LoadTrace {
    available: bool,
    success: bool,
    calls: usize,
    seen: Option<FileSystemHandle>,
    destination: usize,
}
struct RecordingStorage(Arc<Mutex<LoadTrace>>);

struct StorageWithoutLoader;
impl DeferredTriangulation for StorageWithoutLoader {
    fn nb_deferred_triangles(&self) -> i32 {
        1
    }
}

#[test]
fn base_virtual_loader_reports_failure() {
    let mesh = Triangulation::with_deferred(Box::new(StorageWithoutLoader));
    assert!(mesh.has_deferred_data());
    assert!(!mesh.load_deferred_data());
    assert!(mesh.detached_load_deferred_data().is_null());
    assert_eq!(mesh.mesh_purpose(), mesh_purpose::NONE);
}

impl DeferredTriangulation for RecordingStorage {
    fn nb_deferred_triangles(&self) -> i32 {
        i32::from(self.0.lock().unwrap().available)
    }
    fn create_new_entity(&self) -> Handle<Triangulation> {
        let mesh = Handle::new(Triangulation::new());
        mesh.set_mesh_purpose(mesh_purpose::USER);
        mesh
    }
    fn load_deferred_data_with_file_system(
        &self,
        fs: Option<&FileSystemHandle>,
        destination: &Triangulation,
    ) -> bool {
        let mut trace = self.0.lock().unwrap();
        trace.calls += 1;
        trace.seen = fs.cloned();
        trace.destination = std::ptr::from_ref(destination) as usize;
        trace.success
    }
}

#[test]
fn native_filesystem_forwarding_snapshot() {
    let trace = Arc::new(Mutex::new(LoadTrace {
        available: true,
        success: true,
        calls: 0,
        seen: None,
        destination: 0,
    }));
    let mesh = Triangulation::with_deferred(Box::new(RecordingStorage(trace.clone())));
    let fs: FileSystemHandle = Arc::new(FakeFileSystem);
    mesh.set_mesh_purpose(mesh_purpose::CALCULATION);
    let mut output = supplied_snapshot(&mesh, &fs, &trace);
    output.push_str(&detached_snapshot(&mesh, &trace));
    output.push_str(&failed_snapshot(&mesh, &fs, &trace));
    output.push_str(&empty_snapshot(&mesh, &fs, &trace));
    assert_eq!(
        output,
        include_str!("../../../tools/oracle/poly_deferred_filesystem.expected")
    );
}

fn supplied_snapshot(
    mesh: &Triangulation,
    fs: &FileSystemHandle,
    trace: &Mutex<LoadTrace>,
) -> String {
    let loaded = u8::from(mesh.load_deferred_data_with_file_system(Some(fs)));
    let trace = trace.lock().unwrap();
    let same_fs = u8::from(Arc::ptr_eq(trace.seen.as_ref().unwrap(), fs));
    let same_mesh = u8::from(trace.destination == std::ptr::from_ref(mesh) as usize);
    format!(
        "supplied {loaded} {} {same_fs} {same_mesh} {}\n",
        trace.calls,
        mesh.mesh_purpose()
    )
}

fn detached_snapshot(mesh: &Triangulation, trace: &Mutex<LoadTrace>) -> String {
    let detached = mesh.detached_load_deferred_data();
    let trace = trace.lock().unwrap();
    let present = u8::from(!detached.is_null());
    let null_fs = u8::from(trace.seen.is_none());
    let same_mesh =
        u8::from(trace.destination == std::ptr::from_ref(detached.get().unwrap()) as usize);
    format!(
        "detached {present} {} {null_fs} {same_mesh} {} {}\n",
        trace.calls,
        detached.mesh_purpose(),
        mesh.mesh_purpose()
    )
}

fn failed_snapshot(
    mesh: &Triangulation,
    fs: &FileSystemHandle,
    trace: &Mutex<LoadTrace>,
) -> String {
    trace.lock().unwrap().success = false;
    mesh.set_mesh_purpose(mesh_purpose::CALCULATION);
    let loaded = u8::from(mesh.load_deferred_data());
    let detached = mesh.detached_load_deferred_data_with_file_system(Some(fs));
    let trace = trace.lock().unwrap();
    let null_mesh = u8::from(detached.is_null());
    let same_fs = u8::from(Arc::ptr_eq(trace.seen.as_ref().unwrap(), fs));
    format!(
        "failed {loaded} {null_mesh} {} {same_fs} {}\n",
        trace.calls,
        mesh.mesh_purpose()
    )
}

fn empty_snapshot(mesh: &Triangulation, fs: &FileSystemHandle, trace: &Mutex<LoadTrace>) -> String {
    trace.lock().unwrap().available = false;
    let loaded = u8::from(mesh.load_deferred_data_with_file_system(Some(fs)));
    let null_mesh = u8::from(
        mesh.detached_load_deferred_data_with_file_system(Some(fs))
            .is_null(),
    );
    format!(
        "empty {loaded} {null_mesh} {}\n",
        trace.lock().unwrap().calls
    )
}
