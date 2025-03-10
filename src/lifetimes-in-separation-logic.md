# Lifetimes in separation logic

### Disclaimer

I am an amateur when it comes to real computer science.
My introduction to this field by professionals was rather lacking compared to the volume of today applied CS.
I have to do majority of my (flawed) research and writing without any supervision.
As such if you are knowledgable on the subject I would like to ask you for some (constructive) feedback from you, please.

## Introduction

Since Rust's 1.0 release lifetimes proved themself as a natural extension of a substructural type system to many people using Rust.
However underlying framework to formalize Rust safety semantics, i.e. Iris seem to get by without any analogous structure.
In this text I will propose there is a hidden structure within the separation logic used by Iris, which corresponds relativly to lifetime semantics.
I will also propose some (backwards incompatible) language features related to my interpretation of lifetimes to the Rust language.

## Categorification

Let's form a category \\(\mathbf{Sep}\\) using a hom-set of whole Hoare triples,
while precondition \\(P\\) and postcondition \\(Q\\) represent objects of those category.

\\[
\\{P\\}\\ C\\ \\{Q\\}
\\]

That satisfy monoid laws:

\\[1_P = \\{P\\}\\ \texttt{skip}\\ \\{P\\}\\]
\\[\frac{f = \\{P\\}\\ F\\ \\{Q\\} \quad g = \\{Q\\}\\ G\\ \\{R\\} }
{g \circ f = \\{P\\}\\ F;G\\ \\{R\\}}\\]
\\[(\text{Post-1})\quad\\{P\\}\\ C\\ \\{Q\\} = f = 1_Q \circ f = \\{P\\}\\ C;\texttt{skip}\\ \\{Q\\}\\]
\\[(\text{Pre-1})\quad\\{P\\}\\ C\\ \\{Q\\} = f = f \circ 1_P = \\{P\\}\\ \texttt{skip};C\\ \\{Q\\}\\]
\\[(\text{Assoc})\quad\frac{f = \\{P\\}\\ F\\ \\{Q\\} \quad g = \\{Q\\}\\ G\\ \\{R\\} \quad h = \\{R\\}\\ H\\ \\{S\\} }
{f \circ (g \circ h) = \\{P\\}\\ F;G;H\\ \\{S\\} = (f \circ g) \circ h}\\]

To put a restriction on precondition in a triple you can simply construct a new triple.
Lifting of a corestriction from postcondition is done similarly.

\\[
\frac{P \vdash Q \quad \\{Q\\}\\ C\\ \\{R\\} \quad R \vdash S}
{\\{P\\}\\ C\\ \\{S\\}}
\\]

Composition with restricted unit, for example \\(r_S(1_Q) \circ f\\), must be equal to restricted morphism \\(r_S(f)\\).
It could be thought as equality of programs \\(C;\texttt{skip} = C = \texttt{skip};C\\), which I think is beyond original intent for \\(\texttt{skip}\\), so I hope it does not ruin anything. :)

## Lifetimes

The purpose of a lifetime is restrict safe subset of the language to operate only on valid objects and data.
Object is ought to become invalid after some operation on a related object.
A valid or invalid operation occurs on a valid or invalid object respectively.
With that there emerges an order of certain operations.
As an example consider that elimination of a reference happens before elimination of the referenced object.

Let's *roughtly* express the example above using separation logic.
Conditions \\(U_x\\), \\(A_x\\), and \\(D_x\\) denote uninitialized, alive, and dead states of variable \\(x\\).
Operations \\(I_x\\) and \\(E_x\\) denote general introduction and elimination of \\(x\\).
Triples for a single variable (closed-world) lifecycle would predictably be:

\\[\\{U_x\\}\\ I_x\\ \\{A_x\\}
\quad \text{and} \quad
\\{A_x\\}\\ E_x\\ \\{D_x\\}\\]

But consider another variable \\(y\\), which borrows from \\(x\\).
That would put requirements on introduction of \\(y\\) and elimination of \\(x\\),
as \\(I_x\\) has to occur before \\(I_y\\) and \\(E_y\\) before \\(E_x\\):

\\[\\{U_x\\}\\ I_x\\ \\{A_x\\}\\]
\\[\\{A_x \land U_y\\}\\ I_y\\ \\{A_x \land A_y\\}\\]
\\[\\{A_y\\}\\ E_y\\ \\{D_y\\}\\]
\\[\\{A_x \land D_y\\}\\ E_x\\ \\{D_x \land D_y\\}\\]

<!-- TODO: composed-of relation of morphisms/operations -->
