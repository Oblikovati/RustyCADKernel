// Rust port of DataExchange/TKXSBase/Interface/Interface_MSG (.hxx, .cxx: messages, the
// dictionary and its file format) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_MSG`: translated messages from a process-wide dictionary of keys. The
//! number helpers (`Intervalled`, dates, blanks, `Print`) are in `msg_text`.
use rustycad_tkernel::{
    ncollection::DataMap,
    standard::{
        Failure, Handle,
        c_string::{FormatArg, sprintf_into},
        exceptions,
    },
    tcollection::{AsciiString, HAsciiString},
};
use std::io::{self, BufRead, Write};
use std::sync::{OnceLock, PoisonError, RwLock, RwLockWriteGuard};

/// The native statics of `Interface_MSG.cxx`.
struct MsgState {
    thedic: DataMap<AsciiString, Handle<HAsciiString>>,
    thelist: DataMap<AsciiString, i32>,
    thedup: Option<Vec<Handle<HAsciiString>>>,
    theprint: bool,
    therec: bool,
    therun: bool,
    theraise: bool,
}

fn state() -> RwLockWriteGuard<'static, MsgState> {
    static STATE: OnceLock<RwLock<MsgState>> = OnceLock::new();
    let lock = STATE.get_or_init(|| {
        RwLock::new(MsgState {
            thedic: DataMap::default(),
            thelist: DataMap::default(),
            thedup: None,
            theprint: true,
            therec: false,
            therun: false,
            theraise: false,
        })
    });
    lock.write().unwrap_or_else(PoisonError::into_inner)
}

/// `Interface_MSG`: a key, and the message formatted from its translation when built
/// with values (natively the `Sprintf` of the translated C format into 300 chars).
///
/// ```
/// use rustycad_tkxsbase::interface::InterfaceMsg;
/// let msg = InterfaceMsg::with_integer("Entity n0.%d", 12).unwrap();
/// assert_eq!(msg.value().unwrap(), "Entity n0.12");
/// ```
#[derive(Clone, Debug)]
pub struct InterfaceMsg {
    thekey: String,
    theval: Option<String>,
}

impl InterfaceMsg {
    /// `Interface_MSG(key)`: the value is the translation, computed when read.
    pub fn new(key: &str) -> Self {
        Self {
            thekey: key.to_owned(),
            theval: None,
        }
    }
    /// `Interface_MSG(key, i1)`: the translation as a C format of one `%d`.
    pub fn with_integer(key: &str, i1: i32) -> Result<Self, Failure> {
        Self::formatted(key, &[FormatArg::Int(i1)])
    }
    /// `Interface_MSG(key, i1, i2)`: two `%d`.
    pub fn with_integers(key: &str, i1: i32, i2: i32) -> Result<Self, Failure> {
        Self::formatted(key, &[FormatArg::Int(i1), FormatArg::Int(i2)])
    }
    /// `Interface_MSG(key, r1, intervals = -1)`: one real, rounded by `Intervalled` for
    /// a non-negative `intervals`.
    pub fn with_real(key: &str, r1: f64, intervals: i32) -> Result<Self, Failure> {
        let value = if intervals < 0 {
            r1
        } else {
            Self::intervalled(r1, intervals, false)
        };
        Self::formatted(key, &[FormatArg::Real(value)])
    }
    /// `Interface_MSG(key, str)`: one `%s`.
    pub fn with_text(key: &str, text: &str) -> Result<Self, Failure> {
        Self::formatted(key, &[FormatArg::Text(text.as_bytes())])
    }
    /// `Interface_MSG(key, ival, str)`: one `%d` then one `%s`.
    pub fn with_integer_text(key: &str, ival: i32, text: &str) -> Result<Self, Failure> {
        Self::formatted(
            key,
            &[FormatArg::Int(ival), FormatArg::Text(text.as_bytes())],
        )
    }
    fn formatted(key: &str, args: &[FormatArg<'_>]) -> Result<Self, Failure> {
        let form = Self::translated(key)?;
        let mut mess = Vec::new();
        sprintf_into(&mut mess, form.as_bytes(), args).map_err(|e| {
            Failure::new(&format!(
                "Interface_MSG: format {form:?} of key {key:?}: {e}"
            ))
        })?;
        mess.pop(); // the terminating NUL
        let theval = Some(String::from_utf8_lossy(&mess).into_owned());
        Ok(Self {
            thekey: key.to_owned(),
            theval,
        })
    }
    /// `Value` (and `operator const char*`): the formatted message, else the translation.
    pub fn value(&self) -> Result<String, Failure> {
        match &self.theval {
            Some(text) => Ok(text.clone()),
            None => Self::translated(&self.thekey),
        }
    }
    /// `Destroy`: forgets the formatted message (the destructor's work natively).
    pub fn destroy(&mut self) {
        self.theval = None;
    }
    /// `Read(Standard_IStream&)`: reads "@key" lines each followed by its message;
    /// "@@" lines and empty lines are skipped. Returns the count of keys. As
    /// `std::istream::getline(buf, 200)`, a line of 200 characters or more ends the
    /// reading. A raised `Record` (raising mode) ends it with that failure.
    pub fn read(stream: &mut dyn BufRead) -> Result<i32, Failure> {
        let mut nb = 0;
        let mut key = String::new();
        let mut buf = Vec::new();
        loop {
            buf.clear();
            let Ok(count) = stream.read_until(b'\n', &mut buf) else {
                break;
            };
            if count == 0 {
                break;
            }
            if buf.last() == Some(&b'\n') {
                buf.pop();
            }
            if buf.len() >= 200 {
                break;
            }
            let line = String::from_utf8_lossy(&buf).into_owned();
            if line.starts_with("@@") || line.is_empty() {
                continue;
            }
            if let Some(rest) = line.strip_prefix('@') {
                nb += 1;
                key = rest.to_owned();
            } else {
                Self::record(&key, &line)?;
            }
        }
        Ok(nb)
    }
    /// `Read(const char* file)`: -1 when the file cannot be opened.
    pub fn read_file(file: &str) -> Result<i32, Failure> {
        match std::fs::File::open(file) {
            Ok(opened) => Self::read(&mut io::BufReader::new(opened)),
            Err(_) => Ok(-1),
        }
    }
    /// `Write(S, rootkey = "")`: writes the dictionary (the keys starting with
    /// `rootkey`); returns the count of written messages.
    pub fn write(stream: &mut dyn Write, rootkey: &str) -> io::Result<i32> {
        let entries: Vec<(AsciiString, Handle<HAsciiString>)> = {
            let msg = state();
            msg.thedic
                .items()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        };
        if entries.is_empty() {
            return Ok(0);
        }
        if !rootkey.is_empty() {
            writeln!(stream, "@@ ROOT:{rootkey}")?;
        }
        let mut nb = 0;
        for (key, text) in entries.iter().filter(|(k, _)| k.starts_with(rootkey)) {
            stream.write_all(b"@")?;
            stream.write_all(key.as_bytes())?;
            stream.write_all(b"\n")?;
            let Some(text) = text.get() else { continue };
            nb += 1;
            stream.write_all(text.string().as_bytes())?;
            stream.write_all(b"\n")?;
        }
        stream.flush()?;
        Ok(nb)
    }
    /// `IsKey(mess)`: a key begins with '^'.
    pub fn is_key(mess: &str) -> bool {
        mess.starts_with('^')
    }
    /// `Translated(key)`: the recorded item, else the key (traced, and a
    /// `Standard_DomainError` in raising mode); the key itself when not running.
    pub fn translated(key: &str) -> Result<String, Failure> {
        let mut msg = state();
        if !msg.therun {
            return Ok(key.to_owned());
        }
        let akey = AsciiString::new(key);
        if let Some(found) = msg.thedic.seek(&akey) {
            return Ok(String::from_utf8_lossy(found.string().as_bytes()).into_owned());
        }
        if msg.theprint {
            println!(" **  Interface_MSG:Translate ?? {key}  **");
        }
        if msg.therec {
            count_missing(&mut msg.thelist, akey);
        }
        if msg.theraise {
            return Err(exceptions::domain_error(&format!(
                "Interface_MSG : Translate: key {key:?} is not recorded; expected a recorded key"
            )));
        }
        Ok(key.to_owned())
    }
    /// `Record(key, item)`: binds `item`; a redefinition is traced (and raises
    /// `Standard_DomainError` in raising mode, after replacing the item).
    pub fn record(key: &str, item: &str) -> Result<(), Failure> {
        let mut msg = state();
        let str = Handle::new(HAsciiString::new(item));
        let akey = AsciiString::new(key);
        if let Ok(slot) = msg.thedic.change_find(&akey) {
            *slot = str;
        } else {
            msg.thedic.bind(akey, str);
            return Ok(());
        }
        if msg.theprint {
            println!(" **  Interface_MSG:Record ?? {key} ** {item}  **");
        }
        if msg.therec {
            let dup = msg.thedup.get_or_insert_with(Vec::new);
            dup.push(Handle::new(HAsciiString::new(key)));
            dup.push(Handle::new(HAsciiString::new(item)));
        }
        if msg.theraise {
            return Err(exceptions::domain_error(&format!(
                "Interface_MSG : Record: key {key:?} is already recorded; expected a new key"
            )));
        }
        Ok(())
    }
    /// `SetTrace(toprint, torecord)`.
    pub fn set_trace(toprint: bool, torecord: bool) {
        let mut msg = state();
        msg.theprint = toprint;
        msg.therec = torecord;
    }
    /// `SetMode(running, raising)`.
    pub fn set_mode(running: bool, raising: bool) {
        let mut msg = state();
        msg.therun = running;
        msg.theraise = raising;
    }
    /// `PrintTrace`: the recorded redefinitions, then the counts of missing keys.
    pub fn print_trace(stream: &mut dyn Write) -> io::Result<()> {
        let (dups, missing) = {
            let msg = state();
            let missing: Vec<(AsciiString, i32)> =
                msg.thelist.items().map(|(k, v)| (k.clone(), *v)).collect();
            (msg.thedup.clone().unwrap_or_default(), missing)
        };
        for pair in dups.as_chunks::<2>().0 {
            stream.write_all(b"** DUP:")?;
            stream.write_all(pair[0].string().as_bytes())?;
            stream.write_all(b" ** ")?;
            stream.write_all(pair[1].string().as_bytes())?;
            stream.write_all(b"\n")?;
        }
        for (key, count) in missing {
            write!(stream, "** MSG(NB={count}): ")?;
            stream.write_all(key.as_bytes())?;
            stream.write_all(b"\n")?;
        }
        Ok(())
    }
}

fn count_missing(list: &mut DataMap<AsciiString, i32>, key: AsciiString) {
    match list.change_find(&key) {
        Ok(count) => *count += 1,
        Err(_) => {
            list.bind(key, 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dictionary_read_translate_and_write() {
        let text = "@@ comment\n@^TEST.KEY1\nfirst %d\n\n@^TEST.KEY2\nsecond\n";
        assert_eq!(InterfaceMsg::read(&mut text.as_bytes()).unwrap(), 2);
        InterfaceMsg::set_mode(true, false);
        InterfaceMsg::set_trace(false, true);
        assert_eq!(InterfaceMsg::translated("^TEST.KEY2").unwrap(), "second");
        let msg = InterfaceMsg::with_integer("^TEST.KEY1", 5).unwrap();
        assert_eq!(msg.value().unwrap(), "first 5");
        assert_eq!(
            InterfaceMsg::translated("^TEST.MISSING").unwrap(),
            "^TEST.MISSING"
        );
        InterfaceMsg::record("^TEST.KEY2", "again").unwrap();
        let mut out = Vec::new();
        assert_eq!(InterfaceMsg::write(&mut out, "^TEST.KEY2").unwrap(), 1);
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "@@ ROOT:^TEST.KEY2\n@^TEST.KEY2\nagain\n"
        );
        let mut trace = Vec::new();
        InterfaceMsg::print_trace(&mut trace).unwrap();
        let trace = String::from_utf8(trace).unwrap();
        assert!(trace.contains("** DUP:^TEST.KEY2 ** again\n"));
        assert!(trace.contains("** MSG(NB=1): ^TEST.MISSING\n"));
        assert!(InterfaceMsg::is_key("^x") && !InterfaceMsg::is_key("x"));
        assert_eq!(
            InterfaceMsg::read_file("/nonexistent/msg/file").unwrap(),
            -1
        );
        let mut plain = InterfaceMsg::with_integer_text("%d:%s", 3, "a").unwrap();
        plain.destroy();
        assert_eq!(plain.value().unwrap(), "%d:%s");
        // The modes are process-wide: restore the native defaults for the other tests.
        InterfaceMsg::set_mode(false, false);
        InterfaceMsg::set_trace(true, false);
    }
}
