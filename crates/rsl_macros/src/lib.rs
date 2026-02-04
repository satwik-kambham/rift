use proc_macro::TokenStream;
use quote::quote;
use syn::{
    FnArg, ItemFn, Lit, Meta, ReturnType, Token, Type, TypePath, parse::Parser,
    punctuated::Punctuated, spanned::Spanned,
};

#[proc_macro_attribute]
pub fn rsl_native(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = syn::parse_macro_input!(item as ItemFn);
    let fn_ident = &input_fn.sig.ident;

    if let Err(err) = validate_signature(&input_fn) {
        return err.to_compile_error().into();
    }

    let name_override = match parse_name_override(attr) {
        Ok(value) => value,
        Err(err) => return err.to_compile_error().into(),
    };

    let derived_name = to_lower_camel(&fn_ident.to_string());
    let name_literal = name_override.unwrap_or(derived_name);

    let expanded = quote! {
        #input_fn
        rsl::submit_native_function!(#name_literal, #fn_ident);
    };

    expanded.into()
}

fn parse_name_override(attr: TokenStream) -> syn::Result<Option<String>> {
    if attr.is_empty() {
        return Ok(None);
    }

    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let metas = parser.parse(attr)?;
    let mut name_value = None;

    for meta in metas {
        if let Meta::NameValue(nv) = meta
            && nv.path.is_ident("name")
        {
            match nv.value {
                syn::Expr::Lit(expr_lit) => match expr_lit.lit {
                    Lit::Str(lit_str) => {
                        name_value = Some(lit_str.value());
                    }
                    _ => {
                        return Err(syn::Error::new(
                            expr_lit.span(),
                            "expected string literal for name",
                        ));
                    }
                },
                _ => {
                    return Err(syn::Error::new(
                        nv.value.span(),
                        "expected string literal for name",
                    ));
                }
            }
        }
    }

    Ok(name_value)
}

fn validate_signature(input_fn: &ItemFn) -> syn::Result<()> {
    let inputs = &input_fn.sig.inputs;
    if inputs.len() != 1 {
        return Err(syn::Error::new(
            input_fn.sig.inputs.span(),
            "expected signature: fn(Vec<Primitive>) -> Primitive",
        ));
    }

    let Some(FnArg::Typed(arg)) = inputs.first() else {
        return Err(syn::Error::new(
            input_fn.sig.inputs.span(),
            "expected a typed argument",
        ));
    };

    if !is_vec_of_primitive(&arg.ty) {
        return Err(syn::Error::new(
            arg.ty.span(),
            "expected argument type Vec<Primitive>",
        ));
    }

    match &input_fn.sig.output {
        ReturnType::Type(_, ty) if is_primitive_type(ty) => Ok(()),
        _ => Err(syn::Error::new(
            input_fn.sig.output.span(),
            "expected return type Primitive",
        )),
    }
}

fn is_vec_of_primitive(ty: &Type) -> bool {
    let Type::Path(TypePath { path, .. }) = ty else {
        return false;
    };

    let Some(seg) = path.segments.last() else {
        return false;
    };

    if seg.ident != "Vec" {
        return false;
    }

    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
        return false;
    };

    let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() else {
        return false;
    };

    is_primitive_type(inner_ty)
}

fn is_primitive_type(ty: &Type) -> bool {
    let Type::Path(TypePath { path, .. }) = ty else {
        return false;
    };

    path.segments
        .last()
        .map(|seg| seg.ident == "Primitive")
        .unwrap_or(false)
}

fn to_lower_camel(name: &str) -> String {
    let mut parts = name.split('_');
    let Some(first) = parts.next() else {
        return String::new();
    };

    let mut result = String::from(first);
    for part in parts {
        if part.is_empty() {
            continue;
        }
        let mut chars = part.chars();
        if let Some(first_char) = chars.next() {
            result.push(first_char.to_ascii_uppercase());
            result.push_str(chars.as_str());
        }
    }

    result
}
