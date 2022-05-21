use chumsky::prelude::*;

use crate::{lex::Token, Spanned};

pub type DocExpr = Option<Spanned<String>>;

pub fn doc_expr() -> impl Parser<Token, DocExpr, Error = Simple<Token>> + Clone {
    filter_map(|span, tok| match tok {
        Token::Documentation(docstr) => Ok(docstr.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    })
    .map_with_span(|body, span| (body, span))
    .labelled("documentation")
    .or_not()
}
