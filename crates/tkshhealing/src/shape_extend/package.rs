// Rust translation of ModelingAlgorithms/TKShHealing/ShapeExtend/ShapeExtend.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeExtend`: the package functions (the message loader and the status bit codes).
use super::status::ShapeExtendStatus;
use crate::sh_message::SHAPE_US;
use rustycad_tkernel::message::MsgFile;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::atomic::{AtomicBool, Ordering};

/// The message whose presence tells that the Shape Healing messages are loaded.
const PROBE_KEY: &str = "ShapeFix.FixSmallSolid.MSG0";

/// The `static bool init` of `ShapeExtend::Init`: set before the first load, so a later
/// call does nothing even when the first one failed.
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// `ShapeExtend`: the tools common to the Shape Healing packages.
///
/// ```
/// use rustycad_tkshhealing::shape_extend::{ShapeExtend, ShapeExtendStatus};
/// let flag = ShapeExtend::encode_status(ShapeExtendStatus::Done3);
/// assert_eq!(flag, 0x0004);
/// assert!(ShapeExtend::decode_status(flag, ShapeExtendStatus::Done));
/// assert!(!ShapeExtend::decode_status(flag, ShapeExtendStatus::Fail));
/// ```
pub struct ShapeExtend;

impl ShapeExtend {
    /// `Init`: loads the Shape Healing messages into the process-wide catalog once
    /// ([`MsgFile::global`]); Standard_ProgramError when they are still missing. Only the
    /// first call works, as the native function-local flag;
    /// `ShapeExtend::init()?`.
    pub fn init() -> Result<(), Failure> {
        if INITIALIZED.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        Self::load_messages(MsgFile::global())
    }

    /// The body of `Init` on `catalog`: unless the catalog knows the messages already,
    /// loads `SHAPE.<lang>` from the `CSF_SHMessage` directory, else the compiled
    /// [`SHAPE_US`] text; Standard_ProgramError when the check key is still missing.
    /// `ShapeExtend::load_messages(&catalog)?`.
    pub fn load_messages(catalog: &MsgFile) -> Result<(), Failure> {
        let key = AsciiString::new(PROBE_KEY);
        if catalog.has_msg(&key) {
            return Ok(());
        }
        if !catalog.load_from_env(Some("CSF_SHMessage"), "SHAPE", Some(""))? {
            // sizeof(SHMessage_SHAPE_us) - 1: the literal without its NUL.
            catalog.load_from_string(SHAPE_US, SHAPE_US.len() as i32)?;
        }
        if catalog.has_msg(&key) {
            return Ok(());
        }
        Err(Failure::with_message(
            "Standard_ProgramError",
            &format!(
                "Critical Error - message resources for ShapeExtend are invalid or undefined! (no {PROBE_KEY:?} message)"
            ),
        ))
    }

    /// `EncodeStatus`: the bit flag of `status`: `DONEi` is bit `i - 1`, `FAILi` is bit
    /// `i + 7`, `DONE` and `FAIL` are all eight bits of their byte, `OK` is 0;
    /// `ShapeExtend::encode_status(ShapeExtendStatus::Fail1) == 0x0100`.
    pub fn encode_status(status: ShapeExtendStatus) -> i32 {
        match status {
            ShapeExtendStatus::Ok => 0x0000,
            ShapeExtendStatus::Done => 0x00ff,
            ShapeExtendStatus::Fail => 0xff00,
            // DONE1..DONE8 are the enumerators 1..8 and FAIL1..FAIL8 the enumerators
            // 10..17: one bit each, in order, as the native switch spells them.
            done if (done as i32) < (ShapeExtendStatus::Done as i32) => 1 << (done as i32 - 1),
            fail => 1 << (fail as i32 - 2),
        }
    }

    /// `DecodeStatus`: whether `flag` holds the bit of `status`; for `OK`, whether `flag`
    /// holds no bit at all; `ShapeExtend::decode_status(0, ShapeExtendStatus::Ok)`.
    pub fn decode_status(flag: i32, status: ShapeExtendStatus) -> bool {
        if status == ShapeExtendStatus::Ok {
            return flag == 0;
        }
        (flag & Self::encode_status(status)) != 0
    }
}
