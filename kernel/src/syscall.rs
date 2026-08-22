#[repr(u64)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Number {
    Write = 1,
    Yield = 2,
    Open = 3,
    Read = 4,
    FileWrite = 5,
    Exit = 6,
    Close = 7,
    PortCreate = 8,
    PortSend = 9,
    PortRecv = 10,
}

#[repr(u64)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidArgument = u64::MAX - 1,
    InvalidHandle = u64::MAX - 2,
    PermissionDenied = u64::MAX - 3,
    NotFound = u64::MAX - 4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Return {
    Value(u64),
    Exited(u64),
}

pub struct SyscallTable;

impl SyscallTable {
    pub const fn new() -> Self {
        Self
    }

    pub fn dispatch(&self, number: u64, _args: [u64; 3]) -> Result<Return, Error> {
        match number {
            n if n == Number::Yield as u64 => Ok(Return::Value(0)),
            _ => Err(Error::InvalidArgument),
        }
    }
}

pub fn syscall_smoke_test() -> bool {
    let table = SyscallTable::new();
    table.dispatch(Number::Yield as u64, [0; 3]) == Ok(Return::Value(0))
        && table.dispatch(0xFFFF, [0; 3]) == Err(Error::InvalidArgument)
}
