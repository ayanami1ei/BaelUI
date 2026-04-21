use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, ItemMod, Path,
    Expr, ExprTuple,
};

pub fn impl_externed(args: TokenStream, input: TokenStream)->TokenStream{
    // 解析输入为一个模块（`mod { ... }`）
    let mut item_mod = parse_macro_input!(input as ItemMod);

    // 解析参数为路径列表，例如: #[externed(foo::bar, crate::prelude)]。
    // 如果未提供参数，默认注入 `crate::prelude`。
    let mut deps_paths: Vec<Path> = Vec::new();
    if args.is_empty() {
        deps_paths.push(parse_quote!(crate::prelude));
    } else {
        // 把参数包装成元组文本 `(a, b, c)`，然后解析为 ExprTuple，逐项要求为 Path 表达式
        let s = format!("({})", args.to_string());
        match syn::parse_str::<ExprTuple>(&s) {
            Ok(tuple) => {
                for expr in tuple.elems.into_iter() {
                    match expr {
                        Expr::Path(ep) => deps_paths.push(ep.path),
                        _ => return TokenStream::from(quote! { compile_error!("`externed` arguments must be paths like `crate::prelude`"); }),
                    }
                }
            }
            Err(_) => return TokenStream::from(quote! { compile_error!("`externed` arguments must be paths like `crate::prelude`"); }),
        }
    }

    // 仅支持内联模块 `mod name { ... }`，否则报错
    if let Some((_, items)) = &mut item_mod.content {
        // 只在模块末尾添加 `pub use prelude::*;`（忽略参数）
        items.push(parse_quote!(pub use crate::prelude::*;));

        quote! { #item_mod }.into()
    } else {
        let ident = &item_mod.ident;
        TokenStream::from(quote! {
            compile_error!(concat!("`externed` macro requires inline module for ", stringify!(#ident)));
        })
    }
}