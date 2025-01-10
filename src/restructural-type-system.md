## Explaining structural properties

### Contraction (Copy)

One of the main features of Rust is types unable to be copied.
Consider a Rust function:

```
fn foo(a: T, b: T) {
  // ...
}
```

Given function `foo` for types `T: Copy` it is allowed to define a new function:

```
fn bar(a: T) {
  foo(a, a)
}
```

Function definition `bar` simply make a copy of the `a` argument and passes both of them down to `foo`.
As such two arguments are *contracted* into a single one, this is why mathmaticians call it like that, as term originates from formal logic.

### Weakening (Forget)

However, the next structural property is considered essential in Rust's type system.
It have been a major contention point in discussions about Rust.
It is assumed that any object of any type could be safely forgotten, that is be safely destroyed without calling its `drop` implementation.

# Pivot to formal syntactic system

Let define Rust type system semantics using a new syntax.


