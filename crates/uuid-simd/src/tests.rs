use crate::error::Error;
use crate::sa_ascii::AsciiCase;

use simd_abstraction::{OutBuf, OutRef};

fn ok_cases() -> &'static [(&'static str, &'static str)] {
    const A1: &str = "67e5504410b1426f9247bb680e5fe0c8";
    const A2: &str = "00000000000000000000000000000000";

    const OK: &[(&str, &str)] = &[
        (A1, "67e55044-10b1-426f-9247-bb680e5fe0c8"),
        (A1, "67e5504410b1426f9247bb680e5fe0c8"),
        (A1, "{67e55044-10b1-426f-9247-bb680e5fe0c8}"),
        (A1, "urn:uuid:67e55044-10b1-426f-9247-bb680e5fe0c8"),
        (A2, "00000000000000000000000000000000"),
        (A2, "00000000-0000-0000-0000-000000000000"),
        (
            "01020304111221223132414243444546",
            "01020304-1112-2122-3132-414243444546",
        ),
        (
            "F9168C5ECEB24faaB6BF329BF39FA1E4",
            "F9168C5E-CEB2-4faa-B6BF-329BF39FA1E4",
        ),
        (
            "6d93badebd9f4e1389149474e1e3567b",
            "{6d93bade-bd9f-4e13-8914-9474e1e3567b}",
        ),
    ];

    OK
}

fn err_cases() -> &'static [&'static str] {
    const ERR: &[&str] = &[
        "",
        "!",
        "F9168C5E-CEB2-4faa-B6BF-329BF39FA1E45",
        "F9168C5E-CEB2-4faa-BBF-329BF39FA1E4",
        "F9168C5E-CEB2-4faa",
        "{F9168C5E-CEB2-4faa9B6BFF329BF39FA1E41",
        "67e5504410b1426f9247bb680e5fe0c",
        "67e5504410b1426f9247bb680e5fe0c88",
        "67e5504410b1426f9247bb680e5fe0cg8",
        "{00000000000000000000000000000000}",
        "67e5504410b1426f9247bb680e5fe0c",
        "F9168C5E-CEB2-4faa-B6BF1-02BF39FA1E4",
        "231231212212423424324323477343246663",
        "01020304-1112-2122-3132-41424344",
        "F9168C5E-CEB2-4faa-BGBF-329BF39FA1E4",
        "F9168C5E-CEB2F4faaFB6BFF329BF39FA1E4",
        "F9168C5E-CEB2-4faaFB6BFF329BF39FA1E4",
        "F9168C5E-CEB2-4faa-B6BFF329BF39FA1E4",
        "F9168C5E-CEB2-4faaXB6BFF329BF39FA1E4",
        "67e5504410b1426%9247bb680e5fe0c8",
        "67e550X410b1426f9247bb680e5fe0cd",
        "67e550-4105b1426f9247bb680e5fe0c",
        "F9168C5E-CEB-24fa-eB6BFF32-BF39FA1E4",
    ];
    ERR
}

fn format_cases() -> &'static [(&'static str, &'static str)] {
    &[
        (
            "67e5504410b1426f9247bb680e5fe0c8",
            "67e55044-10b1-426f-9247-bb680e5fe0c8",
        ),
        (
            "01020304111221223132414243444546",
            "01020304-1112-2122-3132-414243444546",
        ),
        (
            "00000000000000000000000000000000",
            "00000000-0000-0000-0000-000000000000",
        ),
    ]
}

#[allow(clippy::type_complexity)]
fn safety_unit_test(
    parse: for<'s, 'd> fn(&'s [u8], OutRef<'d, [u8; 16]>) -> Result<&'d mut [u8; 16], Error>,
    format_simple: for<'s, 'd> fn(
        &'s [u8; 16],
        OutRef<'d, [u8; 32]>,
        AsciiCase,
    ) -> &'d mut [u8; 32],
    format_hyphenated: for<'s, 'd> fn(
        &'s [u8; 16],
        OutRef<'d, [u8; 36]>,
        AsciiCase,
    ) -> &'d mut [u8; 36],
) {
    for &(expected, input) in ok_cases() {
        let mut expected_buf = [0u8; 16];
        let expected_bytes =
            hex_simd::decode(expected.as_bytes(), OutBuf::new(&mut expected_buf)).unwrap();

        let mut output_buf = [0; 16];
        let output_bytes = parse(input.as_bytes(), OutRef::new(&mut output_buf)).unwrap();

        assert_eq!(output_bytes, expected_bytes);
    }

    for &input in err_cases() {
        let mut output_buf = [0; 16];
        parse(input.as_bytes(), OutRef::new(&mut output_buf)).unwrap_err();
    }

    for &(input, expected) in format_cases() {
        let mut src = [0; 16];
        hex_simd::decode(input.as_bytes(), OutBuf::new(&mut src)).unwrap();

        let mut output_buf = [0; 32];
        let output = format_simple(&src, OutRef::new(&mut output_buf), AsciiCase::Upper);
        assert_eq!(output.as_slice(), input.to_ascii_uppercase().as_bytes());
        let output = format_simple(&src, OutRef::new(&mut output_buf), AsciiCase::Lower);
        assert_eq!(output.as_slice(), input.to_ascii_lowercase().as_bytes());

        let mut output_buf = [0; 36];
        let output = format_hyphenated(&src, OutRef::new(&mut output_buf), AsciiCase::Upper);
        assert_eq!(output.as_slice(), expected.to_ascii_uppercase().as_bytes());
        let output = format_hyphenated(&src, OutRef::new(&mut output_buf), AsciiCase::Lower);
        assert_eq!(output.as_slice(), expected.to_ascii_lowercase().as_bytes());
    }
}

#[test]
fn test_safety() {
    safety_unit_test(crate::parse, crate::format_simple, crate::format_hyphenated);
}
