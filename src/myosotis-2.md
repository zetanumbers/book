<!-- TODO: isolate samples into files -->

# Addressing concerns and narratives
#### Myosotis: 2.0 You Can (Not) Forget

Today I would like to address one reoccurring problem about names,
talk through concerns about difficulties with adoption and ergonomics,
discuss switching default bounds between editions. But first to establish
some changes to terminology of unforgettable types lets talk about
naming problem first.

## Naming problem

In the [previous post](myosotis.md#trivial-implementation) I have named
the main trait as `Leak` because people were referring to it in this
way. However the term *(resource) leak* has too many sides and usually
describes a larger set of programs than just skipping destruction as
described in the [destruction guarantee]. That's also why this name is
disliked by some developers and it often leads people to make an argument
which would lie outside of the scope of my proposal, for example about
how to leak a value by sending it to another thread.

Alternatively the entire feature have been called *unforgettable
types*, which prompted me to consider renaming trait into `Forget` at
one point, but I only [written down](myosotis.md#trivial-implementation)
the possibility of renaming the trait. After some discussion with others
and noticing problems described in the previous paragraph, I have decided
to rename `Leak` trait into `Forget` and refer to it as such from now on.

```rust
unsafe auto trait Forget {}
```

I would like to see others to also start calling this hypothetical trait
`Forget`, so I hope I've convinced you a little.

## Breaking macros

TODO: figure out (scoped awareness, similar to editions)

```rust
macro_rules! forget {
    ($t:ty, $e:expr) => {{
        // emits `unsafe` keyword = bad
        let _ = unsafe { ::core::mem::transmute::<$t, ::core::mem::ManuallyDrop<$t>>($e) };
    }};
}
```

## Ergonomics and adoption of new default bounds

Unfortunately I have found rustc developers are a bit dismissive when
it comes features with implicit trait bounds due to consensus of such
features being difficult to adopt. I believe the major part of this worry
is not justified due to several unique properties of unforgettable types.

The problem with adding a new default bound is that any type which doesn't
implement this trait is pretty much unless everyone relaxes the bound
when possible.[^ergonomics_problem] To illustrate general case of this
take a look at this code.

```rust
/// An auto trait, implementation of which allows value to be passed into
/// some old std functions, which previously had no such bound. We add
/// it as new default bound, kinda like with `Sized`.
unsafe auto trait AllowSomething {}

// Old std function. Has implicit `T: AllowSomething` bound.
fn something<T>(x: T) {
  // ...
}

/// A new struct from one library
struct NewStruct {
  _disallow_something: PhantomDisallowSomething
  // ...
}

/// A different struct from another. Has implicit `T: AllowSomething` bound.
struct OldStructFromOtherLib<T> {
  // ...
}
```

`something` and `OldStructFromOtherLib` are orthogonal but still interfere
with each other.

## Pessimistic case adoption scenario is not that bad

Unforgettable types:

- Thread join handles
- Task join handles

Support:

- Standard library
- Async runtime libraries

Allows:

- Binary projects to use unforgettable types

### Safe crate migration algorithm

Even tho safe crates may not yet be able to utilize unforgettable types,
it would still be useful to enable explicit `Forget` bounds for forward
compatibility.

This algorithm utilizes clippy to automatically make changes to the code.

1. Make sure your crate compiles and does not get any errors from
clippy. Getting rid of warnings will help too;
1. Add `#![aware(unforgettable_types)]` attribute to the root of your
crate
1. Run `cargo check` to evaluate places where `T: Forget` bounds on
generic argument types are required;
1. Run `cargo clippy -A warnings -`

### Unsafe crate migration algorithm

Because leaking objects in unsafe code (unless it is intentional)
is considered to be a bug, it would only require a bit more awareness
from the reviewers to .

1. Evaluate **public** API of **your crate**, whether or not it enables
shared ownership or leaking objects;
1. Evaluate **internal** API of **your crate**, whether or not it enables
shared ownership or leaking objects;
1. Evaluate public API of your **dependencies**, whether or not it
enables shared ownership or leaking objects;
1. Evaluate **implementation** of **your crate**, whether or not it
**intentionally** shares ownership or leaks objects;
1. Evaluate **implementation** of **your crate**, whether or not it
**unintentionally** shares ownership or leaks objects. This could occur
due to **known bugs** within your crate.

## Switching between editions

TODO: consider removing

-----

[^ergonomics_problem]: [Comment from @bjorn3 within #t-lang > The destruction guarantee and linear types formulation - rust-lang - Zulip](https://rust-lang.zulipchat.com/#narrow/stream/213817-t-lang/topic/The.20destruction.20guarantee.20and.20linear.20types.20formulation/near/417223664)

[destruction guarantee]: myosotis.md#solution
