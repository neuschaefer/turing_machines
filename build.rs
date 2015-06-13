extern crate gcc;

fn main() {
    // NOTE: Using the gcc crate means that, it would link libturingrt into the
    //       rust code and put it into a weird subdirectory, but I'll do it
    //       anyway, because calling gcc manually is just a pain.

    gcc::compile_library("libturingrt.a", &["src/rt.c"]);
}
