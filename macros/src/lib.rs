use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse_macro_input;
use syn::spanned::Spanned;
use syn::{Expr, ItemImpl, LitStr, Token};

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
    let policy_expr = args.policy;
    let item_impl = parse_macro_input!(item as ItemImpl);
    match generate_impl(method, path_lit, policy_expr, item_impl) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn generate_impl(
    method: Method,
    path: LitStr,
    policy: Option<Expr>,
    item_impl: ItemImpl,
) -> syn::Result<proc_macro2::TokenStream> {
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
    let policy_tokens = policy
        .map(|policy_expr| quote! { .with_policy(#policy_expr) })
        .unwrap_or_default();
    let generics = item_impl.generics.clone();
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let tokens = quote! {
        #item_impl

        impl #impl_generics #crate_path::controller::route::HttpRoute for #handler_ty #where_clause {
            fn route() -> #crate_path::endpoint::route::RouteBuilder {
                #crate_path::endpoint::route::EndpointRoute::#builder_ident(#path, #handler_expr)#policy_tokens
            }
        }

        #crate_path::inventory::submit! {
            #crate_path::controller::attribute_route::RegisteredHttpRoute {
                build: || <#handler_ty as #crate_path::controller::route::HttpRoute>::endpoint(),
            }
        }
    };
    Ok(tokens)
}

struct RouteArgs {
    path: LitStr,
    policy: Option<Expr>,
}

impl Parse for RouteArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.is_empty() {
            return Err(syn::Error::new(
                Span::call_site(),
                "route attribute requires a path literal (and optional policy = ...)",
            ));
        }

        let mut path: Option<LitStr> = None;
        let mut policy: Option<Expr> = None;

        if input.peek(syn::LitStr) {
            path = Some(input.parse()?);
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match ident.to_string().as_str() {
                "path" => {
                    if path.is_some() {
                        return Err(syn::Error::new(
                            ident.span(),
                            "path provided more than once",
                        ));
                    }
                    path = Some(input.parse::<LitStr>()?);
                }
                "policy" => {
                    if policy.is_some() {
                        return Err(syn::Error::new(
                            ident.span(),
                            "policy provided more than once",
                        ));
                    }
                    policy = Some(input.parse::<Expr>()?);
                }
                _ => {
                    return Err(syn::Error::new(ident.span(), "expected `path` or `policy`"));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        let path = path.ok_or_else(|| {
            syn::Error::new(
                Span::call_site(),
                "route attribute requires a path literal or path = \"/route\"",
            )
        })?;

        Ok(Self { path, policy })
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
