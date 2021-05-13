#[macro_use]
extern crate macrotk;

#[derive(FromMeta)]
pub struct Test {
    help: macrotk::syn::LitStr,
}

fn main() {
}
