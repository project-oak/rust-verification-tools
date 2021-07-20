---
layout: post
title: Using KLEE on Rust-for-Linux (part 1)
---

![Linux logo](https://cdn.kernel.org/pub/linux/kernel/v2.0/Logo.gif){: style="float: left; width: 10%; padding: 1%"}
The [Rust for Linux] project is working on adding support for the Rust language
to the Linux kernel
with the hope that using Rust will make new code more safe, easier
to refactor and review, and easier to write.
(See the [RFC] for more detail about goals and for the varied
responses of the Linux Kernel community.)

Back in April, I took a look at whether we could use our [Rust verification
tools][RVT git repo] on the [Rust for Linux] repo to provide further
safety.
Most of our work is based on the [KLEE] symbolic execution tool
and I was able to get that to work.
For *reasons*, I did not get to explore this very deeply after that but
I thought it would be useful to describe what I was able to do
and some of the questions raised by the work as a guide to how you might tackle
the problem in the future.

I have split this blog into three parts because it was getting quite long.
In [this part][Using KLEE with Rust for Linux (part 1)],
I'll start by looking at some key questions around what
properties and code we want to check.
The [second part][Using KLEE with Rust for Linux (part 2)],
will dive deeply into how to build Rust-for-Linux in a way
that you can use [KLEE] on it.
(Many people will want to skip this part.)
And the [final part][Using KLEE with Rust for Linux (part 3)],
will return to the questions by creating test harnesses and
stubs that could be used to check the Rust-for-Linux code for bugs.

As with the previous post on [using KLEE with CoreUtils], my goal in this post
is to help others to use tools from the formal verification community to check
code like this rather than to do that checking myself.  In particular, I will
not find any bugs, I will not attempt to provide evidence that this is worth
doing and I will not create a verification system that is ready to integrate
into any project.
These (and other limitations listed at the end of the [last post][Using KLEE with Rust for Linux (part 3)])
all need to be fixed before I would recommend that you try to use
these tools as part of your regular workflow.
But, I hope that this series will give you an idea of what
needs to be done and the problems that need to be fixed
in the future.

One other important thing to note is that the Rust compiler moves very fast
and this causes problems for verification tools.
In particular, what I describe is based on KLEE that recently added support
for LLVM-11 but the latest Rust compiler relies on LLVM-12.
All the detailed instructions are therefore for [an older version of Rust for
Linux](https://github.com/Rust-for-Linux/linux/commit/e71e2ab9686e168aebb086d8bd2643b79f20106e)
from late May instead of the current version.[^partial-fix]

[^partial-fix]:
    If you want to update this to work with the current version of 
    Rust for Linux, I have
    [another branch](https://github.com/alastairreid/linux/tree/r4l_new_llvm12)
    that compiles but, since I was not able to use KLEE, it is untested
    and it has not been cleaned up to make it suitable for merging into the
    main branch.
    But it may be useful as a guide to what needs to be changed.


## What properties would we like to verify?

I think that there are four types of property that we might want to
check:
safety checks;
invariants;
parameterized checks;
and
functional correctness.

(Spoiler: in my experiment I only scraped the surface of safety checks and
parameterized checking.)


### Safety checks

Rust's type system is able to statically check for many problems
that go unchecked in C or C++ code.
And some of Rust's language features support good software engineering
practice to mitigate issues around things like "unsafe" code where
the compiler cannot perform such strong checks: allowing tricky code
to be isolated into areas that can be carefully reviewed instead of
spreading them evenly through the codebase.
This drastically reduces the need to verify low-level properties
such as null pointer checks, use-after-free, locking issues, etc.
While these issues are not completely eliminated, they are probably not the
highest priority for Rust code.

Part of Rust's safety is achieved through dynamic checks inserted by
the compiler in debug mode. For example, array bounds checks and integer
overflow checks.
These are problematic because failing a check triggers a kernel panic from which
there is no good recovery.
If you do not catch the panic, you have to reboot the kernel: losing unsaved data,
ejecting users from the system and affecting other systems that rely on this
one.
If you try to catch the panic, you risk leaving data structures in an
inconsistent state which will cause subtle failures and potential data
corruption later.
Neither is entirely satisfactory and there are strong opinions on which of
these two bad options is better or worse.

There is clearly a need to identify dynamic checks that can never fail
and dynamic checks that could potentially fail.
We can (and probably should) look for checks that can fail using
testing and fuzzing but formal verification tools have the unique ability
to persuade us that none of the checks can fail.
And, if none of the checks can fail, then we can turn off the dynamic
checks safe in the knowledge that it makes no difference.[^realistically]

[^realistically]:
    More realistically, what is likely to happen is that tools find
    some checks that
    could fail (and we fix them), some checks that can't fail (and we eliminate them).
    And then we are left with checks that are too hard for an automatic tool to
    reason about.
    For these, we are forced to either turn to other approaches such as
    interactive theorem proving,
    manual reasoning,
    very thorough testing and fuzzing,
    or simplifying the code.
    (Simplifying the code would be the best option if possible.)


### System invariants

Going further, the correct functioning of system software often relies on a
large number of invariants, it is important that all parts of the system
preserves those invariants, and code changes will inevitably result in
inconsistencies.
Again, the Rust language can help avoid this by making it easier to
isolate all the code that touches a data structure in one place using
more powerful abstraction features and it can
enforce this isolation using the module system.

But, sometimes the code that preserves an invariant is unavoidably smeared
over a very large amount of code.
And, sometimes, the invariant is extremely subtle or hard to ensure.
So we might also want to specify and then check system invariants.

For checking system invariants, it is certainly *nice* if a formal verification
tool can catch all violations or persuade us that the code is correct but, if
possible, we should also aim to use testing and fuzzing.
One way to do this is to write the invariants as executable Rust code:
we can then insert debug-mode checks (i.e., `assert!`) at key places in the
code and check the invariants multiple ways.


### Functional correctness and parameterized checks

Ideally, of course, we would like to check that the code does what it
is supposed to (i.e., that it is "functionally correct").
To do this, we need a specification of what Linux is supposed to do – and
that's where the problem starts...

- There is no specification of Linux.

- Creating a specification of Linux as we wish it was would be a huge amount of
  work (that nobody is likely to do).

- Correcting the specification to match the aspects of Linux that people rely on
  (even if they are not "ideal") would be a huge amount of work with a very long
  tail as one dependency after another is discovered and some hypothetical
  Linux specification committee argues over whether that dependency is a
  bug or a feature.

- Fixing remaining divergences between the specification and the Linux codebase
  would be a huge amount of work.

- Linux would continue to evolve while a specification is being built.

- If you want the specification and the code to be in sync with each other,
  then the specification and testing/verifying activities act as a brake on development:
  you have to agree the changes to the specification before or as you are changing the
  code.
  It is hard to imagine that this would be acceptable.

*Note that there is nothing special about Linux in the above: it is true for
any major software system and, especially for any system that has a single
implementation.*[^special-cases]

[^special-cases]:
    Just because *all* of Linux cannot possibly be specified does not mean
    that individual parts of Linux cannot be specified.
    There is no reason why relatively isolated parts such as device drivers,
    network stacks or filesystems could not be formally specified and even verified.

One way round this situation is to weaken the goals: instead of aiming to
specify *exactly* what Linux should do for *any sequence* of system calls, I/O,
etc. we could, instead, specify *some* aspects of how *some sequences* of system
calls should behave.

As an example of how this might work, if we believe that a particular
system call cannot ever return some error code, we could write a parameterized
check that executes some sequence of operations, invokes the
system call under test and then asserts that the result is not the forbidden
error code.
A traditional test would try this for a few important sequences and function
arguments.
A parameterized check extends this by taking the sequences and function
arguments as parameters which allows you to use the same check for
both fuzzing or for formal verification.
(These [dual-use parameterized
checks][Using PropVerify] are a key part of our overall project
and we [wrote a paper][HATRA 2020] about the overall strategy.)
  

## What code do we want to check?

The hardest bugs to find and deal with often result from misunderstandings
between different teams and from one part of the system being
changed without making corresponding changes elsewhere.
One of the biggest opportunities for misunderstandings and divergence in
Rust for Linux is between the C code and the Rust code in Linux.
It would therefore be great to check the entire system at once
so that we flush out disagreements between the Rust code and the C code.

But, there is a lot of C code in Linux and we cannot possibly consider all
of it without the goal of "verify Rust for Linux" turning into
"verify all of Linux".
So, for now, we should focus on the Rust code and recognise that
we are leaving a major potential source of bugs for another time
when we see a way to make it manageable.

The Rust code in Rust for Linux interacts with "rLinux" (the rest of Linux)[^rLinux]
in two major ways:

[^rLinux]:
    For convenience, I made up the shortened name "rLinux" to refer
    to the "rest of Linux": those parts of Linux that are not written
    in Rust.
    The name is inspired by the Scottish Government's use of the
    term "rUK" to refer to all the parts of the United Kingdom
    that are not Scotland.

1. Calling into rLinux services to do things like allocate memory,
   take a lock, print a message or register a device driver.

2. rLinux invokes Rust code through module initialization/finalization
   functions, virtual function tables provided by device drivers, etc.
   This is accomplished through some wrapper code that converts 
   C data structures and idiomatic C APIs to idiomatic Rust data
   structures and APIs.

A large part of Rust for Linux consists of glue code that
makes these interactions safe and idiomatic.
We have a choice of whether to include this glue code in our
verification or to focus on verifying device drivers, etc.
that interact with this glue code.
There are three competing forces that influence our decision:

1) We would like to verify as much Rust code as possible.
   So we should try *not* to exclude any Rust code.

2) Wherever we draw the boundary between C and Rust, we will
   have to create stub functions of the code that we exclude.
   To minimize work, we should choose a boundary that is small,
   is well documented and is stable.

3) When writing parameterized checks, we would prefer to interact
   with an idiomatic Rust interface so that we can exploit
   the Rust type system, traits, etc. in our checks.

There is probably no right answer here but, for now,
it looks as though we should include almost all of the glue
code that invokes rLinux services but that we should
exclude the glue code that wraps Rust code in C API
and C data structures.


## Should we use Cargo?

The obvious way to build the code is to tweak the existing
Linux build system.
This is the most flexible if we want to start verifying more
of the Linux C code.
But it has one huge disadvantage: the Rust for Linux code
is not built using Cargo so we lose a lot of the Cargo
features that we are used to using.
In particular, our `cargo-verify` tool behaves like `cargo test` and
it invokes Cargo as part of its execution
so, if we don't use Cargo, we will need to achieve
the same results using Makefiles, etc.

I don't know what the best choice is here.
It would be convenient to use Cargo and cargo-verify but
it turns out that kernel code is a bit simpler to
compile than normal Rust crates so it is not unbearable
to avoid Cargo.
So, for now, I will just tweak the Makefiles in the
kernel.[^good-practice]

One major disadvantage of not using Cargo is that it makes it
harder to use the libraries that we have developed elsewhere
in the [RVT project].
In the third part of this series, we will resort to copying those files
into Rust for Linux as a crude workaround and we will only use the
simplest library.


[^good-practice]:
    It turns out that it is hard to integrate cargo into the
    build systems found at large companies where
    that use massively parallel compilation farms,
    compiled code is stored on a massive distributed shared
    database, there is a substantial investment in
    C++, Java/Kotlin, PHP or whatever,
    and all code coming into the company has to be reviewed
    to prevent supply-chain attacks.

    So, if you are like me and you are developing tools that should
    be usable both with conventional cargo-based development and
    some other build system, it is good practice to use something
    other than cargo every now and then to understand the issues
    you will face later.
    And, compared with what you find in large companies, the
    Linux makefiles are relatively simple to work with :-)


## Summary

The Rust language has great promise as a safer systems programming language
and the [Rust for Linux] project is working hard on demonstrating that promise
in the context of the Linux kernel.
This series of posts is looking at how we can further improve safety.

In [this first part of the series][Using KLEE with Rust for Linux (part 1)],
I looked at some key questions around what properties we might want to check
(and what does not seem realistic to consider).
And I looked at how much of the codebase we might want to check: just device
drivers, and other extensions written in Rust; the Rust infrastructure that supports these extensions;
and/or the interaction between Rust and the remainder of Linux (written in C).

In the remaining two parts, I will
[give detailed build instructions for using KLEE with the Rust for Linux codebase][Using KLEE with Rust for Linux (part 2)]
and I will
[show how we can build simple verification frameworks][Using KLEE with Rust for Linux (part 3)].
(However, this series of posts will stop short of showing a *ready to use*
method or even recommending that you start using it yet. I leave this for the
future.)


-----------


[WLLVM]:                          https://github.com/travitch/whole-program-llvm
[Rust For Linux]:                 https://github.com/Rust-for-Linux/linux
[RFC]:                            https://lore.kernel.org/lkml/20210414184604.23473-1-ojeda@kernel.org/
[RVT git repo]:                   {{site.gitrepo}}/
[RVT project]:                    {{site.baseurl}}/
[KLEE]:                           https://klee.github.io/
[Using PropVerify]:               {{site.baseurl}}{% post_url 2020-09-03-using-propverify %}
[HATRA 2020]:                     https://alastairreid.github.io/papers/HATRA_20/
[Vectorized code 2]:              {{site.baseurl}}{% post_url 2021-05-15-verifying-vectorized-code2 %}
[Using KLEE with CoreUtils]:      {{site.baseurl}}{% post_url 2021-07-14-coreutils %}
[Using KLEE with Rust for Linux (part 1)]: {{site.baseurl}}{% post_url 2021-08-22-rust-on-linux-1 %}
[Using KLEE with Rust for Linux (part 2)]: {{site.baseurl}}{% post_url 2021-08-23-rust-on-linux-2 %}
[Using KLEE with Rust for Linux (part 3)]: {{site.baseurl}}{% post_url 2021-08-24-rust-on-linux-3 %}
