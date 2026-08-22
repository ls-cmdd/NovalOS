use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FsError {
    NotFound,
    AlreadyExists,
    InvalidName,
    InvalidHandle,
    PermissionDenied,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rights(u8);

impl Rights {
    pub const READ: Self = Self(1 << 0);
    pub const WRITE: Self = Self(1 << 1);

    pub const fn from_bits(bits: u64) -> Option<Self> {
        if bits == 0 || bits > 3 {
            None
        } else {
            Some(Self(bits as u8))
        }
    }

    pub const fn contains(self, required: Self) -> bool {
        self.0 & required.0 == required.0
    }
}

pub const INIT_PROCESS_ID: u64 = 1;

const NAMESPACE_BITS: u32 = 9;
const NAMESPACE_GENERATION_BITS: u32 = 10;
const SLOT_BITS: u32 = 6;
const GENERATION_BITS: u32 = 6;
const SLOT_MASK: u32 = (1 << SLOT_BITS) - 1;
const GENERATION_MASK: u16 = (1 << GENERATION_BITS) - 1;
const NAMESPACE_GENERATION_MASK: u16 = (1 << NAMESPACE_GENERATION_BITS) - 1;
const NAMESPACE_MASK: u32 = (1 << NAMESPACE_BITS) - 1;

/// Opaque capability token returned to userspace. The raw value is not a
/// global file-table index: it names a slot and generation inside one VFS
/// capability namespace, so a stale or cross-namespace token is rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileHandle(u32);

impl FileHandle {
    pub const fn from_raw(raw: u64) -> Option<Self> {
        if raw == 0 || raw > u32::MAX as u64 {
            None
        } else {
            Some(Self(raw as u32))
        }
    }

    pub const fn raw(self) -> u64 {
        self.0 as u64
    }

    const fn encode(
        namespace: usize,
        namespace_generation: u16,
        slot: usize,
        generation: u16,
    ) -> Option<Self> {
        if namespace as u32 > NAMESPACE_MASK
            || namespace_generation > NAMESPACE_GENERATION_MASK
            || slot as u32 > SLOT_MASK
            || generation > GENERATION_MASK
        {
            return None;
        }
        let raw = 1
            + ((namespace as u32) << (NAMESPACE_GENERATION_BITS + SLOT_BITS + GENERATION_BITS))
            + ((namespace_generation as u32) << (SLOT_BITS + GENERATION_BITS))
            + ((slot as u32) << GENERATION_BITS)
            + generation as u32;
        Some(Self(raw))
    }

    const fn decode(self) -> (usize, u16, usize, u16) {
        let raw = self.0 - 1;
        (
            (raw >> (NAMESPACE_GENERATION_BITS + SLOT_BITS + GENERATION_BITS)) as usize,
            ((raw >> (SLOT_BITS + GENERATION_BITS)) & NAMESPACE_GENERATION_MASK as u32) as u16,
            ((raw >> GENERATION_BITS) & SLOT_MASK) as usize,
            (raw & GENERATION_MASK as u32) as u16,
        )
    }
}

struct FileNode {
    name: String,
    data: Vec<u8>,
}

struct OpenFile {
    name: String,
    rights: Rights,
    offset: usize,
}

struct CapabilityNamespace {
    owner: u64,
    generation: u16,
    handles: Vec<Option<OpenFile>>,
    generations: Vec<u16>,
}

pub struct Vfs {
    files: Vec<FileNode>,
    namespaces: Vec<Option<CapabilityNamespace>>,
    namespace_generations: Vec<u16>,
}

static GLOBAL_VFS: Mutex<Option<Vfs>> = Mutex::new(None);

impl Vfs {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            namespaces: Vec::new(),
            namespace_generations: Vec::new(),
        }
    }

    pub fn create(&mut self, name: &str, data: &[u8]) -> Result<(), FsError> {
        if name.is_empty() || name.contains('/') {
            return Err(FsError::InvalidName);
        }
        if self.files.iter().any(|file| file.name == name) {
            return Err(FsError::AlreadyExists);
        }
        self.files.push(FileNode {
            name: String::from(name),
            data: Vec::from(data),
        });
        Ok(())
    }

    pub fn read(&self, name: &str) -> Result<&[u8], FsError> {
        self.files
            .iter()
            .find(|file| file.name == name)
            .map(|file| file.data.as_slice())
            .ok_or(FsError::NotFound)
    }

    pub fn open(&mut self, name: &str, rights: Rights) -> Result<FileHandle, FsError> {
        self.open_owned(name, rights, 0)
    }

    fn namespace_slot(&mut self, owner: u64) -> Result<usize, FsError> {
        if let Some(slot) = self
            .namespaces
            .iter()
            .position(|namespace| namespace.as_ref().is_some_and(|ns| ns.owner == owner))
        {
            return Ok(slot);
        }
        let slot = self
            .namespaces
            .iter()
            .position(Option::is_none)
            .unwrap_or_else(|| {
                self.namespaces.push(None);
                self.namespaces.len() - 1
            });
        if slot as u32 > NAMESPACE_MASK {
            return Err(FsError::PermissionDenied);
        }
        if slot == self.namespace_generations.len() {
            self.namespace_generations.push(0);
        }
        let generation = self.namespace_generations[slot];
        self.namespaces[slot] = Some(CapabilityNamespace {
            owner,
            generation,
            handles: Vec::new(),
            generations: Vec::new(),
        });
        Ok(slot)
    }

    pub fn open_owned(
        &mut self,
        name: &str,
        rights: Rights,
        owner: u64,
    ) -> Result<FileHandle, FsError> {
        if !self.files.iter().any(|file| file.name == name) {
            return Err(FsError::NotFound);
        }
        let namespace = self.namespace_slot(owner)?;
        let ns = self.namespaces[namespace]
            .as_mut()
            .ok_or(FsError::PermissionDenied)?;
        let slot = ns
            .handles
            .iter()
            .position(Option::is_none)
            .unwrap_or_else(|| {
                ns.handles.push(None);
                ns.generations.push(0);
                ns.handles.len() - 1
            });
        let generation = ns.generations[slot];
        let namespace_generation = ns.generation;
        ns.handles[slot] = Some(OpenFile {
            name: String::from(name),
            rights,
            offset: 0,
        });
        FileHandle::encode(namespace, namespace_generation, slot, generation)
            .ok_or(FsError::PermissionDenied)
    }

    fn namespace_handle_mut(
        &mut self,
        handle: FileHandle,
        owner: u64,
    ) -> Result<&mut OpenFile, FsError> {
        let (namespace, namespace_generation, slot, generation) = handle.decode();
        let ns = self
            .namespaces
            .get_mut(namespace)
            .and_then(Option::as_mut)
            .ok_or(FsError::InvalidHandle)?;
        if ns.owner != owner {
            return Err(FsError::PermissionDenied);
        }
        if ns.generation != namespace_generation {
            return Err(FsError::InvalidHandle);
        }
        if ns.generations.get(slot).copied() != Some(generation) {
            return Err(FsError::InvalidHandle);
        }
        ns.handles
            .get_mut(slot)
            .and_then(Option::as_mut)
            .ok_or(FsError::InvalidHandle)
    }

    pub fn read_handle(&mut self, handle: FileHandle, output: &mut [u8]) -> Result<usize, FsError> {
        self.read_handle_owned(handle, output, 0)
    }

    pub fn read_handle_owned(
        &mut self,
        handle: FileHandle,
        output: &mut [u8],
        owner: u64,
    ) -> Result<usize, FsError> {
        let (name, offset, rights) = {
            let open = self.namespace_handle_mut(handle, owner)?;
            (open.name.clone(), open.offset, open.rights)
        };
        if !rights.contains(Rights::READ) {
            return Err(FsError::PermissionDenied);
        }
        let file = self
            .files
            .iter()
            .find(|file| file.name == name)
            .ok_or(FsError::NotFound)?;
        let available = file.data.len().saturating_sub(offset);
        let count = available.min(output.len());
        output[..count].copy_from_slice(&file.data[offset..offset + count]);
        let open = self.namespace_handle_mut(handle, owner)?;
        open.offset = offset + count;
        Ok(count)
    }

    pub fn write_handle(&mut self, handle: FileHandle, input: &[u8]) -> Result<usize, FsError> {
        self.write_handle_owned(handle, input, 0)
    }

    pub fn write_handle_owned(
        &mut self,
        handle: FileHandle,
        input: &[u8],
        owner: u64,
    ) -> Result<usize, FsError> {
        let (name, offset, rights) = {
            let open = self.namespace_handle_mut(handle, owner)?;
            (open.name.clone(), open.offset, open.rights)
        };
        if !rights.contains(Rights::WRITE) {
            return Err(FsError::PermissionDenied);
        }
        let end = offset.saturating_add(input.len());
        let file = self
            .files
            .iter_mut()
            .find(|file| file.name == name)
            .ok_or(FsError::NotFound)?;
        if end > file.data.len() {
            file.data.resize(end, 0);
        }
        file.data[offset..end].copy_from_slice(input);
        let open = self.namespace_handle_mut(handle, owner)?;
        open.offset = end;
        Ok(input.len())
    }

    pub fn close(&mut self, handle: FileHandle) -> Result<(), FsError> {
        self.close_owned(handle, 0)
    }

    pub fn close_owned(&mut self, handle: FileHandle, owner: u64) -> Result<(), FsError> {
        let (namespace, namespace_generation, slot, generation) = handle.decode();
        let ns = self
            .namespaces
            .get_mut(namespace)
            .and_then(Option::as_mut)
            .ok_or(FsError::InvalidHandle)?;
        if ns.owner != owner {
            return Err(FsError::PermissionDenied);
        }
        if ns.generation != namespace_generation {
            return Err(FsError::InvalidHandle);
        }
        if ns.generations.get(slot).copied() != Some(generation) {
            return Err(FsError::InvalidHandle);
        }
        let entry = ns.handles.get_mut(slot).ok_or(FsError::InvalidHandle)?;
        if entry.take().is_none() {
            return Err(FsError::InvalidHandle);
        }
        ns.generations[slot] = (generation + 1) & GENERATION_MASK;
        Ok(())
    }

    pub fn revoke_owner(&mut self, owner: u64) -> usize {
        let Some(namespace) = self
            .namespaces
            .iter()
            .position(|entry| entry.as_ref().is_some_and(|ns| ns.owner == owner))
        else {
            return 0;
        };
        let generation = self.namespace_generations[namespace];
        self.namespace_generations[namespace] = (generation + 1) & NAMESPACE_GENERATION_MASK;
        let Some(namespace) = self.namespaces[namespace].take() else {
            return 0;
        };
        namespace
            .handles
            .iter()
            .filter(|handle| handle.is_some())
            .count()
    }

    pub fn list(&self) -> impl Iterator<Item = &str> {
        self.files.iter().map(|file| file.name.as_str())
    }

    pub fn unlink(&mut self, name: &str) -> Result<(), FsError> {
        if self
            .namespaces
            .iter()
            .flatten()
            .flat_map(|namespace| namespace.handles.iter())
            .flatten()
            .any(|open| open.name == name)
        {
            return Err(FsError::PermissionDenied);
        }
        let index = self
            .files
            .iter()
            .position(|file| file.name == name)
            .ok_or(FsError::NotFound)?;
        self.files.remove(index);
        Ok(())
    }
}

pub fn init_global_ramdisk() -> bool {
    let mut fs = Vfs::new();
    if fs
        .create("init", include_bytes!("../../ramdisk/init.txt"))
        .is_err()
        || fs.create("init.elf", ramdisk_init_elf()).is_err()
    {
        return false;
    }
    *GLOBAL_VFS.lock() = Some(fs);
    true
}

pub fn global_open(name: &[u8], rights: Rights, owner: u64) -> Result<FileHandle, FsError> {
    let name = core::str::from_utf8(name).map_err(|_| FsError::InvalidName)?;
    GLOBAL_VFS
        .lock()
        .as_mut()
        .ok_or(FsError::NotFound)?
        .open_owned(name, rights, owner)
}

pub fn global_read(handle: FileHandle, output: &mut [u8], owner: u64) -> Result<usize, FsError> {
    GLOBAL_VFS
        .lock()
        .as_mut()
        .ok_or(FsError::NotFound)?
        .read_handle_owned(handle, output, owner)
}

pub fn global_write(handle: FileHandle, input: &[u8], owner: u64) -> Result<usize, FsError> {
    GLOBAL_VFS
        .lock()
        .as_mut()
        .ok_or(FsError::NotFound)?
        .write_handle_owned(handle, input, owner)
}

pub fn global_close(handle: FileHandle, owner: u64) -> Result<(), FsError> {
    GLOBAL_VFS
        .lock()
        .as_mut()
        .ok_or(FsError::NotFound)?
        .close_owned(handle, owner)
}

pub fn global_revoke_owner(owner: u64) -> usize {
    GLOBAL_VFS
        .lock()
        .as_mut()
        .map(|vfs| vfs.revoke_owner(owner))
        .unwrap_or(0)
}

pub fn ramdisk_init_elf() -> &'static [u8] {
    include_bytes!("../../ramdisk/init.elf")
}

pub fn ramdisk_exit_elf() -> &'static [u8] {
    include_bytes!("../../ramdisk/exit.elf")
}

pub fn ramdisk_smoke_test() -> bool {
    let init_data = include_bytes!("../../ramdisk/init.txt");
    let init_elf = ramdisk_init_elf();
    let mut fs = Vfs::new();
    fs.create("init", init_data).is_ok()
        && fs.create("init.elf", init_elf).is_ok()
        && fs
            .read("init")
            .map(|data| data.starts_with(b"NOVEL_OS INIT"))
            .unwrap_or(false)
        && fs
            .read("init.elf")
            .map(|data| data.starts_with(b"\x7fELF"))
            .unwrap_or(false)
        && fs.create("exit.elf", ramdisk_exit_elf()).is_ok()
}

pub fn vfs_handle_smoke_test() -> bool {
    let mut fs = Vfs::new();
    if fs.create("init", b"NOVEL INIT\n").is_err() {
        return false;
    }
    let handle = match fs.open("init", Rights::READ) {
        Ok(handle) => handle,
        Err(_) => return false,
    };
    let mut first = [0_u8; 5];
    let mut second = [0_u8; 5];
    let read_first = fs.read_handle(handle, &mut first) == Ok(5) && &first == b"NOVEL";
    let read_second = fs.read_handle(handle, &mut second) == Ok(5) && &second == b" INIT";
    let read_only = match fs.open("init", Rights::READ) {
        Ok(handle) => handle,
        Err(_) => return false,
    };
    let denied_write = fs.write_handle(read_only, b"X") == Err(FsError::PermissionDenied);
    let _ = fs.close(read_only);
    let closed = fs.close(handle).is_ok();
    let stale_rejected = fs.read_handle(handle, &mut first) == Err(FsError::InvalidHandle);
    let owned = match fs.open_owned("init", Rights::READ, 7) {
        Ok(handle) => handle,
        Err(_) => return false,
    };
    let mut capability_buffer = [0_u8; 1];
    let wrong_owner =
        fs.read_handle_owned(owned, &mut capability_buffer, 8) == Err(FsError::PermissionDenied);
    let owner_read = fs.read_handle_owned(owned, &mut capability_buffer, 7) == Ok(1);
    let revoked = fs.revoke_owner(7) == 1
        && fs.read_handle_owned(owned, &mut capability_buffer, 7) == Err(FsError::InvalidHandle);
    let reopened = match fs.open_owned("init", Rights::READ, 7) {
        Ok(handle) => handle,
        Err(_) => return false,
    };
    let namespace_generation = fs.read_handle_owned(owned, &mut capability_buffer, 7)
        == Err(FsError::InvalidHandle)
        && fs.read_handle_owned(reopened, &mut capability_buffer, 7) == Ok(1);
    read_first
        && read_second
        && denied_write
        && closed
        && stale_rejected
        && wrong_owner
        && owner_read
        && revoked
        && namespace_generation
}

pub fn vfs_smoke_test() -> bool {
    let mut fs = Vfs::new();
    if fs.create("init", b"NOVEL INIT\n").is_err() {
        return false;
    }
    let read_ok = fs.read("init") == Ok(b"NOVEL INIT\n".as_slice());
    let listed = fs.list().count() == 1;
    let removed = fs.unlink("init").is_ok() && fs.read("init") == Err(FsError::NotFound);
    read_ok && listed && removed && vfs_handle_smoke_test()
}
