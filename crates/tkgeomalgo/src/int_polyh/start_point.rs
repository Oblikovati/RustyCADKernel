// Created on: 1999-04-06
// Created by: Fabrice SERVANT
// Copyright (c) 1999-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPolyh/IntPolyh_StartPoint.hxx and
// IntPolyh_StartPoint.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPolyh_StartPoint`: an intersection point of two triangles, on an edge (with its
//! parameter lambda) or a vertex of each.
use super::array::IntPolyhDumpItem;
use super::dump::{c_exp, c_fixed};
use super::triangle::IntPolyhTriangle;
use std::fmt;
use std::io::Write;

/// The coincidence precision of the start points (`MyConfusionPrecision`, 10.0e-12).
const MY_CONFUSION_PRECISION: f64 = 10.0e-12;

/// `IntPolyh_StartPoint`: the point, its parameters on both surfaces, the triangle, edge
/// (-1 for a vertex, -2 when unknown) and edge parameter on each surface, the incidence
/// angle and the section line it belongs to.
///
/// ```
/// use rustycad_tkgeomalgo::int_polyh::IntPolyhStartPoint;
/// let sp = IntPolyhStartPoint::default();
/// assert_eq!((sp.e1(), sp.lambda1(), sp.get_angle()), (-2, -1.0, -2.0));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IntPolyhStartPoint {
    x: f64,
    y: f64,
    z: f64,
    u1: f64,
    v1: f64,
    u2: f64,
    v2: f64,
    lambda1: f64,
    lambda2: f64,
    angle: f64,
    t1: i32,
    e1: i32,
    t2: i32,
    e2: i32,
    chainlist: i32,
}

impl Default for IntPolyhStartPoint {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            u1: 0.0,
            v1: 0.0,
            u2: 0.0,
            v2: 0.0,
            lambda1: -1.0,
            lambda2: -1.0,
            angle: -2.0,
            t1: -1,
            e1: -2,
            t2: -1,
            e2: -2,
            chainlist: -1,
        }
    }
}

impl IntPolyhStartPoint {
    /// The constructor from every field but the angle (-2).
    /// `IntPolyhStartPoint::new([0.0; 3], [0.0; 4], (1, 2, 0.5), (3, 4, 0.25), 0)`.
    pub fn new(
        xyz: [f64; 3],
        uv: [f64; 4],
        (t1, e1, lambda1): (i32, i32, f64),
        (t2, e2, lambda2): (i32, i32, f64),
        chainlist: i32,
    ) -> Self {
        let [x, y, z] = xyz;
        let [u1, v1, u2, v2] = uv;
        Self {
            x,
            y,
            z,
            u1,
            v1,
            u2,
            v2,
            lambda1,
            lambda2,
            angle: -2.0,
            t1,
            e1,
            t2,
            e2,
            chainlist,
        }
    }

    /// `X()`. `sp.x()`.
    pub const fn x(&self) -> f64 {
        self.x
    }
    /// `Y()`. `sp.y()`.
    pub const fn y(&self) -> f64 {
        self.y
    }
    /// `Z()`. `sp.z()`.
    pub const fn z(&self) -> f64 {
        self.z
    }
    /// `U1()`. `sp.u1()`.
    pub const fn u1(&self) -> f64 {
        self.u1
    }
    /// `V1()`. `sp.v1()`.
    pub const fn v1(&self) -> f64 {
        self.v1
    }
    /// `U2()`. `sp.u2()`.
    pub const fn u2(&self) -> f64 {
        self.u2
    }
    /// `V2()`. `sp.v2()`.
    pub const fn v2(&self) -> f64 {
        self.v2
    }
    /// `T1()`. `sp.t1()`.
    pub const fn t1(&self) -> i32 {
        self.t1
    }
    /// `E1()`. `sp.e1()`.
    pub const fn e1(&self) -> i32 {
        self.e1
    }
    /// `Lambda1()`. `sp.lambda1()`.
    pub const fn lambda1(&self) -> f64 {
        self.lambda1
    }
    /// `T2()`. `sp.t2()`.
    pub const fn t2(&self) -> i32 {
        self.t2
    }
    /// `E2()`. `sp.e2()`.
    pub const fn e2(&self) -> i32 {
        self.e2
    }
    /// `Lambda2()`. `sp.lambda2()`.
    pub const fn lambda2(&self) -> f64 {
        self.lambda2
    }
    /// `GetAngle()`. `sp.get_angle()`.
    pub const fn get_angle(&self) -> f64 {
        self.angle
    }
    /// `ChainList()`. `sp.chain_list()`.
    pub const fn chain_list(&self) -> i32 {
        self.chainlist
    }

    /// `GetEdgePoints(Triangle, First, Second, Last)`: the surface of the known edge (1
    /// or 2, 0 for none) and, for an edge 1..=3 of that surface, its two points and the
    /// opposite one (`None` for another edge number, whose native outputs are unchanged).
    /// `sp.get_edge_points(&triangle)`.
    pub fn get_edge_points(&self, triangle: &IntPolyhTriangle) -> (i32, Option<[i32; 3]>) {
        let (surf_id, edge): (i32, i32) = if self.e1 != -1 {
            (1, self.e1)
        } else if self.e2 != -1 {
            (2, self.e2)
        } else {
            return (0, None);
        };
        let (p1, p2, p3) = (
            triangle.first_point(),
            triangle.second_point(),
            triangle.third_point(),
        );
        let points: Option<[i32; 3]> = match edge {
            1 => Some([p1, p2, p3]),
            2 => Some([p2, p3, p1]),
            3 => Some([p3, p1, p2]),
            _ => None,
        };
        (surf_id, points)
    }

    /// `SetXYZ(XX, YY, ZZ)`. `sp.set_xyz(1.0, 2.0, 3.0)`.
    pub fn set_xyz(&mut self, xx: f64, yy: f64, zz: f64) {
        (self.x, self.y, self.z) = (xx, yy, zz);
    }
    /// `SetUV1(UU1, VV1)`. `sp.set_uv1(0.1, 0.2)`.
    pub fn set_uv1(&mut self, uu1: f64, vv1: f64) {
        (self.u1, self.v1) = (uu1, vv1);
    }
    /// `SetUV2(UU2, VV2)`. `sp.set_uv2(0.1, 0.2)`.
    pub fn set_uv2(&mut self, uu2: f64, vv2: f64) {
        (self.u2, self.v2) = (uu2, vv2);
    }
    /// `SetEdge1(IE1)`. `sp.set_edge1(3)`.
    pub fn set_edge1(&mut self, ie1: i32) {
        self.e1 = ie1;
    }
    /// `SetLambda1(LAM1)`. `sp.set_lambda1(0.5)`.
    pub fn set_lambda1(&mut self, lam1: f64) {
        self.lambda1 = lam1;
    }
    /// `SetEdge2(IE2)`. `sp.set_edge2(3)`.
    pub fn set_edge2(&mut self, ie2: i32) {
        self.e2 = ie2;
    }
    /// `SetLambda2(LAM2)`. `sp.set_lambda2(0.5)`.
    pub fn set_lambda2(&mut self, lam2: f64) {
        self.lambda2 = lam2;
    }
    /// `SetCoupleValue(IT1, IT2)`: the triangles. `sp.set_couple_value(1, 2)`.
    pub fn set_couple_value(&mut self, it1: i32, it2: i32) {
        (self.t1, self.t2) = (it1, it2);
    }
    /// `SetAngle(ang)`. `sp.set_angle(0.5)`.
    pub fn set_angle(&mut self, ang: f64) {
        self.angle = ang;
    }
    /// `SetChainList(ChList)`. `sp.set_chain_list(0)`.
    pub fn set_chain_list(&mut self, ch_list: i32) {
        self.chainlist = ch_list;
    }

    /// `CheckSameSP(SP)`: 1 when `sp` is this point (the same edge and parameter on a
    /// surface, or the same parameters on the first surface for a vertex), else 0. With
    /// two unknown edges the native member prints both points and a message to standard
    /// output, as here. `a.check_same_sp(&b)`.
    pub fn check_same_sp(&self, sp: &IntPolyhStartPoint) -> i32 {
        let mut test: i32 = 0;
        if (self.e1 >= -1 && self.e1 == sp.e1) || (self.e2 >= -1 && self.e2 == sp.e2) {
            // The edges are known.
            let same1: bool = self.lambda1 > -MY_CONFUSION_PRECISION
                && (self.lambda1 - sp.lambda1).abs() < MY_CONFUSION_PRECISION;
            let same2: bool = self.lambda2 > -MY_CONFUSION_PRECISION
                && (self.lambda2 - sp.lambda2).abs() < MY_CONFUSION_PRECISION;
            if same1 || same2 {
                test = 1;
            }
        }
        if test == 0 && (self.e1 == -1 || self.e2 == -1) {
            // This point is a vertex.
            if (sp.u1 - self.u1).abs() < MY_CONFUSION_PRECISION
                && (sp.v1 - self.v1).abs() < MY_CONFUSION_PRECISION
            {
                test = 1;
            }
        } else if self.e1 == -2 && self.e2 == -2 {
            let mut text = String::new();
            let _ = self.dump_index(0o0200, &mut text);
            let _ = sp.dump_index(0o0201, &mut text);
            text.push_str("e1==-2 & e2==-2 Can't Check\n");
            let _ = std::io::stdout().write_all(text.as_bytes());
        }
        test
    }

    /// `Dump()`: the native `printf` text. `sp.dump(&mut text)?`.
    pub fn dump(&self, out: &mut dyn fmt::Write) -> fmt::Result {
        writeln!(
            out,
            "\nPoint : x={}g y={}g z={}g u1={}g v1={}g u2={}g v2={}g",
            c_exp(self.x, 8, 3),
            c_exp(self.y, 8, 3),
            c_exp(self.z, 8, 3),
            c_exp(self.u1, 8, 3),
            c_exp(self.v1, 8, 3),
            c_exp(self.u2, 8, 3),
            c_exp(self.v2, 8, 3)
        )?;
        self.dump_tail(out)
    }

    /// `Dump(i)`: the native `printf` text numbered `i`. `sp.dump_index(3, &mut text)?`.
    pub fn dump_index(&self, i: i32, out: &mut dyn fmt::Write) -> fmt::Result {
        writeln!(
            out,
            "\nPoint({i}) : x={}g y={}g z={}g u1={}g v1={}g u2={}g v2={}g",
            c_exp(self.x, 8, 3),
            c_exp(self.y, 8, 3),
            c_exp(self.z, 8, 3),
            c_exp(self.u1, 8, 3),
            c_exp(self.v1, 8, 3),
            c_exp(self.u2, 8, 3),
            c_exp(self.v2, 8, 3)
        )?;
        self.dump_tail(out)
    }

    fn dump_tail(&self, out: &mut dyn fmt::Write) -> fmt::Result {
        writeln!(
            out,
            "Triangle S1:{} Edge S1:{} Lambda1:{} Triangle S2:{} Edge S2:{} Lambda2:{}",
            self.t1,
            self.e1,
            c_fixed(self.lambda1, 0, 6),
            self.t2,
            self.e2,
            c_fixed(self.lambda2, 0, 6)
        )?;
        writeln!(
            out,
            "Angle: {} List Number: {}",
            c_fixed(self.angle, 0, 6),
            self.chainlist
        )
    }
}

impl IntPolyhDumpItem for IntPolyhStartPoint {
    fn dump_item(&self, out: &mut dyn fmt::Write) -> fmt::Result {
        self.dump(out)
    }
}
