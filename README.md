`turing_machines` [![(build status)][tr-img]][tr-link]
======================================================

This [rust] crate contains a library, compiler and interpreter for dealing with
Turing Machines.

It isn't fully baked yet, but patches and bug reports are welcome.

[rust]: https://rust-lang.org/
[tr-img]: https://travis-ci.org/neuschaefer/turing_machines.png
[tr-link]: https://travis-ci.org/neuschaefer/turing_machines


## Using the tools

First, you'll have to compile the crate, with `cargo build`.

`turing_machines` comes with two tools: An interpreter (`turing`), which will
simply follow the transitions of a turing machine until the stop state is
reached, and a compiler (`turingc`), that uses [LLVM] for optimization and code
generation.

[LLVM]: http://llvm.org/

### turing

```sh
$ cargo run --bin turing data/hello.tm </dev/null
HELLO.WORLD!
```

(Which does not work at the moment)

### turingc

```sh
$ target/debug/turingc data/hello.tm 2> hello.ll
$ clang -O2 hello.ll `find target/ -name libturingrt.a` -o hello
warning: overriding the module target triple with x86_64-pc-linux-gnu
1 warning generated.
$ ./hello </dev/null
HELLO.WORLD!
```


## File format

```
# TM format: whitespace separates entries
# transition: state,symbol,movement (no whitspace)
# first state is initial, last state is stopping state
# last symbol is Blank
# movement is: L/R/N (left/right/none)
# - for an unreachable transition
```


## Using the library

First, add this to your `Cargo.toml`:

```toml
[dependencies.turing_machines]
git = "https://github.com/neuschaefer/turing_machines"
```

Then, run `cargo doc --open` to see how the api works ;-)


## License

TODO, it will proabably be BSD-ish
