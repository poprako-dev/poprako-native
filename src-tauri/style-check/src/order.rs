use std::collections::HashMap;

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Item, Path, Type};

fn name(item: &Item) -> Option<String> {
    match item {
        Item::Struct(value) => Some(value.ident.to_string()),
        Item::Enum(value) => Some(value.ident.to_string()),
        Item::Union(value) => Some(value.ident.to_string()),
        Item::Trait(value) => Some(value.ident.to_string()),
        Item::Type(value) => Some(value.ident.to_string()),
        Item::Fn(value) => Some(value.sig.ident.to_string()),
        Item::Const(value) => Some(value.ident.to_string()),
        Item::Static(value) => Some(value.ident.to_string()),
        _ => None,
    }
}

fn implemented(item: &Item) -> Option<String> {
    let Item::Impl(value) = item else {
        return None;
    };

    let Type::Path(value) = value.self_ty.as_ref() else {
        return None;
    };

    value.path.get_ident().map(ToString::to_string)
}

struct Dependencies<'a> {
    declared: &'a HashMap<String, usize>,
    position: usize,
    errors: &'a mut Vec<String>,
}

impl<'ast> Visit<'ast> for Dependencies<'_> {
    fn visit_path(&mut self, path: &'ast Path) {
        if let Some(first) = path.segments.first()
            && self
                .declared
                .get(&first.ident.to_string())
                .is_some_and(|index| *index > self.position)
        {
            self.errors.push(format!(
                "line {}: dependency {} must be defined before its user",
                first.ident.span().start().line,
                first.ident
            ));
        }

        visit::visit_path(self, path);
    }
}

pub fn check(items: &[Item], errors: &mut Vec<String>) {
    let declared: HashMap<_, _> = items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| name(item).map(|name| (name, index)))
        .collect();

    for (index, item) in items.iter().enumerate() {
        if let Some(owner) = implemented(item)
            && let Some(position) = declared.get(&owner)
            && items[position + 1..index]
                .iter()
                .any(|item| implemented(item).as_ref() != Some(&owner))
        {
            errors.push(format!(
                "line {}: impl for {owner} must immediately follow its type",
                item.span().start().line
            ));
        }

        Dependencies {
            declared: &declared,
            position: index,
            errors,
        }
        .visit_item(item);
    }
}
