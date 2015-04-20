Code for Rust Talk in May 2015
==============================

This code implements the seam carving algorithm, but the commit history has is somewhat artificially constructed to show a series on incremental steps to improve performance - without using any tricks on the algorithm itself, such as recalculating only the needed energy.

In other words, this is a fast brute force implementation constructed in steps which can be used to demo how to turn slow Rust code into fast Rust code (ideally with the use of a profiler or two like `perf` or valgrind).

Details on the actual algorithm:

* Spec: http://coursera.cs.princeton.edu/algs4/assignments/seamCarving.html
* FAQ: http://coursera.cs.princeton.edu/algs4/checklists/seamCarving.html
* More sample inputs: http://coursera.cs.princeton.edu/algs4/testing/seamCarving-testing.zip

Sample execution which reduces the width of the bundled image by 300 pixels:

```
cargo run --release -- test-input.png -o /tmp/output.png -W 300
```

