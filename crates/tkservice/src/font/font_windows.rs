//! Pinned OCCT's registry order, supported extensions and relative Windows font paths.
use rustycad_tkernel::tcollection::AsciiString;

pub(super) trait WindowsFontRegistry {
    fn system_windows_directory(&self) -> Option<Vec<u8>>;
    fn registered_font_paths(&self) -> Vec<Vec<u8>>;
}

pub(super) fn registry_font_candidates(registry: &impl WindowsFontRegistry) -> Vec<AsciiString> {
    let Some(mut directory) = registry.system_windows_directory() else {
        return Vec::new();
    };
    directory.extend_from_slice(b"\\Fonts\\");
    registry
        .registered_font_paths()
        .into_iter()
        .filter_map(|path| resolve_registry_font_path(&directory, path))
        .collect()
}

fn resolve_registry_font_path(directory: &[u8], mut path: Vec<u8>) -> Option<AsciiString> {
    let separator = path.iter().rposition(|byte| *byte == b'.')?;
    let extension = &path[separator + 1..];
    let supported = [b"ttf".as_slice(), b"otf", b"ttc", b"pfa", b"pfb"];
    if !supported
        .iter()
        .any(|suffix| extension.eq_ignore_ascii_case(suffix))
    {
        return None;
    }
    if !path.contains(&b'\\') {
        path = [directory, path.as_slice()].concat();
    }
    Some(AsciiString::from_view(&path))
}

#[cfg(windows)]
impl super::FontFontMgr {
    pub(super) fn register_windows_font_database(
        &self,
        library: &super::FontFTLibrary,
    ) -> Result<(), rustycad_tkernel::standard::Failure> {
        let registry = super::font_windows_ffi::SystemFontRegistry;
        for path in registry_font_candidates(&registry) {
            let mut fonts = Vec::new();
            super::font_check::check_font(&mut fonts, library, &path, 0)?;
            self.register_fonts(&fonts, false)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct FakeWindowsFontRegistry {
        directory: Option<Vec<u8>>,
        paths: Vec<Vec<u8>>,
    }
    impl WindowsFontRegistry for FakeWindowsFontRegistry {
        fn system_windows_directory(&self) -> Option<Vec<u8>> {
            self.directory.clone()
        }
        fn registered_font_paths(&self) -> Vec<Vec<u8>> {
            self.paths.clone()
        }
    }
    #[test]
    fn registry_order_absolute_paths_and_case_insensitive_extensions_match_native() {
        let registry = FakeWindowsFontRegistry {
            directory: Some(b"C:\\Windows".to_vec()),
            paths: [
                b"Arial.TTF".as_slice(),
                b"D:\\Shared\\Face.ttc",
                b"broken.exe",
                b"Type1.pfb",
                b"none",
            ]
            .into_iter()
            .map(Vec::from)
            .collect(),
        };
        let paths: Vec<Vec<u8>> = registry_font_candidates(&registry)
            .iter()
            .map(|path| path.as_bytes().to_vec())
            .collect();
        assert_eq!(
            paths,
            [
                b"C:\\Windows\\Fonts\\Arial.TTF".to_vec(),
                b"D:\\Shared\\Face.ttc".to_vec(),
                b"C:\\Windows\\Fonts\\Type1.pfb".to_vec()
            ]
        );
    }
    #[test]
    fn absent_windows_directory_or_empty_registry_yields_no_candidates() {
        let missing = FakeWindowsFontRegistry {
            directory: None,
            paths: vec![b"Arial.ttf".to_vec()],
        };
        assert!(registry_font_candidates(&missing).is_empty());
        let empty = FakeWindowsFontRegistry {
            directory: Some(b"C:\\Windows".to_vec()),
            paths: Vec::new(),
        };
        assert!(registry_font_candidates(&empty).is_empty());
    }
}
