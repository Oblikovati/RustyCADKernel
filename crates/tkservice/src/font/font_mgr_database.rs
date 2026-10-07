//! `Font_FontMgr::InitFontDataBase` of the X11 (fontconfig) build: the font directories of
//! fontconfig (or the X font server configuration) and the default directories, scanned
//! recursively; each directory's `fonts.dir` when present, else every file in it.
use super::font_check::check_font;
use super::fontconfig_ffi::fontconfig_font_dirs;
use super::{FontFTLibrary, FontFontAspect, FontFontMgr, FontSystemFont};
use rustycad_tkernel::message::Message;
use rustycad_tkernel::ncollection::{BaseAllocator, Buffer, Map};
use rustycad_tkernel::osd::{
    CadPath, DirectoryIterator, File, FileIterator, OpenMode, Protection, SingleProtection,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;

/// The extensions of the font files (`extensions()`).
const EXTENSIONS: [&str; 5] = ["ttf", "otf", "ttc", "pfa", "pfb"];
/// The X font server configurations (`fontServiceConfigurations()`).
const FONT_SERVICE_CONFIGURATIONS: [&str; 3] = [
    "/etc/X11/fs/config",
    "/usr/X11R6/lib/X11/fs/config",
    "/usr/X11/lib/X11/fs/config",
];
/// The default font directories (`defaultFontDirectories()`).
const DEFAULT_FONT_DIRECTORIES: [&str; 3] = [
    "/system/fonts",
    "/usr/share/fonts",
    "/usr/local/share/fonts",
];
/// The embedded DejaVu Sans Latin subset (`Font_DejavuSans_Latin_woff`).
const EMBED_FALLBACK_FONT: &[u8] = include_bytes!("dejavu_sans_latin.woff");

/// A directory set in the native `NCollection_Map` order.
type DirectorySet = Map<AsciiString>;

/// `addDirsRecursively`: the directory and its subdirectories, each once.
fn add_dirs_recursively(path: &CadPath, dirs: &mut DirectorySet) {
    let dir_name = path.system_name();
    if !dirs.add(dir_name.clone()) {
        return;
    }
    let mut children = DirectoryIterator::new(path, "*");
    while children.more() {
        let child = children.values().path().system_name();
        children.next();
        if child.is_equal(".") || child.is_equal("..") {
            continue;
        }
        let joined = [dir_name.as_bytes(), b"/", child.as_bytes()].concat();
        add_dirs_recursively(&CadPath::new(joined), dirs);
    }
}

fn read_protection() -> Protection {
    let r = SingleProtection::R;
    Protection::with_values(r, r, r, r)
}

/// Opens a file for reading, `None` when missing or unreadable.
fn open_read(path: &[u8]) -> Option<File> {
    let mut file = File::new(path);
    if !file.exists() {
        return None;
    }
    file.open(OpenMode::ReadOnly, &read_protection()).ok()?;
    file.is_open().then_some(file)
}

/// The font paths of a `catalogue=` line of an X font server configuration.
fn add_catalogue_dirs(line: &AsciiString, dirs: &mut DirectorySet) -> Result<(), Failure> {
    let location = match line.search("catalogue=") {
        -1 => line.search("catalogue ="),
        found => found,
    };
    let path_location = line.search("/");
    if location <= 0 || path_location <= 0 {
        return Ok(());
    }
    let mut paths = line.clone();
    paths = paths.split(path_location - 1)?;
    for index in 1.. {
        let mut font_path = paths.token(":,", index);
        font_path.right_adjust();
        if font_path.is_empty() {
            break;
        }
        add_dirs_recursively(&CadPath::new(font_path.as_bytes()), dirs);
    }
    Ok(())
}

/// The directories of the X font server configurations (the fontconfig-less fallback).
fn add_font_service_dirs(dirs: &mut DirectorySet) -> Result<(), Failure> {
    for configuration in FONT_SERVICE_CONFIGURATIONS {
        let Some(mut file) = open_read(configuration.as_bytes()) else {
            continue;
        };
        let mut line = AsciiString::default();
        while !file.is_at_end()? {
            file.read_line(&mut line, 256)?;
            add_catalogue_dirs(&line, dirs)?;
        }
        file.close()?;
    }
    Ok(())
}

/// The fontconfig directories, else the X font server ones, then the default directories.
fn font_directories() -> Result<DirectorySet, Failure> {
    let mut dirs = DirectorySet::new(1);
    for dir in fontconfig_font_dirs() {
        add_dirs_recursively(&CadPath::new(dir), &mut dirs);
    }
    if dirs.is_empty() {
        Message::send_alarm_text(&AsciiString::from(
            "Font_FontMgr, fontconfig library returns an empty folder list",
        ));
        add_font_service_dirs(&mut dirs)?;
    }
    for dir in DEFAULT_FONT_DIRECTORIES {
        add_dirs_recursively(&CadPath::new(dir), &mut dirs);
    }
    Ok(dirs)
}

impl FontFontMgr {
    /// `InitFontDataBase`: forgets the fonts, then registers every font of the system font
    /// directories. A malformed `fonts.dir` line that natively raises from
    /// `TCollection_AsciiString` stops the scan with that failure.
    /// Example: `manager.init_font_data_base()?`.
    pub fn init_font_data_base(&self) -> Result<(), Failure> {
        self.clear_font_data_base();
        let library = FontFTLibrary::new();
        let dirs = font_directories()?;
        for dir in dirs.iter() {
            let fonts_dir = [dir.as_bytes(), b"/fonts.dir"].concat();
            match open_read(&fonts_dir) {
                Some(file) => self.register_fonts_dir(file, dir, &library)?,
                None => self.register_directory_files(dir, &library)?,
            }
        }
        Ok(())
    }
    /// Every file of a directory without `fonts.dir`.
    fn register_directory_files(
        &self,
        dir: &AsciiString,
        library: &FontFTLibrary,
    ) -> Result<(), Failure> {
        let mut files = FileIterator::new(&CadPath::new(dir.as_bytes()), "*");
        while files.more() {
            let name = files.values().path().system_name();
            files.next();
            let path = AsciiString::from_view(&[dir.as_bytes(), b"/", name.as_bytes()].concat());
            let mut fonts = Vec::new();
            check_font(&mut fonts, library, &path, -1)?;
            self.register_fonts(&fonts, false)?;
        }
        Ok(())
    }
    /// The fonts listed by a `fonts.dir` (its first line, the count, is skipped).
    fn register_fonts_dir(
        &self,
        mut file: File,
        dir: &AsciiString,
        library: &FontFTLibrary,
    ) -> Result<(), Failure> {
        let mut line = AsciiString::default();
        let mut is_first_line = true;
        while !file.is_at_end()? {
            file.read_line(&mut line, 256)?;
            if std::mem::replace(&mut is_first_line, false) {
                continue;
            }
            self.register_fonts_dir_line(&line, dir, library)?;
        }
        file.close()
    }
}

/// The file name and XLFD of a `fonts.dir` line with a supported extension and an
/// `iso8859-1` encoding, `None` otherwise.
fn parse_fonts_dir_line(line: &AsciiString) -> Result<Option<(AsciiString, AsciiString)>, Failure> {
    let extension_position = line.search(".") + 1;
    if extension_position == 0 {
        return Ok(None);
    }
    let end_of_file_name = line.location(" ", extension_position, line.length())? - 1;
    if end_of_file_name < 0 || end_of_file_name < extension_position {
        return Ok(None);
    }
    let mut extension = line.sub_string(extension_position, end_of_file_name)?;
    extension.lower_case();
    let supported = EXTENSIONS.iter().any(|e| extension.is_equal(e));
    if !supported || line.search("iso8859-1\n") <= 0 {
        return Ok(None);
    }
    let xlfd = line.sub_string(end_of_file_name + 2, line.length())?;
    Ok(Some((line.sub_string(1, end_of_file_name)?, xlfd)))
}

/// The aspect of an XLFD's weight and slant tokens.
fn xlfd_aspect(xlfd: &AsciiString) -> FontFontAspect {
    let bold = xlfd.token("-", 3).is_equal("bold");
    let slant = xlfd.token("-", 4);
    let italic = slant.is_equal("i") || slant.is_equal("o");
    match (bold, italic) {
        (true, true) => FontFontAspect::BoldItalic,
        (true, false) => FontFontAspect::Bold,
        (false, true) => FontFontAspect::Italic,
        (false, false) => FontFontAspect::Regular,
    }
}

impl FontFontMgr {
    /// One `fonts.dir` entry: the font file, and the XLFD family when scalable and different.
    fn register_fonts_dir_line(
        &self,
        line: &AsciiString,
        dir: &AsciiString,
        library: &FontFTLibrary,
    ) -> Result<(), Failure> {
        let Some((file_name, xlfd)) = parse_fonts_dir_line(line)? else {
            return Ok(());
        };
        let mut font_path = dir.clone();
        if font_path.search_from_end("/") != font_path.length() {
            font_path.assign_cat("/");
        }
        font_path.assign_cat(file_name);
        let mut fonts = Vec::new();
        if !check_font(&mut fonts, library, &font_path, -1)? {
            return Ok(());
        }
        self.register_fonts(&fonts, false)?;
        if xlfd.is_empty() || xlfd.search("-0-0-0-0-") == -1 {
            return Ok(());
        }
        let x_font = FontSystemFont::new(&xlfd.token("-", 2))?;
        x_font.set_font_path(xlfd_aspect(&xlfd), &font_path, 0)?;
        let x_font = Handle::new(x_font);
        if !fonts[0].is_equal(&x_font)? {
            self.register_font(&x_font, false)?;
        }
        Ok(())
    }
    /// `CheckFont(theFonts, thePath)`: appends the fonts of the file's first face (and its
    /// instances); `manager.check_font_into(&mut fonts, &path)?`.
    pub fn check_font_into(
        &self,
        fonts: &mut Vec<Handle<FontSystemFont>>,
        path: &AsciiString,
    ) -> Result<bool, Failure> {
        check_font(fonts, &FontFTLibrary::new(), path, 0)
    }
    /// `CheckFont(thePath)`: the font of the file's first face, or null; `manager.check_font(&path)?`.
    pub fn check_font(&self, path: &AsciiString) -> Result<Handle<FontSystemFont>, Failure> {
        let mut fonts = Vec::new();
        let found = check_font(&mut fonts, &FontFTLibrary::new(), path, 0)?;
        Ok(if found {
            fonts.swap_remove(0)
        } else {
            Handle::default()
        })
    }
    /// `EmbedFallbackFont`: the embedded DejaVu Sans Latin WOFF data (a copy, as the native
    /// buffer wraps static data). Example: `FontFontMgr::embed_fallback_font()`.
    pub fn embed_fallback_font() -> Handle<Buffer> {
        let size = EMBED_FALLBACK_FONT.len();
        let Ok(mut buffer) = Buffer::new(Some(BaseAllocator::common()), size) else {
            return Handle::default();
        };
        if buffer.size() != size {
            return Handle::default();
        }
        buffer.as_mut_slice()[..size].copy_from_slice(EMBED_FALLBACK_FONT);
        Handle::new(buffer)
    }
}
