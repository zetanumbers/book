# The destruction guarantee and linear types formulation

<a title="Forget-me-nots - Sedum Tauno Erik, CC BY-SA 2.5 &lt;https://creativecommons.org/licenses/by-sa/2.5&gt;, via Wikimedia Commons" href="https://commons.wikimedia.org/wiki/File:Myosotis_arvensis_ois.JPG"><img width="512" alt="Myosotis arvensis ois" src="https://upload.wikimedia.org/wikipedia/commons/thumb/e/eb/Myosotis_arvensis_ois.JPG/512px-Myosotis_arvensis_ois.JPG"></a>

## Background

Currently there is a consensus about absence of
the <dfn id="intro-drop_guarantee"> [drop guarantee](#term-drop_guarantee) </dfn>. To
be precise, in today's Rust you can forget some value via
[`core::mem::forget`](https://doc.rust-lang.org/1.75.0/core/mem/fn.forget.html)
or via some other safe contraption like cyclic shared references `Rc/Arc`.

As you may know in the early days of Rust the
destruction guarantee was intended to exist. Instead of today's
[`std::thread::scope`](https://doc.rust-lang.org/1.75.0/std/thread/fn.scope.html)
there was
[`std::thread::scoped`](https://doc.rust-lang.org/1.0.0/std/thread/fn.scoped.html)
which worked in a similar manner, except it used a guard value
with a drop implementation to join the spawned thread so that it
wouldn't refer to any local stack variable after the parent thread
exited the scope and destroyed them, but due to absence of the drop
guarantee it was found to be unsound and was removed from standard
library.<sup id="cite_ref-1">[\[1\]](#cite_note-1)</sup> Let's
name these two approaches as <dfn id="intro-guarded_closure">
[guarded closure](#term-guarded_closure)
</dfn> and <dfn id="intro-guard_object"> [guard
object](#term-guard_object) </dfn>. Also to note C++20 has analogous
[`std::jthread`](https://en.cppreference.com/w/cpp/thread/jthread)
guard object.

There is also a discussion among Rust theorists about <dfn
id="intro-linear_type"> [linear types](#term-linear_type) </dfn>
which leads them researching (or maybe revisiting) the possible `Leak`
trait. I've noticed some confusion and thus hesitation when people are
trying to define what does leaking a value mean. I will try to clarify
and define what does leak actually mean.

## Problem

There is a class of problems that we will try to solve. In particular,
we return some object from a function or a method that mutably
(exclusively) borrows one of function arguments. While returned object
is alive we could not refer to borrowed value, which can be a useful
property to exploit. You can invalidate some invariant of a borrowed
type but then you restore it inside of returned object's drop. This
is a fine concept until you realize in some circumstances drop is not
called, which would in turn mean that the borrowed type invariant
invalidation may never cause <dfn id="intro-undefined_behavior">
[undefined behavior](#term-undefined_behavior) </dfn> (UB in
short) if left untreated. However, if drop is guaranteed, we could
mess with borrowed type invariant, knowing that the cleanup will
restore the invariant and make impossible to cause UB after. I
found one example of this as once mentioned planned feature
[`Vec::drain_range`](https://github.com/rust-lang/rust/issues/24292#issuecomment-93513451).

One other special case would be owned scoped thread. It may be included
within class of problems mentioned, but I am not sure. Anyway, in the most
trivial case this is the same as once deleted `std::thread::{scoped,
JoinGuard}` described above. However, many C APIs may in some
sense use this via the <dfn id="intro-callback_registration">
[callback registration](#term-callback_registration) </dfn>
pattern, most common for multithreaded client handles. Absence
of a drop guarantee thus implies `'static` lifetime for a
callback so that the user wouldn't use invalidated references
inside of the callback, if client uses guard object API pattern ([see
example](https://docs.rs/tigerbeetle-unofficial-core/0.3.0+0.13.133/tigerbeetle_unofficial_core/struct.Client.html#method.with_callback)).

## Solution

From now on I will use the term <dfn
id="term-destruction_guarantee">"destruction guarantee"</dfn> instead of
the "drop guarantee" because it more precisely describes the underlying
concept. The difference between drop and destruction is that first only
relates to drop functionality of Rust, while latter can relate to those
and any consuming function that destroys object in sense of how it is
defined by library authors, in other words a <dfn>destructor</dfn>. Such
destructors may even disable drop code and cleanup in some other way.

Most importantly in these two cases objects with the destruction
guarantee would be bounded by lifetime arguments. So to define the
destruction guarantee:

```
Destruction guarantee asserts that bounding lifetime of an object
must end only after object is destroyed by drop or any other valid
destructor. Somehow breaking this guarantee can lead to UB.
```

Notice what this implies for `T: 'static` types. Since static lifetime
never ends or ends only after end of program's execution, the drop
may never be called. This property does not conflict with described
use cases. `JoinGuard<'static, T>` indeed doesn't require to ever
be destroyed, since there would be no references that would ever be
invalidated.

In the context of discussion around `Leak` trait some argue it is possible
to implement `core::mem::forget` via threads and an infinite loop.<sup
id="cite_ref-2">[\[2\]](#cite_note-2)</sup> That forget implementation
won't violate a destruction guarantee as defined above, since either
you use regular threads which require `F: 'static` or use scoped threads
which would join this never completing thread thus no drop and no lifetime
end. **That definition only establishes order between object's destruction
and end of a lifetime, but not existence of a lifetime's end inside of
any execution time.** My further advice would be in general to **think
not in terms of execution time but in terms of semantic lifetimes**,
which role would be to conservatively establish order of events if
those ever exist. Alternatively you will be fundamentally limited by the
[halting problem](https://en.wikipedia.org/wiki/Halting_problem).

On the topic of abort or exit, it shouldn't be considered an end to any lifetime,
since otherwise abort and even spontaneous termination of a program like
SIGTERM becomes unsafe.

To move forward let's determine required conditions for destruction
guarantee. Rust language already makes sure you could never use a
value which bounding lifetime has ended. Drop as a fallback to other
destructors is only ever run on owned values, so for a drop to run
on a value, **the value should preserve transitive ownership of it
by functions' stack/local values**. If you familiar with [tracing garbage
collection](https://en.wikipedia.org/wiki/Garbage_collection_(computer_science)#Tracing)
this is similar to it, so that the required alive value should be
traceable from function stack. The value has to not own itself or be
owned by something that would own itself, at least before the end of its
bounding lifetime, otherwise drop would not be called. Last statement
could be simplified, given that **owner of a value transitively must also
satisfy these requirements**, leaving us with just **the value has to not
own itself**. Also reminding you that `'static` values can be moved into
static context like static variables, which lifetime exceeds lifetime
of a program's execution itself, so consider that analogous to calling
`std::process::exit()` before `'static` ends.

## Trivial implementation

One trivial implementation might have already crept into your mind.

```rust
unsafe auto trait Leak {}
```

This is an automatic trait, which would mean that it is
implemented for types in a similar manner to `Send`.<sup
id="cite_ref-3">[\[3\]](#cite_note-3)</sup> Name `Leak` is a subject
for a possible future change. I used it as it came up in many people's
thoughts as `Leak`. Since `T: !Leak` types possibly could leak in a
practical meaning, it can be renamed into `Forget`. Other variants could
be `Lose`, `!Trace` or `!Reach` (last two as in tracing GC), maybe add
`-able` suffix?

This trait would help to forbid `!Leak` values from using problematic
functionality:

 - Obviously `core::mem::forget` should have a `T: Leak` over its generic
 type argument;
 - `core::mem::ManuallyDrop::new` should have leak bound over input type,
 but intrinsically maybe author has some destructor besides the drop
 that would benefit from `ManuallyDrop::new_unchecked` fallback;
 - `Rc` and `Arc` may themselves be put inside of the contained value,
 creating an ownership loop, although there should probably be an unsafe
 (constructors) fallback in case ownership cycles are guaranteed to be
 broken before cleanup;
 - Channel types like inside of `std::sync::mpsc` with a shared buffer of
 `T` are problematic since you can send a receiver through its sender back
 to itself, thus creating an ownership cycle leaking that shared buffer;
  * Rendezvous channels seem to lack this flaw because they wait for
  other thread/task to be ready to take a value instead of running off
  right after sending it;

In any case the library itself dictates appropriate bounds for its types.

Given that `!Leak` implies new restrictions compared to current Rust
value semantics, by default every type is assumed to be `T: Leak`, kinda
like with `Sized`, e.g. implicit `Leak` trait bound on every type and
type argument unless specified otherwise (`T: ?Leak`). I pretty sure this
feature should not introduce any breaking changes. This means working with
new `!Leak` types is opt-in, kinda like library APIs may consider adding
`?Sized` support after release. There could be a way to disable implicit
`T: Leak` bounds between editions, although I do not see it as a desirable
change, since `!Leak` types would be a small minority in my vision.

#### The Unleak wrapper type

To make `!Leak` struct you would need to use new `Unleak` wrapper type:


```rust
// Some usage of Unleak, probably move JoinGuard there

// Unleak definition

#[repr(transparent)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unleak<T>(pub T, PhantomUnleak);

impl<T> Unleak<T> {
    pub const fn new(v: T) -> Self {
        Unleak(v, PhantomUnleak)
    }
}

// This is the essential part of the `Unleak` design.
unsafe impl<T: 'static> Leak for Unleak<T> {}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct PhantomUnleak;

impl !Leak for PhantomUnleak {}
```

This wrapper makes it easy to define `!Leak` data structures. It
implements `Leak` for `'static` case for you. As a rule of thumb
you determine which field (it should contain struct's lifetime or
generic type argument) would require the destruction guarantee, so
if you invalidate safety invariant of a borrowed type, make sure
this borrow is under `Unleak`.  To illustrate how `Unleak` helps
you could look at this example:

```rust
struct Variance<Contra, Co> {
    process: fn(Contra) -> String,
    // invalidate `Co` type's safety invariant before restoring it
    // inside of the drop
    queue: Unleak<Co>,
}
```

If you aware of
[variance](https://doc.rust-lang.org/reference/subtyping.html) then
you should know that contravariant lifetimes (which are placed
inside of arguments of a function pointer) can be extended via
subtyping up to the `'static` lifetime, it is also applied to
lifetime bounds of generic type arguments. So it should be useless
to mark this function pointer with `Unleak`. If we just had
`PhantomUnleak` there - this is what example above would look like
instead:

```rust
struct Variance<Contra, Co> {
    process: fn(Contra) -> String,
    queue: Co,
    _unleak: PhantomUnleak,
}

unsafe impl<Contra, Co: 'static> Leak for Variance<Contra, Co> {}
```

It now requires unsafe impl with a bit unclear type bounds. If user
forgets to add the `Leak` implementation the type would become restricted
as any `!Leak` type even if type itself `'static`, granting nothing of
value. If user messes up and doesn't add appropriate `'static` bounds,
It may lead to unsound API. `Unleak` on the other hand automatically
ensures that `T: 'static => T: Leak`. So the `PhantomUnleak` should
probably be private/unstable.

Now given this a bit awkward situation about `T: 'static => T: Leak`,
impl and dyn trait types can sometimes be meaningless like `Box<dyn
Debug + ?Leak>` or `-> impl Debug + ?Leak` because those are static
unless you add `+ 'a` explicit lifetime bound, so there probably
should be a lint that would warn user about that.

One thing that we should be aware of in the future would be users'
desire of making their types `!Leak` while not actually needing it. The
appropriate example would be `MutexGuard<'a, T>` being `!Leak`. It is
not required, since it is actually safe to forget a value of this type or
to never unlock a mutex, but it can exist. In this case, you can safely
violate `!Leak` bound, making it useless in practice. Thus unnecessary
`!Leak` impls should be avoided. To address users' underlying itch to
do this, they should be informed that **forgetting or leaking a value
is already undesirable and can be considered a logic bug**.

Of course there should be an unsafe `core::mem::forget_unchecked` for
any value if you really know what you're doing, because there are some ways
to implement `core::mem::forget` for any type with unsafe code still,
for example with `core::ptr::write`. There should also probably be safe
`core::mem::forget_static` since you can basically do that using thread
with an endless loop. However `?Leak` types implement `Leak` for static
lifetimes transitively from `Unleak` to satisfy any function's
bounds over types.

```rust
// not sure about variance here
struct JoinGuard<'a, T: 'a> {
    // ...
    _marker: PhantomData<fn() -> T>,
    _unleak: PhantomData<Unleak<&'a ()>>,
}
```

While implementing `!Leak` types you should also make sure you cannot move
a value of this type into itself. In particular `JoinGuard` may be made
`!Send` to ensure that user won't send `JoinGuard` into its inner thread,
creating a reference to itself, thus escaping from a parent thread while
having live references to parent thread local variables.

```rust
// not sure about variance here
struct JoinGuard<'a, T: 'a> {
    // ...
    _marker: PhantomData<fn() -> T>,
    _unleak: PhantomData<Unleak<&'a ()>>,
    _unsend: PhantomData<*mut ()>,
}

unsafe impl<'static, T: 'static> Send for JoinGuard<'static, T> where Self: Leak {}
unsafe impl<'a, T> Sync for JoinGuard<'a, T> {}
```

There is also a way to forbid `JoinGuard` from moving into its
thread if we bound it by a different lifetime which is shorter than
input closure's lifetime. See prototyped `thread::SendJoinGuard`
in leak-playground
[docs](https://zetanumbers.github.io/leak-playground/leak_playground_std/)
and [repo](https://github.com/zetanumbers/leak-playground). Because
there's no `Leak` trait outside of this repo and external libraries
cannot account for it, `!Leak` types usage safety is enforced
manually sometimes. There're also some new possible features for
tokio in
[leak_playground_tokio](https://zetanumbers.github.io/leak-playground/leak_playground_tokio/)
like non-static task support. The doctest code behaves as intended
(except for internally unleak future examples), but I have no formal
proof of it being 100% valid.

One other consequence would be that if a drop of a `!Leak` object panics
it should be safe to use the *referred to* object, basically meaning that
panic or unwind is a valid exit path from the drop implementation. If
`!Leak` type invalidates some safe type invariant of a borrowed object,
then even if the drop implementation panics, it should restore this
invariant, maybe even by replacing the borrowed value with a default
or an empty value or with a good old manual `std::process::abort`. If
designed otherwise the code should abort on a panic from a drop of `!Leak`
value. So **you would have to be careful with panics too**. This also
applies to any other destructor.

#### Internally Unleak coroutines

Consider one other example from
[leak-playground](https://zetanumbers.github.io/leak-playground/leak_playground/):

<div id="internal_unleak_future">

```rust
fn _internal_unleak_future() -> impl std::future::Future<Output = ()> + Leak {
    async {
        let num = std::hint::black_box(0);
        let bor = Unleak::new(&num);
        let () = std::future::pending().await;
        assert_eq!(*bor.0, 0);
    }
}
```

</div>

During the execution of a future, local variables have non-static
lifetimes, however after future yields these lifetimes become static
unless they refer to something outside of it. This is an example of sound
and safe lifetime extension thus making the whole future `Leak`. However,
if when we use `JoinGuard` it becomes a little bit trickier:

```rust
fn _internal_join_guard_future() -> impl std::future::Future<Output = ()> + Leak {
    async {
        let local = 42;
        let thrd = JoinGuard::spawn({
            let local = &local;
            move || {
                let _inner_local = local;
            }
        });
        let () = std::future::pending().await;
        drop(thrd); // This statement is for verbosity and `thrd`
                    // should drop there implicitly anyway
    }
}
```

Code above may lead to use-after-free if we `forget` this future,
meaning that memory holding this future is deallocated without
cancelling (i.e. dropping) this future first, thus spawned `thrd`
now refers to the future's deallocated local state, since we haven't
joined this thread.  But remember that self-referential (`!Unpin`)
future is pinned forever after it starts, which means that it is
guaranteed there is no way (or at least should be no way) to forget
and deallocate the underlying value in safe code (see pin's [drop
guarantee](https://doc.rust-lang.org/std/pin/index.html#drop-guarantee)).
However outside of rust-lang project some people would not follow this
rule because they don't know about it or maybe discard it purposefully
(the *Rust police* is coming for you). Maybe in the future it would be
possible to somehow relax this rule in some cases, but it would be a
different problem.

## Extensions and alternatives

*DISCLAIMER: This section is optional as it contains unpolished concepts,
which are not essential for understanding the overall design of proposed
feature.*

### Disowns (and NeverGives) trait(s)

If you think about `Rc` long enough, the `T: Leak` bound will start to
feel unnecessary strong. Maybe we could add a trait that signify that your
type can never own `Rc` of self, which would allow us to have a new bound:

```rust
impl<T> Rc<T> {
    fn new(v: T) -> Self
    where
        T: Disowns<Rc<T>>
    {
        // ...
    }
}
```

By analogy with that to make sure closure that you pass into a spawned
thread should never capture anything that can give you join guard:

```rust
pub fn scoped<F>(f: F) -> JoinGuard<F>
where
    F: NeverGives<JoinGuard<F>>
{
    // ...
}
```

To help you with understanding:

```
<fn(T)>: NeverGives<T> + Disowns<T>,
<fn() -> T>: !NeverGives<T> + Disowns<T>,
T: !NeverGives<T> + !Disowns<T>,
trait NeverGives<T>: Disowns<T>,
```

### Custom Rc trait

Or, to generalize, maybe there should be a custom automatic trait for Rc, so
that anything that implements it is safely allowed to be held within `Rc`:

```rust
impl<T> Rc<T> {
    fn new(v: T) -> Self
    where
        T: AllowedInRc
    {
        // ...
    }
}

impl<T> Arc<T> {
    fn new(v: T) -> Self
    where
        T: AllowedInRc + Send + Sync
    {
        // ...
    }
}
```

### Ranked Leak trait

While we may allow `T: Leak` types to be held within `Rc`, `U:
Leak2` would be not given that `Rc<T>: Leak2`. And so on. This
allows us to forbid recursive types but also forbids nested enough
within `Rc`s data types. This is similar to [von Neumann hierarchy
of sets](https://en.wikipedia.org/wiki/Von_Neumann_universe) as sets
there have some rank ordinal. Maybe there could be `unsafe auto trait
Leak<const N: usize> {}` for that?

### Turning drop invocations into compiler errors

Perhaps we could an have some automatic trait `RoutineDrop` which if
unimplemented for a type means that dropping this value would result
in compiler error. This may be useful with hypothetical async drop. It
could also help expand linear type functionality.

## Forward compatibility

Since I wrote this text in terms of destructors, it should be play nicely
with hypothetical async drop. Then it could be the case that `JoinGuard`
logic can be extended to analogous `AwaitGuard` representing async tasks.

## Possible problems

Some current std library functionality relies upon forgetting values,
like `Vec` does it in some cases like panic during element's drop. I'm
not sure if anyone relies upon this, so we could use abort instead. Or
instead we can add `std::mem::is_leak::<T>() -> bool` to determine if
we can forget values or not and then act accordingly.

Currently [internally unleak futures](#internal_unleak_future)
examples emit errors where they shouldn't or should emit different errors,
so I guess some compiler hacking is required. There could also be some
niche compilation case, where compiler assumes every type is `Leak`
and purposefully forgets a value.

<!--

## Discarded ideas

*This section may confuse readers so you might not want to skip it. It
is intended to be referred to when discussion touches these thought of
before but then discarded ideas.*

### Leak<'a>

I cannot describe this

-->

## Terminology

<dl>

<dt id="term-linear_type"> <a href="#intro-linear_type" title="Jump up">^</a> Linear type </dt>

<dd>

Value of which should be used at least once, generally speaking. Use is
usually defined within the context.

</dd>

<dl>

<dt id="term-drop_guarantee"> <a href="#intro-drop_guarantee" title="Jump up">^</a> Drop guarantee </dt>

<dd>

Guarantee that drop is run on every created value unless value's drop
is a noop.

This text uses this term only in reference to older discussions. I use
[destruction guarantee](#term-destruction_guarantee) instead to be more
precise and to avoid confusion in future discussions about async drop.

</dd>

<dt id="term-guarded_closure"> <a href="#intro-guarded_closure" title="Jump up">^</a> Guarded closure </dt>

<dd>

A pattern of a safe library API in Rust. It is a mechanism to guarantee
library's cleanup code is run after user code (closure) used some special
object. It is usually used only in situations when this guarantee is
required to achieve API safety, because it is unnecessary unwieldy
otherwise.

```rust
// WARNING: Yes I know you can rewrite this more efficiently, it's just a demonstration

fn main() {
    let mut a = 0;
    foo::scope(|foo| {
        for _ in 0..10 {
            a += foo.get_secret();
            // cannot forget(foo) since we only have a reference to it
        }
    });
    println!("a = {a}");
}

// Implementation

mod foo {
    use std::marker::PhantomData;
    use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};

    pub struct Foo<'scope, 'env> {
        secret: u32,
        // use lifetimes to avoid the error
        // strange lifetimes to achieve invariance over them
        _scope: PhantomData<&'scope mut &'scope ()>,
        _env: PhantomData<&'env mut &'env ()>,
    }

    impl Foo<'_, '_> {
        pub fn get_secret(&self) -> u32 {
            // There should be much more complex code
            self.secret
        }

        fn cleanup(&self) {
            println!("Foo::cleanup");
        }
    }

    pub fn scope<'env, F, T>(f: F) -> T
    where
        F: for<'scope> FnOnce(&'scope Foo<'scope, 'env>) -> T,
    {
        let foo = Foo {
            secret: 42,
            _scope: PhantomData,
            _env: PhantomData,
        };

        // AssertUnwindSafe is fine because we rethrow the panic
        let res = catch_unwind(AssertUnwindSafe(|| f(&foo)));

        foo.cleanup();

        match res {
            Ok(v) => v,
            Err(payload) => resume_unwind(payload),
        }
    }
}
```

Output:

```
Foo::cleanup
a = 420
```

</dd>

<dt id="term-guard_object"> <a href="#intro-guard_object" title="Jump up">^</a> Guard object </dt>

<dd>

A pattern of library APIs like `std::sync::MutexGuard`. Usually these
borrow some local state (like `std::sync::Mutex`) and restore it within
its drop implementation. Since Rust value semantics allow objects to
be forgotten, cleanup code within the drop implementation should not be
essential to preserve safety of your API.

However this proposal aims to relax this restriction, given a new
backwards-compatible set of rules.

</dd>

<dt id="term-callback_registration"> <a href="#intro-callback_registration" title="Jump up">^</a> Callback registration </dt>

<dd>

A pattern of library APIs, especially in
C. It is usually represented as setting some
function as a callback to incoming response for some client handle.
[tigerbeetle_unofficial_core::Client](https://docs.rs/tigerbeetle-unofficial-core/0.3.0+0.13.133/tigerbeetle_unofficial_core/struct.Client.html)
would be an example of that.

</dd>

<dt id="term-undefined_behavior"> <a href="#intro-undefined_behavior" title="Jump up">^</a> Undefined behavior or UB </dt>

<dd>

[Wikipedia explains it better than
me.](https://en.wikipedia.org/wiki/Undefined_behavior)

</dd>

</dl>

## References

1. <a href="#cite_ref-1" id="cite_note-1" title="Jump up">^</a> [rust-lang/rust github issue #24292 - std::thread::JoinGuard (and scoped) are unsound because of reference cycles](https://github.com/rust-lang/rust/issues/24292)

2. <a href="#cite_ref-2" id="cite_note-2" title="Jump up">^</a> [Yoshua Wuyts - Linear Types One-Pager # Updates](https://blog.yoshuawuyts.com/linear-types-one-pager/#updates)

3. <a href="#cite_ref-3" id="cite_note-3" title="Jump up">^</a> [unstable book - auto-traits](https://doc.rust-lang.org/beta/unstable-book/language-features/auto-traits.html)

## Credits

Thanks to @petrochenkov for reviewing and discussing this proposal with me.
