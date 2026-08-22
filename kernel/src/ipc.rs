use spin::Mutex;

const PORT_CAPACITY: usize = 8;
const MESSAGE_CAPACITY: usize = 64;
const QUEUE_CAPACITY: usize = 4;
const SLOT_BITS: u32 = 3;
const GENERATION_BITS: u32 = 8;
const GENERATION_MASK: u16 = (1 << GENERATION_BITS) - 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IpcError {
    InvalidHandle,
    PermissionDenied,
    QueueFull,
    QueueEmpty,
    InvalidMessage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PortHandle(u32);

impl PortHandle {
    fn encode(slot: usize, generation: u16) -> Self {
        Self(1 + ((slot as u32) << GENERATION_BITS) + generation as u32)
    }

    fn decode(self) -> Option<(usize, u16)> {
        if self.0 == 0 {
            return None;
        }
        let raw = self.0 - 1;
        Some((
            (raw >> GENERATION_BITS) as usize,
            (raw & GENERATION_MASK as u32) as u16,
        ))
    }
}

#[derive(Clone, Copy)]
struct Message {
    bytes: [u8; MESSAGE_CAPACITY],
    len: usize,
}

#[derive(Clone, Copy)]
struct Port {
    owner: u64,
    generation: u16,
    queue: [Option<Message>; QUEUE_CAPACITY],
    head: usize,
    tail: usize,
    len: usize,
}

impl Port {
    const fn empty() -> Self {
        Self {
            owner: 0,
            generation: 0,
            queue: [None; QUEUE_CAPACITY],
            head: 0,
            tail: 0,
            len: 0,
        }
    }
}

struct IpcTable {
    ports: [Option<Port>; PORT_CAPACITY],
    generations: [u16; PORT_CAPACITY],
}

impl IpcTable {
    const fn new() -> Self {
        Self {
            ports: [None; PORT_CAPACITY],
            generations: [0; PORT_CAPACITY],
        }
    }

    fn create(&mut self, owner: u64) -> Result<u64, IpcError> {
        let Some(slot) = self.ports.iter().position(Option::is_none) else {
            return Err(IpcError::QueueFull);
        };
        let generation = self.generations[slot];
        self.ports[slot] = Some(Port {
            owner,
            generation,
            ..Port::empty()
        });
        Ok(PortHandle::encode(slot, generation).0 as u64)
    }

    fn port_mut(&mut self, raw: u64, owner: u64) -> Result<&mut Port, IpcError> {
        if raw > u32::MAX as u64 {
            return Err(IpcError::InvalidHandle);
        }
        let handle = PortHandle(raw as u32);
        let (slot, generation) = handle.decode().ok_or(IpcError::InvalidHandle)?;
        let port = self
            .ports
            .get_mut(slot)
            .and_then(Option::as_mut)
            .ok_or(IpcError::InvalidHandle)?;
        if port.generation != generation {
            return Err(IpcError::InvalidHandle);
        }
        if port.owner != owner {
            return Err(IpcError::PermissionDenied);
        }
        Ok(port)
    }

    fn send(&mut self, raw: u64, owner: u64, input: &[u8]) -> Result<usize, IpcError> {
        if input.len() > MESSAGE_CAPACITY {
            return Err(IpcError::InvalidMessage);
        }
        let port = self.port_mut(raw, owner)?;
        if port.len == QUEUE_CAPACITY {
            return Err(IpcError::QueueFull);
        }
        let mut message = Message {
            bytes: [0; MESSAGE_CAPACITY],
            len: input.len(),
        };
        message.bytes[..input.len()].copy_from_slice(input);
        port.queue[port.tail] = Some(message);
        port.tail = (port.tail + 1) % QUEUE_CAPACITY;
        port.len += 1;
        Ok(input.len())
    }

    fn recv(&mut self, raw: u64, owner: u64, output: &mut [u8]) -> Result<usize, IpcError> {
        let port = self.port_mut(raw, owner)?;
        let message = port.queue[port.head].take().ok_or(IpcError::QueueEmpty)?;
        if output.len() < message.len {
            port.queue[port.head] = Some(message);
            return Err(IpcError::InvalidMessage);
        }
        output[..message.len].copy_from_slice(&message.bytes[..message.len]);
        port.head = (port.head + 1) % QUEUE_CAPACITY;
        port.len -= 1;
        Ok(message.len)
    }

    fn revoke_owner(&mut self, owner: u64) -> usize {
        let mut revoked = 0;
        for (slot, port) in self.ports.iter_mut().enumerate() {
            if port.as_ref().is_some_and(|port| port.owner == owner) {
                *port = None;
                self.generations[slot] = (self.generations[slot] + 1) & GENERATION_MASK;
                revoked += 1;
            }
        }
        revoked
    }
}

static IPC: Mutex<IpcTable> = Mutex::new(IpcTable::new());

pub fn create(owner: u64) -> Result<u64, IpcError> {
    IPC.lock().create(owner)
}

pub fn send(handle: u64, owner: u64, input: &[u8]) -> Result<usize, IpcError> {
    IPC.lock().send(handle, owner, input)
}

pub fn recv(handle: u64, owner: u64, output: &mut [u8]) -> Result<usize, IpcError> {
    IPC.lock().recv(handle, owner, output)
}

pub fn revoke_owner(owner: u64) -> usize {
    IPC.lock().revoke_owner(owner)
}

pub fn smoke_test() -> bool {
    let owner = 0xA5;
    let handle = match create(owner) {
        Ok(handle) => handle,
        Err(_) => return false,
    };
    let message = b"ipc";
    let mut output = [0_u8; MESSAGE_CAPACITY];
    send(handle, owner, message) == Ok(message.len())
        && recv(handle, owner, &mut output) == Ok(message.len())
        && &output[..message.len()] == message
        && send(handle, owner + 1, message) == Err(IpcError::PermissionDenied)
        && revoke_owner(owner) == 1
        && send(handle, owner, message) == Err(IpcError::InvalidHandle)
}
