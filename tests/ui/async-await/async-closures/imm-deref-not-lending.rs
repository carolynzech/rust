//@ edition: 2021

#![feature(impl_trait_in_bindings)]

struct FooS {
    precise: i32,
}

fn ref_inside_mut(f: &mut &FooS) {
    let x: impl Fn() -> _ = async move || {
        let y = &f.precise;
    };
}

fn mut_inside_ref(f: &&mut FooS) {
    let x: impl Fn() -> _ = async move || {
        let y = &f.precise;
    };
}

// Expected to fail, no immutable reference here.
fn mut_ref_inside_mut(f: &mut &mut FooS) {
    let x: impl Fn() -> _ = async move || {
        //~^ ERROR async closure does not implement `Fn`
        let y = &f.precise;
    };
}

fn ref_inside_box(f: Box<&FooS>) {
    let x: impl Fn() -> _ = async move || {
        let y = &f.precise;
    };
}

fn box_inside_ref(f: &Box<FooS>) {
    let x: impl Fn() -> _ = async move || {
        let y = &f.precise;
    };
}

// Expected to fail, no immutable reference here.
fn box_inside_box(f: Box<Box<FooS>>) {
    let x: impl Fn() -> _ = async move || {
        //~^ ERROR async closure does not implement `Fn`
        let y = &f.precise;
    };
}

fn main() {}
