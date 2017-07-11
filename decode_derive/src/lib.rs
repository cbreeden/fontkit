extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

// TODO:
//   Gather all `StaticEncodeSize` elements and do eager bounds checks.
//   This seems to generate better assembly.  Also generate `StaticEncodeSize`
//   for structs that only contain `StaticEncodeSize` types.

#[proc_macro_derive(Decode, attributes(WithParam))]
pub fn parse_decode(input: TokenStream) -> TokenStream {
    let source = input.to_string();
    let ast = syn::parse_derive_input(&source).expect("failed to parse rust syntax");
    let gen = impl_parse(&ast);
    let ret: TokenStream = gen.parse().expect("failed to serialize to rust syntax");
    // panic!(ret.to_string());
    ret
}

fn impl_parse(ast: &syn::DeriveInput) -> quote::Tokens {
    use syn::{Body, VariantData};

    let variants = match ast.body {
        Body::Struct(VariantData::Struct(ref vars)) => vars,
        _ => panic!("#[derive(Parse)] is only defined for braced structs"),
    };

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let ident = &ast.ident;
    let fields = variants.iter().map(|field| field.ident.as_ref().unwrap());
    let parse = variants
        .iter()
        .filter(|field| field.ident.as_ref().unwrap() != "buffer")
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let ty = &field.ty;

            let with_param = field
                .attrs
                .iter()
                .filter_map(|f| match f.value {
                                syn::MetaItem::NameValue(ref id, ref lit) => {
                                    if id == "WithParam" { Some(lit) } else { None }
                                }
                                _ => None,
                            })
                .next();

            if let Some(lit) = with_param {
                if let &syn::Lit::Str(ref lit, _) = lit {
                    let mut tokens = quote::Tokens::new();
                    tokens.append(lit);

                    quote! {
                        let #ident = buf.decode_read_with::<#ty>(#tokens)?;
                    }
                } else {
                    quote! {
                        let #ident = buf.decode_read_with::<#ty>(#lit)?;
                    }
                }
            } else {
                quote! {
                    let #ident = buf.decode_read::<#ty>()?;
                }
            }
        });

    let build = variants
        .iter()
        .map(|field| field.ident.as_ref().unwrap())
        .map(|id| quote! { #id : #id });

    quote! {
        impl<'fnt> Decode<'fnt> for #ident #ty_generics #where_clause {
            #[inline]
            fn decode(buffer: &'fnt [u8]) -> Result<Self> {
                let mut buf = buffer;
                #(#parse)*

                Ok(#ident {
                    #(#build),*
                })
            }
        }
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