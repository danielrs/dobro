extern crate gcc;

fn main() {
    gcc::compile_library("libearwax.a", &["earwax.c"]);
}
