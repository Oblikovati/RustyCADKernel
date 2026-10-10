//! Checks the hand-written Tcl/Tk declarations against the installed C headers.
mod header_probe;

use header_probe::{
    c_type, callback_check, compile, constant_check, function_check, offset_checks, size_check,
    translation_unit,
};
use rustycad_tcl::sys::{self, Signature};

const ALL_FUNCTION_TABLES: &[&[Signature]] = &[
    sys::TCL_SIGNATURES,
    #[cfg(unix)]
    sys::TCL_UNIX_SIGNATURES,
    sys::TCL_VARIADIC_SIGNATURES,
    sys::TK_SIGNATURES,
];

/// Every Tcl/Tk name OCCT `src/Draw` calls or names (grep of `\b(Tcl|Tk)_[A-Z]\w+` over
/// the .cxx/.hxx/.mm/.pxx/.lxx files of the pinned revision), minus type names.
const OCCT_DRAW_FUNCTIONS: &[&str] = &[
    "Tcl_Exit",
    "Tk_Init",
    "Tcl_GetStdChannel",
    "Tcl_Eval",
    "Tcl_GetStringResult",
    #[cfg(unix)]
    "Tcl_CreateFileHandler",
    "Tcl_AppendResult",
    "Tk_MainWindow",
    "Tcl_SetVar",
    "Tcl_DStringValue",
    "Tcl_DStringAppend",
    "Tk_SetAppName",
    "Tk_MainLoop",
    "Tcl_Write",
    "Tcl_RegisterChannel",
    "Tcl_Init",
    "Tcl_GetVar",
    "Tcl_DStringInit",
    "Tcl_DStringFree",
    "Tcl_DoOneEvent",
    "Tk_GetUid",
    "Tcl_StaticPackage",
    "Tcl_SetVar2",
    "Tcl_SetResult",
    "Tcl_SetChannelOption",
    "Tcl_CreateExitHandler",
    "Tcl_CreateChannelHandler",
    "Tk_Name",
    "Tk_GeometryRequest",
    "Tcl_VarTraceInfo",
    "Tcl_UntraceVar",
    "Tcl_Flush",
    "Tcl_FindExecutable",
    "Tcl_DeleteInterp",
    "Tcl_CommandComplete",
    "Tk_Main",
    "Tk_HandleEvent",
    "Tk_GetNumMainWindows",
    "Tk_Display",
    #[cfg(unix)]
    "Tk_CreateFileHandler",
    "Tcl_UtfToUniCharDString",
    "Tcl_UnsetVar",
    "Tcl_UniCharLen",
    "Tcl_TraceVar",
    "Tcl_SetStdChannel",
    "Tcl_ResetResult",
    "Tcl_RecordAndEval",
    "Tcl_PutEnv",
    "Tcl_Panic",
    "Tcl_MakeFileChannel",
    "Tcl_Gets",
    "Tcl_ExternalToUtfDString",
    "Tcl_EvalFile",
    #[cfg(unix)]
    "Tcl_DeleteFileHandler",
    "Tcl_DeleteCommand",
    "Tcl_DeleteChannelHandler",
    "Tcl_CreateInterp",
    "Tcl_CreateCommand",
    "Tcl_Close",
    "Tcl_AppendElement",
    "Tcl_AddErrorInfo",
];

/// Names that are C macros: the C expansion and the Rust item that carries it.
const MACRO_EXPANSIONS: &[(&str, &str)] = &[
    (
        "Tcl_DStringValue",
        "field Tcl_DString::string (DString::value)",
    ),
    ("Tk_Name", "sys::Tk_Name reads Tk_FakeWinHead::nameUid"),
    (
        "Tk_Display",
        "sys::Tk_Display reads Tk_FakeWinHead::display",
    ),
    ("Tk_Main", "Tk_MainEx(argc, argv, proc, Tcl_CreateInterp())"),
    #[cfg(unix)]
    ("Tk_CreateFileHandler", "Tcl_CreateFileHandler"),
];

#[test]
fn every_declaration_matches_the_installed_headers() {
    let source = translation_unit(ALL_FUNCTION_TABLES);
    if let Err(diagnostics) = compile(&source) {
        panic!("declarations disagree with tcl.h/tk.h:\n{diagnostics}\n--- probe ---\n{source}");
    }
}

#[test]
fn every_function_occt_draw_uses_is_declared() {
    let declared: Vec<&str> = ALL_FUNCTION_TABLES
        .iter()
        .flat_map(|t| t.iter().map(|s| s.name))
        .collect();
    for name in OCCT_DRAW_FUNCTIONS {
        let is_macro = MACRO_EXPANSIONS
            .iter()
            .any(|(macro_name, _)| macro_name == name);
        assert!(
            declared.contains(name) || is_macro,
            "{name} is used by OCCT Draw but not declared"
        );
    }
}

#[test]
fn the_checker_rejects_a_wrong_return_type() {
    let wrong = Signature {
        name: "Tcl_Eval",
        arguments: &["*mut Tcl_Interp", "*const c_char"],
        returns: "c_uint",
        variadic: false,
    };
    assert!(compile_with(&function_check(&wrong)).is_err());
}

#[test]
fn the_checker_rejects_a_missing_const() {
    let wrong = Signature {
        name: "Tcl_Eval",
        arguments: &["*mut Tcl_Interp", "*mut c_char"],
        returns: "c_int",
        variadic: false,
    };
    assert!(compile_with(&function_check(&wrong)).is_err());
}

#[test]
fn the_checker_rejects_a_wrong_callback_argument() {
    let wrong = Signature {
        name: "Tcl_FileProc",
        arguments: &["ClientData", "c_uint"],
        returns: "()",
        variadic: false,
    };
    assert!(compile_with(&callback_check(&wrong)).is_err());
}

#[test]
fn the_checker_rejects_a_wrong_constant() {
    assert!(compile_with(&constant_check("TCL_GLOBAL_ONLY", 2)).is_err());
    assert!(compile_with(&constant_check("TCL_GLOBAL_ONLY", 1)).is_ok());
}

#[test]
fn the_checker_rejects_a_lost_variadic_marker() {
    let wrong = Signature {
        name: "Tcl_AppendResult",
        arguments: &["*mut Tcl_Interp"],
        returns: "()",
        variadic: false,
    };
    assert!(compile_with(&function_check(&wrong)).is_err());
}

#[test]
fn the_checker_rejects_a_wrong_layout() {
    assert!(compile_with(&offset_checks("Tk_FakeWin", &[("nameUid", 0)])).is_err());
    assert!(compile_with(&size_check("Tcl_DString", 8)).is_err());
}

#[test]
fn rust_types_translate_to_c_declarators() {
    assert_eq!(c_type("*mut *const c_char"), "char const * *");
    assert_eq!(c_type("Option<Tcl_CmdProc>"), "Tcl_CmdProc *");
    assert_eq!(c_type("Tcl_FreeProcPointer"), "Tcl_FreeProc *");
}

/// Compiles one check line after the headers.
fn compile_with(check: &str) -> Result<(), String> {
    compile(&format!(
        "#include <stddef.h>\n#include <tcl.h>\n#include <tk.h>\n{check}"
    ))
}

#[test]
fn tk_window_xid_obeys_the_installed_platform_abi() {
    let expected = if cfg!(all(windows, target_pointer_width = "64")) {
        8
    } else {
        std::mem::size_of::<std::ffi::c_ulong>()
    };
    assert_eq!(std::mem::size_of::<sys::Window>(), expected);
    assert!(compile_with(&size_check("Window", expected)).is_ok());
}
