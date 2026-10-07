"""Named fakes and fixture builders shared by the macro-row generator tests."""

from __future__ import annotations

from api_inventory.inputs import JSONObject

PIN = "a" * 64

# Reduced copies of the tkernel registration macros, enough for the registration scan.
REGISTRATION_MACROS = """
macro_rules! impl_transient {
    ($class:ty, $name:literal, $parent:ty, $field:ident) => {
        $crate::impl_transient!($class, $name, $parent, $field, interfaces: []);
    };
    ($class:ty, $name:literal, $parent:ty, $field:ident, interfaces: [$($interface:ty),* $(,)?]) => {
        impl $crate::standard::RuntimeTyped for $class {
            const TYPE_NAME: &'static str = $name;
            fn static_type() -> &'static RuntimeType {
                register::<Self>($name, Some(<$parent as $crate::standard::RuntimeTyped>::static_type_handle()))
            }
        }
    };
}
macro_rules! impl_transient_interface {
    ($interface:ty, $name:literal, $parent:ty) => {
        impl $crate::standard::RuntimeTyped for $interface {
            const TYPE_NAME: &'static str = $name;
            fn static_type() -> &'static RuntimeType {
                let parent = <$parent as $crate::standard::RuntimeTyped>::static_type_handle();
                register::<Self>($name, Some(parent))
            }
        }
    };
}
macro_rules! define_alert {
    ($(#[$doc:meta])* $alert:ident, $name:literal) => {
        pub struct $alert;
        impl_transient!($alert, $name, AlertBase, parent);
    };
}
"""


class FakeOcctSources:
    """In-memory OCCT tree: `FakeOcctSources({'src/A/a.hxx': '...'})`."""

    def __init__(self, files: dict[str, str]) -> None:
        self.files = files

    def text(self, relative: str) -> str | None:
        return self.files.get(relative)

    def sibling_sources(self, relative: str) -> list[str]:
        folder = relative.rsplit("/", 1)[0]
        return sorted(path for path in self.files if path.startswith(folder + "/") and path.endswith(".cxx"))


class FakeRustSources:
    """In-memory Rust workspace: `FakeRustSources({'crates/tkg2d/src/a.rs': '...'})`."""

    def __init__(self, files: dict[str, str]) -> None:
        self.files = files

    def rust_files(self) -> list[tuple[str, str]]:
        return sorted(self.files.items())

    def text(self, relative: str) -> str | None:
        return self.files.get(relative)


def declaration(
    identity: str,
    name: str,
    kind: str,
    signature: str = "",
    parameters: list[str] | None = None,
    header: str = "src/ModelingData/TKG2d/Geom2d/Geom2d_Line.hxx",
    spelled: tuple[str, int, int] | None = None,
    macro: bool = False,
) -> JSONObject:
    """One compiler inventory record in the shape the crosswalk reads."""
    file, line, column = spelled or (header, 1, 1)
    return {
        "id": identity,
        "qualified_name": name,
        "kind": kind,
        "signature": signature,
        "parameter_types": parameters if parameters is not None else ([] if kind != "CXXRecord" else None),
        "written_parameter_types": parameters if parameters is not None else ([] if kind != "CXXRecord" else None),
        "api_visible": True,
        "source": header,
        "macro_generated": macro,
        "spelling_location": {"file": file, "line": line, "column": column},
        "variadic": False,
    }


def rust_item(path: str, kind: str, source: str = "crates/tkg2d/src/geom2d/line.rs", **extra: object) -> JSONObject:
    """One normalized rustdoc declaration with a pinned source."""
    item: JSONObject = {"path": path, "kind": kind, "source_file": source, "source_sha256": PIN, "source_status": None}
    item.update(extra)
    return item


def rust_document(items: list[JSONObject], crate: str = "rustycad_tkg2d") -> JSONObject:
    return {"crates": [{"crate": crate, "declarations": items}]}


def header_manifest(paths: dict[str, str]) -> JSONObject:
    """`{'src/.../a.hxx': 'TKG2d'}` -> the api-header-manifest shape."""
    return {
        "headers": [{"path": path, "owners": [{"toolkit": toolkit}]} for path, toolkit in paths.items()],
        "toolkits": [{"name": toolkit} for toolkit in sorted(set(paths.values()))],
    }
