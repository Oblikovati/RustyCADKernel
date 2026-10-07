//! Runs the Rust `drawexe` on the DRAW oracle scripts and compares its output with the
//! output native OCCT DRAWEXE produced (`tests/oracle/*.expected`, recorded by
//! `tools/verify_tkdraw_oracle.py --record`, which also compares both executables live).
use std::path::{Path, PathBuf};
use std::process::Command;

/// The oracle scripts, sorted.
fn oracle_scripts() -> Vec<PathBuf> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("oracle");
    let mut scripts: Vec<PathBuf> = std::fs::read_dir(&directory)
        .expect("the oracle directory exists")
        .map(|entry| entry.expect("readable entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "tcl"))
        .collect();
    scripts.sort();
    scripts
}

/// The output of `drawexe -b -f script` (stdout then stderr), with an empty DRAWDEFAULT.
fn run_drawexe(script: &Path, empty_default: &Path) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_drawexe"))
        .args(["-b", "-f"])
        .arg(script.file_name().expect("a script file name"))
        .current_dir(script.parent().expect("a script directory"))
        .env("DRAWDEFAULT", empty_default)
        .env_remove("CSF_OCCTResourcePath")
        .output()
        .expect("drawexe runs");
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    text
}

#[test]
fn drawexe_matches_native_drawexe_on_every_oracle_script() {
    let empty_default =
        std::env::temp_dir().join(format!("tkdraw_empty_default_{}.tcl", std::process::id()));
    std::fs::write(&empty_default, "").expect("write the empty DRAWDEFAULT");
    let scripts = oracle_scripts();
    assert!(!scripts.is_empty(), "no oracle scripts found");
    let mut mismatches: Vec<String> = Vec::new();
    for script in &scripts {
        let expected = std::fs::read_to_string(script.with_extension("expected"))
            .expect("a recorded native output");
        let actual = run_drawexe(script, &empty_default);
        if actual != expected {
            mismatches.push(format!(
                "{}:\n--- native\n{expected}\n--- rust\n{actual}",
                script.display()
            ));
        }
    }
    let _ = std::fs::remove_file(&empty_default);
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}
