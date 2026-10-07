//! `ShapeProcess_ShapeContext::PrintStatistics`: the shell and face counts of the map,
//! sent to the messenger with the PrResult messages.
use super::ShapeProcessShapeContext;
use rustycad_tkernel::message::{Gravity, Messenger, Msg};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The counts of the map: shells to shells (SS) or to nothing (SN), faces to faces (FF),
/// to shells (FS) or to nothing (FN).
#[derive(Default)]
struct MapCounts {
    ss: i32,
    sn: i32,
    ff: i32,
    fs: i32,
    fn_: i32,
}

/// `Messenger::Send(Message_Msg, gravity)`: the text with the pending arguments filled
/// with UNKNOWN (`Message_Msg::Get`).
pub(in crate::shape_process) fn send(
    messenger: &Messenger,
    message: &mut Msg,
    gravity: Gravity,
) -> Result<(), Failure> {
    messenger.send_extended(message.get()?, gravity);
    Ok(())
}

pub(in crate::shape_process) fn message(key: &str) -> Msg {
    Msg::from_key(&AsciiString::from(key))
}

impl ShapeProcessShapeContext {
    /// `PrintStatistics()`: the mapping counts and the preparation ratios of shells and
    /// faces sent as information messages; `context.print_statistics()?`.
    pub fn print_statistics(&self) -> Result<(), Failure> {
        let counts = self.map_counts()?;
        let messenger = self.messenger();
        let send_key = |key: &str| send(&messenger, &mut message(key), Gravity::Info);
        send_key("PrResult.Print.MSG100")?;
        send_key("PrResult.Print.MSG50")?;
        let mut shell_result = message("PrResult.Print.MSG110");
        shell_result.arg_integer(counts.ss)?;
        send(&messenger, &mut shell_result, Gravity::Info)?;
        let mut no_result = message("PrResult.Print.MSG150");
        no_result.arg_integer(counts.sn)?;
        send(&messenger, &mut no_result, Gravity::Info)?;
        // Native resets both messages to their original text to reuse them for faces.
        let (first, second) = (
            shell_result.original().clone(),
            no_result.original().clone(),
        );
        shell_result.set(&first);
        no_result.set(&second);
        send_key("PrResult.Print.MSG55")?;
        let mut face_result = message("PrResult.Print.MSG115");
        face_result.arg_integer(counts.ff)?;
        send(&messenger, &mut face_result, Gravity::Info)?;
        shell_result.arg_integer(counts.fs)?;
        send(&messenger, &mut shell_result, Gravity::Info)?;
        no_result.arg_integer(counts.fn_)?;
        send(&messenger, &mut no_result, Gravity::Info)?;
        self.print_ratios(&messenger, &counts)
    }

    fn print_ratios(&self, messenger: &Messenger, c: &MapCounts) -> Result<(), Failure> {
        let (nb_shells, nb_faces) = (c.ss + c.sn, c.ff + c.fs + c.fn_);
        let shell_ratio = if nb_shells > 0 {
            1. * f64::from(nb_shells - c.sn) / f64::from(nb_shells)
        } else {
            1.
        };
        let face_ratio = if nb_faces > 0 {
            1. * f64::from(nb_faces - c.fn_) / f64::from(nb_faces)
        } else {
            1.
        };
        send(
            messenger,
            &mut message("PrResult.Print.MSG200"),
            Gravity::Info,
        )?;
        let mut shells = message("PrResult.Print.MSG205");
        shells.arg_integer((100. * shell_ratio) as i32)?;
        send(messenger, &mut shells, Gravity::Info)?;
        let mut faces = message("PrResult.Print.MSG210");
        faces.arg_integer((100. * face_ratio) as i32)?;
        send(messenger, &mut faces, Gravity::Info)
    }

    fn map_counts(&self) -> Result<MapCounts, Failure> {
        let mut counts = MapCounts::default();
        let map = self.map();
        for (key, value) in map.items() {
            match key.shape_type()? {
                ShapeEnum::Shell if value.is_null() => counts.sn += 1,
                ShapeEnum::Shell => counts.ss += 1,
                ShapeEnum::Face if value.is_null() => counts.fn_ += 1,
                ShapeEnum::Face if value.shape_type()? == ShapeEnum::Shell => counts.fs += 1,
                ShapeEnum::Face => counts.ff += 1,
                _ => {}
            }
        }
        Ok(counts)
    }
}
