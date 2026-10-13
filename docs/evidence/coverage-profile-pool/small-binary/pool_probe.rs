// Owned coverage experiment; every policy uses this exact instrumented binary.
fn wait_for_release() {
    println!("MARTENSITE_READY");
    let mut release = String::new();
    std::io::stdin().read_line(&mut release).unwrap();
    assert_eq!(release, "release\n");
}
fn signed_class(value: i32) -> i32 {
    if value > 0 {
        return 1;
    }
    if value < 0 {
        return -1;
    }
    0
}
#[test]
fn positive_path() {
    assert_eq!(signed_class(7), 1);
    wait_for_release();
}
#[test]
fn negative_path() {
    assert_eq!(signed_class(-4), -1);
    wait_for_release();
}
#[test]
fn zero_path() {
    assert_eq!(signed_class(0), 0);
    wait_for_release();
}
#[test]
fn mixed_path() {
    assert_eq!(signed_class(-1) + signed_class(0) + signed_class(1), 0);
    wait_for_release();
}
