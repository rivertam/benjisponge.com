fn main() {
    // Watch the whole styles/ dir: input.css @imports the per-section files.
    println!("cargo:rerun-if-changed=styles");
    // Tailwind scans src/**/*.rs for utility classes, so a class first used
    // in a .rs file must rerun this script or it silently never reaches the
    // generated stylesheet. The scan honors .gitignore, so watch that too.
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=.gitignore");
    topcoat::tailwind::BuildConfig::new()
        .input("styles/input.css")
        .render()
        .unwrap();
}
