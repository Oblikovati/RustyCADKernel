//! The real Tk window contract runs on the OS main thread, including under nextest.
use rustycad_tcl::{Completion, Interp, TkApp, tk_init};
fn main() {
    let interp = Interp::new();
    // Tk_Init runs tk.tcl, which needs Tcl's own init.tcl (tcl_findLibrary), as in DRAW.
    assert_eq!(interp.init(), Completion::Ok, "{}", interp.result());
    let app = match tk_init(&interp) {
        Ok(app) => app,
        Err(reason) => {
            assert!(
                std::env::var_os("DISPLAY").is_none(),
                "Tk_Init failed with a display: {reason}"
            );
            assert!(reason.contains("display"), "{reason}");
            return;
        }
    };
    interp.eval("wm withdraw .").unwrap();
    verify_window(&app);
    interp.eval("destroy .").unwrap();
}
fn verify_window(app: &TkApp<'_>) {
    let window = app.main_window().expect("Tk_Init creates the main window");
    let name = window.set_app_name("MartensiteTclTest").unwrap();
    assert!(name.starts_with("MartensiteTclTest"), "{name}");
    window.set_name_uid(app.get_uid(&name).unwrap());
    assert_eq!(window.name(), name);
    assert!(std::ptr::eq(
        app.get_uid("x").unwrap(),
        app.get_uid("x").unwrap()
    ));
    assert!(app.main_window_count() >= 1);
    assert!(!window.display().is_null());
    window.geometry_request(200, 200);
}
