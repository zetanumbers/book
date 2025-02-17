<link rel="stylesheet" type="text/css" href="http://tikzjax.com/v1/fonts.css">
<script src="https://tikzjax.com/v1/tikzjax.js"></script>

<script type="text/javascript">
  window.addEventListener("load", () => {
    const embeds = new Set(document.querySelectorAll("div.tikz-embed").values().map(el => { return { element: el, fixed: false }; }));
    const bg_re = /#fff/gi;
    const fg_re = /#000/gi;
    const scale = 1.5;
    let fix_interval;
    fix_interval = window.setInterval(() => {
      let fixed_count = 0;
      for (const embed of embeds) {
        if (embed.fixed) {
          fixed_count += 1;
          continue;
        }

        const flex_div = embed.element.children[0];
        const page_div = flex_div?.children[0];
        if (page_div != null) {
          if (page_div.classList[0] !== "page") {
            console.error("Page div not found for", page_div);
          } else {
            page_div.classList = [];
            const svg = page_div.children[0];
            const new_width = (scale * parseFloat(flex_div.style.width)) + "pt";
            const new_height = (scale * parseFloat(flex_div.style.height)) + "pt";

            svg.width.baseVal.valueAsString = new_width;
            svg.height.baseVal.valueAsString = new_height;
            flex_div.style.width = "100%";
            flex_div.style.height = new_height;
            flex_div.style.justifyContent = "center";
            // flex_div.style.alignItems = "center";
            page_div.style.width = new_width;

            page_div.innerHTML = page_div.innerHTML
              .replaceAll(bg_re, "var(--bg)")
              .replaceAll(fg_re, "var(--fg)");

            embed.fixed = true;
            fixed_count += 1;
          }
        }
      }
      if (fixed_count == embeds.size) {
        window.clearInterval(fix_interval);
      }
    }, 1000);
  });
</script>

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
    - [Order of events](#order-of-events)
    - [Variable's semantics](#variables-semantics)
    - [Borrows](#borrows)
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

### Order of events

Let's try visualize lifetime of some object \\(a\\) using, what I would call, *events*:

<div class="tikz-embed">
<script type="text/tikz">
  \begin{tikzcd}
    a \arrow[r] & a^{-1}
  \end{tikzcd}
</script>
</div>

The events \\(a\\) and \\(a^{-1}\\) respectivelly stands for the *introduction* of variable \\(a\\) and its *elimination*.
The arrow represents timeflow: elimination of a variable may occure **only after** its introduction.
This relation can be also described as a strict comparison \\(a < a^{-1}\\), forming a *[strict partial order]* over those events.
The comparison's strictness allows to verify event order requirements as long as \\(\alpha < \alpha\\) relation cannot be derived, which usually mean there's a cycle of arrows.
As such you may remember property of comparisons called *transitivity*: \\(\alpha < \gamma\\) if \\(\alpha < \beta\\) and \\(\beta < \gamma\\).
So there could be as many hidden arrows as this rule allows, and to emphisize some of these, they are shown as dashed ones.
Later the notion of equality \\(\alpha = \beta\\) will also play a role of events occuring at the same moment.

### Variable's semantics

To copy a variable \\(b = a\\), an additional requirement \\(a < b < a^{-1}\\) should be put on the order of those events,
meaning variable \\(a\\) must exists at the moment of creation of \\(b\\):

<div class="tikz-embed">
<script type="text/tikz">
  \begin{tikzcd}
    a \arrow[d] \arrow[r]  & a^{-1} \\
    b \arrow[r] \arrow[ru] & b^{-1}
  \end{tikzcd}
</script>
</div>

This is usually all the variable semantics of any simple programming language, however you know Rust is special.
As such let's dive deeper and  express the notion of a \\(b\\) constructor consuming \\(a\\) via \\(b = a^{-1}\\):

<div class="tikz-embed">
<script type="text/tikz">
  \begin{tikzcd}
    a \arrow[d, dashed] \arrow[r] & a^{-1} \arrow[d, dashed] \\
    b \arrow[r] \arrow[ru, equal] & b^{-1}
  \end{tikzcd}
</script>
</div>

Then to immutably borrow a variable \\(b = \\&a\\), the order \\(a < b < b^{-1} < a^{-1}\\) have to be a requirement:

<div class="tikz-embed">
<script type="text/tikz">
  \begin{tikzcd}
    a \arrow[d] \arrow[r]          & a^{-1} \\
    b \arrow[r] \arrow[ru, dashed] & b^{-1} \arrow[u]
  \end{tikzcd}
</script>
</div>

Notice that the previous requirement \\(b < a^{-1}\\) we had for copies is recovered via transitivity.
So if you compare these two diagrams, you will notice \\(b^{-1} < a^{-1}\\) standing out.
This relation is enforsed specifically with the Rust's borrow checker.
However those are not enought to model every possible interaction with values or objects in Rust.

### Borrows

The notion of *unique borrow*, *shared borrow* and *owned value* though gives enough expressiveness to the language.
But combining those is more complicated than giving sensible rules for order of variable introductions and eliminations.

Let's call *introduction* and *elimination* events of a shared borrow \\(b = \\&^{b}\_\mathbf{shr} a \\) and \\(b^{-1} = \\&^{b^{-1}}\_\mathbf{shr} a\\) respectivelly.
For the unique borrow let's pick names \\(c = \\&^c_\mathbf{mut} a\\) and \\(c^{-1} = \\&^{c^{-1}}_\mathbf{mut} a\\).
And obviously \\(\\&^{b}\_\mathbf{shr} a < \\&^{b^{-1}}\_\mathbf{shr} a\\) has to hold true.
Now we could rewrite borrow diargam from above a bit more detailed:

<div class="tikz-embed">
<script type="text/tikz">
\begin{tikzcd}
a \arrow[r] & \&^b_\mathbf{shr} a \arrow[r]  & \&^{b^{-1}}_\mathbf{shr} a \arrow[r] & a^{-1} \\
            & b \arrow[r] \arrow[u, equal] & b^{-1} \arrow[u, equal]        &
\end{tikzcd}
</script>
</div>

But consider this diagram:

<div class="tikz-embed">
<script type="text/tikz">
\begin{tikzcd}
                        & \&^{c}_\mathbf{mut}a \arrow[rr] \arrow[dd]           &  & \&^{c^{-1}}_\mathbf{mut}a \arrow[rd] \arrow[dd] &        \\
a \arrow[rd] \arrow[ru] &                                                      &  &                                                 & a^{-1} \\
                        & \&^{b}_\mathbf{shr}a \arrow[rr] \arrow[rruu, dotted] &  & \&^{b^{-1}}_\mathbf{shr}a \arrow[ru]            &
\end{tikzcd}
</script>
</div>

The relation \\(\\&^{b}\_\textbf{shr}a < \\&^{c^{-1}}\_\textbf{mut}a\\) (denoted by a dotted arrow) would contradict *uniqness* of unique borrows.
To enforce that borrows are actually unique, you need to ensure shared and unique borrow "intervals" do not intersect.
You could do that by adding rules, which allows to derive contradiction \\(\alpha < \alpha\\) if overlap between unique and a shared borrow is present:

\\[\\&^{b^{-1}}\_\mathbf{shr} a < \\&^c_\mathbf{mut} a \quad\text{if either}\quad \\&^{b}\_\mathbf{shr} a < \\&^c_\mathbf{mut} a \quad\text{or}\quad \\&^{b^{-1}}\_\mathbf{shr} a < \\&^{c^{-1}}\_\mathbf{mut} a;\\]

\\[\\&^{c^{-1}}\_\mathbf{mut} a < \\&^{b}\_\mathbf{shr} a \quad\text{if either}\quad \\&^c_\mathbf{mut} a < \\&^{b}\_\mathbf{shr} a \quad\text{or}\quad \\&^{c^{-1}}\_\mathbf{mut} a < \\&^{b^{-1}}\_\mathbf{shr} a,\quad\text{i.e. vice versa.}\\]

I would call those implication relations to be *second order relations* (denoted by a double arrow):

<div class="tikz-embed">
<script type="text/tikz">
\begin{tikzcd}
                        & \&^{c}_\mathbf{mut}a \arrow[rr] \arrow[dd] &    & \&^{c^{-1}}_\mathbf{mut}a \arrow[rd] \arrow[dd] \arrow[lldd, dashed] &        \\
a \arrow[rd] \arrow[ru] & {} \arrow[r, Rightarrow]                   & {} & {} \arrow[l, Rightarrow]                                             & a^{-1} \\
                        & \&^{b}_\mathbf{shr}a \arrow[rr]            &    & \&^{b^{-1}}_\mathbf{shr}a \arrow[ru]                                 &
\end{tikzcd}
</script>
</div>

[Cyclone language]: https://cyclone.thelanguage.org/
[Non-lexical lifetimes]: https://smallcultfollowing.com/babysteps/blog/2016/04/27/non-lexical-lifetimes-introduction/
[strict partial order]: https://en.wikipedia.org/wiki/Partially_ordered_set
[ZST]: https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts
