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
    let fields = match opts.data.take_struct() {
        Some(f) => f,
        None => {
            let e = Error::custom("Could not access fields of struct");
            return e.write_errors().into();
        }
    }
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
        let acc_field = match accessor_field.0 {
            Some(a) => a,
            None => {
                let e = Error::custom("No identifier for the accessor field");
                return e.write_errors().into();
            }
        };
        let acc_attr = accessor_field.3;
        let acc_meta: syn::MetaList = match acc_attr.meta {
            syn::Meta::List(lv) => lv,
            _ => {
                let e = Error::custom("Wrong attribute kind, use 'accessor(MyAccessor)'");
                return e.write_errors().into();
            }
        };
        let acc_id = acc_meta.tokens;

        outputs.push(quote! {
            #[automatically_derived]
            #vis struct #acc_id;
            #[automatically_derived]
            impl datastructurs::intrusive_linked_list::IntrusiveListAccessor<#struct_id> for #acc_id {
                fn get_node(item: &#struct_id) -> &datastructurs::intrusive_linked_list::ListLink {
                    &item.#acc_field
                }

                fn get_node_mut(item: &mut #struct_id) -> &mut datastructurs::intrusive_linked_list::ListLink {
                    &mut item.#acc_field
                }

                unsafe fn from_node(node: &datastructurs::intrusive_linked_list::ListLink) -> &#struct_id {
                    let offset = std::mem::offset_of!(#struct_id, #acc_field);
                    let p_node = node as *const _ as *const u8;
                    let p_struct = unsafe { p_node.sub(offset) } as *const #struct_id;
                    unsafe { &*p_struct }
                }

                unsafe fn from_node_mut(node: &mut datastructurs::intrusive_linked_list::ListLink) -> &mut #struct_id {
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
