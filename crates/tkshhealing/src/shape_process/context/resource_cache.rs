//! `ShapeProcess_Context::LoadResourceManager`: the process-wide cache of the last loaded
//! resource file, reloaded when its name or the modification time of its files changes.
use super::{ShapeProcessContext, ShapeProcessResource};
use rustycad_tkernel::resource::{ResourceHost, ResourceManager};
use rustycad_tkernel::standard::{Failure, SharedState};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::{Arc, Mutex};
use std::time::UNIX_EPOCH;

/// The function statics of the native LoadResourceManager (sRC, sName, sMtime, sUMtime),
/// guarded by the native mutex.
struct ResourceCache {
    manager: Option<ResourceManager>,
    name: Vec<u8>,
    mtime: i64,
    user_mtime: i64,
}

static CACHE: Mutex<ResourceCache> = Mutex::new(ResourceCache {
    manager: None,
    name: Vec::new(),
    mtime: 0,
    user_mtime: 0,
});

/// The modification time in seconds of a resource file; 0 for no path. Natively a failed
/// `stat` leaves the time unread (undefined); Rust reads 0.
fn modification_time(path: &AsciiString) -> i64 {
    if path.is_empty() {
        return 0;
    }
    let path = String::from_utf8_lossy(path.as_bytes()).into_owned();
    std::fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |elapsed| elapsed.as_secs() as i64)
}

impl ResourceCache {
    // Native: a cached manager of another name, or of changed files, is dropped.
    fn invalidate(&mut self, name: &str, times: (i64, i64)) -> bool {
        let mut modified = false;
        if self.manager.is_none() {
            return modified;
        }
        if self.name != name.as_bytes() {
            self.manager = None;
            return modified;
        }
        if self.mtime != times.0 {
            (self.mtime, modified) = (times.0, true);
        }
        if self.user_mtime != times.1 {
            (self.user_mtime, modified) = (times.1, true);
        }
        if modified {
            self.manager = None;
        }
        modified
    }
}

impl ShapeProcessContext {
    /// `LoadResourceManager(name)`: a copy of the cached manager of the resource file
    /// `name` (`$CSF_<name>Defaults` and `$CSF_<name>UserDefaults`), reloaded when the
    /// name or a file time changed ("file is load only once and reloaded only if file date
    /// has changed"); `context.load_resource_manager("DEDefaults")?`.
    pub fn load_resource_manager(&self, name: &str) -> Result<ShapeProcessResource, Failure> {
        let mut cache = CACHE
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let times = (
            modification_time(&ResourceManager::resource_path(name, false)),
            modification_time(&ResourceManager::resource_path(name, true)),
        );
        let modified = cache.invalidate(name, times);
        if cache.manager.is_none() {
            cache.manager = Some(ResourceManager::new(name, false, ResourceHost::native())?);
            if !modified {
                (cache.name, cache.mtime, cache.user_mtime) =
                    (name.as_bytes().to_vec(), times.0, times.1);
            }
        }
        // "Creating copy of sRC for thread safety of Resource_Manager variables".
        let manager = cache
            .manager
            .clone()
            .unwrap_or_else(|| ResourceManager::empty(ResourceHost::native()));
        Ok(Arc::new(SharedState::new(manager)))
    }
}
