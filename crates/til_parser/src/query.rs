use chumsky::{Parser, Stream};
use til_query::{
    common::logical::logicaltype::LogicalType,
    ir::{
        db::Database,
        project::{
            namespace::{self, Namespace},
            Project,
        },
    },
};
use tydi_common::error::{Error, Result};
use tydi_intern::Id;

use crate::{
    expr::Expr,
    ident_expr::IdentExpr,
    lex::lexer,
    namespace::{namespaces_parser, Decl, Statement},
    report::report_errors,
    Spanned,
};

pub fn into_query_storage(src: impl Into<String>) -> Result<Database> {
    let mut _db = Database::default();
    let db = &mut _db;

    let src = src.into();
    let (tokens, mut errs) = lexer().parse_recovery(src.as_str());

    let (ast, parse_errs) = if let Some(tokens) = tokens {
        let len = src.chars().count();
        let (ast, parse_errs) =
            namespaces_parser().parse_recovery(Stream::from_iter(len..len + 1, tokens.into_iter()));
        // TODO? Eval

        println!("{:#?}", ast);

        (ast, parse_errs)
    } else {
        (None, Vec::new())
    };

    report_errors(&src, errs.clone(), parse_errs.clone());

    if errs.len() > 1 || parse_errs.len() > 1 {
        return Err(Error::ParsingError(
            "Errors during parsing, see report.".to_string(),
        ));
    }

    let mut project = Project::new("proj", ".")?;
    if let Some(ast) = ast {
        for (name, parsed_namespace) in ast.into_iter() {
            let mut namespace = Namespace::new(name)?;
        }
    }

    Ok(_db)
}
