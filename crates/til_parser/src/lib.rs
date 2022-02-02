pub mod lex;
pub type Span = std::ops::Range<usize>;

#[cfg(test)]
mod tests {
    use crate::lex::lexer;
    use chumsky::Parser;
    use std::path::Path;

    use super::*;

    fn test_parse(path: impl AsRef<Path>) {
        let src = std::fs::read_to_string(path).unwrap();

        let (tokens, mut errs) = lexer().parse_recovery(src.as_str());

        if let Some(tokens) = tokens {
            println!("Tokens:");
            for token in tokens {
                println!("{:?}", token);
            }
        }

        println!("Errors:");
        for err in errs {
            println!("{}", err);
        }
    }

    #[test]
    fn test_test_til() {
        test_parse("test.til")
    }

    #[test]
    fn test_sample_til() {
        test_parse("sample.til")
    }
}
