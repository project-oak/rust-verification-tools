// Copyright 2021 The Propverify authors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    #[cfg(feature = "verifier-seahorn")]
    seahorn();
}

#[cfg(feature = "verifier-seahorn")]
fn seahorn() {
    println!("cargo:rerun-if-changed=lib/seahorn.c");
    cc::Build::new().file("lib/seahorn.c").compile("seahorn");
}
