# Natural aliasing model

*Draft 2025-07-07*

<!--toc:start-->
- [Natural aliasing model](#natural-aliasing-model)
  - [Motivation](#motivation)
  - [Introduction](#introduction)
  - [Simple aliasing](#simple-aliasing)
  - [Better `Send`](#better-send)
  - [Compound aliasing and borrows](#compound-aliasing-and-borrows)
  - [Immutable borrows](#immutable-borrows)
  - [Allocations and `Forget`](#allocations-and-forget)
  - [`Copy`](#copy)
  - [Ownership and self-referencial types](#ownership-and-self-referencial-types)
  - [`Move` and runtime ownership tracking](#move-and-runtime-ownership-tracking)
  - [Aliasing topology](#aliasing-topology)
  - [Justification for the fundamental aliasing rule](#justification-for-the-fundamental-aliasing-rule)
  - [Afterword](#afterword)
<!--toc:end-->

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
- Self-referencial types
- Etc.

To put it simply: this text is all about abstraction of memory aliasing.
Although I am not a good writer, I've tried to explain things in a manner similar to a Rust programmer.
Nonetheless, due to my lack of experience, I expect this text to contain a good amount a flaws and errors.

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

## Simple aliasing

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

Nonetheless I will immediately contradict myself there, but not really.
You can absolutely define a reasonable subroutine working on aliased memory, although to do that, *you have to make it clear to the user what you are doing*.
A part of that would be the understanding that `&Cell`s outside of the subroutine call aren't used until subroutine returns. 

[`GhostCell`]: https://plv.mpi-sws.org/rustbelt/ghostcell/

## Better `Send`

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
I will name this rule the *fundamental aliasing rule*, although pretentious, in the context of our theory it is worth its name.

Unfortunately it's not possible to realize such thread-safety checking behavior in the type system today.
It would require to extend capabilities of lifetimes, potentially even allowing self-referential types to be defined in safe way,
or even introducing another type of aliasing lifetime.

[higher-ranked lifetimes]: https://doc.rust-lang.org/nomicon/hrtb.html
[stackful]: https://docs.rs/corosensei/0.2.2/corosensei/index.html
[**evident cornerstone**]: https://blaz.is/blog/post/future-send-was-unavoidable/

## Compound aliasing and borrows

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
  let a = &mut *b; // reborrow
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

## Immutable borrows

Immutable borrows allows us to worry less about aliasing.
Rather, restricting mutability of a reference allows us to disregard any aliasing information on that borrow.
That is, aliasing information on an immutable borrow is quite trivial, limited to compound aliasing of borrowed by it mutable references.
Even more trivial case would be of a `static SOME_REFERENCE: &'static T = &T {}`, where static immutable references are ideally what a programmer would like to see.
This is the kind of aliasing functional programming languages use, where every variable should be interpreted "at face value".

## Allocations and `Forget`

So what about a `Box` we would only read from?
Would that be the same as for static immutable references?
Obviously no.
If you've got a hang of Rust, you might draw a comparison between mutable borrows and memory allocators.
In a sense, memory allocation is a borrow of the memory allocator, or rather, a part of its managed memory.
That's why it's sometimes more compelling to implement custom memory allocators using mutable borrows instead of some kind of a `Box` type, like [bumpalo].

The only difference between a `Box` and a mutable borrow is in the party responsible for bookkeeping, either the compiler or the runtime.
However, if something isn't handled by the compiler, it becomes syntactically invisible to us, which then explains why memory leaks are considered safe.
Part of it, the function [`std::mem::forget`] allows anything to escape syntax and render its runtime effects invisible.
In order to guarantee absence of memory leaks, compiler should be aware of this kind of aliasing information too, just like for `&mut T`.
This entails a type of API used by aforementioned memory allocators and arenas, maybe with some portion of runtime bookkeeping via [custom `Box` type] with lifetime.

This is where hypothetical [`Forget`] trait comes to rescue.
While it was satisfying to realize that `Forget` was tightly involved with lifetimes, its lack of connection to memory leaking was uncanny.
But now there's an answer: it comes from the allocator interface design.
If allocation wasn't a free function, but designed as explained above, `!Forget` would have prevented those leaks.
Noticeably, if you consider the rule of aliasing information of a type is being closed under its public interface,
it would be ok to forget allocations, if we also forget about the allocator itself.

Although that warrants a question "wouldn't allocator need to allocate memory from somewhere in order to hold it?"
The answer is yes, allocator is by definition the way of tracking unaliased memory,
thus for every allocator we should establish there's no intersections between allocators, for which we need an allocator.
This leaves us to conclude that there has to be a chain of runtime and compile-time allocators, with the primordial allocator at the beginning.
I'll argue this primordial allocator is your consistent decision on division of memory between allocators,
possibly leaving a single runtime allocator on the entire physical memory.

[bumpalo]: https://docs.rs/bumpalo/3.19.0/bumpalo/struct.Bump.html#method.alloc
[`std::mem::forget`]: https://doc.rust-lang.org/1.88.0/std/mem/fn.forget.html
[custom `Box` type]: https://docs.rs/bumpalo/3.19.0/bumpalo/boxed/struct.Box.html

## `Copy`

Another funny thing to consider is absence `Copy` impl on type as being closed under its API.
That wouldn't make much sense for actual pointers, until we would consider pointers as indices.
Global memory could be thought of as a singleton byte array we index using load and store instructions.
And in reverse, if we would ever consider indices to be pointers with **multiple** memories,
it allows to copy the whole memory region, leaving stored these pseudo-pointers to be valid.
But alas I find this thinking a bit unclear for implementation yet.

[`RefOnce`]: https://docs.rs/scope-lock/0.3.1/scope_lock/struct.RefOnce.html

## Ownership and self-referencial types

What is ownership really?
Coming from above section, I hope you consider an argument that it is about giving/taking something and taking/giving it back.
**In order to give something you have to take it first, and so in order to give something back, you need to take back what you gave.**
First statement is about composition of constructors, how constructor of a structure utilizes its field's constructor.
But the second one is more interesting, as it stands for composition of destructors.
Rust automates destructor nesting largely due to implicit destruction of variables, although there is probably a fitting alternative.
No matter, as we still can make sense of it in a few new ways.

One way is to reexamine, so called, self-referencial types.
Take the infamous doubly-linked list for example.
A list as a whole contains a collection of allocated on a heap nodes with value field, next and previous nullable pointers to respective nodes.
There's a consistent way of deallocating all of these nodes.
For this sequence of nodes we can recursively deallocate its tail, and when we get the empty next node we can start deallocating nodes themselves.
It's just as if it was singly-linked list without the previous node pointer, which forms a tree of nodes.
Usually deallocation of a doubly-linked list is handled with a loop instead,
but that would be the same as if we took tail out of the head node and had the [tail call optimization].

To some extent this thinking of converting types with reference cycles into a tree of references is unavoidable, because of our conceptualization of ownership.
At least this allows to refine our thinking, to **compose destructors and think about them separately**.
This nested ownership of types may resonate in other aspects of Rust language,
even if such feature would be a hypothetical, like [structured concurrency] for async code.

Returning back to doubly linked list,
my suggestion for trying to came up with safe syntax for self-referencial types in this case would be to regard list nodes in two different modes:
as a singly-linked list node, with next pointer resembling a `Box`,
and as a doubly-linked list node with next and previous pointers as arbitrary aliasing mutable borrows.
Top-level, you would consider list of nodes in second mode by creating a `&Cell` borrow of list's head in singly-linked mode.
This is kinda what [`GhostCell`] does already.
Also this sits well with my intuition about async blocks with references to other local variables, which is yet to be put on paper.

[tail call optimization]: https://en.wikipedia.org/wiki/Tail_call
[Structured Concurrency]: https://blog.yoshuawuyts.com/tree-structured-concurrency

## [`Move`] and runtime ownership tracking

I guess this is an appropriate place to mention, that the program stack is also an allocator.
Many uncomfortable consequences stand from this nuance, like restricting values from moving between scopes when borrowed.
But it seems possible to somewhat alleviate this using a primitive like [`RefOnce`] or `&own T` which I've found a use in one of my libraries.
This makes me think that, if stack frame allocation had a syntax with lifetimes,
then inability to move a type would have been expressed as inability to place a type into something with a bigger lifetime.
Otherwise this may lead it to being able to witness that type in outlived/invalid state, which `RefOnce` avoids by borrowing only memory for that type to occupy.

And again, back to `Forget`.
One of this trait's flaws would have been unclear semantics about what type would require of a type to be forgettable.
For example, `Rc` can be placed into itself, introducing a reference cycle.
To handle this it is required to restrict `Rc<'a, T>` with aforementioned aliasing lifetime from being put into itself somehow using lifetimes to track down such case.
But it becomes obvious if we remember that `Rc` shifts responsibility of tracking ownership to runtime,
which usually isn't aware of any syntactic scopes we keep in mind in order to think about ownership.
In order to understand how `Rc`s are tracking memory allocation, appropriately you would need to keep in mind all of them.
More appropriately you would reason about `Rc` as aliasing mutable borrows to allocated memory.

Precisely upon dropping `Rc`s, runtime filters out contexts its allocated memory belongs to, sort of like it's in superposition until then.
On the second to last drop of `Rc` we would know one definite context where its allocated memory is placed,
which currently could be either `Rc` itself or some other syntactic context we have hold on.
This thinking also extends to channels like MPSC, which have exhibit similar unclear/runtime ownership.

[`Move`]: https://blog.yoshuawuyts.com/self-referential-types-2/

## Aliasing topology

I hope it is clear to you why looking at aliasing variables separately hurts programmer's ability to develop reasoning about a program's behavior.
To be more precise, you have to know what happens to different aliases to construct a sound program.
While it is possible to write a public library, working with aliased memory, it is library users' task to put the pieces together to conclude a program.
Otherwise we would call that possible memory corruption.

If you have ever delved into topology, you might recognize that neighborhoods of aliased variables could be expressed with some topology.
Naively we could say two variables alias the same memory whenever they alias same memory addresses.
This means entails map \\(m\\) from the collection of aliasing variables \\(V\\) to a powerset of the address space \\(2^A\\).
However this doesn't account for compound aliasing of reborrowing.

So that means instead of a mapping to boolean domain \\(2\\), we should map from address space to *topological space of aliasing constructions* defined as:
points as strings of the form `b*(f[oi]*)?` or `0` using regex notation
and (infinite) opens sets defined to hold every valid string can we get by appending some suffix and the `0` string standing for unaliased memory address.
This way mutable (aliasing) borrows would map to strings `b+` with each `b` symbol corresponding to one reborrow,
and immutable borrows map to strings `b*f[oi]*` where `f` standing for freezing mutable reference into immutable one and then either of `[oi]*` sequences.
Whenever copying or reborrowing an immutable borrow, we assign old one a new string with added `o` and new one new string with added `i`,
which would ensure that every such variable of immutable borrow forms a singleton open set.
There's a smallest fitting topology, set of open subsets, \\(\tau\\) with open sets defined from preimages of continuous map \\(m\\) which I will name *alias map*.
For any set of aliasing variables \\(V\\) we will call this \\(\tau_V\\) an *aliasing topology* on space \\(V\\).

This description, sadly, is too mechanical to be a good mathematical definition.
However, although I lack confidence in defining it in such way,
I suspect aliasing topology can be expressed as sieves of an appropriate category and alias map to be Grothendieck construction.
Moreover, my intuition about this subject while based in Grothendieck topos of sheaves on a site, I am yet to develop a confidence to express my ideas this way.
But I hope more knowledgeable people would connect dots together if such interpretation was appropriate for the subject of natural aliasing.

## Justification for the fundamental aliasing rule

Now to define product type (pair and tuples) of this theory, it is most fitting to define alias map from the pair of variables as union of alias maps from each variable.
This allows us to disregard individual members of a tuple and view it only as a whole.
It also means that alias map of a pair of borrowed and borrowing variables is the same as alias map of that borrowing variables by itself,
which should make sense if you remember the compound aliasing.

Another important type construction would be exponential types, i.e. closures.
Closures are important for type erasure of a variable, or tuple, and consider any construction of a closure identical by their alias maps.
This makes it possible to abstract any function call as a `FnOnce()` closure and disregard internal contents of the closure except for its captured variables.
Important consequence to note: [β-reduction] on such closure is able to change its alias map,
which is fine as long as **closure's alias map constitutes an open set in the aliasing topology**.
Nonetheless this constitutes the ability to *think about aliasing of variables solely by public interfaces of their constructions*.

Aliasing topology also establishes determinism for applications β-reduction rule, which is another way to say that if we know variables are unaliased,
we could use memory to store and load values in a deterministic and consistent way.

[β-reduction]: https://en.wikipedia.org/wiki/Lambda_calculus#%CE%B2-reduction_2

## Afterword

I would appreciate and credit your contributions if you share me useful improvements to this text.
I hope all this abstract nonsense would help guide rust-lang's and other languages future,
as there are a lots of implications about the memory-safe language design to consider.
