//! A crate to make using `tonic::include_proto` less painful.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

extern crate proc_macro2;
use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

#[derive(Default, Debug)]
struct Namespace {
    children: HashMap<String, Namespace>,
    generate: bool,
}

impl Namespace {
    fn new(generate: bool) -> Self {
        Self {
            children: HashMap::new(),
            generate,
        }
    }
}

/// This macro invokes the macro `tonic::include_proto` for multiple protobuf packages
/// and each of them are placed in the correct namespace.
///
/// # Example
///
/// The code:
///
/// ```
/// tonic_include_proto::namespaced!("x.y", "x.z");
/// ```
///
/// is equivalent to
///
/// ```
/// mod x {
///     mod y {
///         tonic::include_proto!("x.y");
///     }
///     mod z {
///         tonic::include_proto!("x.z");
///     }
/// }
/// ```
#[proc_macro]
pub fn namespaced(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input as a string literal collection.
    let input = input.into_iter().collect::<Vec<_>>();
    let packages: Vec<_> = input
        .iter()
        .filter_map(|item| match item {
            proc_macro::TokenTree::Literal(lit) => {
                let mut s = lit.to_string();
                s.retain(|c| c != '"');
                Some(s)
            }
            _ => None,
        })
        .collect();
    // Now we split each import (which holds a package name, such as x.z.y)
    // into separate namespace names (x, z and y).
    let packages: Vec<_> = packages
        .iter()
        .map(|i| i.split('.').collect::<Vec<_>>())
        .collect();
    // Flatten the structure of multiple packages by trasforming the Vec<Vec> into a Map<Name, Map>.
    let mut namespaces_map: HashMap<String, Namespace> = HashMap::new();
    for package in packages {
        let mut map = &mut namespaces_map;
        for (i, import) in package.iter().enumerate() {
            let entry = map
                .entry(import.to_string())
                .or_insert_with(|| Namespace::new(i == package.len() - 1));
            map = &mut entry.children;
        }
    }
    // Recursively reconstruct the hierarchy of namespaces.
    let tokens: Vec<_> = namespaces_map
        .iter()
        .map(|(name, namespace)| build_namespace_tokens(name.to_string(), name, namespace))
        .collect();
    let tokens = quote! {
        #(#tokens)*
    };
    tokens.into()
}

fn build_namespace_tokens(path: String, name: &str, namespace: &Namespace) -> TokenStream {
    // Compute the inner namespaces' token streams.
    let inner: Vec<_> = namespace
        .children
        .iter()
        .map(|(name, namespace)| {
            build_namespace_tokens(format!("{}.{}", path, name), name, namespace)
        })
        .collect();
    // Format the namespace name.
    let formatted_name = format_ident!("{}", name);
    // When generate=true invoke the tonic macro to include the protobuf.
    let include_token = if namespace.generate {
        quote! {
            tonic::include_proto!(#path);
        }
    } else {
        quote! {}
    };
    quote! {
        pub mod #formatted_name {
            #include_token
            #(#inner)*
        }
    }
}
