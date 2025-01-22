/* SPDX-License-Identifier: MIT
 * origin: musl src/math/floor.c */

//! Generic `floor` algorithm.
//!
//! Note that this uses the algorithm from musl's `floorf` rather than `floor` or `floorl` because
//! performance seems to be better (based on icount) and it does not seem to experience rounding
//! errors on i386.

use super::super::{Float, Int, IntTy, MinInt};

pub fn floor<F: Float>(x: F) -> F {
    let zero = IntTy::<F>::ZERO;

    let mut ix = x.to_bits();
    let e = x.exp_unbiased();

    // If the represented value has no fractional part, no truncation is needed.
    if e >= F::SIG_BITS as i32 {
        return x;
    }

    if e >= 0 {
        // |x| >= 1.0

        let m = F::SIG_MASK >> e.unsigned();
        if ix & m == zero {
            // Portion to be masked is already zero; no adjustment needed.
            return x;
        }

        // Otherwise, raise an inexact exception.
        force_eval!(x + F::MAX);

        if x.is_sign_negative() {
            ix += m;
        }

        ix &= !m;
        F::from_bits(ix)
    } else {
        // |x| < 1.0, raise an inexact exception since truncation will happen (unless x == 0).
        force_eval!(x + F::MAX);

        if x.is_sign_positive() {
            // 0.0 <= x < 1.0; rounding down goes toward +0.0.
            F::ZERO
        } else if ix << 1 != zero {
            // -1.0 < x < 0.0; rounding down goes toward -1.0.
            F::NEG_ONE
        } else {
            // -0.0 remains unchanged
            x
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test against https://en.cppreference.com/w/cpp/numeric/math/floor
    fn spec_test<F: Float>() {
        // Not Asserted: that the current rounding mode has no effect.
        for f in [F::ZERO, F::NEG_ZERO, F::INFINITY, F::NEG_INFINITY].iter().copied() {
            assert_biteq!(floor(f), f);
        }
    }

    /* Skipping f16 / f128 "sanity_check"s due to rejected literal lexing at MSRV */

    #[test]
    #[cfg(f16_enabled)]
    fn spec_tests_f16() {
        spec_test::<f16>();
    }

    #[test]
    fn sanity_check_f32() {
        assert_eq!(floor(0.5f32), 0.0);
        assert_eq!(floor(1.1f32), 1.0);
        assert_eq!(floor(2.9f32), 2.0);
    }

    #[test]
    fn spec_tests_f32() {
        spec_test::<f32>();
    }

    #[test]
    fn sanity_check_f64() {
        assert_eq!(floor(1.1f64), 1.0);
        assert_eq!(floor(2.9f64), 2.0);
    }

    #[test]
    fn spec_tests_f64() {
        spec_test::<f64>();
    }

    #[test]
    #[cfg(f128_enabled)]
    fn spec_tests_f128() {
        spec_test::<f128>();
    }
}
