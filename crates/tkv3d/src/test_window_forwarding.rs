//! Test windows preserve neutral-window behavior while overriding only the observed native callback.
macro_rules! forward_neutral_window_query {
    ($backend:ident, $method:ident, $result:ty) => {
        fn $method(&self) -> $result {
            self.$backend.$method()
        }
    };
}
macro_rules! forward_neutral_window_surface {
    ($backend:ident) => {
        fn window_base(&self) -> &AspectWindowBase {
            self.$backend.window_base()
        }
        fn map(&self) {
            self.$backend.map();
        }
        fn unmap(&self) {
            self.$backend.unmap();
        }
        forward_neutral_window_query!($backend, is_mapped, bool);
        forward_neutral_window_query!($backend, do_resize, AspectTypeOfResize);
        forward_neutral_window_query!($backend, do_mapping, bool);
        forward_neutral_window_query!($backend, ratio, f64);
        forward_neutral_window_query!($backend, position, (i32, i32, i32, i32));
        forward_neutral_window_query!($backend, native_handle, AspectDrawable);
        forward_neutral_window_query!($backend, native_parent_handle, AspectDrawable);
        forward_neutral_window_query!($backend, native_fb_config, AspectFbConfig);
    };
}
