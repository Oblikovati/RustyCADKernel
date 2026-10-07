//! HLRBRep_BiPnt2D.hxx and HLRBRep_BiPoint.hxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkmath::gp::{Point2, Point3, Xy};

/// The four regularity flags of a polyhedral segment.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[allow(missing_docs)] // The native argument names, documented on the type.
pub struct BiPntFlags {
    pub reg1: bool,
    pub regn: bool,
    pub outl: bool,
    pub intl: bool,
}

macro_rules! regularity_accessors {
    () => {
        /// Shape(); `segment.shape()`.
        pub fn shape(&self) -> &Shape {
            &self.shape
        }
        /// Shape(S); `segment.set_shape(&s)`.
        pub fn set_shape(&mut self, s: &Shape) {
            self.shape = s.clone();
        }
        /// Rg1Line(); `segment.rg1_line()`.
        pub fn rg1_line(&self) -> bool {
            self.flags.reg1
        }
        /// Rg1Line(B); `segment.set_rg1_line(true)`.
        pub fn set_rg1_line(&mut self, b: bool) {
            self.flags.reg1 = b;
        }
        /// RgNLine(); `segment.rgn_line()`.
        pub fn rgn_line(&self) -> bool {
            self.flags.regn
        }
        /// RgNLine(B); `segment.set_rgn_line(true)`.
        pub fn set_rgn_line(&mut self, b: bool) {
            self.flags.regn = b;
        }
        /// OutLine(); `segment.out_line()`.
        pub fn out_line(&self) -> bool {
            self.flags.outl
        }
        /// OutLine(B); `segment.set_out_line(true)`.
        pub fn set_out_line(&mut self, b: bool) {
            self.flags.outl = b;
        }
        /// IntLine(); `segment.int_line()`.
        pub fn int_line(&self) -> bool {
            self.flags.intl
        }
        /// IntLine(B); `segment.set_int_line(true)`.
        pub fn set_int_line(&mut self, b: bool) {
            self.flags.intl = b;
        }
    };
}

/// HLRBRep_BiPnt2D: a 2D segment of a polyhedral result with the shape it comes from
/// and its regularity flags.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkhlr::hlr_brep::{BiPnt2D, BiPntFlags};
/// let s = BiPnt2D::new(0.0, 0.0, 1.0, 2.0, &Shape::default(), BiPntFlags::default());
/// assert_eq!(s.p2().y(), 2.0);
/// assert!(!s.out_line());
/// ```
#[derive(Clone, Debug, Default)]
pub struct BiPnt2D {
    p1: Point2,
    p2: Point2,
    shape: Shape,
    flags: BiPntFlags,
}

impl BiPnt2D {
    /// HLRBRep_BiPnt2D(x1, y1, x2, y2, S, reg1, regn, outl, intl);
    /// `BiPnt2D::new(0.0, 0.0, 1.0, 1.0, &shape, flags)`.
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64, s: &Shape, flags: BiPntFlags) -> Self {
        Self {
            p1: Point2::new(x1, y1),
            p2: Point2::new(x2, y2),
            shape: s.clone(),
            flags,
        }
    }

    /// HLRBRep_BiPnt2D(thePoint1, thePoint2, S, reg1, regn, outl, intl);
    /// `BiPnt2D::from_xy(a, b, &shape, flags)`.
    pub fn from_xy(p1: Xy, p2: Xy, s: &Shape, flags: BiPntFlags) -> Self {
        Self {
            p1: Point2::from(p1),
            p2: Point2::from(p2),
            shape: s.clone(),
            flags,
        }
    }

    /// P1(); `segment.p1()`.
    pub fn p1(&self) -> Point2 {
        self.p1
    }

    /// P2(); `segment.p2()`.
    pub fn p2(&self) -> Point2 {
        self.p2
    }

    regularity_accessors!();
}

/// HLRBRep_BiPoint: a 3D segment of a polyhedral result with the shape it comes from
/// and its regularity flags.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkhlr::hlr_brep::{BiPntFlags, BiPoint};
/// let s = BiPoint::new([0.0, 0.0, 0.0], [1.0, 2.0, 3.0], &Shape::default(), BiPntFlags::default());
/// assert_eq!(s.p2().z(), 3.0);
/// ```
#[derive(Clone, Debug, Default)]
pub struct BiPoint {
    p1: Point3,
    p2: Point3,
    shape: Shape,
    flags: BiPntFlags,
}

impl BiPoint {
    /// HLRBRep_BiPoint(x1, y1, z1, x2, y2, z2, S, reg1, regn, outl, intl);
    /// `BiPoint::new([0.0; 3], [1.0; 3], &shape, flags)`.
    pub fn new(p1: [f64; 3], p2: [f64; 3], s: &Shape, flags: BiPntFlags) -> Self {
        Self {
            p1: Point3::new(p1[0], p1[1], p1[2]),
            p2: Point3::new(p2[0], p2[1], p2[2]),
            shape: s.clone(),
            flags,
        }
    }

    /// P1(); `segment.p1()`.
    pub fn p1(&self) -> Point3 {
        self.p1
    }

    /// P2(); `segment.p2()`.
    pub fn p2(&self) -> Point3 {
        self.p2
    }

    regularity_accessors!();
}
