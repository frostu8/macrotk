# macrotk
An extensible macro toolkit for Rust.

I got tired of writing the same functions for handling parameters to macros,
specifically attribute macros. It turns out that I was so right about being
tired of it that you can just write a macro for it. Yep, now we're going full
***macroception***.

This has actually [already been done before](https://crates.io/crates/devise),
but it's old, and I wanted to try out writing something like it.

```rust
use syn::{parse_macro_input, LitStr};

use macrotk::{meta::Meta, FromMeta};

use proc_macro::TokenStream;

#[derive(FromMeta)]
pub struct MacroMeta {
    something: LitStr,
    otherthing: LitStr,
}

#[proc_macro_attribute]
pub fn cool_macro(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as Meta<MacroMeta>);

    // now we can just use these fields!
    let something = &attr.something;
    let otherthing = &attr.something;
    
    // ... do stuff ...

    item
}
```
<span style="font-size: 8px">This should work... I hope</span>
