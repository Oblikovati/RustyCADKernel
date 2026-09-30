use super::{CadPath, c_bytes};
impl CadPath {
    /// Detects OCCT's drive-colon form, including drive-relative spellings; `CadPath::is_dos_path("C:part")`.
    pub fn is_dos_path(path: impl AsRef<[u8]>) -> bool {
        c_bytes(path.as_ref()).get(1) == Some(&b':')
    }
    /// Detects the case-sensitive NT extended prefix; `CadPath::is_nt_extended_path(path)`.
    pub fn is_nt_extended_path(path: impl AsRef<[u8]>) -> bool {
        c_bytes(path.as_ref()).starts_with(br"\\?\")
    }
    /// Detects normal or extended UNC syntax; `CadPath::is_unc_path("//server/share")`.
    pub fn is_unc_path(path: impl AsRef<[u8]>) -> bool {
        let bytes = c_bytes(path.as_ref());
        if bytes.starts_with(br"\\") {
            return bytes.get(2) != Some(&b'?') || Self::is_unc_extended_path(bytes);
        }
        bytes.starts_with(b"//")
    }
    /// Detects the uppercase extended UNC prefix; `CadPath::is_unc_extended_path(path)`.
    pub fn is_unc_extended_path(path: impl AsRef<[u8]>) -> bool {
        c_bytes(path.as_ref()).starts_with(br"\\?\UNC\")
    }
    /// Detects a single leading slash; `CadPath::is_unix_path("/tmp")`.
    pub fn is_unix_path(path: impl AsRef<[u8]>) -> bool {
        let bytes = c_bytes(path.as_ref());
        bytes.first() == Some(&b'/') && bytes.get(1) != Some(&b'/')
    }
    /// Detects Android's content prefix; `CadPath::is_content_protocol_path("content://part")`.
    pub fn is_content_protocol_path(path: impl AsRef<[u8]>) -> bool {
        c_bytes(path.as_ref()).starts_with(b"content://")
    }
    /// Checks the first colon for a following double slash; `CadPath::is_remote_protocol_path("https://host")`.
    pub fn is_remote_protocol_path(path: impl AsRef<[u8]>) -> bool {
        let bytes = c_bytes(path.as_ref());
        let Some(index) = bytes.iter().position(|&byte| byte == b':') else {
            return false;
        };
        index != 0 && bytes[index + 1..].starts_with(b"//")
    }
    /// Classifies path syntax without filesystem access; `CadPath::is_relative_path("../part")`.
    pub fn is_relative_path(path: impl AsRef<[u8]>) -> bool {
        let bytes = path.as_ref();
        !Self::is_unc_path(bytes)
            && !Self::is_dos_path(bytes)
            && !Self::is_nt_extended_path(bytes)
            && !Self::is_unix_path(bytes)
            && !Self::is_remote_protocol_path(bytes)
    }
    /// Complements OCCT's relative syntax classification; `CadPath::is_absolute_path("/part")`.
    pub fn is_absolute_path(path: impl AsRef<[u8]>) -> bool {
        !Self::is_relative_path(path)
    }
}
