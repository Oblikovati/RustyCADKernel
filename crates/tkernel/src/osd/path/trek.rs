//! Trek editing (`UpTrek`, `DownTrek`, `TrekLength`, `RemoveATrek`, `TrekValue`,
//! `InsertATrek`): the Unix build is here, the Windows build in `windows_host`.
//!
//! A failing call may leave the trek partly edited, exactly where the native code raised.
use super::{
    CadPath, PathSyntax,
    text::{self, Text},
    windows_host,
};
use crate::{standard::Failure, tcollection::AsciiString};

fn numeric_error(message: &str) -> Failure {
    Failure::with_type("Standard_NumericError", message, "")
}

impl CadPath {
    /// `UpTrek`: removes the last trek component; `path.up_trek()`.
    ///
    /// Unix build: the last token is removed where it last occurs, then the first `||`
    /// loses one bar. Windows build: truncation before the last `|` run.
    pub fn up_trek(&mut self) {
        if self.syntax == PathSyntax::Windows {
            return windows_host::up_trek(&mut self.trek);
        }
        let length: i32 = self.trek_length();
        if length == 0 {
            return;
        }
        let last: Text = text::token(&self.trek, b"|", length);
        let at: i32 = text::search_from_end(&self.trek, &last);
        // The last token always occurs at its own position, so both removals are in range.
        let _ = text::remove(&mut self.trek, at, text::len(&last));
        self.remove_first_double_bar();
    }
    /// `DownTrek(name)`: appends a component; `path.down_trek("sub")`.
    ///
    /// Unix build: `name` then a `|` unless `name` ends with one. An empty `name` makes
    /// the native code read the byte before its buffer; here it appends `|`. Windows
    /// build: joined with one `|` and normalized.
    pub fn down_trek(&mut self, name: impl AsRef<[u8]>) {
        let name: &[u8] = super::c_bytes(name.as_ref());
        if self.syntax == PathSyntax::Windows {
            return windows_host::down_trek(&mut self.trek, name);
        }
        self.trek.extend_from_slice(name);
        if name.last() != Some(&b'|') {
            self.trek.push(b'|');
        }
    }
    /// `TrekLength`: the number of trek components; `path.trek_length()`.
    pub fn trek_length(&self) -> i32 {
        if self.syntax == PathSyntax::Windows {
            return windows_host::trek_length(&self.trek);
        }
        self.trek
            .split(|&byte| byte == b'|')
            .filter(|part| !part.is_empty())
            .count() as i32
    }
    /// `RemoveATrek(where)`: removes the where-th component; `path.remove_a_trek(2)?`.
    ///
    /// Unix build: raises `Standard_NumericError` for an empty trek or `where` past the
    /// end, and removes the where-th token's text at its *first* occurrence (so
    /// `|ab|b|` minus component 2 is `|a|b|`). Windows build: out-of-range `where` is
    /// ignored; an empty trek raises `Standard_OutOfRange`.
    pub fn remove_a_trek(&mut self, at: i32) -> Result<(), Failure> {
        if self.syntax == PathSyntax::Windows {
            return windows_host::remove_at(&mut self.trek, at);
        }
        let length: i32 = self.trek_length();
        if length <= 0 || at > length {
            return Err(numeric_error(
                "OSD_Path::RemoveATrek : where has an invalid value",
            ));
        }
        let component: Text = text::token(&self.trek, b"|", at);
        let found: i32 = text::search(&self.trek, &component);
        text::remove(&mut self.trek, found, text::len(&component))?;
        self.remove_first_double_bar();
        Ok(())
    }
    /// `RemoveATrek(name)`: removes the first occurrence of `name`; `path.remove_a_trek_named("sub")?`.
    ///
    /// Unix build: plain text search (it may cut inside a longer component), never
    /// fails. Windows build: searches `|name|`; an empty trek or name raises
    /// `Standard_OutOfRange`.
    pub fn remove_a_trek_named(&mut self, name: impl AsRef<[u8]>) -> Result<(), Failure> {
        let name: &[u8] = super::c_bytes(name.as_ref());
        if self.syntax == PathSyntax::Windows {
            return windows_host::remove_named(&mut self.trek, name);
        }
        if self.trek_length() == 0 {
            return Ok(());
        }
        let found: i32 = text::search(&self.trek, name);
        if found != -1 {
            text::remove(&mut self.trek, found, text::len(name))?;
            self.remove_first_double_bar();
        }
        Ok(())
    }
    /// `TrekValue(where)`: the where-th component; `path.trek_value(1)?`.
    ///
    /// Unix build: raises `Standard_NumericError` when there is none. Windows build:
    /// returns an empty string instead, and raises `Standard_OutOfRange` for an empty trek.
    pub fn trek_value(&self, at: i32) -> Result<AsciiString, Failure> {
        let value: Text = match self.syntax {
            PathSyntax::Windows => windows_host::value_at(&self.trek, at)?,
            PathSyntax::Unix => text::token(&self.trek, b"|", at),
        };
        if self.syntax == PathSyntax::Unix && value.is_empty() {
            return Err(numeric_error("OSD_Path::TrekValue : where is invalid"));
        }
        Ok(AsciiString::from_raw(&value))
    }
    /// `InsertATrek(name, where)`: inserts `name` before the where-th component;
    /// `path.insert_a_trek("sub", 1)?`.
    ///
    /// Unix build: raises `Standard_NumericError` unless `1 <= where <= TrekLength()`,
    /// and inserts before the first occurrence of that component's text. Windows build:
    /// inserts after the where-th `|` and ignores an out-of-range `where`.
    pub fn insert_a_trek(&mut self, name: impl AsRef<[u8]>, at: i32) -> Result<(), Failure> {
        let name: &[u8] = super::c_bytes(name.as_ref());
        if self.syntax == PathSyntax::Windows {
            return windows_host::insert_at(&mut self.trek, name, at);
        }
        let length: i32 = self.trek_length();
        if at <= 0 || at > length {
            return Err(numeric_error(
                "OSD_Path::InsertATrek : where has an invalid value",
            ));
        }
        let component: Text = text::token(&self.trek, b"|", at);
        let found: i32 = text::search(&self.trek, &component);
        let mut inserted: Text = name.to_vec();
        inserted.push(b'|');
        text::insert(&mut self.trek, found, &inserted)
    }
    /// `awhere = myTrek.Search("||"); if (awhere != -1) myTrek.Remove(awhere);`
    fn remove_first_double_bar(&mut self) {
        let at: i32 = text::search(&self.trek, b"||");
        if at != -1 {
            self.trek.remove(at as usize - 1);
        }
    }
}
