mod widget;
mod externed;

use proc_macro::TokenStream;            // <- 标准 proc_macro
use crate::widget::impl_widget;

#[proc_macro_attribute]
pub fn widget(args: TokenStream, input: TokenStream) -> TokenStream {
    impl_widget(args, input)
}

#[proc_macro_attribute]
pub fn externed(args: TokenStream, input: TokenStream) -> TokenStream {
    externed::impl_externed(args, input)
}