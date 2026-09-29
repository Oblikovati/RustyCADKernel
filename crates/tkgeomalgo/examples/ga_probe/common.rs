//! Reader extensions and printers shared by the TKGeomAlgo probe; the counterpart of
//! tools/oracle/tkgeomalgo_probe_common.cxx.
pub use super::print::{Reader, attempt, end, flag, int, real, show};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Direction2, Direction3, Point2, Point3, Xy, Xyz};

pub fn xyz(value: Xyz) {
    real(value.x());
    real(value.y());
    real(value.z());
}
pub fn pnt(value: Point3) {
    xyz(value.xyz());
}
pub fn dir(value: Direction3) {
    xyz(value.xyz());
}
pub fn xy(value: Xy) {
    real(value.x());
    real(value.y());
}
pub fn pnt2d(value: Point2) {
    xy(value.xy());
}

/// `ga_text`: a text as one `t<hex bytes>` token.
pub fn text(bytes: &[u8]) {
    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    print!("t{hex} ");
}
/// `ga_captured`: the text a member wrote, then its exception type, if any.
pub fn captured(body: impl FnOnce(&mut String) -> Result<(), Failure>) {
    let mut console = String::new();
    let result = body(&mut console);
    text(console.as_bytes());
    if let Err(error) = result {
        print!("{} ", error.exception_type());
    }
}

/// The GaReader members of tools/oracle/tkgeomalgo_probe_common.cxx.
impl Reader<'_> {
    pub fn xyz(&mut self) -> Xyz {
        let (x, y) = (self.real(), self.real());
        Xyz::new(x, y, self.real())
    }
    pub fn pnt(&mut self) -> Point3 {
        let (x, y) = (self.real(), self.real());
        Point3::new(x, y, self.real())
    }
    pub fn dir(&mut self) -> Direction3 {
        let (x, y) = (self.real(), self.real());
        Direction3::new(x, y, self.real()).expect("probe direction")
    }
    pub fn dir2d(&mut self) -> Direction2 {
        let x = self.real();
        Direction2::new(x, self.real()).expect("probe direction")
    }
    pub fn pnt2d(&mut self) -> Point2 {
        let x = self.real();
        Point2::new(x, self.real())
    }
}
