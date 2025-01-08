//@ edition: 2021
//@ compile-flags: -Zprint-mono-items=eager --crate-type=lib

//~ MONO_ITEM fn async_fn @@
//~ MONO_ITEM fn async_fn::{closure#0} @@
pub async fn async_fn() {}

//~ MONO_ITEM fn closure @@
//~ MONO_ITEM fn closure::{closure#0} @@
pub fn closure() {
    let _ = || {};
}
