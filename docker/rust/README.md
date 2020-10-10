## rust 
This is the first layer on ubuntu:expect 
This layer do two things: 
+ build rust-1.46.0 and cargo from source 
+ install rustup from official script and set compiled stage2 the default compiler
### possible issue 
The `./x.py build` may fail due to network error. To try it for a few times is likely to fix this problem.

