# Rust with localy substructural types

## Explaining structural properties

### Contraction (Copy)

One of the main features of Rust is types unable to be copied.
Consider a Rust function:

```rust
fn pre_cntr(a: T, b: U, c: U) {
  // ...
}
```

Given function `pre_cntr` **only** for types `U: Copy` it is allowed to define a new function:

```rust
fn post_cntr(a: T, b: U) {
  pre_cntr(a, b, b)
}
```

Function definition `post_cntr` simply make a copy of the `b` argument and passes them down to `pre_cntr`.
As such two arguments are *contracted* into a single one, this is why mathmaticians call it like that, as term originates from formal logic.
The argument `a` is there for you to understand that existence of other arguments is permited.

### Weakening (Forget)

However, the next structural property is not considered optional in Rust's type system.
It have been a major contention point in discussions about Rust.
It is assumed that any object of any type could be safely forgotten, that is be safely destroyed without calling its `drop` implementation.

To explain naming again, consider a new function below:

```rust
fn pre_wkn(a: T) {
  // ...
}
```

Given function `pre_wkn` currently for any type `U` it is allowed to define a new function:

```rust
fn post_wkn(a: T, b: U) {
  forget(b);
  pre_wkn(a);
}
```

Function definition `post_wkn` simply forgets the `b` argument and calls `pre_wkn` with `a`.
As such function definition is *weakened* by requiring a new argument `b`, as this makes it less versatile.
The term "weaken" also comes from formal logic.
The argument `a` is there again simply for you to understand that existence or non-existence of other arguments is permited.

### Exchange (Move)

The last named structural rule is exchange, which relates to the type's ability to be moved.

```rust
fn pre_xchg(a: T, b: U, c: W) {
  // ...
}
```

Given function `pre_xchg` currently for any pair of types `U` and `W` it is allowed to define a new function:

```rust
fn post_xchg(a: T, c: W, b: U) {
  pre_xchg(a, b, c);
}
```

This example is not as rigorous to be useful, so we need to pivot to more formal system in order to rigorously define and analyze such rules.

## Pivot to formalism

Our formal system would represent evolution of typed variables available in the scope.

\\[ \left(\begin{matrix}T_{0} & T_{1} & \cdots & T_{n}\end{matrix}\right) \vdash \left(\begin{matrix}U_{0} & U_{1} & \cdots & U_{m}\end{matrix}\right) \\]

Term \\(T_{i}\\) represents a *typed \\(\mathit{T_{i}}\\) variable*, in which variable's name is omitted as being irrelevant.
Sequence of typed variables in parentheses \\(\left(\begin{matrix}T_{0} & T_{1} & \cdots & T_{n}\end{matrix}\right)\\) represent a *logical stack* of available typed variables at a certain point.
It is not exactly the program stack you may be familiar with, but a formal construction in order to express certain properties of a programming language.
As such I will refer to it simply as *stack*, as opposed to the *program stack*.
However they might look similar in certain aspects and you might be familiar with stack-based programming languages such as PostScript.
And finally the whole expression or the *sequent* represents logical stack's evolution.

To be concise it is allowed to express possibly empty sequence of typed variables using distinct greek capital letters, similar to formal logic:

\\[ \left(\begin{matrix} \Gamma & T & U & \Delta \end{matrix}\right) \\]

As such let's introduce first the most obvious sequent:

\\[ \text{Identity} \quad \left(\begin{matrix} \Gamma \end{matrix}\right) \vdash \left(\begin{matrix} \Gamma \end{matrix}\right) \\]

Next would be the structural rules from above. 

\\[ \text{(Copy or Contraction)} \quad \left(\begin{matrix}\Gamma & T \end{matrix}\right) \vdash \left(\begin{matrix}\Gamma & T & T\end{matrix}\right) \\]
\\[ \text{(Forget or Weakening)} \quad \left(\begin{matrix}\Gamma & T \end{matrix}\right) \vdash \left(\begin{matrix}\Gamma\end{matrix}\right) \\]
\\[ \text{(Move or Exchange)} \quad \left(\begin{matrix}\Gamma & T & U & \Delta \end{matrix}\right) \vdash \left(\begin{matrix}\Gamma & U & T & \Delta \end{matrix}\right) \\]

## Memory safety

Would do we usually mean by *memory safety*?
I would say that memory safety is inablity to represent unintentional code.
Think about the `Box<T>` type.
It is a unique memory address by which you are able to refer to inner value of type `T`.
You may think of this address as a "name" of variable, as it behaves very similarly.
Upon writing to a variable (or box) you expect the its consequent reads to return the same value you've written.


