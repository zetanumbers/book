<style>
  div.mdbook-graphviz-output {
    text-align: center;
  }
</style>

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

Let's try visualize lifetime of some object \\(a\\) using, what I would call, *events*:

```dot process
digraph G {
  rankdir="LR";
  node [shape="none",fontname="MathJax_Math"];
  edge [arrowhead=empty]
  ia [label="I(a)"];
  ea [label="E(a)"];  
  ia -> ea;
}
```

The events \\(I(a)\\) and \\(E(a)\\) respectivelly stands for the *introduction* of variable \\(a\\) and its *elimination*.
The arrow represents timeflow: elimination of a variable may occure **strictly after** its introduction.
This relation can be also described as a strict comparison \\(I(a) \le E(a)\\), forming a *[partial order]* over these events.
Keep in mind its *transitivity*: \\(X < Z\\) if \\(X < Y\\) and \\(Y < Z\\).

To copy a variable \\(b = a\\), an additional requirement \\(I(a) < I(b) \le E(a)\\) should be put on the order of those events:

```dot process
digraph G {
  rankdir="LR";
  node [shape="none",fontname="MathJax_Math"];
  edge [arrowhead=empty]
  ia [label="I(a)"];
  ea [label="E(a)"];
  _1 [style=invis];
  ib [label="I(b)"];
  eb [label="E(b)"];
  ia -> ea;
  ia -> ib [constraint=false];
  _1 -> ib [style=invis];
  ib -> eb;
  ib -> ea [constraint=false,arrowhead=normal];
}
```

To copy a variable \\(b = a\\), an additional requirement \\(I(a) < I(b) \le E(a)\\) should be put on the order of those events:

```dot process
digraph G {
  rankdir="LR";
  node [shape="none",fontname="MathJax_Math"];
  edge [arrowhead=empty]
  ia [label="I(a)"];
  ea [label="E(a)"];
  _1 [style=invis];
  ib [label="I(b)"];
  eb [label="E(b)"];
  ia -> ea;
  ia -> ib [constraint=false];
  _1 -> ib [style=invis];
  ib -> eb;
  ib -> ea [constraint=false,arrowhead=normal];
}
```

To immutably borrow a variable \\(b = \\&a\\), the order \\(I(a) < I(b) < E(b) \le E(a)\\) must be a requirement:

```dot process
digraph G {
  rankdir="LR";
  node [shape="none",fontname="MathJax_Math"];
  edge [arrowhead=empty]
  ia [label="I(a)"];
  ea [label="E(a)"];
  ib [label="I(b)"];
  eb [label="E(b)"];
  ia -> ea;
  ia -> ib [constraint=false];
  ib -> eb;
  ib -> ea [style=dashed,constraint=false];
  eb -> ea [arrowhead="",constraint=false];
}
```

Notice that requirement \\(I(b) < E(a)\\) that we had for copies is recovered via transitivity.

```dot process
digraph G {
  rankdir="LR";
  node [shape="none",fontname="MathJax_Math"];
  ia [label="I(a)"];
  ea [label="E(a)"];
  csab [label="CopyStart(a, b)"];
  ceab [label="CopyEnd(a, b)"];
  _1 [style=invis];
  ib [label="I(b)"];
  eb [label="E(b)"];
  csab -> ceab;
  ia -> csab;
  ceab -> ea;
  csab -> ib -> ceab;
  _1 -> ib [style=invis];
  ib -> eb;
}
```

```dot process
digraph G {
  rankdir="LR";
  node [shape="none",fontname="MathJax_Math"];
  ia [label="I(a)"];
  ca [label="Copy(a, b)"];
  ea [label="E(a)"];
  _1 [style=invis];
  ib [label="I(b)"];
  eb [label="E(b)"];
  ia -> ca -> ea;
  ca -> ib [color="#bcbdd0:invis:#bcbdd0",dir=none,constraint=false];
  _1 -> ib [style=invis];
  ib -> eb;
  ib -> ea [style="dashed"];
}
```

[Cyclone language]: https://cyclone.thelanguage.org/
[Non-lexical lifetimes]: https://smallcultfollowing.com/babysteps/blog/2016/04/27/non-lexical-lifetimes-introduction/
[partial order]: https://en.wikipedia.org/wiki/Partially_ordered_set
