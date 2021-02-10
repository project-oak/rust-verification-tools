// Copyright 2020-2021 The Propverify authors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use lazy_static::lazy_static;
use log::info;
use regex::Regex;

/// Detect lines that match #[should_panic(expected = ...)] string.
pub fn is_expected_panic(line: &str, expect: &Option<&str>, name: &str) -> bool {
    lazy_static! {
        static ref PANICKED: Regex = Regex::new(r" panicked at '([^']*)',\s+(.*)").unwrap();
    }

    if let Some(expect) = expect {
        if let Some(caps) = PANICKED.captures(line) {
            let message = caps.get(1).unwrap().as_str();
            let srcloc = caps.get(2).unwrap().as_str();
            if message.contains(expect) {
                info!(
                    "     {}: Detected expected failure '{}' at {}",
                    name, message, srcloc
                );
                info!("     Error message: {}", line);
                return true;
            }
        }
    }

    false
}
