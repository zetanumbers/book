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
              .replaceAll(fg_re, "var(--fg)")
              .replaceAll("cmmi10", "MathJax_Math");

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
  - [Order of operations](#order-of-operations)
    - [Variable's semantics](#variables-semantics)
  - [Interval and set orders](#interval-and-set-orders)
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

## Order of operations

You may have heard, that Rust type system was inspired by the [Cyclone language].
It was a research project, which have implemented a clever region-based memory managment, which relates to the common understanding of how lifetimes work.
To recap: Rust compiler associates each lifetime token to a certain scope, where an object can live until the end of the scope.

Things changes a bit after introduction of [Non-lexical lifetimes].
Scopes were redefined in terms of the control-flow instead of the literal source code in order to lift unnecessary restrictions from safe code.
But what are the "necessary" restrictions in the first place?

Let's try visualize lifetime of some object \\(a\\) or it's operational semantics with a *diagram*:

<div class="tikz-embed">
<script type="text/tikz">
  \begin{tikzcd}
    a \arrow[r] & a^{-1}
  \end{tikzcd}
</script>
</div>

*Operations* \\(a\\) and \\(a^{-1}\\) respectivelly stands for the *introduction* of variable \\(a\\) and its *elimination*, given whatever meaning you would embed those with.
The arrow represents timeflow: elimination of a variable may occure **only after** its introduction.
This relation can be also described as a strict comparison \\(a < a^{-1}\\), forming a *[strict partial order]* over those operations.
The comparison's strictness allows to verify operation order requirements as long as \\(\alpha < \alpha\\) relation cannot be derived, which usually mean there's a cycle of arrows.
As such you may remember property of comparisons called *transitivity*: \\(\alpha < \gamma\\) if \\(\alpha < \beta\\) and \\(\beta < \gamma\\).
Later the notion of equality \\(\alpha = \beta\\) will also play a role of operations occuring simultaneously.

There could be as many hidden arrows as this rule allows, and to emphisize some of these, they are shown as dashed ones.
Operations are allowed to not immediatelly define relations between some of them:

<div class="tikz-embed">
<script type="text/tikz">
  \begin{tikzcd}
  \alpha \arrow[r] \arrow[d] & \beta \arrow[d] \\
  \gamma \arrow[r]           & \delta
  \end{tikzcd}
</script>
</div>

But eventually those operations will be sorted into a sequence with respect to those relations, i.e. will be *[linearly extended]*.

### Variable's semantics

To copy a variable \\(b = a\\), an additional requirement \\(a < b < a^{-1}\\) should be put on the order of those operations,
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
As such let's dive deeper and express the notion of a \\(b\\) constructor consuming \\(a\\) via \\(b = a^{-1}\\):

<div class="tikz-embed">
<script type="text/tikz">
  \begin{tikzcd}
    a \arrow[d, dashed] \arrow[r] & a^{-1} \arrow[d, dashed] \\
    b \arrow[r] \arrow[ru, equal] & b^{-1}
  \end{tikzcd}
</script>
</div>

With this you could state your first contradictory example.
You should not be able to construct \\(c\\) from both \\(a\\) and \\(b\\), if construction of \\(b\\) requires consumption of \\(a\\), i.e. \\(b < c < a^{-1} = b\\):

<div class="tikz-embed">
<script type="text/tikz">
  \begin{tikzcd}
    a \arrow[r] \arrow[rd] & a^{-1} \arrow[r, equal, color=red] & b \arrow[r] \arrow[ld, color=red] & b^{-1} \\
                           & c \arrow[u, color=red] \arrow[rru] \arrow[r]     & c^{-1}                 &
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
This relation is enforsed specifically with the Rust's borrow checker, for which we will later derive a notion of lifetimes.

## Interval and set orders

So far operations were considered to be either equal or non-intersecting, just like points.
But there is a great utility in conceptualizing operations as *intervals*.
That is, one interval could be considered a *subinterval* of another, thus defining a new relation \\(\beta \subseteq \alpha\\) with \\(\beta\\) as a subinterval and \\(\alpha\\) as a *superinterval*.
If interval \\(\alpha\\) is before or after \\(\gamma\\), then the subinterval \\(\beta\\) is respectfully before or after \\(\gamma\\) too:

\\[
\frac{\alpha < \gamma \quad \delta \subseteq \gamma}{\alpha < \delta}
\quad
\frac{\gamma < \beta \quad \delta \subseteq \gamma}{\delta < \beta}
\\]

Visualized with a diagram, double arrows would annotate subinterval relation \\(\subseteq\\) from subinterval to superinterval:

<div class="tikz-embed">
<script type="text/tikz">
  \begin{tikzcd}
    \alpha \arrow[r] \arrow[rd, dashed] & \gamma \arrow[r]                                & \beta \\
                                        & \delta \arrow[u, Rightarrow] \arrow[ru, dashed] &
  \end{tikzcd}
</script>
</div>

Although you shouldn't make a distiction between point-like operations and interval-like operations.
The essential part of this is just its subinterval \\(\subseteq\\) relation, as it allows to implement more modular and sophisticated constructions in our model.
Any interval could be embedded with smaller operations as subintervals, while superinterval embodies an abstraction over those.

However for it to be a proper interval requires from some perspective to be continuous, without holes:

\\[
\frac{\alpha < \delta < \beta \quad \alpha \subseteq \gamma \quad \beta \subseteq \gamma}{\delta \subseteq \gamma}
\\]

Diagram:

<div class="tikz-embed">
<script type="text/tikz">
  \begin{tikzcd}
                                            & \gamma                                 &                              \\
    \alpha \arrow[ru, Rightarrow] \arrow[r] & \delta \arrow[u, Rightarrow, dashed] \arrow[r] & \beta \arrow[lu, Rightarrow]
  \end{tikzcd}
</script>
</div>

Without this it would be a simple old *set*, which is weaker than a interval.

### Borrows

The notion of *unique borrow*, *shared borrow* and *owned value* gives enough expressiveness to the language to build rich safe interfaces.
These can be expressed using superintervals such as \\(\\&^\kappa_\mathbf{shr} a\\), where \\(\mathbf{shr}\\) means *shared* and \\(\kappa\\) is its lifetime to identify borrows:

<div class="tikz-embed">
<script type="text/tikz">
\begin{tikzcd}
a \arrow[r] \arrow[d, dashed]       & \&^\kappa_\mathbf{shr} a \arrow[r] & a^{-1}                                          \\
b \arrow[ru, Rightarrow] \arrow[rr] &                                    & b^{-1} \arrow[lu, Rightarrow] \arrow[u, dashed]
\end{tikzcd}
</script>
</div>

But consider this diagram:

<div class="tikz-embed">
<script type="text/tikz">
\begin{tikzcd}
                        & \&^c_\mathbf{mut}a \arrow[rrdd] \arrow[rr]         &  & \&^{c^{-1}}_\mathbf{mut}a \arrow[rd] &        \\
a \arrow[ru] \arrow[rd] &                                                    &  &                                      & a^{-1} \\
                        & \&^b_\mathbf{shr}a \arrow[rruu, dotted] \arrow[rr] &  & \&^{b^{-1}}_\mathbf{shr}a \arrow[ru] &
\end{tikzcd}
</script>
</div>

The relation \\(\\&^b\_\textbf{shr}a < \\&^{c^{-1}}\_\textbf{mut}a\\) (denoted by a dotted arrow) would contradict *temporal uniqness* of unique borrows.
To enforce that borrows are actually unique, you need to ensure shared and unique borrow *intervals* do not intersect.


You could do that by adding rules, which allows to derive contradiction \\(\alpha < \alpha\\) if overlap between unique and a shared borrow is present:

\\[\\&^{c^{-1}}\_\mathbf{mut} a < \\&^d_\mathbf{mut} a \quad\text{if}\quad \\&^c\_\mathbf{mut} a < \\&^{d^{-1}}_\mathbf{mut} a;\\]

\\[\\&^{b^{-1}}\_\mathbf{shr} a < \\&^c_\mathbf{mut} a \quad\text{if}\quad \\&^b\_\mathbf{shr} a < \\&^{c^{-1}}_\mathbf{mut} a;\\]

\\[\\&^{c^{-1}}\_\mathbf{mut} a < \\&^b\_\mathbf{shr} a \quad\text{if}\quad \\&^c_\mathbf{mut} a < \\&^{b^{-1}}\_\mathbf{shr} a,\quad\text{i.e. vice versa.}\\]

I would call those implication relations to be *second order relations* (denoted by a double arrow):

<div class="tikz-embed">
<script type="text/tikz">
\begin{tikzcd}[row sep=small, column sep=small]
                          & \&^c_\mathbf{mut}a \arrow[rrrrdddd] \arrow[rrrr] &                                      &  &    & \&^{c^{-1}}_\mathbf{mut}a \arrow[lllldddd, dashed] \arrow[rdd] &        \\
                          &                                                  & {} \arrow[rr, Rightarrow, bend left] &  & {} &                                                                &        \\
a \arrow[ruu] \arrow[rdd] &                                                  &                                      &  &    &                                                                & a^{-1} \\
                          &                                                  &                                      &  &    &                                                                &        \\
                          & \&^b_\mathbf{shr}a \arrow[rrrr]                  &                                      &  &    & \&^{b^{-1}}_\mathbf{shr}a \arrow[ruu]                          &
\end{tikzcd}
</script>
</div>

<div class="tikz-embed">
<script type="text/tikz">
\begin{tikzcd}[column sep=small]
a \arrow[r] & \&_\mathbf{shr}^b a \arrow[r] \arrow[rrdd] & \&_\mathbf{shr}^{b^{-1}} a \arrow[r, color=red] & \&_\mathbf{mut}^c a \arrow[r, color=red]                        & \&_\mathbf{mut}^{c^{-1}} a \arrow[r, color=red] & \&_\mathbf{shr}^d a \arrow[r] \arrow[lldd, color=red] & \&_\mathbf{shr}^{d^{-1}} a \arrow[r] & a^{-1} \\
            &                             &                 &                                       &                 &                             &                 &    \\
            &                             &                 & e \arrow[luu, color=red] \arrow[r] \arrow[rrruu] & e^{-1}          &                             &                 &
\end{tikzcd}
</script>
</div>

[Cyclone language]: https://cyclone.thelanguage.org/
[Non-lexical lifetimes]: https://smallcultfollowing.com/babysteps/blog/2016/04/27/non-lexical-lifetimes-introduction/
[strict partial order]: https://en.wikipedia.org/wiki/Partially_ordered_set
[linearly extended]: https://en.wikipedia.org/wiki/Linear_extension
