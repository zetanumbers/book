#![feature(auto_traits, negative_impls)]

use core::marker::PhantomData;

// ANCHOR: leak_trait
unsafe auto trait Leak {}
// ANCHOR_END: leak_trait

// ANCHOR: unleak
#[repr(transparent)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unleak<T>(pub T, PhantomUnleak);

impl<T> Unleak<T> {
    pub const fn new(v: T) -> Self {
        Unleak(v, PhantomUnleak)
    }
}

// This is the essential part of the `Unleak` design.
unsafe impl<T: 'static> Leak for Unleak<T> {}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct PhantomUnleak;

impl !Leak for PhantomUnleak {}
// ANCHOR_END: unleak

// ANCHOR: variance
struct Variance<Contra, Co> {
    process: fn(Contra) -> String,
    // invalidate `Co` type's safety invariant before restoring it
    // inside of the drop
    queue: Unleak<Co>,
}
// ANCHOR_END: variance

// ANCHOR: variance_alt
struct VarianceAlt<Contra, Co> {
    process: fn(Contra) -> String,
    queue: Co,
    _unleak: PhantomUnleak,
}

unsafe impl<Contra, Co: 'static> Leak for VarianceAlt<Contra, Co> {}
// ANCHOR_END: variance_alt

// ANCHOR: join_guard
// not sure about variance here
struct JoinGuard<'a, T: 'a> {
    // ...
    _marker: PhantomData<fn() -> T>,
    _unleak: PhantomData<Unleak<&'a ()>>,
    _unsend: PhantomData<*mut ()>,
}

unsafe impl<T: 'static> Send for JoinGuard<'static, T> {}
unsafe impl<'a, T> Sync for JoinGuard<'a, T> {}
// ANCHOR_END: join_guard

// We are outside of the main function
fn main() {}
