//! Behaviour of the NCollection utility ports: UTF iterator/string, cursor and items views.
//! Bit-exact agreement with OCCT is checked by the `ncoll_utilities_*` differential probe.

mod ncollection_utilities {
    mod cursor_and_views;
    mod utf_iterator;
    mod utf_string;
}
