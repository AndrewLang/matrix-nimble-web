use proc_macro::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse_macro_input;
use syn::spanned::Spanned;
use syn::{ItemImpl, LitStr, Token};

#[proc_macro_attribute]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_route(Method::Get, attr, item)
}

#[proc_macro_attribute]
pub fn post(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_route(Method::Post, attr, item)
}

#[proc_macro_attribute]
pub fn put(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_route(Method::Put, attr, item)
}

#[proc_macro_attribute]
pub fn delete(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_route(Method::Delete, attr, item)
}

enum Method {
    Get,
    Post,
    Put,
    Delete,
}

impl Method {
    fn builder_ident(&self) -> proc_macro2::Ident {
        let name = match self {
            Method::Get => "get",
            Method::Post => "post",
            Method::Put => "put",
            Method::Delete => "delete",
        };
        format_ident!("{}", name)
    }
}

fn expand_route(method: Method, attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as RouteArgs);
    let path_lit = args.path;
    let item_impl = parse_macro_input!(item as ItemImpl);
    match generate_impl(method, path_lit, item_impl) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn generate_impl(method: Method, path: LitStr, item_impl: ItemImpl) -> syn::Result<proc_macro2::TokenStream> {
    let handler_ty = item_impl.self_ty.clone();
    let handler_expr = match handler_ty.as_ref() {
        syn::Type::Path(type_path) => {
            let path = &type_path.path;
            quote! { #path }
        }
        other => {
            return Err(syn::Error::new(
                other.span(),
                "route attributes currently support only concrete type paths",
            ));
        }
    };

    let crate_path = resolve_crate_path();
    let builder_ident = method.builder_ident();
    let generics = item_impl.generics.clone();
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let tokens = quote! {
        #item_impl

        impl #impl_generics #crate_path::controller::route::HttpRoute for #handler_ty #where_clause {
            fn route() -> #crate_path::endpoint::route::RouteBuilder {
                #crate_path::endpoint::route::EndpointRoute::#builder_ident(#path, #handler_expr)
            }
        }
    };
    Ok(tokens)
}

struct RouteArgs {
    path: LitStr,
}

impl Parse for RouteArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.is_empty() {
            return Err(syn::Error::new(
                Span::call_site(),
                "route attribute requires a path literal",
            ));
        }

        if input.peek(syn::LitStr) {
            let path: LitStr = input.parse()?;
            if !input.is_empty() {
                return Err(syn::Error::new(
                    input.span(),
                    "unexpected tokens after route literal",
                ));
            }
            return Ok(Self { path });
        }

        let ident: syn::Ident = input.parse()?;
        if ident != "path" {
            return Err(syn::Error::new(
                ident.span(),
                "expected string literal or path = \"/route\"",
            ));
        }

        input.parse::<Token![=]>()?;
        let path: LitStr = input.parse()?;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
        if !input.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                "unexpected tokens after path literal",
            ));
        }

        Ok(Self { path })
    }
}

fn resolve_crate_path() -> syn::Path {
    fn convert(found: FoundCrate) -> syn::Path {
        match found {
            FoundCrate::Itself => syn::parse_quote!(crate),
            FoundCrate::Name(name) => {
                let ident = syn::Ident::new(&name, Span::call_site());
                syn::parse_quote!(::#ident)
            }
        }
    }

    crate_name("nimble-web")
        .or_else(|_| crate_name("nimble_web"))
        .map(convert)
        .unwrap_or_else(|_| {
            let ident = syn::Ident::new("nimble_web", Span::call_site());
            syn::parse_quote!(::#ident)
        })
}
