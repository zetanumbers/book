# Recovering structural properties of types in Rust

## Introduction

<!-- In this text I would like to tell you how product types (i.e. `structs`) can recover properties like thread-safety, unwind-safety, static lifetime, forgetness for thread-unsafe, unwind-unsafe, proposed unforgettable types respectivelly. -->
This way I can show how structural rules and other properties of types can be recovered in practice.
I will state a generalizing hypothesis of such fact, from which certain useful properties can be proven about Rust objects and code (for example about async blocks).
Then we can rely upon these properties while designing type system to hopefully enrich it even further.

I will try to explain my sense of it using Rust code analogies, however the general knowledge about mathematical formal logic is highly recommended.
Otherwise you should introduce yourself to [The Pain Of Real Linear Types in Rust] by Aria Desires which I have found to be a good introduction to the unfortunate "linear types" discussion.

And just to clarify: Rust being an imperative programming language doesn't strictly follow all the substructual rules which type theory would have.
This formally incomplete text won't fill every hole you could imagine within the Rust type system.
As such please entertain these ideas a bit before rejecting them as useless.

## Mathmaticians cannot name things... or do they?

A quote from the aforementioned [The Pain Of Real Linear Types in Rust]:

> First off, I need to explain something about naming. My experience from discussing this system and its application to Rust and other programming languages is that it’s poorly named, and so are most of the concepts it introduces. As in, I frequently see conversations about this topic quickly get muddied and confused because two people are using the same terms to mean different things.
>
> Yes, the names have a justification; Substructural Types are the natural consequence of applying Substructural Logic to Type Theory. However I haven’t observed this duality to be actually helpful. Substructural Type Systems are actually fairly simple things which are very easy to reason about intuitively. Therefore I’ve found it much more helpful to use a naming scheme that appeals to this intuition and avoids the muddied baggage of the old names.

The first paragraph is quite correct about the existing terminology confusion.
However let me introduce you to a reason why substructural rules are named in such manner.

You may have heard mathmaticians usually like to talk about functional programming languages.
The core concept of these languages is, of course, a function.
As such take a look at our first structural rule, the contraction.

Given the function definition:

```rust
// `ctx` stands for context object
fn f(ctx: X, a: T, b: T) -> R {
  // ...
}
```

Assuming objects of type `T` can be copied, you can quite simply define another function `g`:

```rust
fn g(ctx: X, a: T) -> R {
  f(ctx, a, a)
}
```

This fact corresponds to the contraction rule from the formal logic. Let me be upstraight and show how it would have been written using [sequent calculus]:

\\[ \cfrac{X, T, T \vdash R}{X, T \vdash R} \quad (\mathit{CL}) \\]

The above expression resembles a fraction, which is called used to reason **about** the language.
To simplify top and bottom subexpressions tell 
 
[The Pain Of Real Linear Types in Rust]: https://faultlore.com/blah/linear-rust/
[sequent calculus]: https://en.wikipedia.org/wiki/Sequent_calculus
