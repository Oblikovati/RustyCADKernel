use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Handle, StandardTransient};
#[repr(C)]
#[derive(Default)]
struct TestBase {
    parent: StandardTransient,
}
impl_transient!(TestBase, "TestBase", StandardTransient, parent);
#[repr(C)]
#[derive(Default)]
struct TestDerived {
    parent: TestBase,
}
impl_transient!(TestDerived, "TestDerived", TestBase, parent);
#[repr(C)]
#[derive(Default)]
struct TestOther {
    parent: StandardTransient,
}
impl_transient!(TestOther, "TestOther", StandardTransient, parent);
