fn main() {
    cc::Build::new()
        .file("bar.c")
        .compile("bar_library");
}
