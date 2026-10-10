//! Generates a C translation unit from the `sys` tables and compiles it against the
//! installed `tcl.h`/`tk.h`, so every hand-written declaration is checked by the C
//! compiler: a function is assigned to a pointer of the Rust-declared type, a callback
//! typedef receives a function declared with the Rust argument list, constants and
//! struct layouts are `_Static_assert`ed.
use rustycad_tcl::sys::{self, Signature};
use std::io::Write;
use std::process::{Command, Stdio};

/// Translates the `stringify!` text of a Rust FFI type into a C type.
pub fn c_type(rust: &str) -> String {
    let rust = rust.replace(' ', "");
    if let Some(pointee) = rust.strip_prefix("*mut") {
        return format!("{} *", c_type(pointee));
    }
    if let Some(pointee) = rust.strip_prefix("*const") {
        return format!("{} const *", c_type(pointee));
    }
    if let Some(callback) = rust
        .strip_prefix("Option<")
        .and_then(|r| r.strip_suffix('>'))
    {
        return format!("{callback} *");
    }
    scalar_c_type(&rust).to_string()
}

/// Maps a non-pointer Rust type name to C; an unknown name is a test failure.
fn scalar_c_type(rust: &str) -> &str {
    match rust {
        "c_int" => "int",
        "c_uint" => "unsigned int",
        "c_char" => "char",
        "c_ushort" => "unsigned short",
        "c_ulong" => "unsigned long",
        "c_void" | "()" | "!" => "void",
        "Tcl_FreeProcPointer" => "Tcl_FreeProc *",
        "ClientData" | "Tcl_Interp" | "Tcl_Channel" | "Tcl_Command" | "Tcl_Encoding"
        | "Tcl_DString" | "Tcl_UniChar" | "Tk_Window" | "Tk_Uid" | "XEvent" | "Display" => rust,
        other => panic!("no C mapping for Rust FFI type {other:?}; add it to scalar_c_type"),
    }
}

/// The C parameter list of a signature (`void` when empty).
fn c_parameters(signature: &Signature) -> String {
    let mut parameters: Vec<String> = signature.arguments.iter().map(|a| c_type(a)).collect();
    if signature.variadic {
        parameters.push("...".to_string());
    }
    if parameters.is_empty() {
        "void".to_string()
    } else {
        parameters.join(", ")
    }
}

/// `static RET (*p)(ARGS) = NAME;` for one function.
pub fn function_check(signature: &Signature) -> String {
    format!(
        "static {} (*rustycad_fp_{name})({}) = {name};\n",
        c_type(signature.returns),
        c_parameters(signature),
        name = signature.name
    )
}

/// Declares a function with the Rust argument list and stores it in the C typedef.
pub fn callback_check(signature: &Signature) -> String {
    format!(
        "extern {} rustycad_cb_{name}({});\nstatic {name} *rustycad_cbp_{name} = rustycad_cb_{name};\n",
        c_type(signature.returns),
        c_parameters(signature),
        name = signature.name
    )
}

/// `_Static_assert` of one constant.
pub fn constant_check(name: &str, value: i64) -> String {
    format!("_Static_assert(({name}) == ({value}LL), \"{name} == {value}\");\n")
}

/// `_Static_assert` of a struct size.
pub fn size_check(c_struct: &str, size: usize) -> String {
    format!("_Static_assert(sizeof({c_struct}) == {size}, \"sizeof {c_struct}\");\n")
}

/// `_Static_assert`s of field offsets.
pub fn offset_checks(c_struct: &str, offsets: &[(&str, usize)]) -> String {
    let check = |(field, offset): &(&str, usize)| {
        format!(
            "_Static_assert(offsetof({c_struct}, {field}) == {offset}, \"{c_struct}.{field}\");\n"
        )
    };
    offsets.iter().map(check).collect()
}

/// The full translation unit for the given function tables.
pub fn translation_unit(functions: &[&[Signature]]) -> String {
    let mut text = String::from(
        "#include <stddef.h>\n#include <stdint.h>\n#include <tcl.h>\n#include <tk.h>\n",
    );
    text += &sys::CALLBACK_SIGNATURES
        .iter()
        .map(callback_check)
        .collect::<String>();
    for table in functions {
        text += &table.iter().map(function_check).collect::<String>();
    }
    text += &sys::CONSTANT_VALUES
        .iter()
        .map(|(n, v)| constant_check(n, *v))
        .collect::<String>();
    text += "_Static_assert((intptr_t)TCL_VOLATILE == 1 && (intptr_t)TCL_STATIC == 0, \"free procs\");\n";
    text += &dstring_layout();
    text += &tk_window_layout();
    text
}

fn dstring_layout() -> String {
    use std::mem::{offset_of, size_of};
    use sys::Tcl_DString as D;
    size_check("Tcl_DString", size_of::<D>())
        + &offset_checks(
            "Tcl_DString",
            &[
                ("string", offset_of!(D, string)),
                ("length", offset_of!(D, length)),
                ("spaceAvl", offset_of!(D, spaceAvl)),
                ("staticSpace", offset_of!(D, staticSpace)),
            ],
        )
}

fn tk_window_layout() -> String {
    use std::mem::offset_of;
    use sys::Tk_FakeWinHead as W;
    let offsets = [
        ("display", offset_of!(W, display)),
        ("screenNum", offset_of!(W, screenNum)),
        ("visual", offset_of!(W, visual)),
        ("depth", offset_of!(W, depth)),
        ("window", offset_of!(W, window)),
        ("parentPtr", offset_of!(W, parentPtr)),
        ("pathName", offset_of!(W, pathName)),
        ("nameUid", offset_of!(W, nameUid)),
    ];
    // Only the prefix is mirrored, so the size is not compared; the offsets are.
    offset_checks("Tk_FakeWin", &offsets)
}

/// Compiles `source` with `-fsyntax-only -Werror`; returns the compiler diagnostics on failure.
pub fn compile(source: &str) -> Result<(), String> {
    let include = format!("-I{}", env!("RUSTYCAD_TCL_INCLUDE"));
    let compiler = if cfg!(windows) { "clang" } else { "cc" };
    let mut child = Command::new(compiler)
        .args(["-x", "c", "-std=c11", "-fsyntax-only", "-Wall", "-Werror"])
        .args(["-Wincompatible-pointer-types", "-Wno-unused-variable"])
        .arg(include)
        .arg("-")
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|error| {
            panic!("C compiler {compiler}: expected runnable header checker: {error}")
        });
    child
        .stdin
        .take()
        .expect("piped")
        .write_all(source.as_bytes())
        .expect("write the probe");
    let output = child.wait_with_output().expect("header compiler finishes");
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}
