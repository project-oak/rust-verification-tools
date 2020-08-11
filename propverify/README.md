# `propverify`

This is a library/DSL for writing verification harnesses.

We see formal verification and testing as two parts of the same
activity and so this library is designed to be compatible
with the
[proptest](https://github.com/AltSysrq/proptest)
fuzzing/property testing library
so that you can use the same harness with
either a formal verification tool or with a fuzzing tool.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.

## Acknowledgements

This crate is heavily based on the design and API of the wonderful
[proptest](https://github.com/AltSysrq/proptest)
fuzz-testing library.
The implementation also borrows techniques, tricks and code
from the implementation - you can learn a lot about how to write
an embedded DSL from reading the proptest code.

In turn, proptest was influenced by
the [Rust port of QuickCheck](https://github.com/burntsushi/quickcheck)
and
the [Hypothesis](https://hypothesis.works/) fuzzing/property testing library Python.
(proptest also acknowledges `regex_generate` – but we have not yet implemented
regex strategies for this library.)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as
above, without any
additional terms or conditions.
