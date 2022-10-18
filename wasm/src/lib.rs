//! Internal WASM implementation of momo.
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::*;
use syn::{fold::Fold, punctuated::Punctuated};

#[derive(Copy, Clone)]
// All conversions we support.  Check references to this type for an idea how to add more.
enum Conversion<'a> {
    Into(&'a Type),
    AsRef(&'a Type),
    AsMut(&'a Type),
}

impl<'a> Conversion<'a> {
    fn target_type(&self) -> Type {
        match *self {
            Conversion::Into(ty) => ty.clone(),
            Conversion::AsRef(ty) => parse_quote!(&#ty),
            Conversion::AsMut(ty) => parse_quote!(&mut #ty),
        }
    }

    fn conversion_expr(&self, i: Ident) -> Expr {
        match *self {
            Conversion::Into(_) => parse_quote!(#i.into()),
            Conversion::AsRef(_) => parse_quote!(#i.as_ref()),
            Conversion::AsMut(_) => parse_quote!(#i.as_mut()),
        }
    }
}

fn parse_bounded_type(ty: &Type) -> Option<Ident> {
    if let Type::Path(TypePath {
        qself: None,
        ref path,
    }) = ty
    {
        if path.segments.len() == 1 {
            return Some(path.segments[0].ident.clone());
        }
    }
    None
}

fn parse_bounds(bounds: &Punctuated<TypeParamBound, Token![+]>) -> Option<Conversion> {
    if bounds.len() != 1 {
        return None;
    }
    if let TypeParamBound::Trait(ref tb) = bounds.first().unwrap() {
        if let Some(seg) = tb.path.segments.iter().last() {
            if let PathArguments::AngleBracketed(ref gen_args) = seg.arguments {
                if gen_args.args.len() != 1 {
                    return None;
                }
                if let GenericArgument::Type(ref arg_ty) = gen_args.args.first().unwrap() {
                    if seg.ident == "Into" {
                        return Some(Conversion::Into(arg_ty));
                    } else if seg.ident == "AsRef" {
                        return Some(Conversion::AsRef(arg_ty));
                    } else if seg.ident == "AsMut" {
                        return Some(Conversion::AsMut(arg_ty));
                    }
                }
            }
        }
    }
    None
}

// create a map from generic type to Conversion
fn parse_generics(decl: &Signature) -> (HashMap<Ident, Conversion<'_>>, Generics) {
    let mut ty_conversions = HashMap::new();
    let mut params = Punctuated::new();
    for gp in decl.generics.params.iter() {
        if let GenericParam::Type(ref tp) = gp {
            if let Some(conversion) = parse_bounds(&tp.bounds) {
                ty_conversions.insert(tp.ident.clone(), conversion);
                continue;
            }
        }
        params.push(gp.clone());
    }
    let where_clause = if let Some(ref wc) = decl.generics.where_clause {
        let mut predicates = Punctuated::new();
        for wp in wc.predicates.iter() {
            if let WherePredicate::Type(ref pt) = wp {
                if let Some(ident) = parse_bounded_type(&pt.bounded_ty) {
                    if let Some(conversion) = parse_bounds(&pt.bounds) {
                        ty_conversions.insert(ident, conversion);
                        continue;
                    }
                }
            }
            predicates.push(wp.clone());
        }
        Some(WhereClause {
            predicates,
            ..wc.clone()
        })
    } else {
        None
    };
    (
        ty_conversions,
        Generics {
            params,
            where_clause,
            ..decl.generics.clone()
        },
    )
}

fn pat_to_ident(pat: &Pat) -> Ident {
    if let Pat::Ident(ref pat_ident) = *pat {
        return pat_ident.ident.clone();
    }
    unimplemented!("No non-ident patterns for now!");
}

fn pat_to_expr(pat: &Pat) -> Expr {
    let ident = pat_to_ident(pat);
    parse_quote!(#ident)
}

fn convert<'a>(
    inputs: &'a Punctuated<FnArg, Token![,]>,
    ty_conversions: &HashMap<Ident, Conversion<'a>>,
) -> (
    Punctuated<FnArg, Token![,]>,
    Conversions,
    Punctuated<Expr, Token![,]>,
    bool,
) {
    let mut argtypes = Punctuated::new();
    let mut conversions = Conversions {
        intos: Vec::new(),
        as_refs: Vec::new(),
        as_muts: Vec::new(),
    };
    let mut argexprs = Punctuated::new();
    let mut has_self = false;
    inputs.iter().for_each(|input| match input {
        FnArg::Receiver(..) => {
            has_self = true;
            argtypes.push(input.clone());
        }
        // FIXME: Should tick has_self on typed receivers too.
        FnArg::Typed(PatType {
            ref pat,
            ref ty,
            ref colon_token,
            ..
        }) => match **ty {
            Type::ImplTrait(TypeImplTrait { ref bounds, .. }) => {
                if let Some(conv) = parse_bounds(bounds) {
                    argtypes.push(FnArg::Typed(PatType {
                        attrs: Vec::new(),
                        pat: pat.clone(),
                        colon_token: *colon_token,
                        ty: Box::new(conv.target_type()),
                    }));
                    let ident = pat_to_ident(pat);
                    conversions.add(ident.clone(), conv);
                    argexprs.push(conv.conversion_expr(ident));
                }
            }
            Type::Path(..) => {
                if let Some(ident) = parse_bounded_type(ty) {
                    if let Some(conv) = ty_conversions.get(&ident) {
                        argtypes.push(FnArg::Typed(PatType {
                            attrs: Vec::new(),
                            pat: pat.clone(),
                            colon_token: *colon_token,
                            ty: Box::new(conv.target_type()),
                        }));
                        let ident = pat_to_ident(pat);
                        conversions.add(ident, *conv);
                        argexprs.push(conv.conversion_expr(pat_to_ident(pat)));
                    }
                }
            }
            _ => {
                argtypes.push(input.clone());
                argexprs.push(pat_to_expr(pat));
            }
        },
    });
    (argtypes, conversions, argexprs, has_self)
}

struct Conversions {
    intos: Vec<Ident>,
    as_refs: Vec<Ident>,
    as_muts: Vec<Ident>,
}

impl Conversions {
    fn add(&mut self, ident: Ident, conv: Conversion) {
        match conv {
            Conversion::Into(_) => self.intos.push(ident),
            Conversion::AsRef(_) => self.as_refs.push(ident),
            Conversion::AsMut(_) => self.as_muts.push(ident),
        }
    }
}

fn has_conversion(idents: &[Ident], expr: &Expr) -> bool {
    if let Expr::Path(ExprPath { ref path, .. }) = *expr {
        if path.segments.len() == 1 {
            let seg = path.segments.iter().last().unwrap();
            return idents.iter().any(|i| i == &seg.ident);
        }
    }
    false
}

#[allow(clippy::collapsible_if)]
impl Fold for Conversions {
    fn fold_expr(&mut self, expr: Expr) -> Expr {
        //TODO: Also catch `Expr::Call` with suitable paths & args
        if let Expr::MethodCall(ref mc) = expr {
            if mc.args.is_empty() {
                if mc.method == "into" {
                    if has_conversion(&self.intos, &mc.receiver) {
                        return *mc.receiver.clone();
                    }
                } else if mc.method == "as_ref" {
                    if has_conversion(&self.as_refs, &mc.receiver) {
                        return *mc.receiver.clone();
                    }
                } else if mc.method == "as_mut" {
                    if has_conversion(&self.as_muts, &mc.receiver) {
                        return *mc.receiver.clone();
                    }
                }
            }
        }
        syn::fold::fold_expr(self, expr)
    }
}

#[no_mangle]
/// Implementation of the momo macro.
pub extern "C" fn momo(code: TokenStream, _attr: TokenStream) -> TokenStream {
    //TODO: alternatively parse ImplItem::Method
    let code_clone = code.clone();
    let fn_item: Item = match syn::parse2(code) {
        Ok(input) => input,
        Err(err) => return err.to_compile_error(),
    };

    if let Item::Fn(ref item_fn) = fn_item {
        let inner_ident =
            syn::parse_str::<Ident>(&format!("_{}_inner", item_fn.sig.ident)).unwrap();
        let (ty_conversions, generics) = parse_generics(&item_fn.sig);
        let (argtypes, mut conversions, argexprs, has_self) =
            convert(&item_fn.sig.inputs, &ty_conversions);
        let new_item = Item::Fn(ItemFn {
            block: if has_self {
                parse_quote!({ self.#inner_ident(#argexprs) })
            } else {
                parse_quote!({ #inner_ident(#argexprs) })
            },
            ..item_fn.clone()
        });
        let mut new_inner_item = ItemFn {
            vis: Visibility::Inherited,
            sig: Signature {
                ident: inner_ident,
                generics,
                inputs: argtypes,
                ..item_fn.sig.clone()
            },
            ..item_fn.clone()
        };
        new_inner_item.block = Box::new(conversions.fold_block(std::mem::replace(
            new_inner_item.block.as_mut(),
            parse_quote!({}),
        )));
        let new_inner_item = Item::Fn(new_inner_item);
        quote!(#new_item #[inline(never)] #[allow(unused_mut)] #new_inner_item)
    } else {
        // FIXME: Warn user about inappropriate location. proc_macro_error
        // is a bit messy in watt.
        code_clone
    }
}
