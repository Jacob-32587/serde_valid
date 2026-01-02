use std::collections::HashMap;

use super::{attr::get_ser_and_de_rename, case::RenameRule};
use crate::types::{CommaSeparatedMetas, Field, NamedField};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub type RenameMap = HashMap<String, TokenStream>;

pub fn collect_serde_rename_map(
    container_attributes: &[syn::Attribute],
    fields: &syn::FieldsNamed,
    is_container_enum: bool,
) -> RenameMap {
    let maybe_serde_attribute = container_attributes
        .iter()
        .find(|x| x.path().is_ident("serde"))
        .into_iter()
        .next();
    let mut renames = RenameMap::new();

    let rename_rule = if let Some(serde_attribute) = maybe_serde_attribute {
        let mut rule = RenameRule::None;
        serde_attribute
            .parse_nested_meta(|meta| {
                if meta.path.is_ident("rename_all") {
                    if let Ok((_, Some(de))) = get_ser_and_de_rename(&meta) {
                        if let Some(found_rule) = RenameRule::from_str(&de) {
                            rule = found_rule;
                        }
                    };
                };
                Ok(())
            })
            .ok();
        rule
    } else {
        RenameRule::None
    };

    for field in fields.named.iter() {
        let named_field = NamedField::new(field);
        for attribute in named_field.attrs() {
            if attribute.path().is_ident("serde") {
                if let Some(rename) = find_rename_from_serde_attributes(attribute) {
                    renames.insert(
                        field.ident.to_token_stream().to_string(),
                        quote!(std::borrow::Cow::from(#rename)),
                    );
                }
            } else if is_container_enum && rename_rule.will_variant_change() {
                let rename =
                    rename_rule.apply_to_variant(&field.ident.to_token_stream().to_string());
                renames.insert(
                    field.ident.to_token_stream().to_string(),
                    quote!(std::borrow::Cow::from(#rename)),
                );
            } else if !is_container_enum && rename_rule.will_field_change() {
                let rename = rename_rule.apply_to_field(&field.ident.to_token_stream().to_string());
                renames.insert(
                    field.ident.to_token_stream().to_string(),
                    quote!(std::borrow::Cow::from(#rename)),
                );
            }
        }
    }
    renames
}

fn find_rename_from_serde_attributes(attribute: &syn::Attribute) -> Option<TokenStream> {
    if let syn::Meta::List(serde_list) = &attribute.meta {
        if let Ok(serde_nested_meta) =
            serde_list.parse_args_with(CommaSeparatedMetas::parse_terminated)
        {
            for serde_meta in serde_nested_meta {
                if let Some(rename) = find_rename_from_serde_rename_attributes(&serde_meta) {
                    return Some(rename);
                }
            }
        }
    }
    None
}

fn find_rename_from_serde_rename_attributes(serde_meta: &syn::Meta) -> Option<TokenStream> {
    match serde_meta {
        syn::Meta::NameValue(rename_name_value) => {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit_str),
                ..
            }) = &rename_name_value.value
            {
                Some(lit_str.to_token_stream())
            } else {
                None
            }
        }
        syn::Meta::List(rename_list) => {
            if let Ok(nested) = rename_list.parse_args_with(CommaSeparatedMetas::parse_terminated) {
                for rename_meta in nested {
                    if !rename_meta.path().is_ident("deserialize") {
                        continue;
                    }
                    if let syn::Meta::NameValue(deserialize_name_value) = rename_meta {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(lit_str),
                            ..
                        }) = &deserialize_name_value.value
                        {
                            return Some(lit_str.to_token_stream());
                        }
                    }
                }
            }

            None
        }
        _ => None,
    }
}
