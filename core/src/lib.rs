//! # The Rust Core Library
//!
//! The Rust Core Library is the dependency-free[^free] foundation of [The
//! Rust Standard Library](../std/index.html). It is the portable glue
//! between the language and its libraries, defining the intrinsic and
//! primitive building blocks of all Rust code. It links to no
//! upstream libraries, no system libraries, and no libc.
//!
//! [^free]: Strictly speaking, there are some symbols which are needed but
//!          they aren't always necessary.
//!
//! The core library is *minimal*: it isn't even aware of heap allocation,
//! nor does it provide concurrency or I/O. These things require
//! platform integration, and this library is platform-agnostic.
//!
//! # How to use the core library
//!
//! Please note that all of these details are currently not considered stable.
//!
// FIXME: Fill me in with more detail when the interface settles
//! This library is built on the assumption of a few existing symbols:
//!
//! * `memcpy`, `memcmp`, `memset`, `strlen` - These are core memory routines which are
//!   often generated by LLVM. Additionally, this library can make explicit
//!   calls to these functions. Their signatures are the same as found in C.
//!   These functions are often provided by the system libc, but can also be
//!   provided by the [compiler-builtins crate](https://crates.io/crates/compiler_builtins).
//!
//! * `rust_begin_panic` - This function takes four arguments, a
//!   `fmt::Arguments`, a `&'static str`, and two `u32`'s. These four arguments
//!   dictate the panic message, the file at which panic was invoked, and the
//!   line and column inside the file. It is up to consumers of this core
//!   library to define this panic function; it is only required to never
//!   return. This requires a `lang` attribute named `panic_impl`.
//!
//! * `rust_eh_personality` - is used by the failure mechanisms of the
//!    compiler. This is often mapped to GCC's personality function, but crates
//!    which do not trigger a panic can be assured that this function is never
//!    called. The `lang` attribute is called `eh_personality`.

// Since core defines many fundamental lang items, all tests live in a
// separate crate, libcoretest (library/core/tests), to avoid bizarre issues.
//
// Here we explicitly #[cfg]-out this whole crate when testing. If we don't do
// this, both the generated test artifact and the linked libtest (which
// transitively includes core) will both define the same set of lang items,
// and this will cause the E0152 "found duplicate lang item" error. See
// discussion in #50466 for details.
//
// This cfg won't affect doc tests.
#![cfg(not(test))]
// To run core tests without x.py without ending up with two copies of core, Miri needs to be
// able to "empty" this crate. See <https://github.com/rust-lang/miri-test-libstd/issues/4>.
// rustc itself never sets the feature, so this line has no affect there.
#![cfg(any(not(feature = "miri-test-libstd"), test, doctest))]
#![stable(feature = "core", since = "1.6.0")]
#![doc(
    html_playground_url = "https://play.rust-lang.org/",
    issue_tracker_base_url = "https://github.com/rust-lang/rust/issues/",
    test(no_crate_inject, attr(deny(warnings))),
    test(attr(allow(dead_code, deprecated, unused_variables, unused_mut)))
)]
#![doc(cfg_hide(
    not(test),
    any(not(feature = "miri-test-libstd"), test, doctest),
    no_fp_fmt_parse,
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64",
    target_has_atomic = "8",
    target_has_atomic = "16",
    target_has_atomic = "32",
    target_has_atomic = "64",
    target_has_atomic = "ptr",
    target_has_atomic_equal_alignment = "8",
    target_has_atomic_equal_alignment = "16",
    target_has_atomic_equal_alignment = "32",
    target_has_atomic_equal_alignment = "64",
    target_has_atomic_equal_alignment = "ptr",
    target_has_atomic_load_store = "8",
    target_has_atomic_load_store = "16",
    target_has_atomic_load_store = "32",
    target_has_atomic_load_store = "64",
    target_has_atomic_load_store = "ptr",
))]
#![no_core]
#![rustc_coherence_is_core]
//
// Lints:
#![deny(rust_2021_incompatible_or_patterns)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(fuzzy_provenance_casts)]
#![warn(deprecated_in_future)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![allow(explicit_outlives_requirements)]
#![allow(incomplete_features)]
#![warn(multiple_supertrait_upcastable)]
//
// Library features:
// tidy-alphabetical-start
#![feature(char_indices_offset)]
#![feature(const_align_of_val)]
#![feature(const_align_of_val_raw)]
#![feature(const_align_offset)]
#![feature(const_alloc_layout)]
#![feature(const_arguments_as_str)]
#![feature(const_array_from_ref)]
#![feature(const_array_into_iter_constructors)]
#![feature(const_bigint_helper_methods)]
#![feature(const_black_box)]
#![feature(const_caller_location)]
#![feature(const_cell_into_inner)]
#![feature(const_char_from_u32_unchecked)]
#![feature(const_cmp)]
#![feature(const_cstr_methods)]
#![feature(const_discriminant)]
#![feature(const_eval_select)]
#![feature(const_exact_div)]
#![feature(const_float_bits_conv)]
#![feature(const_float_classify)]
#![feature(const_fmt_arguments_new)]
#![feature(const_hash)]
#![feature(const_heap)]
#![feature(const_index_range_slice_index)]
#![feature(const_inherent_unchecked_arith)]
#![feature(const_int_unchecked_arith)]
#![feature(const_intrinsic_forget)]
#![feature(const_ipv4)]
#![feature(const_ipv6)]
#![feature(const_is_char_boundary)]
#![feature(const_likely)]
#![feature(const_maybe_uninit_as_mut_ptr)]
#![feature(const_maybe_uninit_assume_init)]
#![feature(const_maybe_uninit_uninit_array)]
#![feature(const_nonnull_new)]
#![feature(const_option)]
#![feature(const_option_ext)]
#![feature(const_pin)]
#![feature(const_pointer_byte_offsets)]
#![feature(const_pointer_is_aligned)]
#![feature(const_ptr_as_ref)]
#![feature(const_ptr_is_null)]
#![feature(const_ptr_read)]
#![feature(const_ptr_sub_ptr)]
#![feature(const_ptr_write)]
#![feature(const_raw_ptr_comparison)]
#![feature(const_replace)]
#![feature(const_result_drop)]
#![feature(const_size_of_val)]
#![feature(const_size_of_val_raw)]
#![feature(const_slice_from_raw_parts_mut)]
#![feature(const_slice_from_ref)]
#![feature(const_slice_index)]
#![feature(const_slice_ptr_len)]
#![feature(const_slice_split_at_mut)]
#![feature(const_str_from_utf8_unchecked_mut)]
#![feature(const_swap)]
#![feature(const_transmute_copy)]
#![feature(const_try)]
#![feature(const_type_id)]
#![feature(const_type_name)]
#![feature(const_unicode_case_lookup)]
#![feature(const_unsafecell_get_mut)]
#![feature(const_waker)]
#![feature(core_panic)]
#![feature(duration_consts_float)]
#![feature(ip)]
#![feature(is_ascii_octdigit)]
#![feature(maybe_uninit_uninit_array)]
#![feature(ptr_alignment_type)]
#![feature(ptr_metadata)]
#![feature(set_ptr_value)]
#![feature(slice_ptr_get)]
#![feature(slice_split_at_unchecked)]
#![feature(str_internals)]
#![feature(str_split_inclusive_remainder)]
#![feature(str_split_remainder)]
#![feature(strict_provenance)]
#![feature(utf16_extra)]
#![feature(utf16_extra_const)]
#![feature(variant_count)]
// tidy-alphabetical-end
//
// Language features:
// tidy-alphabetical-start
#![feature(abi_unadjusted)]
#![feature(adt_const_params)]
#![feature(allow_internal_unsafe)]
#![feature(allow_internal_unstable)]
#![feature(asm_const)]
#![feature(associated_type_bounds)]
#![feature(auto_traits)]
#![feature(c_unwind)]
#![feature(cfg_sanitize)]
#![feature(cfg_target_has_atomic)]
#![feature(cfg_target_has_atomic_equal_alignment)]
#![feature(const_closures)]
#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_for)]
#![feature(const_mut_refs)]
#![feature(const_precise_live_drops)]
#![feature(const_refs_to_cell)]
#![feature(decl_macro)]
#![feature(deprecated_suggestion)]
#![feature(doc_cfg)]
#![feature(doc_cfg_hide)]
#![feature(doc_notable_trait)]
#![feature(exhaustive_patterns)]
#![feature(extern_types)]
#![feature(fundamental)]
#![feature(generic_arg_infer)]
#![feature(if_let_guard)]
#![feature(inline_const)]
#![feature(intra_doc_pointers)]
#![feature(intrinsics)]
#![feature(lang_items)]
#![feature(link_llvm_intrinsics)]
#![feature(macro_metavar_expr)]
#![feature(min_specialization)]
#![feature(multiple_supertrait_upcastable)]
#![feature(must_not_suspend)]
#![feature(negative_impls)]
#![feature(never_type)]
#![feature(no_core)]
#![feature(no_coverage)] // rust-lang/rust#84605
#![feature(platform_intrinsics)]
#![feature(prelude_import)]
#![feature(repr_simd)]
#![feature(rustc_allow_const_fn_unstable)]
#![feature(rustc_attrs)]
#![feature(rustdoc_internals)]
#![feature(simd_ffi)]
#![feature(staged_api)]
#![feature(stmt_expr_attributes)]
#![feature(target_feature_11)]
#![feature(trait_alias)]
#![feature(transparent_unions)]
#![feature(try_blocks)]
#![feature(unboxed_closures)]
#![feature(unsized_fn_params)]
// tidy-alphabetical-end
//
// Target features:
// tidy-alphabetical-start
#![feature(arm_target_feature)]
#![feature(avx512_target_feature)]
#![feature(hexagon_target_feature)]
#![feature(mips_target_feature)]
#![feature(powerpc_target_feature)]
#![feature(riscv_target_feature)]
#![feature(rtm_target_feature)]
#![feature(sse4a_target_feature)]
#![feature(tbm_target_feature)]
#![feature(wasm_target_feature)]
// tidy-alphabetical-end

// allow using `core::` in intra-doc links
#[allow(unused_extern_crates)]
extern crate self as core;

#[prelude_import]
#[allow(unused)]
use prelude::v1::*;

#[cfg(not(test))] // See #65860
#[macro_use]
mod macros;

// We don't export this through #[macro_export] for now, to avoid breakage.
// See https://github.com/rust-lang/rust/issues/82913
#[cfg(not(test))]
#[unstable(feature = "assert_matches", issue = "82775")]
/// Unstable module containing the unstable `assert_matches` macro.
pub mod assert_matches {
    #[unstable(feature = "assert_matches", issue = "82775")]
    pub use crate::macros::{assert_matches, debug_assert_matches};
}

#[macro_use]
mod internal_macros;

#[path = "num/shells/int_macros.rs"]
#[macro_use]
mod int_macros;

#[path = "num/shells/i128.rs"]
pub mod i128;
#[path = "num/shells/i16.rs"]
pub mod i16;
#[path = "num/shells/i32.rs"]
pub mod i32;
#[path = "num/shells/i64.rs"]
pub mod i64;
#[path = "num/shells/i8.rs"]
pub mod i8;
#[path = "num/shells/isize.rs"]
pub mod isize;

#[path = "num/shells/u128.rs"]
pub mod u128;
#[path = "num/shells/u16.rs"]
pub mod u16;
#[path = "num/shells/u32.rs"]
pub mod u32;
#[path = "num/shells/u64.rs"]
pub mod u64;
#[path = "num/shells/u8.rs"]
pub mod u8;
#[path = "num/shells/usize.rs"]
pub mod usize;

#[path = "num/f32.rs"]
pub mod f32;
#[path = "num/f64.rs"]
pub mod f64;

#[macro_use]
pub mod num;

/* The core prelude, not as all-encompassing as the std prelude */

pub mod prelude;

/* Core modules for ownership management */

pub mod hint;
pub mod intrinsics;
pub mod mem;
pub mod ptr;

/* Core language traits */

pub mod borrow;
pub mod clone;
pub mod cmp;
pub mod convert;
pub mod default;
pub mod error;
pub mod marker;
pub mod ops;

/* Core types and methods on primitives */

pub mod any;
pub mod array;
pub mod ascii;
pub mod asserting;
#[unstable(feature = "async_iterator", issue = "79024")]
pub mod async_iter;
pub mod cell;
pub mod char;
pub mod ffi;
pub mod iter;
pub mod net;
pub mod option;
pub mod panic;
pub mod panicking;
pub mod pin;
pub mod result;
pub mod sync;

pub mod fmt;
pub mod hash;
pub mod slice;
pub mod str;
pub mod time;

pub mod unicode;

/* Async */
pub mod future;
pub mod task;

/* Heap memory allocator trait */
#[allow(missing_docs)]
pub mod alloc;

// note: does not need to be public
mod bool;
mod tuple;
mod unit;

#[stable(feature = "core_primitive", since = "1.43.0")]
pub mod primitive;

// Pull in the `core_arch` crate directly into core. The contents of
// `core_arch` are in a different repository: rust-lang/stdarch.
//
// `core_arch` depends on core, but the contents of this module are
// set up in such a way that directly pulling it here works such that the
// crate uses the this crate as its core.
#[path = "../../stdarch/crates/core_arch/src/mod.rs"]
#[allow(
    missing_docs,
    missing_debug_implementations,
    dead_code,
    unused_imports,
    unsafe_op_in_unsafe_fn
)]
#[allow(rustdoc::bare_urls)]
// FIXME: This annotation should be moved into rust-lang/stdarch after clashing_extern_declarations is
// merged. It currently cannot because bootstrap fails as the lint hasn't been defined yet.
#[allow(clashing_extern_declarations)]
#[unstable(feature = "stdsimd", issue = "48556")]
mod core_arch;

#[stable(feature = "simd_arch", since = "1.27.0")]
pub mod arch;

// Pull in the `core_simd` crate directly into core. The contents of
// `core_simd` are in a different repository: rust-lang/portable-simd.
//
// `core_simd` depends on core, but the contents of this module are
// set up in such a way that directly pulling it here works such that the
// crate uses this crate as its core.
#[path = "../../portable-simd/crates/core_simd/src/mod.rs"]
#[allow(missing_debug_implementations, dead_code, unsafe_op_in_unsafe_fn, unused_unsafe)]
#[allow(rustdoc::bare_urls)]
#[unstable(feature = "portable_simd", issue = "86656")]
mod core_simd;

#[doc = include_str!("../../portable-simd/crates/core_simd/src/core_simd_docs.md")]
#[unstable(feature = "portable_simd", issue = "86656")]
pub mod simd {
    #[unstable(feature = "portable_simd", issue = "86656")]
    pub use crate::core_simd::simd::*;
}

include!("primitive_docs.rs");
