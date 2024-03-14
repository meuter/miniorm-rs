fn main() {
    println!("cargo:rerun-if-changed=db/migrations");
}

