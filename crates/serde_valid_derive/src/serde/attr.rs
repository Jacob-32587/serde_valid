pub(crate) fn get_ser_and_de_rename(meta: &syn::meta::ParseNestedMeta) -> syn::Result<SerAndDe> {
    let mut ser_meta = None;
    let mut de_meta = None;

    let lookahead = meta.input.lookahead1();
    if lookahead.peek(syn::Token![=]) {
        if let Some(both) = get_lit_str2(meta)?.map(|x| x.value()) {
            ser_meta = Some(both.clone());
            de_meta = Some(both);
        }
    } else if lookahead.peek(syn::token::Paren) {
        meta.parse_nested_meta(|meta| {
            if meta.path.is_ident("serialize") {
                ser_meta = get_lit_str2(&meta)?.map(|x| x.value());
            } else if meta.path.is_ident("deserialize") {
                de_meta = get_lit_str2(&meta)?.map(|x| x.value());
            } else {
                return Err(meta.error(
                    "malformed attribute, expected `(serialize = ..., deserialize = ...)`",
                ));
            }
            Ok(())
        })?;
    } else {
        return Err(lookahead.error());
    }

    Ok((ser_meta, de_meta))
}

pub(crate) type SerAndDe = (Option<String>, Option<String>);

fn get_lit_str2(meta: &syn::meta::ParseNestedMeta) -> syn::Result<Option<syn::LitStr>> {
    let expr: syn::Expr = meta.value()?.parse()?;
    let mut value = &expr;
    while let syn::Expr::Group(e) = value {
        value = &e.expr;
    }
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(lit),
        ..
    }) = value
    {
        Ok(Some(lit.clone()))
    } else {
        Ok(None)
    }
}
