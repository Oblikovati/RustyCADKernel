use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Handle, RuntimeTyped, StandardTransient, Transient};
#[repr(C)]
#[derive(Default)]
struct TransientRoot {
    parent: StandardTransient,
}
impl_transient!(TransientRoot, "TransientRoot", StandardTransient, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass00_50 {
    parent: TransientRoot,
}
impl_transient!(QaClass00_50, "QaClass00_50", TransientRoot, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass01_50 {
    parent: QaClass00_50,
}
impl_transient!(QaClass01_50, "QaClass01_50", QaClass00_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass02_50 {
    parent: QaClass01_50,
}
impl_transient!(QaClass02_50, "QaClass02_50", QaClass01_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass03_50 {
    parent: QaClass02_50,
}
impl_transient!(QaClass03_50, "QaClass03_50", QaClass02_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass04_50 {
    parent: QaClass03_50,
}
impl_transient!(QaClass04_50, "QaClass04_50", QaClass03_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass05_50 {
    parent: QaClass04_50,
}
impl_transient!(QaClass05_50, "QaClass05_50", QaClass04_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass06_50 {
    parent: QaClass05_50,
}
impl_transient!(QaClass06_50, "QaClass06_50", QaClass05_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass07_50 {
    parent: QaClass06_50,
}
impl_transient!(QaClass07_50, "QaClass07_50", QaClass06_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass08_50 {
    parent: QaClass07_50,
}
impl_transient!(QaClass08_50, "QaClass08_50", QaClass07_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass09_50 {
    parent: QaClass08_50,
}
impl_transient!(QaClass09_50, "QaClass09_50", QaClass08_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass10_50 {
    parent: QaClass09_50,
}
impl_transient!(QaClass10_50, "QaClass10_50", QaClass09_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass11_50 {
    parent: QaClass10_50,
}
impl_transient!(QaClass11_50, "QaClass11_50", QaClass10_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass12_50 {
    parent: QaClass11_50,
}
impl_transient!(QaClass12_50, "QaClass12_50", QaClass11_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass13_50 {
    parent: QaClass12_50,
}
impl_transient!(QaClass13_50, "QaClass13_50", QaClass12_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass14_50 {
    parent: QaClass13_50,
}
impl_transient!(QaClass14_50, "QaClass14_50", QaClass13_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass15_50 {
    parent: QaClass14_50,
}
impl_transient!(QaClass15_50, "QaClass15_50", QaClass14_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass16_50 {
    parent: QaClass15_50,
}
impl_transient!(QaClass16_50, "QaClass16_50", QaClass15_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass17_50 {
    parent: QaClass16_50,
}
impl_transient!(QaClass17_50, "QaClass17_50", QaClass16_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass18_50 {
    parent: QaClass17_50,
}
impl_transient!(QaClass18_50, "QaClass18_50", QaClass17_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass19_50 {
    parent: QaClass18_50,
}
impl_transient!(QaClass19_50, "QaClass19_50", QaClass18_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass20_50 {
    parent: QaClass19_50,
}
impl_transient!(QaClass20_50, "QaClass20_50", QaClass19_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass21_50 {
    parent: QaClass20_50,
}
impl_transient!(QaClass21_50, "QaClass21_50", QaClass20_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass22_50 {
    parent: QaClass21_50,
}
impl_transient!(QaClass22_50, "QaClass22_50", QaClass21_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass23_50 {
    parent: QaClass22_50,
}
impl_transient!(QaClass23_50, "QaClass23_50", QaClass22_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass24_50 {
    parent: QaClass23_50,
}
impl_transient!(QaClass24_50, "QaClass24_50", QaClass23_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass25_50 {
    parent: QaClass24_50,
}
impl_transient!(QaClass25_50, "QaClass25_50", QaClass24_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass26_50 {
    parent: QaClass25_50,
}
impl_transient!(QaClass26_50, "QaClass26_50", QaClass25_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass27_50 {
    parent: QaClass26_50,
}
impl_transient!(QaClass27_50, "QaClass27_50", QaClass26_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass28_50 {
    parent: QaClass27_50,
}
impl_transient!(QaClass28_50, "QaClass28_50", QaClass27_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass29_50 {
    parent: QaClass28_50,
}
impl_transient!(QaClass29_50, "QaClass29_50", QaClass28_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass30_50 {
    parent: QaClass29_50,
}
impl_transient!(QaClass30_50, "QaClass30_50", QaClass29_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass31_50 {
    parent: QaClass30_50,
}
impl_transient!(QaClass31_50, "QaClass31_50", QaClass30_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass32_50 {
    parent: QaClass31_50,
}
impl_transient!(QaClass32_50, "QaClass32_50", QaClass31_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass33_50 {
    parent: QaClass32_50,
}
impl_transient!(QaClass33_50, "QaClass33_50", QaClass32_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass34_50 {
    parent: QaClass33_50,
}
impl_transient!(QaClass34_50, "QaClass34_50", QaClass33_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass35_50 {
    parent: QaClass34_50,
}
impl_transient!(QaClass35_50, "QaClass35_50", QaClass34_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass36_50 {
    parent: QaClass35_50,
}
impl_transient!(QaClass36_50, "QaClass36_50", QaClass35_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass37_50 {
    parent: QaClass36_50,
}
impl_transient!(QaClass37_50, "QaClass37_50", QaClass36_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass38_50 {
    parent: QaClass37_50,
}
impl_transient!(QaClass38_50, "QaClass38_50", QaClass37_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass39_50 {
    parent: QaClass38_50,
}
impl_transient!(QaClass39_50, "QaClass39_50", QaClass38_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass40_50 {
    parent: QaClass39_50,
}
impl_transient!(QaClass40_50, "QaClass40_50", QaClass39_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass41_50 {
    parent: QaClass40_50,
}
impl_transient!(QaClass41_50, "QaClass41_50", QaClass40_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass42_50 {
    parent: QaClass41_50,
}
impl_transient!(QaClass42_50, "QaClass42_50", QaClass41_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass43_50 {
    parent: QaClass42_50,
}
impl_transient!(QaClass43_50, "QaClass43_50", QaClass42_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass44_50 {
    parent: QaClass43_50,
}
impl_transient!(QaClass44_50, "QaClass44_50", QaClass43_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass45_50 {
    parent: QaClass44_50,
}
impl_transient!(QaClass45_50, "QaClass45_50", QaClass44_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass46_50 {
    parent: QaClass45_50,
}
impl_transient!(QaClass46_50, "QaClass46_50", QaClass45_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass47_50 {
    parent: QaClass46_50,
}
impl_transient!(QaClass47_50, "QaClass47_50", QaClass46_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass48_50 {
    parent: QaClass47_50,
}
impl_transient!(QaClass48_50, "QaClass48_50", QaClass47_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass49_50 {
    parent: QaClass48_50,
}
impl_transient!(QaClass49_50, "QaClass49_50", QaClass48_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass50_50 {
    parent: QaClass49_50,
}
impl_transient!(QaClass50_50, "QaClass50_50", QaClass49_50, parent);
#[repr(C)]
#[derive(Default)]
struct QaClass50_50Anon {
    parent: QaClass49_50,
}
mod qa_namespace {
    #[repr(C)]
    #[derive(Default)]
    pub(super) struct QaClass50_50 {
        pub(super) parent: super::QaClass49_50,
    }
}
