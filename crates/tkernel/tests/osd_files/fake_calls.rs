//! `FakePosixCalls`: an in-memory POSIX file system with Linux errno values, so the OSD
//! file classes are tested without touching the disk or the process umask.
use rustycad_tkernel::osd::{
    AccessMode, CallResult, Errno, LocalTime, NodeStatus, NodeType, OpenFlags, OpenMode,
    PosixFileCalls, RecordLock, SeekOrigin,
};
use std::{collections::BTreeMap, sync::Mutex};

pub const ENOENT: i32 = 2;
pub const EBADF: i32 = 9;
pub const EACCES: i32 = 13;
pub const EEXIST: i32 = 17;
pub const ENOTDIR: i32 = 20;
pub const EISDIR: i32 = 21;
pub const EINVAL: i32 = 22;
pub const ENOTEMPTY: i32 = 39;

#[derive(Clone, Debug)]
pub enum FakeNode {
    Directory,
    File(Vec<u8>),
    Socket,
}

#[derive(Clone, Debug)]
struct FakeDescriptor {
    path: Vec<u8>,
    position: usize,
    access: OpenMode,
    append: bool,
}

#[derive(Default)]
struct FakeState {
    nodes: BTreeMap<Vec<u8>, (FakeNode, u32)>,
    descriptors: BTreeMap<i32, FakeDescriptor>,
    umask: u32,
    serial: u32,
    locks: Vec<(i32, RecordLock)>,
    order: BTreeMap<Vec<u8>, Vec<Vec<u8>>>,
}

/// The fake; `FakePosixCalls::new().with_file("/w/a", b"data")`.
#[derive(Default)]
pub struct FakePosixCalls {
    state: Mutex<FakeState>,
    /// `st_blksize` reported for every node (the stdio buffer size).
    pub block_size: u64,
}

/// Resolves `//`, `.` and `..` the way the kernel's path walk does.
fn normalize(path: &[u8]) -> Vec<u8> {
    let absolute: bool = path.first() == Some(&b'/');
    let mut parts: Vec<&[u8]> = Vec::new();
    for part in path.split(|&byte| byte == b'/') {
        match part {
            b"" | b"." => {}
            b".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    let joined: Vec<u8> = parts.join(&b'/');
    match (absolute, joined.is_empty()) {
        (true, _) => [b"/".as_slice(), &joined].concat(),
        (false, true) if !path.is_empty() => b".".to_vec(),
        _ => joined,
    }
}

fn parent(path: &[u8]) -> &[u8] {
    match path.iter().rposition(|&byte| byte == b'/') {
        Some(0) => b"/",
        Some(index) => &path[..index],
        None => b".",
    }
}

impl FakePosixCalls {
    pub fn new() -> Self {
        let calls = Self {
            block_size: 4096,
            ..Self::default()
        };
        calls.state.lock().unwrap().umask = 0o022;
        calls.add(b"/", FakeNode::Directory, 0o755);
        calls.add(b".", FakeNode::Directory, 0o755);
        calls
    }
    pub fn with_block_size(block_size: u64) -> Self {
        Self {
            block_size,
            ..Self::new()
        }
    }
    pub fn add(&self, path: &[u8], node: FakeNode, mode: u32) {
        self.state
            .lock()
            .unwrap()
            .nodes
            .insert(normalize(path), (node, mode));
    }
    pub fn with_file(self, path: &str, contents: &[u8]) -> Self {
        self.add(path.as_bytes(), FakeNode::File(contents.to_vec()), 0o644);
        self
    }
    pub fn with_directory(self, path: &str) -> Self {
        self.add(path.as_bytes(), FakeNode::Directory, 0o755);
        self
    }
    /// Fixes the `readdir` order of a directory's names (default: sorted, `.` and `..` first).
    pub fn with_order(self, path: &str, names: &[&str]) -> Self {
        let names: Vec<Vec<u8>> = names.iter().map(|name| name.as_bytes().to_vec()).collect();
        self.state
            .lock()
            .unwrap()
            .order
            .insert(normalize(path.as_bytes()), names);
        self
    }
    pub fn contents(&self, path: &str) -> Option<Vec<u8>> {
        match self.state.lock().unwrap().nodes.get(path.as_bytes()) {
            Some((FakeNode::File(data), _)) => Some(data.clone()),
            _ => None,
        }
    }
    pub fn mode(&self, path: &str) -> Option<u32> {
        self.state
            .lock()
            .unwrap()
            .nodes
            .get(path.as_bytes())
            .map(|(_, mode)| *mode)
    }
    pub fn umask_value(&self) -> u32 {
        self.state.lock().unwrap().umask
    }
    pub fn locks(&self) -> Vec<(i32, RecordLock)> {
        self.state.lock().unwrap().locks.clone()
    }
    pub fn open_descriptors(&self) -> usize {
        self.state.lock().unwrap().descriptors.len()
    }
}

fn node_status(node: &FakeNode, mode: u32, block_size: u64) -> NodeStatus {
    let (node_type, size) = match node {
        FakeNode::Directory => (NodeType::Directory, 4096),
        FakeNode::File(data) => (NodeType::Regular, data.len() as u64),
        FakeNode::Socket => (NodeType::Socket, 0),
    };
    NodeStatus {
        node_type,
        permissions: mode,
        size,
        change_time: 318_000_000,
        block_size,
    }
}

impl FakeState {
    fn node(&self, path: &[u8]) -> CallResult<&(FakeNode, u32)> {
        if path.is_empty() {
            return Err(Errno(ENOENT));
        }
        self.nodes.get(path).ok_or(Errno(ENOENT))
    }
    fn parent_is_directory(&self, path: &[u8]) -> CallResult<()> {
        match self.node(parent(path))? {
            (FakeNode::Directory, _) => Ok(()),
            _ => Err(Errno(ENOTDIR)),
        }
    }
    fn descriptor(&mut self, fd: i32) -> CallResult<&mut FakeDescriptor> {
        self.descriptors.get_mut(&fd).ok_or(Errno(EBADF))
    }
    fn new_descriptor(&mut self, descriptor: FakeDescriptor) -> i32 {
        let fd: i32 = (3..)
            .find(|number| !self.descriptors.contains_key(number))
            .unwrap();
        self.descriptors.insert(fd, descriptor);
        fd
    }
    fn file_data(&mut self, path: &[u8]) -> CallResult<&mut Vec<u8>> {
        match self.nodes.get_mut(path) {
            Some((FakeNode::File(data), _)) => Ok(data),
            _ => Err(Errno(EBADF)),
        }
    }
}

impl PosixFileCalls for FakePosixCalls {
    fn access(&self, path: &[u8], mode: AccessMode) -> CallResult<()> {
        let path: &[u8] = &normalize(path);
        let state = self.state.lock().unwrap();
        let (_, permissions) = state.node(path)?;
        let denied: bool = (mode.read && permissions & 0o400 == 0)
            || (mode.write && permissions & 0o200 == 0)
            || (mode.execute && permissions & 0o100 == 0);
        if denied { Err(Errno(EACCES)) } else { Ok(()) }
    }
    fn stat(&self, path: &[u8]) -> CallResult<NodeStatus> {
        let path: &[u8] = &normalize(path);
        let state = self.state.lock().unwrap();
        let (node, mode) = state.node(path)?;
        Ok(node_status(node, *mode, self.block_size))
    }
    fn fstat(&self, fd: i32) -> CallResult<NodeStatus> {
        let mut state = self.state.lock().unwrap();
        let path: Vec<u8> = state.descriptor(fd)?.path.clone();
        let (node, mode) = state.node(&path)?;
        Ok(node_status(node, *mode, self.block_size))
    }
    fn open(&self, path: &[u8], flags: OpenFlags, mode: u32) -> CallResult<i32> {
        let path: &[u8] = &normalize(path);
        let mut state = self.state.lock().unwrap();
        match state.nodes.get(path) {
            None if !flags.create => return Err(Errno(ENOENT)),
            None => {
                state.parent_is_directory(path)?;
                let masked: u32 = mode & !state.umask;
                state
                    .nodes
                    .insert(path.to_vec(), (FakeNode::File(Vec::new()), masked));
            }
            Some((FakeNode::Directory, _)) if flags.access != OpenMode::ReadOnly => {
                return Err(Errno(EISDIR));
            }
            Some(_) => {}
        }
        if flags.truncate {
            if let Ok(data) = state.file_data(path) {
                data.clear();
            }
        }
        let descriptor = FakeDescriptor {
            path: path.to_vec(),
            position: 0,
            access: flags.access,
            append: flags.append,
        };
        Ok(state.new_descriptor(descriptor))
    }
    fn close(&self, fd: i32) -> CallResult<()> {
        let mut state = self.state.lock().unwrap();
        state.descriptors.remove(&fd).map(drop).ok_or(Errno(EBADF))
    }
    fn read(&self, fd: i32, buffer: &mut [u8]) -> CallResult<usize> {
        let mut state = self.state.lock().unwrap();
        let descriptor: FakeDescriptor = state.descriptor(fd)?.clone();
        if descriptor.access == OpenMode::WriteOnly {
            return Err(Errno(EBADF));
        }
        let data: Vec<u8> = state
            .file_data(&descriptor.path)
            .map_err(|_| Errno(EISDIR))?
            .clone();
        let start: usize = descriptor.position.min(data.len());
        let count: usize = buffer.len().min(data.len() - start);
        buffer[..count].copy_from_slice(&data[start..start + count]);
        state.descriptor(fd)?.position = start + count;
        Ok(count)
    }
    fn write(&self, fd: i32, buffer: &[u8]) -> CallResult<usize> {
        let mut state = self.state.lock().unwrap();
        let descriptor: FakeDescriptor = state.descriptor(fd)?.clone();
        if descriptor.access == OpenMode::ReadOnly {
            return Err(Errno(EBADF));
        }
        let data: &mut Vec<u8> = state.file_data(&descriptor.path)?;
        let start: usize = if descriptor.append {
            data.len()
        } else {
            descriptor.position
        };
        if data.len() < start + buffer.len() {
            data.resize(start + buffer.len(), 0);
        }
        data[start..start + buffer.len()].copy_from_slice(buffer);
        state.descriptor(fd)?.position = start + buffer.len();
        Ok(buffer.len())
    }
    fn lseek(&self, fd: i32, offset: i64, origin: SeekOrigin) -> CallResult<i64> {
        let mut state = self.state.lock().unwrap();
        let descriptor: FakeDescriptor = state.descriptor(fd)?.clone();
        let size: i64 = state
            .file_data(&descriptor.path)
            .map_or(0, |data| data.len() as i64);
        let base: i64 = match origin {
            SeekOrigin::Start => 0,
            SeekOrigin::Current => descriptor.position as i64,
            SeekOrigin::End => size,
        };
        let target: i64 = base + offset;
        if target < 0 {
            return Err(Errno(EINVAL));
        }
        state.descriptor(fd)?.position = target as usize;
        Ok(target)
    }
    fn record_lock(&self, fd: i32, lock: RecordLock) -> CallResult<()> {
        let mut state = self.state.lock().unwrap();
        state.descriptor(fd)?;
        state.locks.push((fd, lock));
        Ok(())
    }
    fn rename(&self, from: &[u8], to: &[u8]) -> CallResult<()> {
        let (from, to): (&[u8], &[u8]) = (&normalize(from), &normalize(to));
        let mut state = self.state.lock().unwrap();
        state.node(from)?;
        state.parent_is_directory(to)?;
        let node = state.nodes.remove(from).unwrap();
        state.nodes.insert(to.to_vec(), node);
        Ok(())
    }
    fn unlink(&self, path: &[u8]) -> CallResult<()> {
        let path: &[u8] = &normalize(path);
        let mut state = self.state.lock().unwrap();
        if matches!(state.node(path)?.0, FakeNode::Directory) {
            return Err(Errno(EISDIR));
        }
        state.nodes.remove(path);
        Ok(())
    }
    fn rmdir(&self, path: &[u8]) -> CallResult<()> {
        let path: &[u8] = &normalize(path);
        let mut state = self.state.lock().unwrap();
        if !matches!(state.node(path)?.0, FakeNode::Directory) {
            return Err(Errno(ENOTDIR));
        }
        let mut prefix: Vec<u8> = path.to_vec();
        prefix.push(b'/');
        if state.nodes.keys().any(|key| key.starts_with(&prefix)) {
            return Err(Errno(ENOTEMPTY));
        }
        state.nodes.remove(path);
        Ok(())
    }
    fn mkdir(&self, path: &[u8], mode: u32) -> CallResult<()> {
        let path: &[u8] = &normalize(path);
        let mut state = self.state.lock().unwrap();
        if path.is_empty() {
            return Err(Errno(ENOENT));
        }
        if state.nodes.contains_key(path) {
            return Err(Errno(EEXIST));
        }
        state.parent_is_directory(path)?;
        let masked: u32 = mode & !state.umask;
        state
            .nodes
            .insert(path.to_vec(), (FakeNode::Directory, masked));
        Ok(())
    }
    fn chmod(&self, path: &[u8], mode: u32) -> CallResult<()> {
        let path: &[u8] = &normalize(path);
        let mut state = self.state.lock().unwrap();
        state.node(path)?;
        state.nodes.get_mut(path).unwrap().1 = mode & 0o7777;
        Ok(())
    }
    fn umask(&self, mask: u32) -> u32 {
        std::mem::replace(&mut self.state.lock().unwrap().umask, mask)
    }
    fn mkstemp(&self, template: &[u8]) -> CallResult<(i32, Vec<u8>)> {
        let mut state = self.state.lock().unwrap();
        state.serial += 1;
        let name: Vec<u8> = [
            template.strip_suffix(b"XXXXXX").unwrap(),
            format!("{:06}", state.serial).as_bytes(),
        ]
        .concat();
        state
            .nodes
            .insert(name.clone(), (FakeNode::File(Vec::new()), 0o600));
        let descriptor = FakeDescriptor {
            path: name.clone(),
            position: 0,
            access: OpenMode::ReadWrite,
            append: false,
        };
        Ok((state.new_descriptor(descriptor), name))
    }
    fn mkdtemp(&self, template: &[u8]) -> CallResult<Vec<u8>> {
        let mut state = self.state.lock().unwrap();
        state.serial += 1;
        let name: Vec<u8> = [
            template.strip_suffix(b"XXXXXX").unwrap(),
            format!("{:06}", state.serial).as_bytes(),
        ]
        .concat();
        state
            .nodes
            .insert(name.clone(), (FakeNode::Directory, 0o700));
        Ok(name)
    }
    fn read_directory(&self, path: &[u8]) -> CallResult<Vec<Vec<u8>>> {
        let path: &[u8] = &normalize(path);
        let state = self.state.lock().unwrap();
        if !matches!(state.node(path)?.0, FakeNode::Directory) {
            return Err(Errno(ENOTDIR));
        }
        if let Some(order) = state.order.get(path) {
            return Ok(order.clone());
        }
        let mut prefix: Vec<u8> = path.to_vec();
        prefix.push(b'/');
        let mut names: Vec<Vec<u8>> = vec![b".".to_vec(), b"..".to_vec()];
        for key in state.nodes.keys() {
            if let Some(rest) = key.strip_prefix(prefix.as_slice()) {
                if !rest.is_empty() && !rest.contains(&b'/') {
                    names.push(rest.to_vec());
                }
            }
        }
        Ok(names)
    }
    fn local_time(&self, seconds: i64) -> Option<LocalTime> {
        // 318000000 s after the epoch is 1980-01-29 13:20:00 UTC; the fake is in UTC.
        (seconds == 318_000_000).then_some(LocalTime {
            month: 1,
            day: 29,
            year: 1980,
            hour: 13,
            minute: 20,
            second: 0,
        })
    }
}
