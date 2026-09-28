//! FSD_Base64.
use rustycad_tkernel::fsd::Base64;

#[test]
fn encoding_pads_and_sizes_like_rfc_4648() {
    for (data, text) in [
        (&b""[..], ""),
        (b"f", "Zg=="),
        (b"fo", "Zm8="),
        (b"foo", "Zm9v"),
        (b"foob", "Zm9vYg=="),
    ] {
        assert_eq!(Base64::encode(data).as_bytes(), text.as_bytes());
        assert_eq!(Base64::encode_into(None, data), text.len());
    }
    let mut short = [b'*'; 3];
    assert_eq!(Base64::encode_into(Some(&mut short), b"fo"), 0);
    assert_eq!(short, [b'*'; 3]);
}

#[test]
fn decoding_follows_the_native_padding_rules() {
    assert_eq!(Base64::decode_into(None, b""), 0);
    assert_eq!(Base64::decode_into(None, b"Zg=="), 1);
    assert_eq!(Base64::decode_into(None, b"Zm8="), 2);
    assert_eq!(
        Base64::decode_into(None, b"Zm9"),
        2,
        "a short group still decodes its bytes"
    );
    assert_eq!(
        Base64::decode(b"Zm9vYg==").get().unwrap().as_slice(),
        b"foob"
    );
    assert_eq!(
        Base64::decode(b"-_,.").get().unwrap().as_slice(),
        &[0xFB, 0xFF, 0xFE]
    );
    let empty = Base64::decode(b"");
    assert!(
        !empty.get().unwrap().is_empty(),
        "an empty result is still allocated"
    );
    let mut short = [0_u8; 1];
    assert_eq!(Base64::decode_into(Some(&mut short), b"Zm8="), 0);
}
