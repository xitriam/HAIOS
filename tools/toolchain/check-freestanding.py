#!/usr/bin/env python3
"""Compile a library object, never an OS entry point; reject hosted std."""
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import tempfile

rustc = str(Path.home() / '.cargo/bin/rustc')
with tempfile.TemporaryDirectory(prefix='haios-toolchain-') as directory:
    root = Path(directory)
    source = root / 'probe.rs'
    source.write_text('#![no_std]\n#[unsafe(no_mangle)]\npub extern "C" fn haios_toolchain_probe(a: u64, b: u64) -> u64 { a.wrapping_add(b) }\n')
    obj = root / 'probe.o'
    command = [rustc, '+1.99.0', '--edition=2024', '--crate-type=lib', '--target',
               'x86_64-unknown-none', '-C', 'panic=abort', '--emit=obj', str(source), '-o', str(obj)]
    subprocess.run(command, check=True, capture_output=True)
    data = obj.read_bytes()
    assert data[:6] == b'\x7fELF\x02\x01', 'Expected little-endian ELF64'
    kind, machine = struct.unpack_from('<HH', data, 16)
    assert (kind, machine) == (1, 62), 'Expected relocatable x86-64 object'
    source.write_text('pub fn hosted() { std::println!("hosted"); }\n')
    negative = subprocess.run(command, capture_output=True, text=True)
    assert negative.returncode != 0 and 'std' in negative.stderr and 'E0463' in negative.stderr
    report = {'host': subprocess.check_output(['hostname'], text=True).strip(),
              'rustc': subprocess.check_output([rustc, '+1.99.0', '-Vv'], text=True),
              'target': 'x86_64-unknown-none', 'positive_exit': 0, 'elf_class': 64,
              'elf_type': 'ET_REL', 'elf_machine': 62, 'object_sha256': hashlib.sha256(data).hexdigest(),
              'std_rejected': True, 'negative_exit': negative.returncode,
              'kernel_entry_implemented': False, 'linked_image_tested': False}
    print(json.dumps(report, indent=2))
