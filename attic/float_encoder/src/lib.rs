#![warn(bare_trait_objects)]

mod encode;
mod decode;

pub use self::encode::*;
pub use self::decode::*;

#[cfg(test)]
mod test {
    use super::*;

    use std::f64;

    #[test]
    pub fn can_encode_sequence() {
        let mut target = vec![];

        squish_float(&mut target, 0.0, 0.5).unwrap();
        squish_float(&mut target, 0.5, 2.4).unwrap();

        assert!(target.len() == 4);
    }

    #[test]
    pub fn can_decode_zero() {
        let mut target = vec![];

        squish_float(&mut target, 0.0, 0.0).unwrap();

        let mut src: &[u8] = &target;
        let res = unsquish_float(&mut src, 0.0).unwrap();

        assert!((res-0.0).abs() < 0.01);
    }

    #[test]
    pub fn can_decode_one_point_five() {
        let mut target = vec![];

        squish_float(&mut target, 0.0, 1.5).unwrap();

        let mut src: &[u8] = &target;
        let res = unsquish_float(&mut src, 0.0).unwrap();

        assert!((res-1.5).abs() < 0.01);
    }

    #[test]
    pub fn can_decode_128() {
        let mut target = vec![];

        squish_float(&mut target, 0.0, 128.0).unwrap();

        let mut src: &[u8] = &target;
        let res = unsquish_float(&mut src, 0.0).unwrap();

        assert!((res-128.0).abs() < 0.01);
    }

    #[test]
    pub fn can_decode_big_value() {
        let mut target = vec![];

        squish_float(&mut target, 0.0, 700_000.25).unwrap();

        let mut src: &[u8] = &target;
        let res = unsquish_float(&mut src, 0.0).unwrap();

        assert!((res-700_000.25).abs() < 0.01);
    }

    #[test]
    pub fn can_decode_minus_one_point_five() {
        let mut target = vec![];

        squish_float(&mut target, 0.0, -1.5).unwrap();

        let mut src: &[u8] = &target;
        let res = unsquish_float(&mut src, 0.0).unwrap();

        assert!((res- -1.5).abs() < 0.01);
    }

    #[test]
    pub fn can_decode_127() {
        let mut target = vec![];

        squish_float(&mut target, 0.0, 127.0).unwrap();

        let mut src: &[u8] = &target;
        let res = unsquish_float(&mut src, 0.0).unwrap();

        assert!((res-127.0).abs() < 0.01);
    }

    #[test]
    pub fn can_decode_minus_127() {
        let mut target = vec![];

        squish_float(&mut target, 0.0, -127.0).unwrap();

        let mut src: &[u8] = &target;
        let res = unsquish_float(&mut src, 0.0).unwrap();

        assert!((res- -127.0).abs() < 0.01);
    }

    #[test]
    pub fn can_decode_126() {
        let mut target = vec![];

        squish_float(&mut target, 0.0, 126.0).unwrap();

        let mut src: &[u8] = &target;
        let res = unsquish_float(&mut src, 0.0).unwrap();

        assert!((res-126.0).abs() < 0.01);
    }

    #[test]
    pub fn can_decode_minus_126() {
        let mut target = vec![];

        squish_float(&mut target, 0.0, -126.0).unwrap();

        let mut src: &[u8] = &target;
        let res = unsquish_float(&mut src, 0.0).unwrap();

        assert!((res- -126.0).abs() < 0.01);
    }

    #[test]
    pub fn can_decode_sequence() {
        let mut target = vec![];

        squish_float(&mut target, 0.0, 1.5).unwrap();
        squish_float(&mut target, 1.5, 0.3).unwrap();
        squish_float(&mut target, 0.3, 4093.2).unwrap();
        squish_float(&mut target, 4093.2, 4085.25).unwrap();

        let mut src: &[u8] = &target;
        let res1 = unsquish_float(&mut src, 0.0).unwrap();
        let res2 = unsquish_float(&mut src, res1).unwrap();
        let res3 = unsquish_float(&mut src, res2).unwrap();
        let res4 = unsquish_float(&mut src, res3).unwrap();

        assert!((res1-1.5).abs() < 0.01);
        assert!((res2-0.3).abs() < 0.01);
        assert!((res3-4093.2).abs() < 0.01);
        assert!((res4-4085.25).abs() < 0.01);
    }

    #[test]
    pub fn can_decode_nan() {
        let mut target = vec![];

        squish_float(&mut target, 0.0, f64::NAN).unwrap();

        let mut src: &[u8] = &target;
        let res = unsquish_float(&mut src, 0.0).unwrap();

        assert!(res.is_nan());
    }

    #[test]
    pub fn can_decode_infinity() {
        let mut target = vec![];

        squish_float(&mut target, 0.0, f64::INFINITY).unwrap();

        let mut src: &[u8] = &target;
        let res = unsquish_float(&mut src, 0.0).unwrap();

        assert!(res.is_infinite());
    }

    #[test]
    pub fn can_recover_from_nan() {
        let mut target = vec![];

        squish_float(&mut target, f64::NAN, 42.0).unwrap();

        let mut src: &[u8] = &target;
        let res = unsquish_float(&mut src, f64::NAN).unwrap();

        assert!((res-42.0).abs() < 0.01);
    }

    #[test]
    pub fn can_recover_from_infinity() {
        let mut target = vec![];

        squish_float(&mut target, f64::INFINITY, 42.0).unwrap();

        let mut src: &[u8] = &target;
        let res = unsquish_float(&mut src, f64::INFINITY).unwrap();

        assert!((res-42.0).abs() < 0.01);
    }

    #[test]
    fn zero_is_small() {
        let mut target = vec![];

        squish_float(&mut target, 0.0, 0.0).unwrap();
        assert!(target.len() == 2);
    }

    #[test]
    fn one_is_small() {
        let mut target = vec![];

        squish_float(&mut target, 0.0, 1.0).unwrap();
        assert!(target.len() == 2);
    }

    #[test]
    fn minus_one_is_small() {
        let mut target = vec![];

        squish_float(&mut target, 0.0, -1.0).unwrap();
        assert!(target.len() == 2);
    }
}
