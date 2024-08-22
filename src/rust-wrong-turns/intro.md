# Rust-lang's wrong turns?

In this volume I would like to tell about times when Rust got it wrong and pathways into the future.
We will be talking about such features as async code, `Send`, `Sync` and other auto(-ish) traits and linear types, the things I've learned while I was working on Rust features such as asynchronous drop.

I would like to clarify that I do not put the blame on anyone here, especially on rust-lang's authors and top contributors.
The discovery process was entirely different back when some of affected features were introduced or stabilized.
There was much less code utilizing those features and it wasn't clear what new features would be more useful in the future to assess their compatibility.

But this writing is not about the blame, what was right or wrong anyway, that's why there is a question mark in the title.
It's only about **causality**, what choice leads to what consequences in the language.

As such I am unable to tell what Rust would look like in the future.
I am unable to decide that, but I can roughly tell what is possible.

To access your feedback I'll add a link to relevant discussion's thread, so please use it if you want to share something.
Let's move forward.
