pub mod expr;
pub mod ident_expr;
pub mod lex;
pub mod namespace;
pub mod struct_parse;
pub type Span = std::ops::Range<usize>;
pub type Spanned<T> = (T, Span);
