//! NCollection_Vec2/Vec3/Vec4 and NCollection_Mat3/Mat4 oracle counterpart; prints the same
//! token stream as tools/oracle/tkmath_ncollection_vec_probe.cxx.
use rustycad_tkernel::ncollection::{Vec2, Vec3, Vec4};
use rustycad_tkernel::standard::dump::DumpStream;
mod matrices;
mod vectors;
pub use matrices::ncollection_mat_case;
pub use vectors::ncollection_vec_case;

/// One printed element: float kinds as the bits of the exact double promotion, ints as text.
pub trait Printed: Copy {
    fn print(self);
}
impl Printed for f32 {
    fn print(self) {
        print!("{} ", f64::from(self).to_bits());
    }
}
impl Printed for f64 {
    fn print(self) {
        print!("{} ", self.to_bits());
    }
}
impl Printed for i32 {
    fn print(self) {
        print!("{self} ");
    }
}
pub fn flag(value: bool) {
    print!("{} ", i32::from(value));
}
pub fn vec2<T: Printed>(v: Vec2<T>) {
    v.data().into_iter().for_each(Printed::print);
}
pub fn vec3<T: Printed>(v: Vec3<T>) {
    v.data().into_iter().for_each(Printed::print);
}
pub fn vec4<T: Printed>(v: Vec4<T>) {
    v.data().into_iter().for_each(Printed::print);
}
/// Prints a JSON dump followed by the native probe's ` | ` separator.
pub fn json(write: impl FnOnce(&mut DumpStream) -> std::io::Result<()>) {
    let mut dump = DumpStream::default();
    write(&mut dump).unwrap();
    print!("{} | ", dump.text());
}
/// Prints `DumpJson` output for float kinds; integer vectors print nothing, as the native
/// probe skips them.
pub trait JsonDump {
    fn dump(&self);
}
macro_rules! json_dump {
    ($($vector:ident),+ ; float) => {
        $(
            impl JsonDump for $vector<f32> {
                fn dump(&self) {
                    json(|d| self.dump_json(d));
                }
            }
            impl JsonDump for $vector<f64> {
                fn dump(&self) {
                    json(|d| self.dump_json(d));
                }
            }
            impl JsonDump for $vector<i32> {
                fn dump(&self) {}
            }
        )+
    };
}
json_dump!(Vec2, Vec3, Vec4; float);
