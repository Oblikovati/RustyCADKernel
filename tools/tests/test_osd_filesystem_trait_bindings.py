"""Validate exact OSD file-system override paths against captured inventories."""

import gzip
import json
import unittest
from pathlib import Path

from api_crosswalk import (
    cpp_candidates,
    cpp_indexes,
    header_owners,
    match_api_row,
    rust_candidates,
    rust_crates,
)
from api_inventory.inputs import JSONValue

ROOT = Path(__file__).resolve().parents[2]
BINDINGS: tuple[tuple[str, str, str], ...] = (
    (
        "OSD_LocalFileSystem::IsSupportedPath(const TCollection_AsciiString&) const",
        "c4d69a26a191a9ab405a8a77ab69bd8ad1d763bc1d9d63f1b76c6ddb246b6483",
        "rustycad_tkernel::osd::LocalFileSystem::<rustycad_tkernel::osd::FileSystem>::is_supported_path",
    ),
    (
        "OSD_LocalFileSystem::IsOpenIStream(const std::shared_ptr<std::istream>&) const",
        "2bf2866f246423205425db396599708fb68835274ea95925a536ca2f9dc61ed1",
        "rustycad_tkernel::osd::LocalFileSystem::<rustycad_tkernel::osd::FileSystem>::is_open_istream",
    ),
    (
        "OSD_LocalFileSystem::IsOpenOStream(const std::shared_ptr<std::ostream>&) const",
        "1ac9d132a957f6e6882916fd1bdc571fee8d714f76258691997fa629f527c03a",
        "rustycad_tkernel::osd::LocalFileSystem::<rustycad_tkernel::osd::FileSystem>::is_open_ostream",
    ),
    (
        "OSD_LocalFileSystem::OpenStreamBuffer(const TCollection_AsciiString&,std::ios_base::openmode,int64_t,int64_t*)",
        "039d07133eb9e8fb41ba702c07487da0309b65f1165c531152a03322b3210d13",
        "rustycad_tkernel::osd::LocalFileSystem::<rustycad_tkernel::osd::FileSystem>::open_stream_buffer",
    ),
    (
        "OSD_CachedFileSystem::IsSupportedPath(const TCollection_AsciiString&) const",
        "9874ebef7771698cc812557a1e883c7f1fde45c4648ff31b8b6943e6a79afb7e",
        "rustycad_tkernel::osd::CachedFileSystem::<rustycad_tkernel::osd::FileSystem>::is_supported_path",
    ),
    (
        "OSD_CachedFileSystem::IsOpenIStream(const std::shared_ptr<std::istream>&) const",
        "5d542852929850b1e7e7baeb17038aefe3226316cc2b1342fd27986d4f2228c4",
        "rustycad_tkernel::osd::CachedFileSystem::<rustycad_tkernel::osd::FileSystem>::is_open_istream",
    ),
    (
        "OSD_CachedFileSystem::IsOpenOStream(const std::shared_ptr<std::ostream>&) const",
        "49f57d3c6af22e58a0032b33f3a87541e32cfcdca03898ea5ce19a6f40279640",
        "rustycad_tkernel::osd::CachedFileSystem::<rustycad_tkernel::osd::FileSystem>::is_open_ostream",
    ),
    (
        "OSD_CachedFileSystem::OpenIStream(const TCollection_AsciiString&,std::ios_base::openmode,int64_t,const std::shared_ptr<std::istream>&)",
        "3f50e7b8a001c5510ba70045a5329e4bc0c0a1c761b1f2abcd83029f248572d9",
        "rustycad_tkernel::osd::CachedFileSystem::<rustycad_tkernel::osd::FileSystem>::open_istream",
    ),
    (
        "OSD_CachedFileSystem::OpenOStream(const TCollection_AsciiString&,std::ios_base::openmode)",
        "671ce465696e034ac2913bed1a29ed9047ed87b1b47ed9baab650b8797c2195f",
        "rustycad_tkernel::osd::CachedFileSystem::<rustycad_tkernel::osd::FileSystem>::open_ostream",
    ),
    (
        "OSD_CachedFileSystem::OpenStreamBuffer(const TCollection_AsciiString&,std::ios_base::openmode,int64_t,int64_t*)",
        "2c2b6b74c99432faf450754ecfbf1ea50949cc02e49c1520d044cd105db02c8d",
        "rustycad_tkernel::osd::CachedFileSystem::<rustycad_tkernel::osd::FileSystem>::open_stream_buffer",
    ),
    (
        "OSD_FileSystemSelector::IsSupportedPath(const TCollection_AsciiString&) const",
        "6acd9012519521c938fed74f751c9d42b08b3493718836f310045d9faff6d039",
        "rustycad_tkernel::osd::FileSystemSelector::<rustycad_tkernel::osd::FileSystem>::is_supported_path",
    ),
    (
        "OSD_FileSystemSelector::IsOpenIStream(const std::shared_ptr<std::istream>&) const",
        "0b4f040eff46bb4ba90e23a77ef9bf62b045ca375a7bc6ff2287ef7b675e1377",
        "rustycad_tkernel::osd::FileSystemSelector::<rustycad_tkernel::osd::FileSystem>::is_open_istream",
    ),
    (
        "OSD_FileSystemSelector::IsOpenOStream(const std::shared_ptr<std::ostream>&) const",
        "66c3d43fe6451eae8bb2833445474dc18a3c4642fc5c4275ace6002b87b55536",
        "rustycad_tkernel::osd::FileSystemSelector::<rustycad_tkernel::osd::FileSystem>::is_open_ostream",
    ),
    (
        "OSD_FileSystemSelector::OpenIStream(const TCollection_AsciiString&,std::ios_base::openmode,int64_t,const std::shared_ptr<std::istream>&)",
        "7c7b11c95c2fe411ab1f9d8ca164c2ec47b6ef4477c489dde77b092aeb19cb2c",
        "rustycad_tkernel::osd::FileSystemSelector::<rustycad_tkernel::osd::FileSystem>::open_istream",
    ),
    (
        "OSD_FileSystemSelector::OpenOStream(const TCollection_AsciiString&,std::ios_base::openmode)",
        "c6f03322319d3e5cb1de2161f81018f8412eb06952e0f28c5bca1571d776cccc",
        "rustycad_tkernel::osd::FileSystemSelector::<rustycad_tkernel::osd::FileSystem>::open_ostream",
    ),
    (
        "OSD_FileSystemSelector::OpenStreamBuffer(const TCollection_AsciiString&,std::ios_base::openmode,int64_t,int64_t*)",
        "deeccef2083b3f741056ef9c87f851303e0b5f48762ae13cef402f1fe5637201",
        "rustycad_tkernel::osd::FileSystemSelector::<rustycad_tkernel::osd::FileSystem>::open_stream_buffer",
    ),
)


class OSDFileSystemTraitBindingTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.declarations = read_lines(
            "porting/api-inventory/occt-declarations.jsonl.gz"
        )
        cls.rust_inventory = read_json_gzip("porting/api-inventory/rust-api.json.gz")
        cls.parity = json.loads((ROOT / "porting/parity.json").read_text())
        cls.manifest = json.loads(
            (ROOT / "porting/api-header-manifest.json").read_text()
        )
        cls.by_name, _, _ = cpp_indexes(cls.declarations, header_owners(cls.manifest))
        cls.rust = rust_crates(cls.rust_inventory)

    def test_overrides_resolve_uniquely_to_trait_impls(self) -> None:
        rows: dict[str, dict[str, JSONValue]] = {
            row["upstream_id"]: row for row in self.parity["api"]
        }
        known_sources = {header["path"] for header in self.manifest["headers"]}
        self.assertEqual(len(BINDINGS), 16)
        for selector, cpp_id, rust_path in BINDINGS:
            self.assert_binding(selector, cpp_id, rust_path, rows, known_sources)

    def assert_binding(
        self,
        selector: str,
        cpp_id: str,
        rust_path: str,
        rows: dict[str, dict[str, JSONValue]],
        known_sources: set[str],
    ) -> None:
        cpp_status, cpp = cpp_candidates(selector, "TKernel", self.by_name)
        rust_status, rust = rust_candidates(rust_path, self.rust)
        self.assertEqual(
            (cpp_status, [item["id"] for item in cpp]), ("unique", [cpp_id])
        )
        self.assertEqual(
            (rust_status, [item["path"] for item in rust]), ("unique", [rust_path])
        )
        result = match_api_row(rows[selector], self.by_name, self.rust, known_sources)
        self.assertEqual(result["resolution"], "implemented", selector)


def read_lines(relative: str) -> list[dict[str, JSONValue]]:
    with gzip.open(ROOT / relative, "rt") as stream:
        return [json.loads(line) for line in stream]


def read_json_gzip(relative: str) -> dict[str, JSONValue]:
    with gzip.open(ROOT / relative, "rt") as stream:
        return json.load(stream)
