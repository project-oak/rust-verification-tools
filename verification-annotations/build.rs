fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    #[cfg(feature = "verifier-seahorn")]
    seahorn();
}

#[cfg(feature = "verifier-seahorn")]
fn seahorn() {
    println!("cargo:rerun-if-changed=lib/seahorn.c");
    cc::Build::new()
        .file("lib/seahorn.c")
        .compile("seahorn");
}
