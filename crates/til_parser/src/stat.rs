use crate::expr::{Expr, Spanned};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Decl {
    TypeDecl(Spanned<String>, Box<Spanned<Expr>>),
    ImplDecl(Spanned<String>, Box<Spanned<Expr>>),
    PortsDecl(Spanned<String>, Box<Spanned<Expr>>),
    StreamletDecl(Spanned<String>, Box<Spanned<Expr>>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Statement {
    Import,
    Decl(Decl),
}
