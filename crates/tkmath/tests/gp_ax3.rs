// Copyright (c) 2025 OPEN CASCADE SAS
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

//! OCC29406: preserve handedness when replacing frame directions.
#![allow(non_snake_case)]
use rustycad_tkmath::gp::{Axis1, Axis3, Direction3, Point3};
#[test]
fn gp_Ax3_Test__OCC29406_SetDirectionPreservesOrientation() {
    check_main_direction();
    check_x_direction();
    check_y_direction();
}
fn check_main_direction() {
    for index in 0..4 {
        let mut frame = Axis3::default();
        if index >= 2 {
            frame.z_reverse();
        }
        let direction = if index % 2 == 0 {
            Direction3::X
        } else {
            Direction3::X.reversed()
        };
        let direct = frame.direct().unwrap();
        frame.set_direction(direction).unwrap();
        assert_eq!(direct, frame.direct().unwrap());
        assert!(direction.is_equal(&frame.direction(), 1e-12));
    }
    for direction in [Direction3::X, Direction3::X.reversed()] {
        let mut frame = Axis3::default();
        let axis = Axis1::new(Point3::default(), direction);
        let direct = frame.direct().unwrap();
        frame.set_axis(axis).unwrap();
        assert_eq!(direct, frame.direct().unwrap());
        assert!(axis.direction().is_equal(&frame.direction(), 1e-12));
    }
}
fn check_x_direction() {
    for index in 0..4 {
        let mut frame = Axis3::default();
        if index >= 2 {
            frame.x_reverse();
        }
        let direction = if index % 2 == 0 {
            Direction3::Z
        } else {
            Direction3::Z.reversed()
        };
        let direct = frame.direct().unwrap();
        frame.set_x_direction(direction).unwrap();
        assert_eq!(direct, frame.direct().unwrap());
        if index == 0 {
            assert_x_result(frame);
        }
    }
}
fn assert_x_result(frame: Axis3) {
    let good = frame.direction().crossed(Direction3::Z).unwrap();
    if frame.direct().unwrap() {
        assert!(good.is_equal(&frame.y_direction(), 1e-12));
    } else {
        assert!(good.is_opposite(frame.y_direction(), 1e-12));
    }
}
fn check_y_direction() {
    for index in 0..4 {
        let mut frame = Axis3::default();
        if index >= 2 {
            frame.y_reverse();
        }
        let direction = if index % 2 == 0 {
            Direction3::Z
        } else {
            Direction3::Z.reversed()
        };
        let direct = frame.direct().unwrap();
        frame.set_y_direction(direction).unwrap();
        assert_eq!(direct, frame.direct().unwrap());
        if index == 0 {
            assert_y_result(frame);
        }
    }
}
fn assert_y_result(frame: Axis3) {
    let good = frame.direction().crossed(Direction3::Z).unwrap();
    if frame.direct().unwrap() {
        assert!(good.is_opposite(frame.x_direction(), 1e-12));
    } else {
        assert!(good.is_equal(&frame.x_direction(), 1e-12));
    }
}
