extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

macro_rules! token_lit {
    ($expr:expr) => ({
        let mut t = quote::Tokens::new();
        t.append($expr);
        t
    })
}

// TODO:
//   Gather all `StaticEncodeSize` elements and do eager bounds checks.
//   This seems to generate better assembly.  Also generate `StaticEncodeSize`
//   for structs that only contain `StaticEncodeSize` types.

#[proc_macro_derive(Decode, attributes(WithParam, DecodeDebug))]
pub fn parse_decode(input: TokenStream) -> TokenStream {
    let source = input.to_string();
    let ast = syn::parse_derive_input(&source).expect("failed to parse rust syntax");
    let gen = impl_decode(&ast);
    let ret: TokenStream = gen.parse().expect("failed to serialize to rust syntax");

    // Check for DecodeDebug attribute
    let debug = ast.attrs.iter()
        .filter(|attr|
            match attr.value {
                syn::MetaItem::Word(ref ident) => ident == "DecodeDebug",
                _ => false,
            })
        .next()
        .is_some();

    if debug {
        panic!("{:?}", ret.to_string())
    }

    ret
}

fn impl_decode(ast: &syn::DeriveInput) -> quote::Tokens {
    use syn::{Body, VariantData};

    let variants = match ast.body {
        Body::Struct(VariantData::Struct(ref vars)) => vars,
        _ => panic!("#[derive(Parse)] is only defined for braced structs"),
    };

    let (_, ty_generics, where_clause) = ast.generics.split_for_impl();
    let ident = &ast.ident;

    let decode = variants.iter()
        .filter(|field| !is_param_field(field))
        .map(decode_field);

    let build = variants.iter().map(build_field);

    match decode_params(variants) {
        None =>
            quote! {
                impl<'fnt> Decode<'fnt> for #ident #ty_generics #where_clause {
                    #[inline]
                    fn decode(buffer: &'fnt [u8]) -> Result<Self> {
                        let mut buf = buffer;
                        #(#decode)*
                        Ok(#ident { #(#build),* })
                    }
                }
            },

        Some((_trait, params)) =>
            quote! {
                impl<'fnt> #_trait for #ident #ty_generics #where_clause {
                    #[inline]
                    fn decode(#params) -> Result<Self> {
                        let mut buf = buffer;
                        #(#decode)*
                        Ok(#ident { #(#build),* })
                    }
                }
            },
    }
}

fn is_param_field(field: &syn::Field) -> bool {
    // A field will be taken from a parameter if the ident starts with __dundar__.
    let ident = field.ident.as_ref().unwrap().as_ref();
    ident.starts_with("__")
}

fn decode_params(fields: &[syn::Field]) -> Option<(quote::Tokens, quote::Tokens)> {
    let mut it = fields
        .iter()
        .filter(|field| is_param_field(field))
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            match discarded_type(&field.ty) {
                Some(ty) => {
                    let b = quote! { #ident: #ty };
                    let a = quote! { #ty };
                    (a, b)
                },
                None => {
                    let ty = &field.ty;
                    let b = quote! { #ident: #ty };
                    let a = quote! { #ty };
                    (a, b)
                }
            }
        }).collect::<Vec<_>>();

    if it.len() > 0 {
        let mut params = quote::Tokens::new();
        let mut _trait = quote::Tokens::new();

        params.append("buffer: &'fnt [u8]");

        match it.len() {
            1 => _trait.append("Decode1<'fnt"),
            2 => _trait.append("Decode2<'fnt"),
            _ => panic!("too many parameters"),
        }

        for (ty, param) in it {
            params.append(", ");
            params.append(param);
            _trait.append(", ");
            _trait.append(ty);
        }

        _trait.append(">");
        Some((_trait, params))
    } else {
        None
    }
}

fn build_field(field: &syn::Field) -> quote::Tokens {
    let id = field.ident.as_ref().unwrap();
    if discarded_type(&field.ty).is_some() {
        quote! { #id : Discarded(PhantomData) }
    } else {
        quote! { #id : #id }
    }
}

fn decode_field(field: &syn::Field) -> quote::Tokens {
    let ident = field.ident.as_ref().unwrap();
    let ty = &field.ty;

    // Handle `Discarded<T>` by parsing `T` instead.
    if let Some(ty) = discarded_type(ty) {
        return quote! {
            let #ident = <#ty>::decode(buf)?;
            let buf = buf.split_at(#ident .encode_size()).1;
        }
    }

    // Gather `WithParam`s, if they exist.
    let params = field_params(&field.attrs);

    match params {
        Some(params) => {
            quote! {
                let #ident = <#ty>::decode(#params)?;
                let buf = buf.split_at(#ident .encode_size()).1;
            }
        },
        None => {
            quote! {
                let #ident = <#ty>::decode(buf)?;
                let buf = buf.split_at(#ident .encode_size()).1;
            }
        }
    }
}

fn discarded_type(ty: &syn::Ty) -> Option<quote::Tokens> {
    if let syn::Ty::Path(_, ref path) = *ty {
        // take the last path segment, ie: `path::to::Discarded<T>`
        // We aren't interested if it's not a `Discarded` type
        let seg = path.segments.last().unwrap();
        if !seg.ident.as_ref().starts_with("Discarded") {
            return None
        }

        // Otherwise extract the `T` in `Discarded<T>`
        let ty_path = match seg.parameters {
            syn::PathParameters::AngleBracketed(ref data) => {
                data.types.first().unwrap()
            },
            _ => panic!("malformed `Discarded` parameter"),
        };

        use quote::ToTokens;
        let ty = match *ty_path {
            syn::Ty::Path(_, ref path) => {
                assert!(path.segments.len() == 1, "malformed `Discarded` parameter");
                path.segments.first().unwrap().ident.as_ref()
            },
            ref t => {
                let mut toks = quote::Tokens::new();
                t.to_tokens(&mut toks);
                return Some(toks);
            }
        };
        Some(token_lit!(ty))
    } else {
        None
    }
}

fn field_params(attrs: &[syn::Attribute]) -> Option<quote::Tokens> {
    let mut params = attrs
        .iter()
        .filter_map(|f|
            if let syn::MetaItem::NameValue(ref id, ref lit) = f.value {
                if id == "WithParam" { Some(lit) } else { None }
            } else {
                None
            })
        .map(|lit| match *lit {
            syn::Lit::Str(ref s, _) => s,
            _ => panic!("parameters must be a literal &str"),
        }).peekable();

    if params.peek().is_some() {
        let mut t = quote::Tokens::new();
        t.append("buf");
        for param in params {
            t.append(", ");
            t.append(param);
        }
        Some(t)
    } else {
        None
    }
}


#[proc_macro_derive(StaticEncodeSize)]
pub fn parse_static_size(input: TokenStream) -> TokenStream {
    let source = input.to_string();
    let ast = syn::parse_derive_input(&source).expect("failed to parse rust syntax");
    let gen = impl_static_size(&ast);
    let ret: TokenStream = gen.parse().expect("failed to serialize to rust syntax");
    // panic!(ret.to_string());
    ret
}

fn impl_static_size(ast: &syn::DeriveInput) -> quote::Tokens {
    use syn::{Body, VariantData};

    let variants = match ast.body {
        Body::Struct(VariantData::Struct(ref vars)) => vars,
        _ => panic!("#[derive(Parse)] is only defined for braced structs"),
    };

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let ident = &ast.ident;
    let tys = variants
        .iter()
        .filter(|field| field.ident.as_ref().unwrap() != "buffer")
        .map(|field| &field.ty);

    quote! {
        impl #impl_generics StaticEncodeSize for #ident #ty_generics #where_clause {
            #[inline]
            fn size() -> usize {
                #(<#tys as StaticEncodeSize>::size())+*
            }
        }
    }
}