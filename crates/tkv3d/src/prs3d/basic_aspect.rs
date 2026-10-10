//! `Prs3d_BasicAspect`, the abstract root of the Prs3d aspects, and the shared helpers of the
//! aspect classes.
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Handle, StandardTransient, Transient};
use rustycad_tkservice::graphic3d::Graphic3dAspectsSource;
use rustycad_tkservice::json_dump::field_dumped;
use std::io;

/// An aspect of a presentation; every aspect class dumps itself.
pub trait Prs3dBasicAspect: Transient {
    /// `DumpJson`; `aspect.dump_json(&mut stream, -1)?`.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()>;
}
rustycad_tkernel::impl_transient_interface!(
    dyn Prs3dBasicAspect,
    "Prs3d_BasicAspect",
    StandardTransient
);

/// The native `*theTarget->Aspect() = *theSource->Aspect()`: copies every Graphic3d_Aspects
/// field of `source` into `target`. The source is read before the target is locked, so the
/// same object on both sides is a no-op instead of a deadlock.
/// Example: `prs3d_copy_aspects(&*own.aspect(), &*linked.aspect())`.
pub fn prs3d_copy_aspects(
    target: &dyn Graphic3dAspectsSource,
    source: &dyn Graphic3dAspectsSource,
) {
    let fields = source.aspects_base().fields.snapshot();
    *target.aspects_base().fields.write() = fields;
}

/// `OCCT_DUMP_FIELD_VALUES_DUMPED` of an optional nested aspect: skipped for a null handle.
pub(crate) fn dump_aspect_field<
    T: Prs3dBasicAspect + rustycad_tkernel::standard::RuntimeTyped + ?Sized,
>(
    stream: &mut DumpStream,
    depth: i32,
    field: &str,
    aspect: &Handle<T>,
) -> io::Result<()> {
    let Some(aspect) = aspect.get() else {
        return Ok(());
    };
    field_dumped(stream, depth, field, &|o, d| aspect.dump_json(o, d))
}

/// `OCCT_DUMP_FIELD_VALUES_DUMPED` of an optional Graphic3d aspects handle.
pub(crate) fn dump_graphic_aspects<
    T: Graphic3dAspectsSource + rustycad_tkernel::standard::RuntimeTyped + ?Sized,
>(
    stream: &mut DumpStream,
    depth: i32,
    field: &str,
    aspect: &Handle<T>,
) -> io::Result<()> {
    let Some(aspect) = aspect.get() else {
        return Ok(());
    };
    field_dumped(stream, depth, field, &|o, d| aspect.dump_json(o, d))
}
