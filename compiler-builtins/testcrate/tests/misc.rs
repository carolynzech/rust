use testcrate::*;

/// Make sure that the the edge case tester and randomized tester don't break, and list examples of
/// fuzz values for documentation purposes.
#[test]
fn fuzz_values() {
    const VALS: [u16; 47] = [
        0b0, // edge cases
        0b1111111111111111,
        0b1111111111111110,
        0b1111111111111100,
        0b1111111110000000,
        0b1111111100000000,
        0b1110000000000000,
        0b1100000000000000,
        0b1000000000000000,
        0b111111111111111,
        0b111111111111110,
        0b111111111111100,
        0b111111110000000,
        0b111111100000000,
        0b110000000000000,
        0b100000000000000,
        0b11111111111111,
        0b11111111111110,
        0b11111111111100,
        0b11111110000000,
        0b11111100000000,
        0b10000000000000,
        0b111111111,
        0b111111110,
        0b111111100,
        0b110000000,
        0b100000000,
        0b11111111,
        0b11111110,
        0b11111100,
        0b10000000,
        0b111,
        0b110,
        0b100,
        0b11,
        0b10,
        0b1,
        0b1010110100000, // beginning of random fuzzing
        0b1100011001011010,
        0b1001100101001111,
        0b1101010100011010,
        0b100010001,
        0b1000000000000000,
        0b1100000000000101,
        0b1100111101010101,
        0b1100010111111111,
        0b1111110101111111,
    ];
    let mut i = 0;
    fuzz(10, |x: u16| {
        assert_eq!(x, VALS[i]);
        i += 1;
    });
}

#[test]
fn leading_zeros() {
    use compiler_builtins::int::__clzsi2;
    use compiler_builtins::int::leading_zeros::{
        usize_leading_zeros_default, usize_leading_zeros_riscv,
    };
    fuzz(N, |x: usize| {
        let lz = x.leading_zeros() as usize;
        let lz0 = __clzsi2(x);
        let lz1 = usize_leading_zeros_default(x);
        let lz2 = usize_leading_zeros_riscv(x);
        if lz0 != lz {
            panic!("__clzsi2({}): std: {}, builtins: {}", x, lz, lz0);
        }
        if lz1 != lz {
            panic!(
                "usize_leading_zeros_default({}): std: {}, builtins: {}",
                x, lz, lz1
            );
        }
        if lz2 != lz {
            panic!(
                "usize_leading_zeros_riscv({}): std: {}, builtins: {}",
                x, lz, lz2
            );
        }
    })
}

#[test]
fn float_extend() {
    fuzz_float(N, |x: f32| {
        let tmp0 = x as f64;
        let tmp1: f64 = compiler_builtins::float::extend::__extendsfdf2(x);
        if !compiler_builtins::float::Float::eq_repr(tmp0, tmp1) {
            panic!("__extendsfdf2({}): std: {}, builtins: {}", x, tmp0, tmp1);
        }
    });
}

// This doesn't quite work because of issues related to
// https://github.com/rust-lang/rust/issues/73920.
// TODO how do we resolve this?
/*
macro_rules! pow {
    ($($f:ty, $fn:ident);*;) => {
        $(
            fuzz_float_2(N, |x: $f, y: $f| {
                let n = y as i32;
                let tmp0: $f = x.powi(n);
                let tmp1: $f = $fn(x, n);
                if tmp0 != tmp1 {
                    panic!(
                        "{}({}, {}): std: {}, builtins: {}",
                        stringify!($fn), x, y, tmp0, tmp1
                    );
                }
            });
        )*
    };
}

#[test]
fn float_pow() {
    use compiler_builtins::float::pow::{__powidf2, __powisf2};

    pow!(
        f32, __powisf2;
        f64, __powidf2;
    );
}
*/
