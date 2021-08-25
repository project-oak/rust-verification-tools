---
layout: post
title: Retrospective
---

There are lots of hard problems in industrial research
but one of the hardest is timing.
If you start a project too early, your users
are focused on other problems and don't yet feel a
need for your solution.
And, if you start a project too late, your 
users have already hit the problem and you
have to work round whatever solution they have
found.
And so it is with this project.[^but-also-other-things]
At the moment, our potential users are still working on all the things
you must have to support a new language in a large company
and there is not a strong pull for a new, unproven research
idea.
So, as we close the project down, I thought it would be good
to have a bit of a retrospective on what the project was
trying to do and how.

[^but-also-other-things]:
    I am, of course, oversimplifying when I say that timing
    is *the* issue.
    There's a number of other factors in closing
    the project  but they mostly come
    down to how much work there was to do and how many people
    there were to do it.
    Which is really just another way of saying "timing".

## A usability project ...

The project was primarily a usability project: looking for ways to
make it easier for developers to use the tools created by the
formal verification community.
We wrote [a paper][HATRA 2020] with usability researchers about the issues we anticipated
and the project plan called for a series of case studies and pilots
to figure out where the benefits lay and then to work with development
teams to measure that benefit.
The hope was that, if that benefit was large enough and achievable
by enough developers, it would justify
the necessary investment in improving the tools, supporting the tools,
creating training materials, etc.

## ... that started by developing tools

Before you can do a usability case study, you need to have something
for people to use.
The only problem was that, when we started, [the tools available][Survey 2020]
were quite limited: they could only handle small programs with limited
dependencies.
A particular problem was that many tools could not handle
features found in real Rust crates like
[command line arguments][Using argv]
and
[SIMD intrinsics][Fixing SIMD]
so we spent most of the first year extending KLEE,
writing tools, writing libraries, etc.
to create tools good enough to let us
do the case studies and run pilots.

After a year's work, the tools and our understanding
are getting close to being good enough to start the
case studies.
For example, we could apply the tools to
[CoreUtils][Using KLEE with CoreUtils]
and to
[Rust for Linux][Using KLEE with Rust for Linux (part 1)].
In both cases though, we hit rough edges in the tools
that made it too hard to actually try verifying the code.
A little more work is needed.

## Using "shift left" ...

We had a fairly unusual goal for a formal methods project

> Our goal was to make developers more productive by using
> tools and techniques from the formal verification community.

This is not the usual kind of goal for a formal methods project.
It is much more common to pitch formal as a way to dramatically
and measurably improve quality at the cost of reduced developer velocity.
That is, it will take you longer but the result will be better.

The way that we hoped to improve developer velocity is through
a "shift left" process.
The name "shift left" is a reference to the graphs used by project
managers to think about when the product is ready to release.
If you plot the number of bugs found against time, you know that your product
cannot be released if you are still finding a significant number
of bugs every week.
If you want to release earlier, the most effective thing to
do is to find the bugs earlier.
That is, to shift the bug curve left.

![Illustration of shift-left concept with time increasing towards the right and two similar bug curves beside each other and an arrow indicating movement from the right curve to the left curve.]({{site.baseurl}}/images/retrospective-shift-left.png)

Finding the bugs earlier has two benefits.
The most obvious is that the bug curve will flatten out sooner.
The more important benefit is that we all know that the *cost*
of a bug increases with time: the longer the code sits in your system,
the more impact it will have on the rest of the system and the harder
it will be to fix the bug.

Rust already helps with this: its combination of static checks, dynamic
checks, better API design and less footguns in the language design means that
many bugs are caught as the code is typed into your editor/IDE
or when you first build the code or when you first run it.
Rust's emphasis on testing in [the book](https://nostarch.com/Rust2018),
in tools and in the open source community further helps find bugs earlier.

Our belief is that formal verification tools and techniques have the potential
to find bugs even earlier: pushing the bug curve even further to the left.
And, when you can prove the absence of (certain classes of) bugs, you also
reduce the uncertainty about whether the bug curve has flattened out yet.



## ... to drive adoption

We set ourselves a very ambitious target

> Our target was to get 50% of Rust developers at Google to
> use our tools to make themselves more productive.

Why 50%?

One reason is that it is useful to set ambitious targets: it pushes you harder,
you architect your system so that it can scale, and it forces you to focus on
the summit instead of getting lost in the foothills.
But, to be honest, even 10% would be an ambitious target that we could
be happy to achieve.

The problem with a target of 10% is that it is important to think about
*who* that 10% are.
Is it just the early adopters?
Is it just people who have been taught formal methods at university?
If we assume some distribution on developer ability (whatever you think
ability means), are we just targeting the top 10% of developers?

![Rough sketch of a normal distribution curve highlighting the top 10% of developers on the right as our target users]({{site.baseurl}}/images/retrospective-vertical-10.png)

The advantage of a target like 50% is that there is no way to cheat: we have
to target the median developer.
In the worst case, the distribution of users is like this.

![Rough sketch of a normal distribution curve with a vertical line halfway across the curve and the area to the right of the line highlighted as our target users]({{site.baseurl}}/images/retrospective-vertical-50.png)

In the best case, the distribution is like this.

![Rough sketch of a normal distribution curve with a horizontal line halfway up the curve and the area below the line highlighted as our target users]({{site.baseurl}}/images/retrospective-horizontal-50.png)

And, what we expect is something somewhere between the two.

Achieving a distribution like this is critical to long-term
adoption of formal methods into the practice of industrial
software engineering.
I don't think it can be achieved
by targeting the *most difficult* or the most critical of projects or
by targeting the *most receptive, best trained or smartest developers*.
I think it can only be achieved by targeting *all developers*.


## Testing vs bug-hunting vs proofs: why not both?

Our goal was to [create a continuum of verification options
that all implement the same interface](https://alastairreid.github.io/why-not-both/): allowing you to choose
the right one for what you are doing.
If you just wrote some code, you might start by fuzzing it for some quick
feedback
or using a symbolic execution tools like [KLEE] to hunt for
bugs.
But if you are about to submit the code for code review,
you might want to prove that the code is correct.

We have used [KLEE] as our primary tool during this project
because the research team that developed KLEE are very supportive
to people using and extending their tool and because KLEE is a
mature, robust tool that can handle larger programs.

But KLEE is primarily a tool for bug-hunting so we also
spent time working with [Crux-MIR][Install Crux],
[SeaHorn],
[MIRAI] (see [this branch](https://github.com/project-oak/rust-verification-tools/tree/mirai)),
and
[Rust Model Checker (RMC)][RMC] (see [this branch](https://github.com/project-oak/rust-verification-tools/tree/RMC_support)).
These tools put more emphasis on proving properties of the code
than on finding bugs
so they are an essential part of the goal of building a continuum of tools
that support multiple levels of assurance, scalability, ease of use, etc.

## Contributing to Rust verification

Everyone benefits if Rust has a great formal verification story.
If there are great tools, developers can build better software.
If popular crates are verified, we can be more confident in
any software that depends on those crates.
If verification helps drive Rust adoption in [the systems that society
depends on on a daily basis][Using KLEE with Rust for Linux (part 1)],
we make the world a better place.
etc.

So our goal has always been to contribute to the Rust verification
community and to try to complement the efforts of others.
All the code we wrote is dual licensed to make it easy to incorporate
into other tools.
The [SIMD emulation library][Fixing SIMD] was written with the intention
that any other Rust verification tool would be able to use it.
The [blog posts on this website]({{site.baseurl}}) are written to help other tool builders
understand the problems we hit and the way that we solved them.

## The end

We are proud of the work we have done in this project
and we hope that others will be able to pick up what we
have done and run with it.

Enjoy!

Alastair Reid
<br>
Shaked Flur

----------

[SeaHorn]:                        https://seahorn.github.io/
[Install Crux]:                   {{site.baseurl}}{% post_url 2020-09-07-install-crux %}
[Using KLEE with CoreUtils]:      {{site.baseurl}}{% post_url 2021-07-14-coreutils %}
[Using KLEE with Rust for Linux (part 1)]: {{site.baseurl}}{% post_url 2021-08-22-rust-on-linux-1 %}
[HATRA 2020]:                     https://alastairreid.github.io/papers/HATRA_20/
[Survey 2020]: https://alastairreid.github.io/rust-verification-tools/
[Survey 2021]: https://alastairreid.github.io/automatic-rust-verification-tools-2021/
[Using ARGV]:                     {{site.baseurl}}{% post_url 2020-09-09-using-argv %}
[Fixing SIMD]:                    {{site.baseurl}}{% post_url 2021-05-15-verifying-vectorized-code2 %}
[KLEE]:                           https://klee.github.io/
[RMC]: https://github.com/model-checking/rmc
[MIRAI]: https://github.com/facebookexperimental/MIRAI
