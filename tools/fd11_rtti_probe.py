#!/usr/bin/env python3
"""Locate Itanium C++ RTTI objects and likely vtables in a 64-bit Mach-O.

This is intentionally dependency-free.  Final Draft 11's stripped executable
still contains RTTI names, and the pointers around those names provide useful
anchors for disassembling otherwise anonymous C++ methods.
"""

from __future__ import annotations

import argparse
import re
import struct
from dataclasses import dataclass
from pathlib import Path


LC_SEGMENT_64 = 0x19
MH_MAGIC_64 = 0xFEEDFACF


@dataclass(frozen=True)
class Section:
    segment: str
    name: str
    address: int
    size: int
    offset: int

    def contains_address(self, address: int) -> bool:
        return self.address <= address < self.address + self.size

    def contains_offset(self, offset: int) -> bool:
        return self.offset <= offset < self.offset + self.size


def fixed_string(value: bytes) -> str:
    return value.split(b"\0", 1)[0].decode("ascii", errors="replace")


def parse_sections(data: bytes) -> list[Section]:
    if len(data) < 32:
        raise ValueError("file is too small to be a 64-bit Mach-O")
    magic, _, _, _, command_count, _, _, _ = struct.unpack_from("<8I", data)
    if magic != MH_MAGIC_64:
        raise ValueError("only little-endian 64-bit Mach-O files are supported")

    sections: list[Section] = []
    command_offset = 32
    for _ in range(command_count):
        command, command_size = struct.unpack_from("<2I", data, command_offset)
        if command == LC_SEGMENT_64:
            (
                _,
                _,
                segment_name,
                _,
                _,
                _,
                _,
                _,
                _,
                section_count,
                _,
            ) = struct.unpack_from("<2I16s4Q4I", data, command_offset)
            section_offset = command_offset + 72
            for index in range(section_count):
                values = struct.unpack_from(
                    "<16s16s2Q7I3I", data, section_offset + index * 80
                )
                section_name, owner, address, size, offset = values[:5]
                sections.append(
                    Section(
                        fixed_string(owner),
                        fixed_string(section_name),
                        address,
                        size,
                        offset,
                    )
                )
        command_offset += command_size
    return sections


def find_all(data: bytes, needle: bytes) -> list[int]:
    hits: list[int] = []
    start = 0
    while True:
        hit = data.find(needle, start)
        if hit < 0:
            return hits
        hits.append(hit)
        start = hit + 1


def section_for_offset(sections: list[Section], offset: int) -> Section | None:
    return next((section for section in sections if section.contains_offset(offset)), None)


def section_for_address(sections: list[Section], address: int) -> Section | None:
    return next((section for section in sections if section.contains_address(address)), None)


def address_for_offset(sections: list[Section], offset: int) -> int:
    section = section_for_offset(sections, offset)
    if section is None:
        raise ValueError(f"file offset 0x{offset:x} is not in a section")
    return section.address + offset - section.offset


def describe_address(sections: list[Section], address: int) -> str:
    section = section_for_address(sections, address)
    if section is None:
        return f"0x{address:x}"
    return f"0x{address:x} ({section.segment},{section.name}+0x{address - section.address:x})"


def cstrings(data: bytes, section: Section) -> list[tuple[int, str]]:
    result: list[tuple[int, str]] = []
    blob = data[section.offset : section.offset + section.size]
    start = 0
    for match in re.finditer(b"\0", blob):
        raw = blob[start : match.start()]
        if raw:
            result.append(
                (section.offset + start, raw.decode("utf-8", errors="replace"))
            )
        start = match.end()
    return result


def u64(data: bytes, offset: int) -> int:
    return struct.unpack_from("<Q", data, offset)[0]


def likely_vtable_functions(
    data: bytes, sections: list[Section], typeinfo_address: int
) -> list[tuple[int, list[int]]]:
    result: list[tuple[int, list[int]]] = []
    for reference in find_all(data, struct.pack("<Q", typeinfo_address)):
        if reference < 8 or u64(data, reference - 8) != 0:
            continue
        functions: list[int] = []
        cursor = reference + 8
        while cursor + 8 <= len(data) and len(functions) < 256:
            candidate = u64(data, cursor)
            section = section_for_address(sections, candidate)
            if section is None or (section.segment, section.name) != ("__TEXT", "__text"):
                break
            functions.append(candidate)
            cursor += 8
        if functions:
            result.append((address_for_offset(sections, reference + 8), functions))
    return result


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("binary", type=Path)
    parser.add_argument(
        "--exact", action="store_true", help="match complete decoded RTTI names"
    )
    parser.add_argument(
        "--max-slots", type=int, default=256, help="maximum vtable slots to print"
    )
    parser.add_argument(
        "patterns",
        nargs="+",
        help="case-sensitive substrings in decoded Itanium RTTI names",
    )
    args = parser.parse_args()

    data = args.binary.read_bytes()
    sections = parse_sections(data)
    string_sections = [
        section
        for section in sections
        if section.segment == "__TEXT"
        and section.name in {"__cstring", "__const", "__objc_classname"}
    ]
    strings = [item for section in string_sections for item in cstrings(data, section)]

    matches = [
        (offset, value)
        for offset, value in strings
        if any(
            (re.sub(r"^\d+", "", value) == pattern if args.exact else pattern in value)
            for pattern in args.patterns
        )
        and re.fullmatch(r"\d+[A-Za-z_][A-Za-z0-9_]*", value)
    ]

    for string_offset, encoded_name in matches:
        name = re.sub(r"^\d+", "", encoded_name)
        string_address = address_for_offset(sections, string_offset)
        print(f"{name}: RTTI name {describe_address(sections, string_address)}")
        name_references = find_all(data, struct.pack("<Q", string_address))
        if not name_references:
            print("  no pointer to RTTI name found")
            continue
        for name_reference in name_references:
            typeinfo_offset = name_reference - 8
            typeinfo_address = address_for_offset(sections, typeinfo_offset)
            print(f"  typeinfo {describe_address(sections, typeinfo_address)}")
            direct_references = find_all(data, struct.pack("<Q", typeinfo_address))
            print(
                "  typeinfo refs "
                + ", ".join(
                    describe_address(sections, address_for_offset(sections, offset))
                    for offset in direct_references
                    if section_for_offset(sections, offset) is not None
                )
            )
            for address_point, functions in likely_vtable_functions(
                data, sections, typeinfo_address
            ):
                print(
                    f"  vtable address point {describe_address(sections, address_point)}"
                )
                for index, function in enumerate(functions[: args.max_slots]):
                    print(f"    [{index:02d}] {describe_address(sections, function)}")


if __name__ == "__main__":
    main()
