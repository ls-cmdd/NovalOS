#!/usr/bin/env python3
import struct
from pathlib import Path

BASE = 0x400000
CODE_OFFSET = 0x100
MESSAGE = b"init from elf\n"
FILE_NAME = b"init"
IPC_MESSAGE = b"ipc"
STACK_BUFFER = 0x7FF100

# x86_64 machine code using the documented Novel OS int80 ABI:
# rax=number, rdi=pointer/arg1, rsi=length/arg2, rdx=extra/arg3.
code = bytearray()
code += b"\x48\xc7\xc0\x01\x00\x00\x00"       # mov rax, SYS_WRITE
code += b"\x48\xbf" + struct.pack("<Q", BASE + CODE_OFFSET + 64) # mov rdi, message
code += b"\x48\xc7\xc6" + struct.pack("<I", len(MESSAGE))            # mov rsi, length
code += b"\xcd\x80"                                  # int 0x80
# Open/read/write/close the initramfs file through the same user ABI.
code += b"\x48\xc7\xc0\x03\x00\x00\x00"       # SYS_OPEN
code += b"\x48\xbf" + struct.pack("<Q", BASE + CODE_OFFSET + 80)
code += b"\x48\xc7\xc6" + struct.pack("<I", len(FILE_NAME))
code += b"\x48\xc7\xc2\x01\x00\x00\x00"       # Rights::READ
code += b"\xcd\x80"
code += b"\x48\x89\xc3"                           # save handle in rbx
code += b"\x48\xc7\xc0\x04\x00\x00\x00"       # SYS_READ
code += b"\x48\xbf" + struct.pack("<Q", STACK_BUFFER)
code += b"\x48\xc7\xc6" + struct.pack("<I", len(MESSAGE))
code += b"\x48\x89\xda"                           # handle in rdx
code += b"\xcd\x80"
code += b"\x48\xc7\xc0\x01\x00\x00\x00"       # SYS_WRITE
code += b"\x48\xbf" + struct.pack("<Q", STACK_BUFFER)
code += b"\x48\xc7\xc6" + struct.pack("<I", len(MESSAGE))
code += b"\xcd\x80"
code += b"\x48\xc7\xc0\x07\x00\x00\x00"       # SYS_CLOSE
code += b"\x48\x89\xdf"                           # handle in rdi
code += b"\xcd\x80"
# Create/send/receive a bounded IPC message through opaque port capabilities.
code += b"\x48\xc7\xc0\x08\x00\x00\x00"       # SYS_PORT_CREATE
code += b"\xcd\x80"
code += b"\x48\x89\xc3"                           # save port handle in rbx
code += b"\x48\xc7\xc0\x09\x00\x00\x00"       # SYS_PORT_SEND
code += b"\x48\xbf" + struct.pack("<Q", BASE + CODE_OFFSET + 88)
code += b"\x48\xc7\xc6" + struct.pack("<I", len(IPC_MESSAGE))
code += b"\x48\x89\xda"                           # port handle in rdx
code += b"\xcd\x80"
code += b"\x48\xc7\xc0\x0a\x00\x00\x00"       # SYS_PORT_RECV
code += b"\x48\xbf" + struct.pack("<Q", STACK_BUFFER)
code += b"\x48\xc7\xc6" + struct.pack("<I", len(IPC_MESSAGE))
code += b"\x48\x89\xda"                           # port handle in rdx
code += b"\xcd\x80"
code += b"\x48\x31\xc0"                              # xor rax, rax
loop_start = len(code)
code += b"\x48\xff\xc0"                              # inc rax
code += b"\x48\x3d\x00\x00\x00\x02"                  # cmp rax, 0x02000000
branch_disp = loop_start - (len(code) + 2)
code += b"\x75" + struct.pack("<b", branch_disp)       # jne loop body
# Force a real user-mode page fault at the unmapped guard page.
code += b"\x48\xa1" + struct.pack("<Q", 0x7FE000)    # mov rax, [guard]
code += b"\xeb\xfe"                                  # unreachable safety loop

payload = bytearray(CODE_OFFSET + 88 + len(IPC_MESSAGE))
payload[:len(code)] = code
payload[CODE_OFFSET + 64:CODE_OFFSET + 64 + len(MESSAGE)] = MESSAGE
payload[CODE_OFFSET + 80:CODE_OFFSET + 80 + len(FILE_NAME)] = FILE_NAME
payload[CODE_OFFSET + 88:CODE_OFFSET + 88 + len(IPC_MESSAGE)] = IPC_MESSAGE

# ELF64 header and one executable/readable PT_LOAD segment.
ehdr = struct.pack(
    "<16sHHIQQQIHHHHHH",
    b"\x7fELF" + bytes([2, 1, 1, 0]) + bytes(8),
    2, 0x3E, 1, BASE, 64, 0, 0, 64, 56, 1, 0, 0, 0,
)
phdr = struct.pack(
    "<IIQQQQQQ",
    1, 5, 120, BASE, 0, len(payload), 0x1000, 0x1000,
)
Path("ramdisk").mkdir(exist_ok=True)
Path("ramdisk/init.elf").write_bytes(ehdr + phdr + payload)
print(f"wrote ramdisk/init.elf: {len(ehdr) + len(phdr) + len(payload)} bytes")
