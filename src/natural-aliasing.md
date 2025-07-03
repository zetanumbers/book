# Natural aliasing model

## Motivation

I have in a sense conflicting feelings about Rust.
In my opinion it is the most expressive compiled language as of 2025 that I've yet seen.
It is really a miracle that such a complicated programming language became mainstream.
It is a proof that language's complexity could be beneficial up to defining its public image.
However I can't get rid of the occasional feeling that some suboptimal decisions about Rust's development were made.
Furthermore Rust's aim at everlasting stability makes me more sensitive to such decisions.

More than a year later after my initial suspicions, today I've found a way to substantiate some of my alternative vision on the language's type system.
In this text I'll touch upon several aspects of our type system:

- Why and how `&Cell` is a natural extension of mutable borrows;
- Alternative, more general than `Send`, approach to thread-safety;
- Why and how `Send` futures may contain `!Send` types like `Rc`;
- Why and how hypothesized [`Forget`] marker trait does and does not prevent memory leaks and use-after-free bugs;
- The general role of less or more so copyable types in Rust's type system;
- Etc.

[`Forget`]: ./myosotis.md

## Introduction

Let's first focus on the power of `Cell`.
In usual memory-safe languages (Java, JavaScript, Python, etc) objects are conceptualized as arbitrary aliased pointers with reference counters or GC, just like `Rc<Cell<T>>`.
I have found this approach to lack needed control for the complexity of those object semantics.
Rust grants me this control with lifetimes and complicated library of generic types.

My grudge with object semantics of other memory-safe languages comes down to this:

```javascript
a.c = 13
b.c = 42
assert(a.c == 13) // may fail if `b = a`
```

For myself I found this failing code to be very unintuitive, from my assumption that names `a` and `b` represent two distinct entities.
But in javascript such aliasing is permitted.
It is, however, becomes intuitive once I am aware, when such aliasing could is taking place:

```javascript
b = a
a.c = 13
b.c = 42
assert(a.c != 13)
```

Rust allows to make a distinction between aliased and unaliased borrows:

```rust
// compare usual code
fn assert_unaliased(a: &mut S, b: &mut S) {
  a.c = 13;
  b.c = 42;
  assert_eq!(a.c, 13); // won't fail
}

fn assert_unaliased(a: &Cell<S>, b: &Cell<S>) {
  a.set(S { c: 13, ..a.get() });
  b.set(S { c: 42, ..b.get() });
  assert_eq!(a.get().c, 13); // may fail
}
```

To achieve this Rust restricts mutable borrows to be uncopyable, ensuring a mutable borrow is aliased in context exclusively by one variable's name.
This rule relates to the second JS case when we were aware of aliasing taking place, as it rules out information about aliasing at least one important way.
But what if it was more than one way?

### Simple aliasing

Consider adding a marker lifetime to the `Cell<'a, T>` type, to establish aliasing at the type level.
Although I am simplifying, now it is possible to express aliasing requirements like:

```rust
fn assert_aliased<'a>(a: &Cell<'a, S>, b: &Cell<'a, S>) {
  a.set(S { c: 13, ..a.get() });
  b.set(S { c: 42, ..b.get() });
  assert_eq!(a.get().c, 42); // Change to check if `a` contains value from `b`, won't fail
}
```

The same marker lifetime establishes that these cells alias the same memory region.
Compiler would complain otherwise if such `Cell` is designed properly (like [`GhostCell`] is).
This syntax essentially expresses the notion of "if you have put something in `a` or `b` you will get it from `a` and `b`", for aliasing references `a` and `b`.
However it is indivisible, as you cannot look at only one of two variables, without knowing what happens to the second one.
Instead of picking memory regions at random, programmers rely on memory allocators to ensure their memory is generally unaliased.
*Aliasing information is essential to develop a reasonable program*.

[`GhostCell`]: https://plv.mpi-sws.org/rustbelt/ghostcell/

### Better `Send`

This comes with a cool consequence of alternative definition of thread-safe/unsafe types.
It would be safe to send a type across the thread boundary only if it's aliased memory region isn't aliased anywhere else.
To avoid to talk about plain borrows, consider `Rc<'a, T>` implemented using new `Cell<'a, usize>` as a reference counter.
It is safe to send `a: Rc<'a, T>` to another thread if there isn't any other `b: Rc<'a, T>` left on the old thread.
But more than that, if there is another `b: Rc<'a, T>`, we still could send both of them `(a, b)` across threads.
I have found type annotation for [higher-ranked lifetimes] `(a, b): for<'a> (Rc<'a, T>, Rc<'a, T>)`, although formally ambiguous, to be quite fitting.
Now you can see yourself why `&mut T` would be just a non-copyable version of `for<'a> &Cell<'a, T>`.

From this we could even restore the original `Send` oriented design.
The `!Send` implementation on a type essentially tells that utilized memory region could be (non-atomically, without synchronization) aliased from the *current thread*.
This stems from the assumption that the function body execution always stays on the same thread until its finished.
That assumption is the reason of some limitations on stackless (async blocks) and [stackful] coroutines around `Send`.
This also allows to store `!Send` types in thread locals, which then becomes the [**evident cornerstone**] of problems with async and `Send`.

The solution to that problem would be to abstract assumption into a type, let's say, `ThreadLocalKey<'a>` zero-sized type that would allow thread-unsafe access to thread locals.
But you shouldn't be able to prove that `'a` aliasing lifetime does not occur somewhere else, so you won't ever be able to send it across threads.
Any function requiring thread-unsafe access to thread-locals would have to get this type through its arguments.
This then would be reflected in the function signature, which would inform whether function body is sendable across threads or not.

This way you could imagine a `Future` gets `ThreadLocalKey<'a>` through its `poll` method,
which explains why storing any thread-unsafe type `T: 'a` should make the compiler assume future is thread-unsafe as a whole.
Unless that future's internal structure contains types only with `for<'a>` bounded aliasing lifetimes!

**You should notice that now the thread-safe property of a type could be defined solely from the *type's boundary*, i.e. its safe public interface.**

Unfortunately it's not possible to realize such thread-safety checking behavior in the type system today.
It would require to extend capabilities of lifetimes, potentially even allowing self-referential types to be defined in safe way,
or even introducing another type of aliasing lifetime.

[higher-ranked lifetimes]: https://doc.rust-lang.org/nomicon/hrtb.html
[stackful]: https://docs.rs/corosensei/0.2.2/corosensei/index.html
[**evident cornerstone**]: https://blaz.is/blog/post/future-send-was-unavoidable/

### Compound aliasing and borrows

On that note, this analogously explains why regular lifetimes inside of an async block is "squashed" to `'static` from the outside perspective.
Such lifetimes simply aren't reflected in the future's type boundary.

But to dive a bit deeper, we have to develop this connection of borrows and aliasing further.
What does (re)borrowing actually mean?
For this let's investigate a difference between two aliasing cell references and one mutable reborrow of a mutable reference:

```rust
// notice symmetry between `a` and `b`
fn assert_aliased_cell<'a>(a: &Cell<'a, S>, b: &Cell<'a, S>) {
  a.set(S { c: 13, ..a.get() });
  b.set(S { c: 42, ..b.get() });
  assert_eq!(a.get().c, 42); // ok!
}

fn assert_aliased_mut(a: &mut S) {
  a.c = 13;
  let b = &mut *a; // reborrow
  b.c = 42;
  assert_eq!(a.c, 42); // obviously ok!
}

// what if we swap `a` and `b`?
// now notice the antisymmetry
fn assert_aliased_mut_bad(b: &mut S) {
  let b = &mut *a; // reborrow
  a.c = 13;
  b.c = 42; // compiler error!
  assert_eq!(a.c, 42);
}
```

So it looks like that it isn't actually correct to call mutable references unique.
Rather, mutable borrows allow aliasing in a compound fashion.
Pick the `assert_aliased_mut` example.
As you can see, from `a`'s perspective `b` aliases it, while from `b`'s point of view nothing aliases it at the moment, it is *exclusive*.
At this moment it is as reasonable to look at `b` alone and to look at both `a` and `b`, while considering only `a` won't tell you much about program's behavior.
In this sense `a`'s aliasing info is included in `b`'s aliasing info.

## Justification

I hope it is clear to you why looking at aliasing variables separately hurts programmer's ability to develop reasoning about a program's behavior.
To be more precise, you have to know what happens to different aliases to construct a sound program.
While it is possible to write a public library, working with aliased memory, it is library users' task to put the pieces together to conclude a program.
Otherwise we would call that possible memory corruption.

If you have ever delved into topology, you might recognize that neighborhoods of aliased variables could be expressed with some topology.
Naively we could say two variables alias the same memory whenever they alias same memory addresses.
This means entails map \\(m\\) from the collection of aliasing variables \\(V\\) to a powerset of the address space \\(2^A\\).
However this doesn't account for compound aliasing of reborrowing.
Instead we should specify compoundness level using natural numbers: \\((1 + \mathbb{N})^A\\).
Call this \\(m\\) an *address map*.

Neighborhoods in topology on a collection of aliasing variables are grouped by their mapping to same addresses,
while substracting the number of lowest compoundness levels over some addresses also gives an open set.
In other words, assuming \\(\mathbb{N}\\) is a topological space, with, for every natural number \\(n\\), open set defined as every natural number below \\(n\\),
there's a smallest fitting topology \\(\tau\\) with open sets defined from preimages of continuous map \\(m\\).
For any set of aliasing variables \\(V\\) we will call this \\(\tau_V\\) an *aliasing topology*.

In the first half of the 20th century mathematicians were investigating bounds of the classical logic.
As one of significant results of this work Gerhard Gentzen proved the cut-elimination theorem, which relates to the consistency of logic.
Proof's essential part is reasoning about logical connectives and their introduction and elimination.
This thinking will suit our purposes for proving that some types' properties are closed under interactions with them.

First consider type product, i.e. pairs and tuples.
Address map of tuple of variables has to be the union of address maps of individual variables.
For pair:

\\[\forall \\{x: X, y: Y\\} \subseteq V : m((x, y): X \times Y) := m(x) \cup m(y)\\]
