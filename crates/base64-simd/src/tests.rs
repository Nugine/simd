use crate::Base64;

#[test]
fn test_forgiving() {
    let inputs = ["ab", "abc", "abcd"];
    let outputs: &[&[u8]] = &[&[105], &[105, 183], &[105, 183, 29]];

    for i in 0..inputs.len() {
        let (src, expected) = (inputs[i], outputs[i]);
        let mut buf = src.to_owned().into_bytes();
        let ans = Base64::forgiving_decode_inplace(&mut buf).unwrap();
        assert_eq!(ans, expected, "src = {:?}, expected = {:?}", src, expected);
    }
}
