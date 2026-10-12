//! Keep renderer forwarding backed by the shared TKService view protocol.
//! The alias preserves existing callers while independently packaged toolkit tests use the same body.
pub(crate) use rustycad_tkservice::martensite_retained_cview_methods as retained_view_methods;
