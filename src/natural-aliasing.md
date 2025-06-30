# Natural aliasing model

# Motivation

I have in a sense conflicting feelings about Rust.
In my opinion it is the most expressive and intentive compiled language as of 2025 that I've yet seen.
It is really a miracle that such a complicated programming language became mainstream.
It is a proof that language's complexity could be beneficial up to defining its public image.
However I can't get rid of the occasional feeling that some suboptimal descisions about Rust's development were made.
Furthermore Rust's aim at everlasting stability makes me more sensitive to such descisions.

Today I've found a way to substantiate some of my alternative vision on the language's type system.
In this text I'll touch upon several aspects of our type system:

- Why and how `Send` futures may contain `!Send` types like `Rc`;
- Why hypothesized `Forget` marker trait does not prevent memory leaks and remains useful;
- Why `&Cell` is a natural extension of mutable borrows.
- The general role of less or more so copyable types in Rust's type system;
- Etc.

# Introduction

In the first half of the 20th century mathematicians were investigating bounds of the classical logic.
As one of significant results of this work Gerhard Gentzen prooved the cut-elimination theorem, which relates to the consistency of logic.
Proof's essential part is reasoning about logical connectives and their introduction and elimination.
This thinking will suit our purposes for proving that some types' properties are closed under interactions with them.
