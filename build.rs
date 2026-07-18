fn main() {
    // Watch the whole styles/ dir: input.css @imports the per-section files.
    println!("cargo:rerun-if-changed=styles");
    topcoat::tailwind::BuildConfig::new()
        .input("styles/input.css")
        .render()
        .unwrap();
}
