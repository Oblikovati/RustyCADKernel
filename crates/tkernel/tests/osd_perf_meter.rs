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

//! Full legacy meter suite with a fresh injected registry per fixture.
#![allow(non_snake_case, unused_mut)]
use rustycad_tkernel::osd::{NativeCpuClock, PerfMeter, PerfRegistry};
use rustycad_tkernel::tcollection::AsciiString;
use std::{hint::black_box, rc::Rc};
fn work(registry: &PerfRegistry, duration: f64) {
    let meter = PerfMeter::new(registry.clone(), "WorkMeter", true);
    while meter.elapsed() < duration {
        computational_work();
    }
    meter.kill();
}
fn computational_work() {
    let mut result: f64 = 0.0;
    for i in 0..10000 {
        let value: f64 = f64::from(i);
        result = black_box(result + (value * 0.001).sin() * (value * 0.002).cos());
        result = black_box(result + (value + 1.0).sqrt());
        result = black_box(result + (value * 0.1).powf(1.5));
        let mut text = AsciiString::new("Test");
        text += &AsciiString::from_integer(i);
        black_box(text.len());
    }
    black_box(result);
}

#[test]
fn OSD_PerfMeterTest__BasicCreationWithAutoStart() {
    let registry = PerfRegistry::new(Rc::new(NativeCpuClock));
    registry.reset_all();

    // Create an auto-started meter
    let mut meter = PerfMeter::new(registry.clone(), "TestMeter", true);

    // Do some work
    work(&registry, 0.1);

    // Stop the meter
    meter.stop();

    // Check elapsed time is positive
    let elapsed: f64 = meter.elapsed();
    assert!(elapsed > 0.0);
    registry.reset_all();
}

#[test]
fn OSD_PerfMeterTest__ManualStartStop() {
    let registry = PerfRegistry::new(Rc::new(NativeCpuClock));
    registry.reset_all();

    // Create a meter without auto-start
    let mut meter = PerfMeter::new(registry.clone(), "ManualMeter", false);

    // Start the meter
    meter.start();

    // Do some work
    work(&registry, 0.1);

    // Stop the meter
    meter.stop();

    // Check elapsed time is positive
    let elapsed: f64 = meter.elapsed();
    assert!(elapsed > 0.0);
    registry.reset_all();
}

#[test]
fn OSD_PerfMeterTest__DefaultConstructorAndInit() {
    let registry = PerfRegistry::new(Rc::new(NativeCpuClock));
    registry.reset_all();

    // Create meter with default constructor
    let mut meter = PerfMeter::empty(registry.clone());

    // Initialize it
    meter.init("InitializedMeter");

    // Start it manually
    meter.start();

    // Do some work
    work(&registry, 0.1);

    // Stop and check elapsed time
    meter.stop();
    assert!(meter.elapsed() > 0.0);
    registry.reset_all();
}

#[test]
fn OSD_PerfMeterTest__SharedMetersByName() {
    let registry = PerfRegistry::new(Rc::new(NativeCpuClock));
    registry.reset_all();

    let meterName: &str = "SharedMeter";

    // Create and start first meter
    let mut meter1 = PerfMeter::new(registry.clone(), meterName, true);

    // Do some work
    work(&registry, 0.1);

    // Create second meter with the same name, but don't auto-start
    let mut meter2 = PerfMeter::empty(registry.clone());
    meter2.init(meterName);

    // Do more work
    work(&registry, 0.1);

    // Stop both meters
    meter1.stop();

    // The elapsed time should account for both work segments
    let elapsed: f64 = meter1.elapsed();
    assert!(elapsed > 0.0);

    // Stopping meter2 should have no effect since they share the same meter
    // and meter1 already stopped it
    meter2.stop();
    let elapsed2: f64 = meter2.elapsed();
    assert_eq!(elapsed, elapsed2);
    registry.reset_all();
}

#[test]
fn OSD_PerfMeterTest__RestartMeter() {
    let registry = PerfRegistry::new(Rc::new(NativeCpuClock));
    registry.reset_all();

    // Create and start meter
    let mut meter = PerfMeter::new(registry.clone(), "RestartMeter", true);

    // Do some work
    work(&registry, 0.1);

    // Stop and get elapsed time
    meter.stop();
    let elapsed1: f64 = meter.elapsed();
    assert!(elapsed1 > 0.0);

    // Restart the meter
    meter.start();

    // Do more work
    work(&registry, 0.2);

    // Stop and get elapsed time again
    meter.stop();
    let elapsed2: f64 = meter.elapsed();
    assert!(elapsed2 > 0.0);

    // Second elapsed time should be different from first
    assert_ne!(elapsed1, elapsed2);
    registry.reset_all();
}

#[test]
fn OSD_PerfMeterTest__RelativeTimingAccuracy() {
    let registry = PerfRegistry::new(Rc::new(NativeCpuClock));
    registry.reset_all();

    // Create two meters
    let mut meter1 = PerfMeter::new(registry.clone(), "ShortMeter", true);
    let mut meter2 = PerfMeter::new(registry.clone(), "LongMeter", true);

    // Short work for meter1
    work(&registry, 0.1);
    meter1.stop();

    // Long work for meter2
    work(&registry, 0.5);
    meter2.stop();

    // Long work should take more time than short work
    let shortElapsed: f64 = meter1.elapsed();
    let longElapsed: f64 = meter2.elapsed();

    assert!(shortElapsed > 0.0);
    assert!(longElapsed > 0.0);
    assert!(longElapsed > shortElapsed);
    registry.reset_all();
}

#[test]
fn OSD_PerfMeterTest__PrintALL() {
    let registry = PerfRegistry::new(Rc::new(NativeCpuClock));
    registry.reset_all();

    // Create and use several meters
    let mut meter1 = PerfMeter::new(registry.clone(), "Meter1", true);
    work(&registry, 0.1);
    meter1.stop();

    let mut meter2 = PerfMeter::new(registry.clone(), "Meter2", true);
    work(&registry, 0.2);
    meter2.stop();

    let mut meter3 = PerfMeter::new(registry.clone(), "Meter3", true);
    work(&registry, 0.3);
    meter3.stop();

    // Get the printed output
    let output: String = registry.print_all();

    // Check that the output contains all meter names
    assert!(output.contains("Meter1"));
    assert!(output.contains("Meter2"));
    assert!(output.contains("Meter3"));
    registry.reset_all();
}

#[test]
fn OSD_PerfMeterTest__ResetALL() {
    let registry = PerfRegistry::new(Rc::new(NativeCpuClock));
    registry.reset_all();

    // Create and use several meters
    let mut meter1 = PerfMeter::new(registry.clone(), "ResetMeter1", true);
    work(&registry, 0.1);
    meter1.stop();

    let mut meter2 = PerfMeter::new(registry.clone(), "ResetMeter2", true);
    work(&registry, 0.2);
    meter2.stop();

    // Both meters should have positive elapsed times
    assert!(meter1.elapsed() > 0.0);
    assert!(meter2.elapsed() > 0.0);

    // Reset all meters
    registry.reset_all();

    // After reset, all meters should have zero elapsed time
    assert_eq!(meter1.elapsed(), 0.0);
    assert_eq!(meter2.elapsed(), 0.0);
    registry.reset_all();
}

#[test]
fn OSD_PerfMeterTest__UnusedMeter() {
    let registry = PerfRegistry::new(Rc::new(NativeCpuClock));
    registry.reset_all();

    // Create a meter but don't start it
    let mut meter = PerfMeter::new(registry.clone(), "UnusedMeter", false);

    // Elapsed time should be zero
    assert_eq!(meter.elapsed(), 0.0);

    // Stopping not started meter should have no effect
    meter.stop();
    assert_eq!(meter.elapsed(), 0.0);
    registry.reset_all();
}

#[test]
fn OSD_PerfMeterTest__PrintMethod() {
    let registry = PerfRegistry::new(Rc::new(NativeCpuClock));
    registry.reset_all();

    // Create and start the meter
    let meterName: &str = "PrintTestMeter";
    let mut meter = PerfMeter::new(registry.clone(), meterName, true);

    // Do some work
    work(&registry, 0.1);

    // Stop the meter
    meter.stop();

    // Get the printed output
    let output: String = meter.print();

    // Verify output contains necessary information
    assert!(output.contains(meterName));

    // Verify output contains elapsed time info (we can't check exact time,
    // but there should be numbers)
    assert!(output.bytes().any(|byte| byte.is_ascii_digit()));

    assert!(output.contains("sec"));
    registry.reset_all();
}

#[test]
fn OSD_PerfMeterTest__KillMethod() {
    let registry = PerfRegistry::new(Rc::new(NativeCpuClock));
    registry.reset_all();

    // Create and start the meter
    let mut meter = PerfMeter::new(registry.clone(), "KillTestMeter", true);

    // Do some work
    work(&registry, 0.1);

    // Stop the meter
    meter.stop();

    // Verify we have non-zero elapsed time
    let elapsed: f64 = meter.elapsed();
    assert!(elapsed > 0.0);

    // Kill the meter
    meter.kill();

    // Verify elapsed time is reset
    let elapsedAfterKill: f64 = meter.elapsed();
    assert_eq!(elapsedAfterKill, 0.0);

    // Verify killing a meter makes it disappear from the global list
    let allMeters: String = registry.print_all();
    assert!(!allMeters.contains("KillTestMeter"));
    registry.reset_all();
}

#[test]
fn OSD_PerfMeterTest__KillRunningMeter() {
    let registry = PerfRegistry::new(Rc::new(NativeCpuClock));
    registry.reset_all();

    // Create and start the meter
    let mut meter = PerfMeter::new(registry.clone(), "KillRunningMeter", true);

    // Don't stop the meter, kill it while running
    meter.kill();

    // Verify elapsed time is reset
    let elapsedAfterKill: f64 = meter.elapsed();
    assert_eq!(elapsedAfterKill, 0.0);

    // Start the meter again to verify it's usable after being killed
    meter.init("KillRunningMeter");
    meter.start();
    work(&registry, 0.1);
    meter.stop();

    // Verify it records time correctly after being killed and restarted
    assert!(meter.elapsed() > 0.0);
    registry.reset_all();
}
