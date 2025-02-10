# Refining the Rust type system

<!--toc:start-->
- [Refining the Rust type system](#refining-the-rust-type-system)
  - [Introduction](#introduction)
  - [How do lifetimes work?](#how-do-lifetimes-work)
<!--toc:end-->

## Introduction

Today various issues related to coroutines, Send, Sync, unforgettable types, async drop plague the Rust type system.
Developers of the Rust language commonly seek solutions only to implement a patchwork in a grand scheme of things.
When it comes to design of new major language features, language designers might do something "because it doesn't seem to be wrong to do".
This in turn require patchwork solutions, if such thing even possible.
If you find this paragraph to touch some of the rust-lang design pain points, you might be interested in the subject of this text.

Many people have tackled with the problems such as:

- Self-referential types
- Thread-safe coroutines with `!Send` locals
- Unforgettable and immovable types

I believe I have found a strong framework for designing such difficult features.
**Be warned: I will be using a bit of the Category Theory to describe some arising constructions, but I'll try making it clear if you are unfamiliar with it.**
I hope you will consider it in your further work.
So let's start from the begining.

## How do lifetimes work?

You may have heard, that Rust type system was inspired by the [Cyclone language].
It was a research project, which have implemented a clever region-based memory managment, which relates to the common understanding of how lifetimes work.
To recap: Rust compiler associates each lifetime token to a certain scope, where an object can live until the end of the scope.

Things changes a bit after introduction of [Non-lexical lifetimes].
Scopes were redefined in terms of the control-flow instead of the literal source code in order to lift unnecessary restrictions from safe code.
But what are the "necessary" restrictions in the first place?

In case of object `b: Box<T>` where `T: Copy`, its borrow `br: &Box<T>` or arguably the equivalent `r: &T` shouldn't be used (read from) after `b` is freed (dropped),
i.e. we empose a restriction that `drop(b)` doesn't succeed `read(r)`.
Restriction comes from the fact, that reading freed memory could result in loading another object's bytes.
This makes the program behaviour too difficult to analyze, esspecially with the assumptions we take when coding operations on those objects.
Thus most languages simply prohibit such order of operations as `drop(b),read(r)`.

Rust expresses these rules using two types of borrows: unique (mutable) and shared (immutable) borrows.
Previous paragraph showed when shared borrows are useful.
Unique borrows by analogy for any operation `op(b)` could prohibit `r=borrow_mut(b),op(b),write(r)`,
which becomes useful when working, for example, on enums with fields (check out [`cell-project`] documentation on its limitations).

On the day of writing this text, Rust borrow checker (from what I know) works on the aforementioned scope-based model of borrows.
Let's try instead expressing only the necessary restrictions using lifetimes tokens and relations between them.
Walking back from types for a moment, we have the *lifetime tokens* (further simply *tokens*) such as `'a`, and the *lifetime inclusion* relation `'b: 'a`.
This is enough to establish a [*preorder*] over those tokens, so let's establish that `'b: 'a` means \\(b ≥ a\\) by the rustacean de-facto convetion.
Additionally we have `'static` as the largest lifetime token, i.e. for any `'a` holds `'static: 'a` or \\(\mathrm{static} ≥ a\\).

It is also enough to form a *thin category*, with lifetime tokens as its objects and lifetime inclusion as its morphisms,
which means the same thing except for larget attention to properties such as:

- Transitivity (if `'b: 'a` and `'c: 'b` then `'c: 'a`);
- Reflexivity (`'a: 'a`);
- Bonus: the terminal object (`'static`).

<!-- TODO: borrowtime(b) >= lifetime(r) -->

[Cyclone language]: https://cyclone.thelanguage.org/
[Non-lexical lifetimes]: https://smallcultfollowing.com/babysteps/blog/2016/04/27/non-lexical-lifetimes-introduction/
[`cell-project`]: https://docs.rs/cell-project/0.1.4/cell_project/index.html
[*preorder*]: https://en.wikipedia.org/wiki/Preorder "basically a partial order, except that for 'a ≤ b' and `b ≤ a` judgement 'a = b' may not hold true"
