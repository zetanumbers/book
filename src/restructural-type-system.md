## Explaining structural properties

### Contraction (Copy)

One of the main features of Rust is types unable to be copied.
Consider a Rust function:

```rust
fn before_contraction(a: T, b: U, c: U) {
  // ...
}
```

Given function `before_contraction` **only** for types `U: Copy` it is allowed to define a new function:

```rust
fn after_contraction(a: T, b: U) {
  before_contraction(a, b, b)
}
```

Function definition `after_contraction` simply make a copy of the `b` argument and passes them down to `before_contraction`.
As such two arguments are *contracted* into a single one, this is why mathmaticians call it like that, as term originates from formal logic.
The argument `a` is there for you to understand that existence of other arguments is permited.

### Weakening (Forget)

However, the next structural property is not considered optional in Rust's type system.
It have been a major contention point in discussions about Rust.
It is assumed that any object of any type could be safely forgotten, that is be safely destroyed without calling its `drop` implementation.

To explain naming again, consider a new function below:

```rust
fn before_weakening(a: T) {
  // ...
}
```

Given function `before_weakening` currently for any type it is allowed to define a new function:

```rust
fn after_weakening(a: T, b: U) {
  forget(b);
  before_weakening(a);
}
```

Function definition `after_weakening` simply forgets the `b` argument and calls `before_contraction` with `a`.
As such function definition is *weakened* by requiring a new argument `b`, as this makes it less versatile.
The term "weaken" also comes from formal logic.
The argument `a` is there again simply for you to understand that existence or non-existence of other arguments is permited.

### Exchange (Move)



# Pivot to formalism

Let's define a new formal system in order to analyze rust type system.


