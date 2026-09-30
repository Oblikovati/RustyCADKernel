#[test]
fn TCollection_ExtendedStringTest__LeftAdjust_RemovesLeadingSpaces() {
    let mut string = ExtendedString::new("   Hello World");
    string.left_adjust();
    text(&string, "Hello World");
}
#[test]
fn TCollection_ExtendedStringTest__LeftAdjust_NoLeadingSpaces() {
    let mut string = ExtendedString::new("Hello World");
    string.left_adjust();
    text(&string, "Hello World");
}
#[test]
fn TCollection_ExtendedStringTest__RightAdjust_RemovesTrailingSpaces() {
    let mut string = ExtendedString::new("Hello World   ");
    string.right_adjust();
    text(&string, "Hello World");
}
#[test]
fn TCollection_ExtendedStringTest__RightAdjust_NoTrailingSpaces() {
    let mut string = ExtendedString::new("Hello World");
    string.right_adjust();
    text(&string, "Hello World");
}
#[test]
fn TCollection_ExtendedStringTest__Capitalize_UppercasesFirstLetter() {
    let mut string = ExtendedString::new("hello WORLD");
    string.capitalize();
    text(&string, "Hello world");
}
#[test]
fn TCollection_ExtendedStringTest__Capitalize_AlreadyCapitalized() {
    let mut string = ExtendedString::new("Hello");
    string.capitalize();
    text(&string, "Hello");
}
#[test]
fn TCollection_ExtendedStringTest__LeftAdjust_AllSpaces() {
    let mut string = ExtendedString::new("     ");
    string.left_adjust();
    assert!(string.is_empty());
}
#[test]
fn TCollection_ExtendedStringTest__LeftAdjust_EmptyString() {
    let mut string = ExtendedString::new("");
    string.left_adjust();
    assert!(string.is_empty());
}
#[test]
fn TCollection_ExtendedStringTest__RightAdjust_AllSpaces() {
    let mut string = ExtendedString::new("     ");
    string.right_adjust();
    assert!(string.is_empty());
}
#[test]
fn TCollection_ExtendedStringTest__Capitalize_EmptyString() {
    let mut string = ExtendedString::new("");
    string.capitalize();
    assert!(string.is_empty());
}
#[test]
fn TCollection_ExtendedStringTest__LeftJustify_ExtendsWithFiller() {
    let mut string = ExtendedString::new("Hello");
    string.left_justify(10, 45).unwrap();
    assert_eq!(string.len(), 10);
    text(&string, "Hello-----");
}
#[test]
fn TCollection_ExtendedStringTest__LeftJustify_NoChangeIfShorter() {
    let mut string = ExtendedString::new("Hello");
    string.left_justify(3, 45).unwrap();
    assert_eq!(string.len(), 5);
    text(&string, "Hello");
}
#[test]
fn TCollection_ExtendedStringTest__RightJustify_ExtendsWithFiller() {
    let mut string = ExtendedString::new("Hello");
    string.right_justify(10, 45).unwrap();
    assert_eq!(string.len(), 10);
    text(&string, "-----Hello");
}
#[test]
fn TCollection_ExtendedStringTest__RightJustify_NoChangeIfShorter() {
    let mut string = ExtendedString::new("Hello");
    string.right_justify(3, 45).unwrap();
    assert_eq!(string.len(), 5);
    text(&string, "Hello");
}
#[test]
fn TCollection_ExtendedStringTest__Center_CentersWithFiller() {
    let mut string = ExtendedString::new("Hi");
    string.center(6, 45).unwrap();
    assert_eq!(string.len(), 6);
    text(&string, "--Hi--");
}
#[test]
fn TCollection_ExtendedStringTest__Center_OddWidth() {
    let mut string = ExtendedString::new("Hi");
    string.center(7, 45).unwrap();
    assert_eq!(string.len(), 7);
    text(&string, "--Hi---");
}
#[test]
fn TCollection_ExtendedStringTest__Center_NoChangeIfShorter() {
    let mut string = ExtendedString::new("Hello");
    string.center(3, 45).unwrap();
    assert_eq!(string.len(), 5);
}
#[test]
fn TCollection_ExtendedStringTest__Prepend_InsertsAtBeginning() {
    let mut string = ExtendedString::new("World");
    let prefix = ExtendedString::new("Hello ");
    string.prepend(&prefix);
    text(&string, "Hello World");
}
#[test]
fn TCollection_ExtendedStringTest__Prepend_EmptyPrefix() {
    let mut string = ExtendedString::new("Hello");
    let prefix = ExtendedString::new("");
    string.prepend(&prefix);
    text(&string, "Hello");
}
#[test]
fn TCollection_ExtendedStringTest__Prepend_ToEmptyString() {
    let mut string = ExtendedString::new("");
    let prefix = ExtendedString::new("Hello");
    string.prepend(&prefix);
    text(&string, "Hello");
}
