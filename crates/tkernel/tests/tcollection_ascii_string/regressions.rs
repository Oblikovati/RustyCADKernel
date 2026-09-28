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

#[test]
fn TCollection_AsciiStringTest__AssignCat_MultipleInLoop() {
    let mut definition = AsciiString::default();
    definition.assign_cat("Enum");
    definition.assign_cat(format!(" [in {}-{}]", 0, 1).as_str());
    text(&definition, "Enum [in 0-1]");
    for i in 0..=1 {
        let value = if i == 0 { "Off" } else { "On" };
        definition.assign_cat(format!(" {i}:{value}").as_str());
    }
    text(&definition, "Enum [in 0-1] 0:Off 1:On");
    definition.assign_cat(" , alpha: ");
    definition.assign_cat("On");
    definition.assign_cat(format!("On:{} ", 1).as_str());
    definition.assign_cat("Off");
    definition.assign_cat(format!("Off:{} ", 0).as_str());
    for pattern in ["Enum", "[in 0-1]", "0:Off", "1:On", "alpha:"] {
        assert!(contains(&definition, pattern));
    }
}
#[test]
fn TCollection_AsciiStringTest__BUC60724_EmptyStringInitialization() {
    let first = AsciiString::new("");
    assert!(!first.as_bytes_with_nul().as_ptr().is_null());
    assert_eq!(first.len(), 0);
    assert_eq!(first.as_bytes_with_nul()[0], 0);
    let second = AsciiString::from_char(0);
    assert!(!second.as_bytes_with_nul().as_ptr().is_null());
    assert_eq!(second.len(), 0);
    assert_eq!(second.as_bytes_with_nul()[0], 0);
}
#[test]
fn TCollection_AsciiStringTest__BUC60773_HAsciiStringInitialization() {
    use rustycad_tkernel::tcollection::HAsciiString;
    let handle = std::sync::Arc::new(HAsciiString::default());
    assert!(!std::sync::Arc::as_ptr(&handle).is_null());
    let bytes = handle.as_bytes_with_nul();
    assert!(!bytes.as_ptr().is_null());
    let ascii = AsciiString::from_bytes(bytes);
    assert_eq!(ascii.len(), 0);
    assert!(ascii.is_empty());
}
#[test]
fn TCollection_AsciiStringTest__OCC6794_LargeConcatenation() {
    let mut ascii = AsciiString::default();
    for _ in 1..=10000 {
        ascii += AsciiString::new("a");
    }
    assert_eq!(ascii.len(), 10000);
    assert!(!ascii.is_empty());
}
fn comprehensive_constructors(i: i32, source: &str) -> (AsciiString, AsciiString, AsciiString) {
    let a = AsciiString::new(&source[i as usize..]);
    text(&a, &source[i as usize..]);
    let b = AsciiString::with_max_length(&source[i as usize..], 3);
    assert_eq!(b.len(), 3);
    assert_eq!(b.as_bytes(), &source.as_bytes()[i as usize..i as usize + 3]);
    let c = AsciiString::from_integer(i);
    assert!(c.is_integer_value());
    assert_eq!(c.integer_value().unwrap(), i);
    let d = AsciiString::from_real(0.1 * f64::from(i));
    assert!(d.is_real_value(true));
    assert!(!AsciiString::new("3.3!").is_real_value(true));
    assert!(AsciiString::new("3.3!").is_real_value(false));
    text(&AsciiString::from_real(3.3), "3.3");
    (a, c, d)
}
fn comprehensive_concatenation(
    a: &AsciiString,
    c: &mut AsciiString,
    d: &mut AsciiString,
    source: &str,
) -> AsciiString {
    let e = d.clone();
    assert_eq!(d.as_bytes(), e.as_bytes());
    assert_eq!(d.len(), e.len());
    let f = e.cat(7_u8);
    assert_eq!(f.len(), e.len() + 1);
    assert_eq!(&f.as_bytes()[..e.len()], e.as_bytes());
    assert_eq!(f.value(f.length()).unwrap(), 7);
    let g = f.cat(source);
    assert_eq!(g.len(), f.len() + source.len());
    assert_eq!(&g.as_bytes()[..f.len()], f.as_bytes());
    assert_eq!(g.search(source), f.len() as i32 + 1);
    let h = d.cat(a);
    assert_eq!(h.len(), d.len() + a.len());
    assert_eq!(&h.as_bytes()[..d.len()], d.as_bytes());
    assert_eq!(&h.as_bytes()[d.len()..], a.as_bytes());
    c.assign_cat(a.as_bytes());
    assert_eq!(c.len(), 1 + a.len());
    assert_eq!(c.search(a), 2);
    let previous = d.len();
    d.assign_cat(a);
    assert_eq!(d.len(), previous + a.len());
    assert_eq!(d.search(a), previous as i32 + 1);
    h
}
fn comprehensive_edits(i: usize, source: &str, d: &mut AsciiString, h: &AsciiString) {
    let mut capital = AsciiString::new("aBC");
    capital.capitalize();
    text(&capital, "Abc");
    d.copy(&source[i..]);
    text(d, &source[i..]);
    d.copy(h);
    assert_eq!(d.as_bytes(), h.as_bytes());
    let previous = d.len();
    d.insert(2, source).unwrap();
    assert_eq!(d.len(), previous + source.len());
    assert_eq!(&d.as_bytes()[1..1 + source.len()], source.as_bytes());
    d.copy(source);
    d.insert_char(i as i32 + 1, b'i').unwrap();
    assert_eq!(d.len(), source.len() + 1);
    assert_eq!(d.value(i as i32 + 1).unwrap(), b'i');
    assert_eq!(&d.as_bytes()[i + 1..], &source.as_bytes()[i..]);
    d.copy(source);
    d.insert(i as i32 + 1, AsciiString::new("i")).unwrap();
    assert_eq!(d.len(), source.len() + 1);
    assert_eq!(d.value(i as i32 + 1).unwrap(), b'i');
    assert_eq!(&d.as_bytes()[i + 1..], &source.as_bytes()[i..]);
    assert!(d.is_different(source));
    assert!(d.is_different("theStr"));
    assert!(d.is_different(""));
    assert!(!d.is_different(d.as_bytes()));
}
#[test]
fn TCollection_AsciiStringTest__OCC11758_ComprehensiveConstructorsAndMethods() {
    let source = "0123456789";
    for i in 0..5 {
        let (a, mut c, mut d) = comprehensive_constructors(i, source);
        let h = comprehensive_concatenation(&a, &mut c, &mut d, source);
        comprehensive_edits(i as usize, source, &mut d, &h);
    }
}
