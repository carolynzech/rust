// NOTE we intentionally avoid using the `quote` crate here because it doesn't work with the
// `x86_64-unknown-linux-musl` target.

// NOTE usually the only thing you need to do to test a new math function is to add it to one of the
// macro invocations found in the bottom of this file.

extern crate rand;

use std::error::Error;
use std::fmt::Write as _0;
use std::fs::{self, File};
use std::io::Write as _1;
use std::{i16, u32, u8};

use rand::{Rng, SeedableRng, XorShiftRng};

// Number of test cases to generate
const NTESTS: usize = 10_000;

// TODO tweak this function to generate edge cases (zero, infinity, NaN) more often
fn f32(rng: &mut XorShiftRng) -> f32 {
    let sign = if rng.gen_bool(0.5) { 1 << 31 } else { 0 };
    let exponent = (rng.gen_range(0, u8::MAX) as u32) << 23;
    let mantissa = rng.gen_range(0, u32::MAX) & ((1 << 23) - 1);

    f32::from_bits(sign + exponent + mantissa)
}

// fn(f32) -> f32
macro_rules! f32_f32 {
    ($($intr:ident,)+) => {
        fn f32_f32(rng: &mut XorShiftRng) -> Result<(), Box<Error>> {
            // MUSL C implementation of the function to test
            extern "C" {
                $(fn $intr(_: f32) -> f32;)+
            }

            $(
                let mut cases = String::new();
                for _ in 0..NTESTS {
                    let inp = f32(rng);
                    let out = unsafe { $intr(inp) };

                    let inp = inp.to_bits();
                    let out = out.to_bits();

                    write!(cases, "({}, {})", inp, out).unwrap();
                    cases.push(',');
                }

                let mut f = File::create(concat!("tests/", stringify!($intr), ".rs"))?;
                write!(f, "
                    extern crate libm;

                    #[test]
                    fn {0}() {{
                        const CASES: &[(u32, u32)] = &[
                            {1}
                        ];

                        for case in CASES {{
                            let (inp, expected) = *case;

                            let outf = libm::{0}(f32::from_bits(inp));
                            let outi = outf.to_bits();

                            if !((outf.is_nan() && f32::from_bits(expected).is_nan()) ||
                                 libm::_eqf(outi, expected)) {{
                                panic!(
                                    \"input: {{}}, output: {{}}, expected: {{}}\",
                                    inp,
                                    outi,
                                    expected,
                                );
                            }}
                        }}
                    }}
",
                       stringify!($intr),
                       cases)?;
            )+

            Ok(())
        }
    }
}

macro_rules! f32f32_f32 {
    ($($intr:ident,)+) => {
        fn f32f32_f32(rng: &mut XorShiftRng) -> Result<(), Box<Error>> {
            extern "C" {
                $(fn $intr(_: f32, _: f32) -> f32;)+
            }

            $(
                let mut cases = String::new();
                for _ in 0..NTESTS {
                    let i1 = f32(rng);
                    let i2 = f32(rng);
                    let out = unsafe { $intr(i1, i2) };

                    let i1 = i1.to_bits();
                    let i2 = i2.to_bits();
                    let out = out.to_bits();

                    write!(cases, "(({}, {}), {})", i1, i2, out).unwrap();
                    cases.push(',');
                }

                let mut f = File::create(concat!("tests/", stringify!($intr), ".rs"))?;
                write!(f, "
                    extern crate libm;

                    #[test]
                    fn {0}() {{
                        const CASES: &[((u32, u32), u32)] = &[
                            {1}
                        ];

                        for case in CASES {{
                            let ((i1, i2), expected) = *case;

                            let outf = libm::{0}(f32::from_bits(i1), f32::from_bits(i2));
                            let outi = outf.to_bits();

                            if !((outf.is_nan() && f32::from_bits(expected).is_nan()) ||
                                 libm::_eqf(outi, expected)) {{
                                panic!(
                                    \"input: {{:?}}, output: {{}}, expected: {{}}\",
                                    (i1, i2),
                                    outi,
                                    expected,
                                );
                            }}
                        }}
                    }}
",
                       stringify!($intr),
                       cases)?;
            )+

            Ok(())
        }
    };
}

macro_rules! f32i32_f32 {
    ($($intr:ident,)+) => {
        fn f32i32_f32(rng: &mut XorShiftRng) -> Result<(), Box<Error>> {
            extern "C" {
                $(fn $intr(_: f32, _: i32) -> f32;)+
            }

            $(
                let mut cases = String::new();
                for _ in 0..NTESTS {
                    let i1 = f32(rng);
                    let i2 = rng.gen_range(i16::MIN, i16::MAX);
                    let out = unsafe { $intr(i1, i2 as i32) };

                    let i1 = i1.to_bits();
                    let out = out.to_bits();

                    write!(cases, "(({}, {}), {})", i1, i2, out).unwrap();
                    cases.push(',');
                }

                let mut f = File::create(concat!("tests/", stringify!($intr), ".rs"))?;
                write!(f, "
                    extern crate libm;

                    #[test]
                    fn {0}() {{
                        const CASES: &[((u32, i16), u32)] = &[
                            {1}
                        ];

                        for case in CASES {{
                            let ((i1, i2), expected) = *case;

                            let outf = libm::{0}(f32::from_bits(i1), i2 as i32);
                            let outi = outf.to_bits();

                            if !((outf.is_nan() && f32::from_bits(expected).is_nan()) ||
                                 libm::_eqf(outi, expected)) {{
                                panic!(
                                    \"input: {{:?}}, output: {{}}, expected: {{}}\",
                                    (i1, i2),
                                    outi,
                                    expected,
                                );
                            }}
                        }}
                    }}
",
                       stringify!($intr),
                       cases)?;
            )+

            Ok(())
        }
    };
}

fn main() -> Result<(), Box<Error>> {
    fs::remove_dir_all("tests").ok();
    fs::create_dir("tests")?;

    let mut rng = XorShiftRng::from_rng(&mut rand::thread_rng())?;

    f32_f32(&mut rng)?;
    f32f32_f32(&mut rng)?;
    f32i32_f32(&mut rng)?;

    Ok(())
}

/* Functions to test */

// With signature `fn(f32) -> f32`
f32_f32! {
    fabsf,
    sqrtf,
}

// With signature `fn(f32, f32) -> f32`
f32f32_f32! {
    fmodf,
    powf,
}

// With signature `fn(f32, i32) -> f32`
f32i32_f32! {
    scalbnf,
}
