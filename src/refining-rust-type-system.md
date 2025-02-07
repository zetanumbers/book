# Refining the Rust type system

<!--toc:start-->
- [Refining the Rust type system](#refining-the-rust-type-system)
  - [Introduction](#introduction)
  - [What is a lifetime?](#what-is-a-lifetime)
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
I hope you will consider it in your further work.
So let's start from the begining.

## How do lifetimes work?

You may have heard, that Rust type system was inspired by the [Cyclone language].
It was a research project, which have implemented a clever region-based memory managment, which relates to the common understanding of how lifetimes work.
To recap: Rust compiler associates each lifetime token to a certain scope, where an object can live until the end of the scope.

Things changes a bit after introduction of [Non-lexical lifetimes].
Scopes were redefined in terms of the control-flow instead of the literal source code in order to lift unnecessary restrictions from safe code.
But what are "necessary" restrictions in the first place?



[Cyclone language]: https://cyclone.thelanguage.org/
[Non-lexical lifetimes]: https://smallcultfollowing.com/babysteps/blog/2016/04/27/non-lexical-lifetimes-introduction/
