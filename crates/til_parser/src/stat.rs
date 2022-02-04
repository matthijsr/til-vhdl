use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::{prelude::*, primitive::Just, stream::Stream};
use std::{collections::HashMap, env, fmt, fs, hash::Hash, path::PathBuf};
use til_query::{
    common::{
        logical::logicaltype::stream::{Direction, Synchronicity, Throughput},
        physical::complexity::Complexity,
    },
    ir::physical_properties::InterfaceDirection,
};
use tydi_common::{
    name::{Name, PathName},
    numbers::{NonNegative, Positive, PositiveReal},
};

use crate::{
    expr::Expr,
    lex::{DeclKeyword, Operator, Token, TypeKeyword},
    Span, Spanned,
};

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
