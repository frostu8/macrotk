#[doc(inline)]
pub use macrotk_core::*;
#[doc(inline)]
pub use macrotk_derive::*;

#[cfg(test)]
mod tests {
    #[test]
    pub fn from_meta_derive() {
        let t = trybuild::TestCases::new();
        t.pass("tests/from_meta_derive.rs");
    }
}
