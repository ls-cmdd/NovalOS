#!/usr/bin/env python3
import struct
from pathlib import Path

BASE = 0x400000
CODE_OFFSET = 0x100
MESSAGE = b"exit from elf\n"
code = bytearray()
code += b"\x48\xc7\xc0\x01\x00\x00\x00"       # SYS_WRITE
code += b"\x48\xbf" + struct.pack("<Q", BASE + CODE_OFFSET + 64)
code += b"\x48\xc7\xc6" + struct.pack("<I", len(MESSAGE))
code += b"\xcd\x80"
code += b"\x48\xc7\xc0\x06\x00\x00\x00"       # SYS_EXIT
code += b"\x48\x31\xff"                           # status = 0
code += b"\xcd\x80"
code += b"\xeb\xfe"

payload = bytearray(CODE_OFFSET + 64 + len(MESSAGE))
payload[:len(code)] = code
payload[CODE_OFFSET + 64:CODE_OFFSET + 64 + len(MESSAGE)] = MESSAGE

ehdr = struct.pack("<16sHHIQQQIHHHHHH", b"\x7fELF" + bytes([2, 1, 1, 0]) + bytes(8), 2, 0x3E, 1, BASE, 64, 0, 0, 64, 56, 1, 0, 0, 0)
phdr = struct.pack("<IIQQQQQQ", 1, 5, 120, BASE, 0, len(payload), 0x1000, 0x1000)
Path("ramdisk").mkdir(exist_ok=True)
Path("ramdisk/exit.elf").write_bytes(ehdr + phdr + payload)
print(f"wrote ramdisk/exit.elf: {len(ehdr) + len(phdr) + len(payload)} bytes")
