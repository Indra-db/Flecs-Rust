// Helper routines for the `Component` derive and related component utilities.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Data, DeriveInput, Expr, Fields, Ident, LitStr, Path, Result, Token, Type, parenthesized,
    parse::ParseStream,
};

// Parse #[flecs(...)] attribute and build calls to _component.add_trait::<flecs::...>();
// Additionally parse special options like `meta`, `on_registration`, and `name = "..."`.
pub(crate) fn collect_flecs_traits_calls(
    input: &DeriveInput,
) -> (TokenStream, bool, bool, Option<LitStr>) {
    use syn::{
        parenthesized, parse::Parse, parse::ParseStream, punctuated::Punctuated, token::Comma,
    };

    enum Item {
        Single(Path),
        Pair(Path, Path),
        Name(LitStr),
        Meta,
        OnRegistration,
        Add(Vec<Type>),
        Set(Vec<Expr>),
        Traits(Vec<Item>),
        Hooks(Vec<Item>),
        OnAdd(Expr),
        OnSetHook(Expr),
        OnRemove(Expr),
        OnReplace(Expr),
    }

    impl Parse for Item {
        fn parse(input: ParseStream) -> Result<Self> {
            if input.peek(syn::token::Paren) {
                let inner;
                parenthesized!(inner in input);
                let first: Path = inner.parse()?;
                inner.parse::<Comma>()?;
                let second: Path = inner.parse()?;
                Ok(Item::Pair(first, second))
            } else if input.peek(Ident) && input.peek2(Token![=]) {
                // name = "..."
                let ident: Ident = input.parse()?;
                input.parse::<Token![=]>()?;
                let value: LitStr = input.parse()?;
                if ident == "name" {
                    Ok(Item::Name(value))
                } else {
                    Err(syn::Error::new(
                        ident.span(),
                        "Unsupported flecs option. Expected `name = \"...\"`",
                    ))
                }
            } else if input.peek(Ident) && input.peek2(syn::token::Paren) {
                // function-like entries: add(...), set(...), traits(...), hooks(...), on_*(...)
                let ident: Ident = input.parse()?;
                if ident == "add" {
                    let inner;
                    parenthesized!(inner in input);
                    let mut tys: Vec<Type> = Vec::new();
                    while !inner.is_empty() {
                        let ty: Type = inner.parse()?;
                        tys.push(ty);
                        if inner.peek(Comma) {
                            inner.parse::<Comma>()?;
                        } else {
                            break;
                        }
                    }
                    Ok(Item::Add(tys))
                } else if ident == "set" {
                    let inner;
                    parenthesized!(inner in input);
                    let mut exprs: Vec<Expr> = Vec::new();
                    while !inner.is_empty() {
                        let expr: Expr = inner.parse()?;
                        exprs.push(expr);
                        if inner.peek(Comma) {
                            inner.parse::<Comma>()?;
                        } else {
                            break;
                        }
                    }
                    Ok(Item::Set(exprs))
                } else if ident == "traits" {
                    let inner;
                    parenthesized!(inner in input);
                    let mut items: Vec<Item> = Vec::new();
                    while !inner.is_empty() {
                        if inner.peek(syn::token::Paren) {
                            let p;
                            parenthesized!(p in inner);
                            let first: Path = p.parse()?;
                            p.parse::<Comma>()?;
                            let second: Path = p.parse()?;
                            items.push(Item::Pair(first, second));
                        } else if inner.peek(Ident) {
                            let fork = inner.fork();
                            let ident_peek: Ident = fork.parse()?;
                            if ident_peek == "meta" {
                                // consume the ident
                                let _ = inner.parse::<Ident>()?;
                                items.push(Item::Meta);
                            } else {
                                let p: Path = inner.parse()?;
                                items.push(Item::Single(p));
                            }
                        } else {
                            let p: Path = inner.parse()?;
                            items.push(Item::Single(p));
                        }
                        if inner.peek(Comma) {
                            inner.parse::<Comma>()?;
                        } else {
                            break;
                        }
                    }
                    Ok(Item::Traits(items))
                } else if ident == "hooks" {
                    let inner;
                    parenthesized!(inner in input);
                    let mut items: Vec<Item> = Vec::new();
                    while !inner.is_empty() {
                        let hook_ident: Ident = inner.parse()?;
                        if hook_ident == "on_add"
                            || hook_ident == "on_set"
                            || hook_ident == "on_remove"
                            || hook_ident == "on_replace"
                        {
                            let par;
                            parenthesized!(par in inner);
                            let expr: Expr = par.parse()?;
                            let which = hook_ident.to_string();
                            match which.as_str() {
                                "on_add" => items.push(Item::OnAdd(expr)),
                                "on_set" => items.push(Item::OnSetHook(expr)),
                                "on_remove" => items.push(Item::OnRemove(expr)),
                                "on_replace" => items.push(Item::OnReplace(expr)),
                                _ => unreachable!(),
                            }
                            if inner.peek(Comma) {
                                inner.parse::<Comma>()?;
                            }
                        } else {
                            return Err(syn::Error::new(
                                hook_ident.span(),
                                "Unknown hook in hooks(...). Expected on_add/on_set/on_remove/on_replace",
                            ));
                        }
                    }
                    Ok(Item::Hooks(items))
                } else if ident == "on_add"
                    || ident == "on_set"
                    || ident == "on_remove"
                    || ident == "on_replace"
                {
                    let inner;
                    parenthesized!(inner in input);
                    let expr: Expr = inner.parse()?;
                    if !inner.is_empty() {
                        return Err(syn::Error::new(
                            inner.span(),
                            "Expected a single hook expression",
                        ));
                    }
                    match &*ident.to_string() {
                        "on_add" => Ok(Item::OnAdd(expr)),
                        "on_set" => Ok(Item::OnSetHook(expr)),
                        "on_remove" => Ok(Item::OnRemove(expr)),
                        "on_replace" => Ok(Item::OnReplace(expr)),
                        _ => unreachable!(),
                    }
                } else {
                    Err(syn::Error::new(
                        ident.span(),
                        "Unknown flecs function. Expected `add(...)` or `set(...)` or `traits(...)` or `hooks(...)`",
                    ))
                }
            } else {
                // Bare identifier/path entry. Recognize `meta` and `on_registration` specially.
                if input.peek(Ident) {
                    let fork = input.fork();
                    let ident_peek: Ident = fork.parse()?;
                    if ident_peek == "meta" {
                        let _ = input.parse::<Ident>()?;
                        Ok(Item::Meta)
                    } else if ident_peek == "on_registration" {
                        let _ = input.parse::<Ident>()?;
                        Ok(Item::OnRegistration)
                    } else {
                        let p: Path = input.parse()?;
                        Ok(Item::Single(p))
                    }
                } else {
                    let p: Path = input.parse()?;
                    Ok(Item::Single(p))
                }
            }
        }
    }

    fn qualify(p: &Path) -> TokenStream {
        if p.segments.len() == 1 {
            let ident = &p.segments.first().unwrap().ident;
            quote! { flecs_ecs::core::flecs::#ident }
        } else {
            let first = p.segments.first().unwrap();
            if first.ident == "flecs" {
                let rest = p.segments.iter().skip(1).map(|seg| {
                    let ident = &seg.ident;
                    quote! { :: #ident }
                });
                quote! { flecs_ecs::core::flecs #( #rest )* }
            } else {
                quote! { #p }
            }
        }
    }

    fn should_be_unqualified(p: &Path) -> bool {
        p.segments
            .last()
            .map(|s| {
                let ident = &s.ident;
                ident == "With" || ident == "OneOf" || ident == "IsA" || ident == "ChildOf"
            })
            .unwrap_or(false)
    }

    let mut out = TokenStream::new();
    let mut has_flecs_meta = false;
    let mut has_on_registration = false;
    let mut flecs_name: Option<LitStr> = None;
    // Track ordering across all #[flecs(...)] attributes as encountered
    let mut position: usize = 0;
    let mut name_pos: Option<usize> = None;
    let mut meta_pos: Option<usize> = None;
    for attr in &input.attrs {
        if attr.path().is_ident("flecs") {
            let args: Result<Punctuated<Item, Token![,]>> =
                attr.parse_args_with(Punctuated::<Item, Token![,]>::parse_terminated);
            if let Ok(args) = args {
                for item in args.iter() {
                    position += 1;
                    match item {
                        Item::Traits(items) => {
                            for t in items {
                                match t {
                                    Item::Single(p) => {
                                        let q = qualify(p);
                                        out.extend(quote! { _component.add_trait::<#q>(); });
                                    }
                                    Item::Pair(p1, p2) => {
                                        let q1 = qualify(p1);
                                        let q2 = if should_be_unqualified(p1) {
                                            quote! { #p2 }
                                        } else {
                                            qualify(p2)
                                        };
                                        out.extend(
                                            quote! { _component.add_trait::<(#q1, #q2)>(); },
                                        );
                                    }
                                    Item::Meta => {
                                        has_flecs_meta = true;
                                        if meta_pos.is_none() {
                                            meta_pos = Some(position);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Item::Hooks(items) => {
                            for h in items {
                                match h {
                                    Item::OnAdd(hook) => {
                                        out.extend(quote! { _component.on_add(#hook); });
                                    }
                                    Item::OnSetHook(hook) => {
                                        out.extend(quote! { _component.on_set(#hook); });
                                    }
                                    Item::OnRemove(hook) => {
                                        out.extend(quote! { _component.on_remove(#hook); });
                                    }
                                    Item::OnReplace(hook) => {
                                        out.extend(quote! { _component.on_replace(#hook); });
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Item::Meta => {
                            has_flecs_meta = true;
                            if meta_pos.is_none() {
                                meta_pos = Some(position);
                            }
                        }
                        Item::OnRegistration => {
                            has_on_registration = true;
                        }
                        Item::Add(tys) => {
                            for ty in tys {
                                match ty {
                                    syn::Type::Tuple(tup) => {
                                        let elems = &tup.elems;
                                        if elems.len() == 2 {
                                            let first = &elems[0];
                                            let second = &elems[1];
                                            out.extend(quote! { _component.add((<#first>::id(), <#second>::id())); });
                                        } else {
                                            out.extend(quote! { compile_error!("add((...)) only supports pairs with exactly two types"); });
                                        }
                                    }
                                    _ => {
                                        out.extend(quote! { _component.add(<#ty>::id()); });
                                    }
                                }
                            }
                        }
                        Item::Set(exprs) => {
                            for expr in exprs {
                                match expr {
                                    syn::Expr::Tuple(tup) => {
                                        let elems = &tup.elems;
                                        if elems.len() == 2 {
                                            let first = &elems[0];
                                            let second = &elems[1];
                                            let first_is_value =
                                                !matches!(first, syn::Expr::Path(_));
                                            let second_is_value =
                                                !matches!(second, syn::Expr::Path(_));
                                            if first_is_value && !second_is_value {
                                                // (ValueExpr, TypePath)
                                                out.extend(quote! { _component.set_first(#first, <#second>::id()); });
                                            } else if !first_is_value && second_is_value {
                                                // (TypePath, ValueExpr)
                                                out.extend(quote! { _component.set_second(<#first>::id(), #second); });
                                            } else {
                                                out.extend(quote! { compile_error!("set((...)) expects exactly one value expression and one type path"); });
                                            }
                                        } else {
                                            out.extend(quote! { compile_error!("set((...)) only supports pairs with exactly two elements"); });
                                        }
                                    }
                                    _ => {
                                        out.extend(quote! { _component.set(#expr); });
                                    }
                                }
                            }
                        }
                        Item::Name(name) => {
                            // capture name; if multiple provided, raise a compile-time error later
                            if flecs_name.is_none() {
                                flecs_name = Some(name.clone());
                                name_pos = Some(position);
                            } else {
                                out.extend(quote! { compile_error!("Duplicate `name` in #[flecs(...)] attribute"); });
                            }
                        }
                        Item::Single(_) | Item::Pair(_, _) => {
                            out.extend(quote! { compile_error!("Traits should be wrapped in traits(...). Use #[flecs(traits(YourTrait))]"); });
                        }
                        _ => {
                            out.extend(quote! { compile_error!("Unexpected item in #[flecs(...)] attribute"); });
                        }
                    }
                }
            }
        }
    }
    // Validate ordering: if name/meta are provided, they must occupy the first two positions in any order.
    let mut ordering_error: Option<String> = None;
    match (name_pos, meta_pos) {
        (Some(n), Some(m)) => {
            if !(n == 1 && m == 2 || n == 2 && m == 1) {
                ordering_error = Some(format!(
                    "`name` and `meta` must be the first two items of #[flecs(...)] (found at positions {n} and {m})",
                ));
            }
        }
        (Some(n), None) => {
            if n != 1 {
                ordering_error = Some(format!(
                    "`name` must be the first item of #[flecs(...)] when present (found at position {n})",
                ));
            }
        }
        (None, Some(m)) => {
            if m != 1 {
                ordering_error = Some(format!(
                    "`meta` must be the first item of #[flecs(...)] when present (found at position {m})",
                ));
            }
        }
        (None, None) => {}
    }

    if let Some(msg) = ordering_error {
        let lit = LitStr::new(&msg, Span::call_site());
        out.extend(quote! { compile_error!(#lit); });
    }

    // If meta was requested, ensure we invoke it during registration.
    let out = if has_flecs_meta {
        quote! {
            _component.meta();
            #out
        }
    } else {
        out
    };

    (out, has_flecs_meta, has_on_registration, flecs_name)
}

pub(crate) fn impl_meta(
    input: &DeriveInput,
    has_repr_c: bool,
    struct_name: Ident,
    has_flecs_meta: bool,
) -> TokenStream {
    let has_meta_attribute = has_flecs_meta;

    if !has_meta_attribute {
        return quote! {};
    }

    let mut meta_fields_impl = Vec::new();

    match input.data.clone() {
        Data::Struct(data_struct) => {
            if let Fields::Named(fields_named) = &data_struct.fields {
                for field in &fields_named.named {
                    let is_ignored = field
                        .attrs
                        .iter()
                        .any(|attr| attr.path().is_ident("flecs_skip"));

                    if is_ignored {
                        continue;
                    }

                    let field_name = &field.ident;
                    let field_type = &field.ty;

                    if let Some(field_name) = field_name {
                        meta_fields_impl.push(quote! {
                            .member(id!(world, #field_type), (stringify!(#field_name), flecs_ecs::addons::meta::Count(0), core::mem::offset_of!(#struct_name, #field_name)))
                        });
                    } else {
                        meta_fields_impl.push( quote! {
                            compile_error!("Meta expects named fields, unnamed fields are not supported");
                        }
                        );
                    }
                }
            }
        }
        Data::Enum(data_enum) => {
            if !has_repr_c {
                meta_fields_impl.push( quote! {
                    compile_error!("Meta currently does not support Rust Algebraic enums, please use #[repr(C)] if it's a C compatible enum");
                }
                );
            } else {
                for variant in &data_enum.variants {
                    let is_ignored = variant
                        .attrs
                        .iter()
                        .any(|attr| attr.path().is_ident("flecs_skip"));

                    if is_ignored {
                        continue;
                    }

                    let variant_name = &variant.ident;

                    meta_fields_impl.push(quote! {
                        .constant(stringify!(#variant_name), #struct_name::#variant_name)
                    });
                }
            }
        }
        _ => return quote! { compile_error!("The type is neither a struct nor an enum!"); },
    };

    let meta_fn_impl = quote! {
        use flecs_ecs::addons::meta::*;
        use flecs_ecs::core::WorldProvider;
        let world = component.world();
        let id = #struct_name::get_id(world);
        component
        #( #meta_fields_impl )*;
    };

    meta_impl_return(meta_fn_impl, struct_name)
}

#[cfg(feature = "flecs_meta")]
pub(crate) fn meta_impl_return(meta_fn_impl: TokenStream, struct_name: Ident) -> TokenStream {
    quote! {
        impl flecs_ecs::addons::Meta<#struct_name> for #struct_name {
            fn meta(component: flecs_ecs::core::Component<'_, #struct_name>) {
                #meta_fn_impl
            }
        }
    }
}

#[cfg(not(feature = "flecs_meta"))]
pub(crate) fn meta_impl_return(_meta_fn_impl: TokenStream, struct_name: Ident) -> TokenStream {
    quote! {
        impl flecs_ecs::addons::Meta<#struct_name> for #struct_name {
            fn meta(component: flecs_ecs::core::Component<'_, #struct_name>) {
                println!("Meta is not enabled, please enable the `flecs_meta` feature to use the meta attribute");
                // Optionally, you can leave this empty or provide an alternative implementation
            }
        }
    }
}

pub(crate) fn generate_tag_trait(has_fields: bool) -> proc_macro2::TokenStream {
    if has_fields {
        quote! {
            const IS_TAG: bool = false;
            type TagType = flecs_ecs::core::component_registration::FlecsNotATag;
        }
    } else {
        quote! {
            const IS_TAG: bool = true;
            type TagType = flecs_ecs::core::component_registration::FlecsIsATag;
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct GenericTypeInfo {
    contains_type_bound: bool,
    contains_generic_type: bool,
    is_bound_default: bool,
    is_bound_clone: bool,
    is_bound_partial_ord: bool,
    is_bound_partial_eq: bool,
}

impl GenericTypeInfo {
    pub fn set_contains_type_bound(&mut self) {
        self.contains_type_bound = true;
    }

    pub fn set_contains_generic_type(&mut self) {
        self.contains_generic_type = true;
    }

    pub fn set_is_bound_default(&mut self) {
        self.is_bound_default = true;
    }

    pub fn set_is_bound_clone(&mut self) {
        self.is_bound_clone = true;
    }

    pub fn set_contains_partial_ord_bound(&mut self) {
        self.is_bound_partial_ord = true;
    }

    pub fn set_contains_partial_eq_bound(&mut self) {
        self.is_bound_partial_eq = true;
    }
}

// This function generates a series of trait implementations for structs.
// The implementations depend on the presence or absence of fields in the struct.
pub(crate) fn impl_cached_component_data_struct(
    ast: &mut syn::DeriveInput, // Name of the structure
    has_fields: bool,
    is_tag: &TokenStream,
    has_on_registration: bool,
    flecs_traits_calls: &TokenStream,
    flecs_name: &Option<LitStr>,
) -> proc_macro2::TokenStream {
    let is_generic = !ast.generics.params.is_empty();

    ast.generics.make_where_clause();

    let name = &ast.ident;

    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();
    let iter = &ast.generics.params.iter();
    let iter_where = where_clause.iter();
    let mut contains_lifetime_bound = false;

    let mut type_info_vec: Vec<(syn::Ident, GenericTypeInfo)> = Vec::new();

    //populate map with all the type generics
    iter.clone().for_each(|param| {
        if let syn::GenericParam::Type(type_param) = param {
            type_info_vec.push((type_param.ident.clone(), Default::default()));
        }
    });

    iter.clone().for_each(|param| {
        if let syn::GenericParam::Type(type_param) = param {
            // Find the corresponding entry in the vector.
            if let Some((_, type_info)) = type_info_vec
                .iter_mut()
                .find(|(id, _)| *id == type_param.ident)
            {
                type_info.set_contains_generic_type();

                if !type_param.bounds.empty_or_trailing() {
                    type_info.set_contains_type_bound();
                    type_param.bounds.iter().for_each(|bound| {
                        if let syn::TypeParamBound::Trait(trait_bound) = bound {
                            if trait_bound.path.is_ident("Default") {
                                type_info.set_is_bound_default();
                            } else if trait_bound.path.is_ident("Clone") {
                                type_info.set_is_bound_clone();
                            } else if trait_bound.path.is_ident("PartialOrd") {
                                type_info.set_contains_partial_ord_bound();
                            } else if trait_bound.path.is_ident("PartialEq") {
                                type_info.set_contains_partial_eq_bound();
                            }
                        }
                    });
                }
            }
        } else if let syn::GenericParam::Lifetime(_) = param {
            contains_lifetime_bound = true;
        }
    });

    iter_where.for_each(|where_clause| {
        for predicate in where_clause.predicates.iter() {
            if let syn::WherePredicate::Type(predicate_type) = predicate {
                // Extract the Ident from the bounded type
                if let Type::Path(type_path) = &predicate_type.bounded_ty
                    && let Some(segment) = type_path.path.segments.first()
                {
                    let type_ident = &segment.ident;
                    for bound in predicate_type.bounds.iter() {
                        if let syn::TypeParamBound::Trait(trait_bound) = bound {
                            if trait_bound.path.is_ident("Default") {
                                if let Some((_, gtype_info)) =
                                    type_info_vec.iter_mut().find(|(id, _)| *id == *type_ident)
                                {
                                    gtype_info.set_is_bound_default();
                                }
                            } else if trait_bound.path.is_ident("Clone") {
                                if let Some((_, gtype_info)) =
                                    type_info_vec.iter_mut().find(|(id, _)| *id == *type_ident)
                                {
                                    gtype_info.set_is_bound_clone();
                                }
                            } else if trait_bound.path.is_ident("PartialOrd") {
                                if let Some((_, gtype_info)) =
                                    type_info_vec.iter_mut().find(|(id, _)| *id == *type_ident)
                                {
                                    gtype_info.set_contains_partial_ord_bound();
                                }
                            } else if trait_bound.path.is_ident("PartialEq")
                                && let Some((_, gtype_info)) =
                                    type_info_vec.iter_mut().find(|(id, _)| *id == *type_ident)
                            {
                                gtype_info.set_contains_partial_eq_bound();
                            }
                        }
                    }
                }
            }
        }
    });

    let mut contains_any_type_bound = false;
    let mut contains_any_generic_type = false;
    let mut contains_all_default_bound = true;
    let mut contains_all_clone_bound = true;
    let mut contains_all_partial_ord_bound = true;
    let mut contains_all_partial_eq_bound = true;

    type_info_vec.iter().for_each(|(_, type_info)| {
        if type_info.contains_type_bound {
            contains_any_type_bound = true;
        }
        if type_info.contains_generic_type {
            contains_any_generic_type = true;
        }
        contains_all_default_bound &= type_info.is_bound_default;
        contains_all_clone_bound &= type_info.is_bound_clone;
        contains_all_partial_ord_bound &=
            type_info.contains_type_bound && type_info.is_bound_partial_ord;
        contains_all_partial_eq_bound &=
            type_info.contains_type_bound && type_info.is_bound_partial_eq;
    });

    let mut contains_where_bound = false;
    if let Some(where_clause) = &ast.generics.where_clause {
        contains_where_bound = !where_clause.predicates.is_empty();
    }

    let hook_impl = if !is_generic {
        quote! {

            fn __register_default_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
                use flecs_ecs::core::component_registration::ComponentInfo;
                const IMPLS_DEFAULT: bool =  #name::IMPLS_DEFAULT;
                const IS_ENUM: bool =  <#name as ComponentInfo>::IS_ENUM;

                if IMPLS_DEFAULT {
                    flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,#name>as flecs_ecs::core::component_registration::FlecsDefaultType> ::Type, >(type_hooks);
                } else if !IS_ENUM {
                    flecs_ecs::core::lifecycle_traits::register_ctor_panic_lifecycle_actions::<#name>(
                        type_hooks,
                    );
                }
            }

            fn __register_clone_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
                use flecs_ecs::core::component_registration::ComponentInfo;
                const IMPLS_CLONE: bool = #name::IMPLS_CLONE;

                if IMPLS_CLONE {
                    flecs_ecs::core::lifecycle_traits::register_copy_lifecycle_action:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_CLONE,#name>as flecs_ecs::core::component_registration::FlecsCloneType> ::Type, >(type_hooks);
                } else {
                    flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<#name>(
                        type_hooks,
                    );
                }
            }

            fn __register_compare_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
                use flecs_ecs::core::component_registration::ComponentInfo;
                const IMPLS_PARTIAL_ORD: bool = #name::IMPLS_PARTIAL_ORD;

                if IMPLS_PARTIAL_ORD {
                    flecs_ecs::core::lifecycle_traits::register_partial_ord_lifecycle_action::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_PARTIAL_ORD,#name>as flecs_ecs::core::component_registration::FlecsPartialOrdType> ::Type, >(
                        type_hooks,
                    );
                } else {
                    flecs_ecs::core::lifecycle_traits::register_partial_ord_panic_lifecycle_action::<#name>(
                        type_hooks,
                    );
                }
            }

            fn __register_equals_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
                use flecs_ecs::core::component_registration::ComponentInfo;
                const IMPLS_PARTIAL_EQ: bool = #name::IMPLS_PARTIAL_EQ;

                if IMPLS_PARTIAL_EQ {
                    flecs_ecs::core::lifecycle_traits::register_partial_eq_lifecycle_action::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_PARTIAL_EQ,#name>as flecs_ecs::core::component_registration::FlecsPartialEqType> ::Type, >(
                        type_hooks,
                    );
                } else {
                    flecs_ecs::core::lifecycle_traits::register_partial_eq_panic_lifecycle_action::<#name>(
                        type_hooks,
                    );
                }
            }
        }
    } else if contains_lifetime_bound && !contains_any_generic_type {
        quote! {

            fn __register_default_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
                use flecs_ecs::core::component_registration::ComponentInfo;
                const IMPLS_DEFAULT: bool =  #name::<'_>::IMPLS_DEFAULT;
                const IS_ENUM: bool =  <#name::<'_> as ComponentInfo>::IS_ENUM;

                if IMPLS_DEFAULT {
                    flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,#name #type_generics>as flecs_ecs::core::component_registration::FlecsDefaultType> ::Type, >(type_hooks);
                } else if !IS_ENUM {
                    flecs_ecs::core::lifecycle_traits::register_ctor_panic_lifecycle_actions::<#name #type_generics>(
                        type_hooks,
                    );
                }
            }

            fn __register_clone_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
                use flecs_ecs::core::component_registration::ComponentInfo;
                const IMPLS_CLONE: bool = #name::<'_>::IMPLS_CLONE;

                if IMPLS_CLONE {
                    flecs_ecs::core::lifecycle_traits::register_copy_lifecycle_action:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_CLONE,#name #type_generics>as flecs_ecs::core::component_registration::FlecsCloneType> ::Type, >(type_hooks);
                } else {
                    flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<#name>(
                        type_hooks,
                    );
                }
            }

            fn __register_compare_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
                use flecs_ecs::core::component_registration::ComponentInfo;
                const IMPLS_PARTIAL_ORD: bool = #name::<'_>::IMPLS_PARTIAL_ORD;

                if IMPLS_PARTIAL_ORD {
                    flecs_ecs::core::lifecycle_traits::register_partial_ord_lifecycle_action::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_PARTIAL_ORD,#name #type_generics>as flecs_ecs::core::component_registration::FlecsPartialOrdType> ::Type, >(
                        type_hooks,
                    );
                } else {
                    flecs_ecs::core::lifecycle_traits::register_partial_ord_panic_lifecycle_action::<#name #type_generics>(
                        type_hooks,
                    );
                }
            }

            fn __register_equals_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
                use flecs_ecs::core::component_registration::ComponentInfo;
                const IMPLS_PARTIAL_EQ: bool = #name::<'_>::IMPLS_PARTIAL_EQ;

                if IMPLS_PARTIAL_EQ {
                    flecs_ecs::core::lifecycle_traits::register_partial_eq_lifecycle_action::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_PARTIAL_EQ,#name #type_generics>as flecs_ecs::core::component_registration::FlecsPartialEqType> ::Type, >(
                        type_hooks,
                    );
                } else {
                    flecs_ecs::core::lifecycle_traits::register_partial_eq_panic_lifecycle_action::<#name #type_generics>(
                        type_hooks,
                    );
                }
            }
        }
    } else {
        quote! {
            fn __register_clone_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
                    flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<#name #type_generics>(
                        type_hooks,
                    );
            }
        }
    };

    let component_info_impl = quote! {
        #[inline(always)]
        fn index() -> u32 {
            static INDEX: ::core::sync::atomic::AtomicU32 = ::core::sync::atomic::AtomicU32::new(u32::MAX);
            Self::get_or_init_index(&INDEX)
        }

        fn __register_lifecycle_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t)  {
            flecs_ecs::core::lifecycle_traits::register_lifecycle_actions::<#name #type_generics>(type_hooks);
        }

        #hook_impl
    };

    let is_generic_const = if !contains_any_generic_type {
        quote! {
            const IS_GENERIC: bool = false;
        }
    } else {
        quote! {
            const IS_GENERIC: bool = true;
        }
    };
    let clone_default = if !is_generic
        || (contains_lifetime_bound && !contains_any_generic_type)
        || (contains_any_generic_type && contains_all_default_bound && contains_all_clone_bound)
    {
        quote! {
            const IMPLS_CLONE: bool = {
                use flecs_ecs::core::utility::traits::DoesNotImpl;
                flecs_ecs::core::utility::types::ImplementsClone::<#name #type_generics>::IMPLS
            };
            const IMPLS_DEFAULT: bool = {
                use flecs_ecs::core::utility::traits::DoesNotImpl;
                flecs_ecs::core::utility::types::ImplementsDefault::<#name #type_generics>::IMPLS
            };
        }
    } else if contains_any_generic_type && contains_all_default_bound {
        quote! {
            const IMPLS_CLONE: bool = false;
            const IMPLS_DEFAULT: bool = {
                use flecs_ecs::core::utility::traits::DoesNotImpl;
                flecs_ecs::core::utility::types::ImplementsDefault::<#name #type_generics>::IMPLS
            };
        }
    } else if contains_any_generic_type && contains_all_clone_bound {
        quote! {
            const IMPLS_CLONE: bool = {
                use flecs_ecs::core::utility::traits::DoesNotImpl;
                flecs_ecs::core::utility::types::ImplementsClone::<#name #type_generics>::IMPLS
            };
            const IMPLS_DEFAULT: bool = false;
        }
    } else {
        quote! {
            const IMPLS_CLONE: bool = false;
            const IMPLS_DEFAULT: bool = false;
        }
    };

    let partial_ord_bound = if !is_generic
        || (contains_lifetime_bound && !contains_any_generic_type)
        || (contains_any_generic_type && contains_all_partial_ord_bound)
    {
        quote! {
            const IMPLS_PARTIAL_ORD: bool = {
                use flecs_ecs::core::utility::traits::DoesNotImpl;
                flecs_ecs::core::utility::types::ImplementsPartialOrd::<#name #type_generics>::IMPLS
            };
        }
    } else {
        quote! {
            const IMPLS_PARTIAL_ORD: bool = false;
        }
    };

    let partial_eq_bound = if !is_generic
        || (contains_lifetime_bound && !contains_any_generic_type)
        || (contains_any_generic_type && contains_all_partial_eq_bound)
    {
        quote! {
            const IMPLS_PARTIAL_EQ: bool = {
                use flecs_ecs::core::utility::traits::DoesNotImpl;
                flecs_ecs::core::utility::types::ImplementsPartialEq::<#name #type_generics>::IMPLS
            };
        }
    } else {
        quote! {
            const IMPLS_PARTIAL_EQ: bool = false;
        }
    };

    // Common trait implementation for ComponentType and ComponentId
    let common_traits = {
        quote! {
            impl #impl_generics  flecs_ecs::core::ComponentType<flecs_ecs::core::Struct> for #name #type_generics #where_clause{}

            impl #impl_generics flecs_ecs::core::component_registration::ComponentInfo for #name #type_generics #where_clause {
                #is_generic_const
                const IS_ENUM: bool = false;

                #is_tag
                #clone_default
                #partial_ord_bound
                #partial_eq_bound
                const IS_REF: bool = false;
                const IS_MUT: bool = false;
            }
        }
    };
    let where_clause_quote = if contains_where_bound {
        quote! { #where_clause Self: 'static }
    } else {
        quote! {
            where
            Self: 'static
        }
    };

    let component_id = quote! {
        impl #impl_generics flecs_ecs::core::component_registration::ComponentId for #name #type_generics
        #where_clause_quote
        {
            type UnderlyingType = #name #type_generics;
            type UnderlyingEnumType = flecs_ecs::core::component_registration::NoneEnum;
            type UnderlyingTypeOfEnum = flecs_ecs::core::component_registration::NoneEnum;
            #component_info_impl
        }
    };

    // Specific trait implementation based on the presence of fields
    let is_empty_component_trait = if has_fields {
        quote! { impl #impl_generics flecs_ecs::core::DataComponent for #name #type_generics #where_clause{} }
    } else {
        quote! {
            impl #impl_generics flecs_ecs::core::TagComponent for #name #type_generics #where_clause {}

            impl #impl_generics flecs_ecs::core::IntoEntity for #name #type_generics #where_clause {
                const IS_TYPED_PAIR: bool = false;
                const IS_TYPED: bool = true;
                const IF_ID_IS_DEFAULT: bool = true;
                const IS_TYPED_SECOND: bool = false;
                const IF_ID_IS_DEFAULT_SECOND: bool = false;
                const IS_ENUM: bool = false;
                const IS_TYPE_TAG: bool = true;
                const IS_TYPED_REF: bool = false;
                const IS_TYPED_MUT_REF: bool = false;
                type CastType = Self;
                type IsTyped = flecs_ecs::core::FlecsIsTyped;
                type IsTag = <Self as flecs_ecs::core::ComponentInfo>::TagType;
                fn into_entity<'a>(
                    self,
                    world: impl flecs_ecs::core::WorldProvider<'a>,
                ) -> flecs_ecs::core::Entity {
                    world.world().component_id::<Self>()
                }
            }

            impl #impl_generics flecs_ecs::core::IntoEntity for &'static #name #type_generics #where_clause {
                const IS_TYPED_PAIR: bool = false;
                const IS_TYPED: bool = true;
                const IF_ID_IS_DEFAULT: bool = true;
                const IS_TYPED_SECOND: bool = false;
                const IF_ID_IS_DEFAULT_SECOND: bool = false;
                const IS_ENUM: bool = false;
                const IS_TYPE_TAG: bool = true;
                const IS_TYPED_REF: bool = true;
                const IS_TYPED_MUT_REF: bool = false;
                type CastType = Self;
                type IsTyped = flecs_ecs::core::FlecsIsTyped;
                type IsTag = <Self as flecs_ecs::core::ComponentInfo>::TagType;
                fn into_entity<'a>(
                    self,
                    world: impl flecs_ecs::core::WorldProvider<'a>,
                ) -> flecs_ecs::core::Entity {
                    world.world().component_id::<Self>()
                }
            }

            impl #impl_generics flecs_ecs::core::IntoEntity for &'static mut #name #type_generics #where_clause {
                const IS_TYPED_PAIR: bool = false;
                const IS_TYPED: bool = true;
                const IF_ID_IS_DEFAULT: bool = true;
                const IS_TYPED_SECOND: bool = false;
                const IF_ID_IS_DEFAULT_SECOND: bool = false;
                const IS_ENUM: bool = false;
                const IS_TYPE_TAG: bool = true;
                const IS_TYPED_REF: bool = false;
                const IS_TYPED_MUT_REF: bool = true;
                type CastType = Self;
                type IsTyped = flecs_ecs::core::FlecsIsTyped;
                type IsTag = <Self as flecs_ecs::core::ComponentInfo>::TagType;
                fn into_entity<'a>(
                    self,
                    world: impl flecs_ecs::core::WorldProvider<'a>,
                ) -> flecs_ecs::core::Entity {
                    world.world().component_id::<Self>()
                }
            }

        }
    };

    let internal_on_component_registration = {
        let pre_name = if let Some(name) = flecs_name {
            quote! {
                #[inline(always)]
                fn internal_pre_registration_name() -> Option<&'static str> { Some(#name) }
            }
        } else {
            quote! {}
        };
        quote! {
        impl #impl_generics flecs_ecs::core::component_registration::InternalComponentHooks for #name #type_generics {
            #pre_name
            #[inline(always)]
            fn internal_on_component_registration(world: flecs_ecs::core::WorldRef, component_id: flecs_ecs::core::Entity) {
                let _component = flecs_ecs::core::Component::<Self>::new_w_id(world, component_id);

                #flecs_traits_calls

                <Self as flecs_ecs::core::component_registration::OnComponentRegistration>::on_component_registration(world, component_id);
            }
        }
        }
    };

    let on_component_registration = if has_on_registration {
        quote! {}
    } else {
        quote! {
            impl #impl_generics flecs_ecs::core::component_registration::OnComponentRegistration for #name #type_generics {
                #[inline(always)]
                fn on_component_registration(_world: flecs_ecs::core::WorldRef, _component_id: flecs_ecs::core::Entity) {}
            }
        }
    };

    // Combine common and specific trait implementations
    quote! {
        #is_empty_component_trait
        #common_traits
        #component_id
        #on_component_registration
        #internal_on_component_registration
    }
}

pub(crate) fn generate_variant_constructor(
    variant: &syn::Variant,
    name: &syn::Ident,
) -> proc_macro2::TokenStream {
    let variant_ident = &variant.ident;
    match &variant.fields {
        syn::Fields::Unit => quote! { #name::#variant_ident },
        syn::Fields::Unnamed(fields) => {
            let defaults: Vec<_> = fields
                .unnamed
                .iter()
                .map(|_| quote! { Default::default() })
                .collect();
            quote! { #name::#variant_ident(#(#defaults),*) }
        }
        syn::Fields::Named(fields) => {
            let field_names: Vec<_> = fields
                .named
                .iter()
                .map(|f| f.ident.as_ref().unwrap())
                .collect();
            let defaults: Vec<_> = field_names
                .iter()
                .map(|_| quote! { Default::default() })
                .collect();
            quote! { #name::#variant_ident { #(#field_names: #defaults),* } }
        }
    }
}

pub(crate) fn generate_variant_match_arm(
    variant: &syn::Variant,
    name: &syn::Ident,
    use_index: bool,
    index: usize,
) -> proc_macro2::TokenStream {
    let variant_ident = &variant.ident;

    let inner = if use_index {
        quote! { #index }
    } else {
        quote! {
            unsafe {
                let slice = concat!(stringify!(#variant_ident), "\0").as_bytes();
                core::ffi::CStr::from_bytes_with_nul_unchecked(slice)
            }
        }
    };

    match &variant.fields {
        syn::Fields::Unnamed(fields) => {
            let field_names: Vec<_> = fields.unnamed.iter().map(|_| quote!(_)).collect();
            quote! {
                #name::#variant_ident(#(#field_names),*) => {
                    #inner
                }
            }
        }
        syn::Fields::Named(fields) => {
            let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
            quote! {
                #name::#variant_ident { #(#field_names),* } => {
                    #inner
                }
            }
        }
        syn::Fields::Unit => {
            quote! {
                #name::#variant_ident => {
                    #inner
                }
            }
        }
    }
}

pub(crate) fn impl_cached_component_data_enum(
    ast: &mut syn::DeriveInput,
    has_on_registration: bool,
    underlying_enum_type: TokenStream,
    flecs_traits_calls: &TokenStream,
    flecs_name: &Option<LitStr>,
) -> proc_macro2::TokenStream {
    let is_generic = !ast.generics.params.is_empty();

    ast.generics.make_where_clause();

    let name = &ast.ident;

    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    // Ensure it's an enum and get the variants
    let variants = match &ast.data {
        syn::Data::Enum(data_enum) => &data_enum.variants,
        _ => panic!("#[derive(VariantName)] is only defined for enums"),
    };

    let variant_constructors: Vec<_> = variants
        .iter()
        .map(|variant| generate_variant_constructor(variant, name))
        .collect();

    let variant_name_arms: Vec<_> = variants
        .iter()
        .map(|variant| generate_variant_match_arm(variant, name, false, 0))
        .collect();

    let variant_index_arms: Vec<_> = variants
        .iter()
        .enumerate()
        .map(|(index, variant)| generate_variant_match_arm(variant, name, true, index))
        .collect();

    let has_variants = !variants.is_empty();
    let size_variants = variants.len() as u32;
    let not_empty_trait_or_error = if has_variants {
        quote! { impl #impl_generics flecs_ecs::core::DataComponent for #name #type_generics #where_clause {} }
    } else {
        quote! { compile_error!("Enum components should have at least one variant!"); }
    };

    let cached_enum_data_impl = quote! {
        const SIZE_ENUM_FIELDS: u32 = #size_variants;
        type VariantIterator = std::vec::IntoIter<#name #impl_generics>;

        fn name_cstr(&self) -> &core::ffi::CStr {
            match self {
                #(#variant_name_arms),*
            }
        }

        fn enum_index(&self) -> usize {
            //const _: () = assert!(core::mem::size_of::<#name>()  == 4, "Enum size is not 4 bytes. For Flecs enum behaviour, the enum size must be 4 bytes");
            match self {
                #(#variant_index_arms),*
            }
        }

        fn __enum_data_mut() -> *mut u64 {
            static mut ENUM_FIELD_ENTITY_ID: [u64; #size_variants as usize] = [0; #size_variants as usize];
            unsafe { ENUM_FIELD_ENTITY_ID.as_mut_ptr() }
        }

        fn iter() -> Self::VariantIterator {
            vec![#(#variant_constructors),*].into_iter()
        }
    };

    let cached_enum_data = quote! {
        impl #impl_generics flecs_ecs::core::EnumComponentInfo for #name #type_generics #where_clause{
            #cached_enum_data_impl
        }

    };

    let component_info_impl = quote! {
            #[inline(always)]
            fn index() -> u32 {
                static INDEX: ::core::sync::atomic::AtomicU32 = ::core::sync::atomic::AtomicU32::new(u32::MAX);
                Self::get_or_init_index(&INDEX)
            }

            fn __register_lifecycle_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t)  {
                flecs_ecs::core::lifecycle_traits::register_lifecycle_actions::<#name>(type_hooks);
            }

            fn __register_default_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
                use flecs_ecs::core::component_registration::ComponentInfo;
                const IMPLS_DEFAULT: bool =  #name::IMPLS_DEFAULT;
                const IS_ENUM: bool =  <#name as ComponentInfo>::IS_ENUM;

                if IMPLS_DEFAULT {
                    flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,#name>as flecs_ecs::core::component_registration::FlecsDefaultType> ::Type, >(type_hooks);
                } else if !IS_ENUM {
                    flecs_ecs::core::lifecycle_traits::register_ctor_panic_lifecycle_actions::<#name>(
                        type_hooks,
                    );
                }
            }

            fn __register_clone_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
                use flecs_ecs::core::component_registration::ComponentInfo;
                const IMPLS_CLONE: bool = #name::IMPLS_CLONE;

                if IMPLS_CLONE {
                    flecs_ecs::core::lifecycle_traits::register_copy_lifecycle_action:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_CLONE,#name>as flecs_ecs::core::component_registration::FlecsCloneType> ::Type, >(type_hooks);
                } else {
                    flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<#name>(
                        type_hooks,
                    );
                }
            }

            fn __register_compare_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
                use flecs_ecs::core::component_registration::ComponentInfo;
                const IMPLS_PARTIAL_ORD: bool = #name::IMPLS_PARTIAL_ORD;

                if IMPLS_PARTIAL_ORD {
                    flecs_ecs::core::lifecycle_traits::register_partial_ord_lifecycle_action::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_PARTIAL_ORD,#name>as flecs_ecs::core::component_registration::FlecsPartialOrdType> ::Type, >(
                        type_hooks,
                    );
                } else {
                    flecs_ecs::core::lifecycle_traits::register_partial_ord_panic_lifecycle_action::<#name>(
                        type_hooks,
                    );
                }
            }

            fn __register_equals_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
                use flecs_ecs::core::component_registration::ComponentInfo;
                const IMPLS_PARTIAL_EQ: bool = #name::IMPLS_PARTIAL_EQ;

                if IMPLS_PARTIAL_EQ {
                    flecs_ecs::core::lifecycle_traits::register_partial_eq_lifecycle_action::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_PARTIAL_EQ,#name>as flecs_ecs::core::component_registration::FlecsPartialEqType> ::Type, >(
                        type_hooks,
                    );
                } else {
                    flecs_ecs::core::lifecycle_traits::register_partial_eq_panic_lifecycle_action::<#name>(
                        type_hooks,
                    );
                }
            }
    };

    let component_id = if !is_generic {
        quote! {
            impl #impl_generics flecs_ecs::core::component_registration::ComponentId for #name #type_generics #where_clause{
                type UnderlyingType = #name;
                type UnderlyingEnumType = #name;
                type UnderlyingTypeOfEnum = #underlying_enum_type;

                #component_info_impl
            }
        }
    } else {
        quote! {}
    };

    let component_info = if !is_generic {
        quote! {
            impl #impl_generics flecs_ecs::core::component_registration::ComponentInfo for #name #type_generics #where_clause {
                const IS_GENERIC: bool = false;
                const IS_ENUM: bool = true;
                const IS_TAG: bool = false;
                type TagType =
                flecs_ecs::core::component_registration::FlecsNotATag;
                const IMPLS_CLONE: bool = {
                    use flecs_ecs::core::utility::traits::DoesNotImpl;
                    flecs_ecs::core::utility::types::ImplementsClone::<#name #type_generics>::IMPLS
                };
                const IMPLS_DEFAULT: bool = {
                    use flecs_ecs::core::utility::traits::DoesNotImpl;
                    flecs_ecs::core::utility::types::ImplementsDefault::<#name #type_generics>::IMPLS
                };
                const IMPLS_PARTIAL_ORD: bool = {
                    use flecs_ecs::core::utility::traits::DoesNotImpl;
                    flecs_ecs::core::utility::types::ImplementsPartialOrd::<#name #type_generics>::IMPLS
                };
                const IMPLS_PARTIAL_EQ: bool = {
                    use flecs_ecs::core::utility::traits::DoesNotImpl;
                    flecs_ecs::core::utility::types::ImplementsPartialEq::<#name #type_generics>::IMPLS
                };
                const IS_REF: bool = false;
                const IS_MUT: bool = false;
            }
        }
    } else {
        quote! {
            impl #impl_generics flecs_ecs::core::component_registration::ComponentInfo for #name #type_generics #where_clause {
                const IS_GENERIC: bool = true;
                const IS_ENUM: bool = true;
                const IS_TAG: bool = false;
                type TagType =
                flecs_ecs::core::component_registration::FlecsNotATag;
                const IMPLS_CLONE: bool = {
                    use flecs_ecs::core::utility::traits::DoesNotImpl;
                    flecs_ecs::core::utility::types::ImplementsClone::<#name #type_generics>::IMPLS
                };
                const IMPLS_DEFAULT: bool = {
                    use flecs_ecs::core::utility::traits::DoesNotImpl;
                    flecs_ecs::core::utility::types::ImplementsDefault::<#name #type_generics>::IMPLS
                };
                const IMPLS_PARTIAL_ORD: bool = {
                    use flecs_ecs::core::utility::traits::DoesNotImpl;
                    flecs_ecs::core::utility::types::ImplementsPartialOrd::<#name #type_generics>::IMPLS
                };
                const IMPLS_PARTIAL_EQ: bool = {
                    use flecs_ecs::core::utility::traits::DoesNotImpl;
                    flecs_ecs::core::utility::types::ImplementsPartialEq::<#name #type_generics>::IMPLS
                };
                const IS_REF: bool = false;
                const IS_MUT: bool = false;
            }
        }
    };

    let internal_on_component_registration = {
        let pre_name = if let Some(name) = flecs_name {
            quote! {
                #[inline(always)]
                fn internal_pre_registration_name() -> Option<&'static str> { Some(#name) }
            }
        } else {
            quote! {}
        };
        quote! {
            impl #impl_generics flecs_ecs::core::component_registration::InternalComponentHooks for #name #type_generics {
                #pre_name
                #[inline(always)]
                fn internal_on_component_registration(world: flecs_ecs::core::WorldRef, component_id: flecs_ecs::core::Entity) {
                    let _component = flecs_ecs::core::Component::<Self>::new_w_id(world, component_id);

                    #flecs_traits_calls

                    <Self as flecs_ecs::core::component_registration::OnComponentRegistration>::on_component_registration(world, component_id);
                }
            }
        }
    };

    let on_component_registration = if has_on_registration {
        quote! {}
    } else {
        quote! {
            impl #impl_generics flecs_ecs::core::component_registration::OnComponentRegistration for #name #type_generics {
                #[inline(always)]
                fn on_component_registration(_world: flecs_ecs::core::WorldRef, _component_id: flecs_ecs::core::Entity) {}
            }
        }
    };

    let into_entity = quote! {
        impl #impl_generics flecs_ecs::core::IntoEntity for #name #type_generics #where_clause {
            const IS_TYPED_PAIR: bool = false;
            const IS_TYPED: bool = true;
            const IF_ID_IS_DEFAULT: bool = <Self as flecs_ecs::core::ComponentInfo>::IMPLS_DEFAULT;
            const IS_TYPED_SECOND: bool = true;
            const IF_ID_IS_DEFAULT_SECOND: bool = false;
            const IS_ENUM: bool = true;
            const IS_TYPE_TAG: bool = true;
            const IS_TYPED_REF: bool = false;
            const IS_TYPED_MUT_REF: bool = false;
            type CastType = Self;
            type IsTyped = flecs_ecs::core::FlecsIsTyped;
            type IsTag = <Self as flecs_ecs::core::ComponentInfo>::TagType;

            fn into_entity<'a>(self, world: impl flecs_ecs::core::WorldProvider<'a>) -> flecs_ecs::core::Entity {
                let world = world.world();
                *<Self as flecs_ecs::core::EnumComponentInfo>::id_variant(&self, world)
            }
        }
    };

    quote! {
        impl #impl_generics flecs_ecs::core::ComponentType<flecs_ecs::core::Enum> for #name #type_generics #where_clause {}
        impl #impl_generics flecs_ecs::core::ComponentType<flecs_ecs::core::Struct> for #name #type_generics #where_clause {}

        #component_info

        #component_id

        #not_empty_trait_or_error

        #cached_enum_data

        #internal_on_component_registration

        #on_component_registration

        #into_entity
    }
}

pub(crate) fn check_repr_c(input: &syn::DeriveInput) -> (bool, TokenStream) {
    let mut token_stream = TokenStream::new();

    for attr in &input.attrs {
        if attr.path().is_ident("repr") {
            let result = attr.parse_args_with(|input: ParseStream| {
                let mut found_repr_c = false;
                while !input.is_empty() {
                    let path = input.call(syn::Path::parse_mod_style)?;

                    if path.is_ident("C") {
                        found_repr_c = true;

                        // get the underlying ident type as tokenstream
                        token_stream = quote! { i32 };
                        break;
                    } else if path.is_ident("i8")
                        || path.is_ident("u8")
                        || path.is_ident("i16")
                        || path.is_ident("u16")
                        || path.is_ident("i32")
                        || path.is_ident("u32")
                        || path.is_ident("i64")
                        || path.is_ident("u64")
                    {
                        found_repr_c = true;

                        // get the underlying ident type as tokenstream
                        let ident = path.get_ident().cloned().unwrap();
                        token_stream = quote! { #ident };
                        break;
                    }
                }
                Ok(found_repr_c)
            });

            if let Ok(found_repr_c) = result
                && found_repr_c
            {
                return (true, token_stream); // Return true immediately if `#[repr(C)]` is found
            }
        }
    }

    (
        false,
        quote! { flecs_ecs::core::component_registration::NoneEnum },
    ) // Return false if no `#[repr(C)]` and variants is found
}

/// Expansion function for the `Component` derive macro.
///
/// This function handles the complete expansion logic for deriving the Component trait,
/// including determining component type (struct/enum), checking for tag/data components,
/// and generating appropriate trait implementations.
///
/// # Arguments
///
/// * `input` - A `DeriveInput` containing the parsed struct or enum to derive Component for
///
/// # Returns
///
/// A `TokenStream` containing all generated trait implementations
pub(crate) fn expand_component_derive(mut input: syn::DeriveInput) -> proc_macro2::TokenStream {
    use alloc::vec::Vec;

    // Collect #[flecs(...)] trait requests and options (e.g., meta) to apply on registration
    let (flecs_traits_calls, has_flecs_meta, has_on_registration, flecs_name) =
        collect_flecs_traits_calls(&input);

    let has_repr_c = check_repr_c(&input);

    let mut generated_impls: Vec<proc_macro2::TokenStream> = Vec::new();

    match input.data.clone() {
        syn::Data::Struct(data_struct) => {
            let has_fields = match data_struct.fields {
                syn::Fields::Named(ref fields) => !fields.named.is_empty(),
                syn::Fields::Unnamed(ref fields) => !fields.unnamed.is_empty(),
                syn::Fields::Unit => false,
            };
            let is_tag = generate_tag_trait(has_fields);
            generated_impls.push(impl_cached_component_data_struct(
                &mut input,
                has_fields,
                &is_tag,
                has_on_registration,
                &flecs_traits_calls,
                &flecs_name,
            ));
        }
        syn::Data::Enum(_) => {
            let is_tag = generate_tag_trait(!has_repr_c.0);
            if !has_repr_c.0 {
                generated_impls.push(impl_cached_component_data_struct(
                    &mut input,
                    true,
                    &is_tag,
                    has_on_registration,
                    &flecs_traits_calls,
                    &flecs_name,
                ));
            } else {
                generated_impls.push(impl_cached_component_data_enum(
                    &mut input,
                    has_on_registration,
                    has_repr_c.1,
                    &flecs_traits_calls,
                    &flecs_name,
                ));
            }
        }
        _ => {
            return quote! { compile_error!("The type is neither a struct nor an enum!"); };
        }
    };

    input.generics.make_where_clause();

    let meta_impl = impl_meta(&input, has_repr_c.0, input.ident.clone(), has_flecs_meta);

    quote! {
        #( #generated_impls )*
        #meta_impl
    }
}
