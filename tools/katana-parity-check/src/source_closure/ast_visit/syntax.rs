use syn::{Expr, Pat};

use super::super::lexical_resolver;

pub(super) fn path_resolution_hint(
    path: &syn::Path,
    root: &std::path::Path,
    current: &std::path::Path,
    resolver: Option<&lexical_resolver::LexicalPathResolver>,
) -> (String, Option<String>) {
    let names = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    let Some(first) = names.first() else {
        return ("unresolved:empty path".to_string(), None);
    };
    if resolver.is_some_and(|resolver| resolver.has_alias(first)) {
        return (format!("unresolved:alias `{first}`"), None);
    }
    if names.len() == 1 {
        return ("bare_local_candidate".to_string(), None);
    }
    match resolver.map(|resolver| resolver.resolve(root, current, &names)) {
        Some(lexical_resolver::LexicalResolution::Local(path)) => (
            "local_source_path".to_string(),
            Some(
                root.canonicalize()
                    .ok()
                    .and_then(|root| {
                        path.strip_prefix(root)
                            .ok()
                            .map(std::path::Path::to_path_buf)
                    })
                    .unwrap_or(path)
                    .to_string_lossy()
                    .replace(std::path::MAIN_SEPARATOR, "/"),
            ),
        ),
        Some(lexical_resolver::LexicalResolution::Ambiguous { .. })
        | Some(lexical_resolver::LexicalResolution::AmbiguousAlias { .. }) => {
            ("unresolved:ambiguous local path".to_string(), None)
        }
        Some(lexical_resolver::LexicalResolution::Unresolved { reason }) => {
            (format!("unresolved:{reason}"), None)
        }
        None => ("unresolved:lexical resolver unavailable".to_string(), None),
    }
}

pub(super) fn path_to_syntax(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

pub(super) fn pat_to_syntax(pattern: &Pat) -> String {
    match pattern {
        Pat::Guard(value) => format!(
            "{} if {}",
            pat_to_syntax(&value.pat),
            expr_to_syntax(&value.guard)
        ),
        Pat::Ident(value) => value.ident.to_string(),
        Pat::Wild(_) => "_".to_string(),
        Pat::Lit(value) => match &value.lit {
            syn::Lit::Bool(lit) => lit.value.to_string(),
            syn::Lit::Int(lit) => lit.base10_digits().to_string(),
            _ => "<literal-unresolved>".to_string(),
        },
        Pat::Path(value) => path_to_syntax(&value.path),
        Pat::Tuple(value) => format!(
            "({})",
            value
                .elems
                .iter()
                .map(pat_to_syntax)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Pat::TupleStruct(value) => format!(
            "{}({})",
            path_to_syntax(&value.path),
            value
                .elems
                .iter()
                .map(pat_to_syntax)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Pat::Or(value) => value
            .cases
            .iter()
            .map(pat_to_syntax)
            .collect::<Vec<_>>()
            .join(" | "),
        Pat::Reference(value) => format!("&{}", pat_to_syntax(&value.pat)),
        Pat::Paren(value) => format!("({})", pat_to_syntax(&value.pat)),
        Pat::Struct(value) => format!("{} {{ ... }}", path_to_syntax(&value.path)),
        _ => "<pattern-unresolved>".to_string(),
    }
}

pub(super) fn expr_to_syntax(expr: &Expr) -> String {
    match expr {
        Expr::Path(path) => path_to_syntax(&path.path),
        Expr::Field(field) => format!(
            "{}.{}",
            expr_to_syntax(&field.base),
            match &field.member {
                syn::Member::Named(name) => name.to_string(),
                syn::Member::Unnamed(index) => index.index.to_string(),
            }
        ),
        Expr::Index(index) => format!(
            "{}[{}]",
            expr_to_syntax(&index.expr),
            expr_to_syntax(&index.index)
        ),
        Expr::MethodCall(call) => format!("{}.{}", expr_to_syntax(&call.receiver), call.method),
        Expr::Call(call) => format!("{}()", expr_to_syntax(call.func.as_ref())),
        Expr::Binary(binary) => format!(
            "{} {} {}",
            expr_to_syntax(&binary.left),
            binary_operator_syntax(&binary.op),
            expr_to_syntax(&binary.right)
        ),
        Expr::Paren(paren) => format!("({})", expr_to_syntax(&paren.expr)),
        Expr::Unary(unary) => format!(
            "{}{}",
            unary_operator_syntax(&unary.op),
            expr_to_syntax(&unary.expr)
        ),
        Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Int(value) => value.base10_digits().to_string(),
            syn::Lit::Bool(value) => value.value.to_string(),
            _ => "<literal>".to_string(),
        },
        _ => "<expr>".to_string(),
    }
}

fn binary_operator_syntax(operator: &syn::BinOp) -> &'static str {
    match operator {
        syn::BinOp::Add(_) => "+",
        syn::BinOp::Sub(_) => "-",
        syn::BinOp::Mul(_) => "*",
        syn::BinOp::Div(_) => "/",
        syn::BinOp::Rem(_) => "%",
        syn::BinOp::And(_) => "&&",
        syn::BinOp::Or(_) => "||",
        syn::BinOp::BitXor(_) => "^",
        syn::BinOp::BitAnd(_) => "&",
        syn::BinOp::BitOr(_) => "|",
        syn::BinOp::Shl(_) => "<<",
        syn::BinOp::Shr(_) => ">>",
        syn::BinOp::Eq(_) => "==",
        syn::BinOp::Lt(_) => "<",
        syn::BinOp::Le(_) => "<=",
        syn::BinOp::Ne(_) => "!=",
        syn::BinOp::Ge(_) => ">=",
        syn::BinOp::Gt(_) => ">",
        syn::BinOp::AddAssign(_) => "+=",
        syn::BinOp::SubAssign(_) => "-=",
        syn::BinOp::MulAssign(_) => "*=",
        syn::BinOp::DivAssign(_) => "/=",
        syn::BinOp::RemAssign(_) => "%=",
        syn::BinOp::BitXorAssign(_) => "^=",
        syn::BinOp::BitAndAssign(_) => "&=",
        syn::BinOp::BitOrAssign(_) => "|=",
        syn::BinOp::ShlAssign(_) => "<<=",
        syn::BinOp::ShrAssign(_) => ">>=",
        _ => "<operator>",
    }
}

fn unary_operator_syntax(operator: &syn::UnOp) -> &'static str {
    match operator {
        syn::UnOp::Deref(_) => "*",
        syn::UnOp::Not(_) => "!",
        syn::UnOp::Neg(_) => "-",
        _ => "<operator>",
    }
}
