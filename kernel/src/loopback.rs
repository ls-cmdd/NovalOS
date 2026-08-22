const QUEUE_CAPACITY: usize = 4;
const MTU: usize = 128;

#[derive(Clone, Copy)]
struct Packet {
    bytes: [u8; MTU],
    len: usize,
    checksum: u16,
}

impl Packet {
    fn new(payload: &[u8]) -> Option<Self> {
        if payload.len() > MTU {
            return None;
        }
        let mut bytes = [0; MTU];
        bytes[..payload.len()].copy_from_slice(payload);
        Some(Self {
            bytes,
            len: payload.len(),
            checksum: checksum(payload),
        })
    }

    fn valid(&self) -> bool {
        checksum(&self.bytes[..self.len]) == self.checksum
    }
}

struct LoopbackDevice {
    queue: [Option<Packet>; QUEUE_CAPACITY],
    head: usize,
    tail: usize,
    len: usize,
}

impl LoopbackDevice {
    const fn new() -> Self {
        Self {
            queue: [None; QUEUE_CAPACITY],
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    fn send(&mut self, packet: Packet) -> bool {
        if self.len == QUEUE_CAPACITY {
            return false;
        }
        self.queue[self.tail] = Some(packet);
        self.tail = (self.tail + 1) % QUEUE_CAPACITY;
        self.len += 1;
        true
    }

    fn recv(&mut self) -> Option<Packet> {
        let packet = self.queue[self.head].take()?;
        self.head = (self.head + 1) % QUEUE_CAPACITY;
        self.len -= 1;
        Some(packet)
    }
}

fn checksum(bytes: &[u8]) -> u16 {
    let mut sum = 0_u32;
    for chunk in bytes.chunks(2) {
        let word = chunk[0] as u32 | ((chunk.get(1).copied().unwrap_or(0) as u32) << 8);
        sum = (sum & 0xFFFF) + (sum >> 16) + word;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

pub fn loopback_smoke_test() -> bool {
    let mut device = LoopbackDevice::new();
    let packet = Packet::new(b"loopback").unwrap();
    let sent = device.send(packet);
    let received = device.recv();
    sent && received
        .is_some_and(|packet| packet.valid() && &packet.bytes[..packet.len] == b"loopback")
}

pub fn loopback_datagram_smoke_test() -> bool {
    let mut device = LoopbackDevice::new();
    let payload = [0x03, 0xE8, 0x07, 0xD0, 0, 4, b'p', b'i', b'n', b'g'];
    let packet = Packet::new(&payload).unwrap();
    let roundtrip = device.send(packet)
        && device
            .recv()
            .is_some_and(|p| p.valid() && p.len == payload.len());
    let mut corrupt = Packet::new(&payload).unwrap();
    corrupt.bytes[0] ^= 1;
    let corruption_rejected = !corrupt.valid();
    let mut filled = true;
    for _ in 0..QUEUE_CAPACITY {
        filled &= device.send(Packet::new(b"queued").unwrap());
    }
    let backpressure = !device.send(Packet::new(b"overflow").unwrap());
    roundtrip && corruption_rejected && filled && backpressure
}
