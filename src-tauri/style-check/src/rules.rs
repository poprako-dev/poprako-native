use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Block, ExprIf, ExprUnsafe, File, ItemMod, PatIdent, Signature, UseTree, Visibility};

use crate::order;

#[derive(Default)]
struct Rules {
    errors: Vec<String>,
}

impl<'ast> Visit<'ast> for Rules {
    fn visit_visibility(&mut self, visibility: &'ast Visibility) {
        if matches!(visibility, Visibility::Restricted(_)) {
            self.errors.push(format!(
                "line {}: scoped visibility is forbidden",
                visibility.span().start().line
            ));
        }

        visit::visit_visibility(self, visibility);
    }

    fn visit_expr_if(&mut self, expression: &'ast ExprIf) {
        if expression.else_branch.is_some() {
            self.errors.push(format!(
                "line {}: use a guard or match instead of else",
                expression.span().start().line
            ));
        }

        visit::visit_expr_if(self, expression);
    }

    fn visit_expr_unsafe(&mut self, expression: &'ast ExprUnsafe) {
        self.errors.push(format!(
            "line {}: unsafe is forbidden",
            expression.span().start().line
        ));

        visit::visit_expr_unsafe(self, expression);
    }

    fn visit_signature(&mut self, signature: &'ast Signature) {
        if signature.unsafety.is_some() {
            self.errors.push(format!(
                "line {}: unsafe function is forbidden",
                signature.span().start().line
            ));
        }

        visit::visit_signature(self, signature);
    }

    fn visit_use_tree(&mut self, tree: &'ast UseTree) {
        if let UseTree::Group(group) = tree
            && group
                .items
                .iter()
                .any(|item| matches!(item, UseTree::Path(_) | UseTree::Group(_)))
        {
            self.errors.push(format!(
                "line {}: import braces may only group leaf names",
                tree.span().start().line
            ));
        }

        visit::visit_use_tree(self, tree);
    }

    fn visit_pat_ident(&mut self, pattern: &'ast PatIdent) {
        if ["tx", "rx", "sender", "receiver", "inlet", "outlet"]
            .contains(&pattern.ident.to_string().as_str())
        {
            self.errors.push(format!(
                "line {}: channel halves use send/recv",
                pattern.span().start().line
            ));
        }

        visit::visit_pat_ident(self, pattern);
    }

    fn visit_block(&mut self, block: &'ast Block) {
        for pair in block.stmts.windows(2) {
            if pair[1].span().start().line <= pair[0].span().end().line + 1 {
                self.errors.push(format!(
                    "line {}: separate statements with a blank line",
                    pair[1].span().start().line
                ));
            }
        }

        visit::visit_block(self, block);
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        if let Some((_, items)) = &module.content {
            order::check(items, &mut self.errors);
        }

        visit::visit_item_mod(self, module);
    }
}

pub fn check(file: &File) -> Vec<String> {
    let mut rules = Rules::default();

    order::check(&file.items, &mut rules.errors);

    rules.visit_file(file);

    rules.errors
}

#[cfg(test)]
mod test;
