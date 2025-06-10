use darling::{Error, FromDeriveInput, FromField, util::Ignored};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, Ident, Type, Visibility, parse_macro_input};

mod symbols;
use symbols::*;

#[derive(Debug)]
#[allow(unused)]
struct ParsedAccField(Option<Ident>, Type, Visibility, Attribute);

#[derive(FromField, Debug)]
#[darling(attributes(intrusive_list), forward_attrs)]
struct DataField {
    ident: Option<Ident>,
    vis: syn::Visibility,
    ty: syn::Type,
    attrs: Vec<syn::Attribute>,
}

macro_rules! trace {
    ($($stuff:tt)+) => {
        println!("datastructu_rs::{}::{}: {}", file!(), line!(),format_args!($($stuff)+))
    };
}

#[derive(FromDeriveInput, Debug)]
#[darling(
    attributes(IntoIntrusiveList, accessor),
    supports(struct_any),
    forward_attrs
)]
struct DataStruct {
    ident: Ident,
    data: darling::ast::Data<Ignored /* idc about enums */, DataField>,
}

#[proc_macro_derive(IntoIntrusiveList, attributes(accessor))]
pub fn derive_intrusive_linked_list(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item);
    let opts = match DataStruct::from_derive_input(&input) {
        Ok(o) => o,
        Err(e) => {
            return e.write_errors().into();
        }
    };
    let struct_id = opts.ident;
    let fields = opts
        .data
        .take_struct()
        .expect("only works for structs")
        .fields;

    let accessor_fields: Vec<ParsedAccField> = fields
        .into_iter()
        .filter(|field| {
            field.attrs.iter().any(|attr| match &attr.meta {
                syn::Meta::List(lv) => lv.path == ACCESSOR,
                _other => false,
            })
        })
        .map(|field| ParsedAccField(field.ident, field.ty, field.vis, field.attrs[0].clone()))
        .collect();

    if accessor_fields.is_empty() {
        let e = Error::custom("need at least one field with 'accessor(MyAccessor)'");
        return e.write_errors().into();
    }

    let mut outputs = Vec::new();
    for accessor_field in accessor_fields {
        let vis: Visibility = accessor_field.2;
        let acc_field = accessor_field.0.expect("ident is none??????????");
        let acc_attr = accessor_field.3;
        let acc_meta: syn::MetaList = match acc_attr.meta {
            syn::Meta::List(lv) => lv,
            _ => panic!("weird attribute kind"),
        };
        let acc_id = acc_meta.tokens;

        outputs.push(quote! {
            #[automatically_derived]
            #vis struct #acc_id;
            #[automatically_derived]
            impl IntrusiveListAccessor<#struct_id> for #acc_id {
                fn get_node(item: &#struct_id) -> &ListLink {
                    &item.#acc_field
                }

                fn get_node_mut(item: &mut #struct_id) -> &mut ListLink {
                    &mut item.#acc_field
                }

                unsafe fn from_node(node: &ListLink) -> &#struct_id {
                    let offset = std::mem::offset_of!(#struct_id, #acc_field);
                    let p_node = node as *const _ as *const u8;
                    let p_struct = unsafe { p_node.sub(offset) } as *const #struct_id;
                    unsafe { &*p_struct }
                }

                unsafe fn from_node_mut(node: &mut ListLink) -> &mut #struct_id {
                    let offset = std::mem::offset_of!(#struct_id, #acc_field);
                    let p_node = node as *const _ as *const u8;
                    let p_struct = unsafe { p_node.sub(offset) } as *mut #struct_id;
                    unsafe { &mut *p_struct }
                }
            }
        });
    }

    let output = {
        let mut t = quote! {};
        for im in outputs {
            t = quote! {
                #t
                #im
            };
        }
        t
    };
    #[cfg(debug_assertions)]
    trace!(
        "datastructurs_ill_proc_macro: The following code was generated:\n=====\n{output}\n=====\n"
    );
    output.into()
}
