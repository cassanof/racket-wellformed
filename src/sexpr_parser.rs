use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

use crate::{
    errors::ParsingError,
    sexpr::TokInfo,
    sexpr::{Atom, Sexpr},
};

#[derive(Parser)]
#[grammar = "./sexpr.pest"]
/// Sexpr parser with extended atom variants and comments.
pub struct SexprParser;

impl SexprParser {
    pub fn parse_all(input: &str) -> Result<Vec<Sexpr<TokInfo>>, ParsingError> {
        let parsed = Self::parse(Rule::program, input)?
            .next()
            .unwrap()
            .into_inner();
        let mut result = Vec::new();
        for p in parsed {
            let rule = p.clone().as_rule();
            if rule != Rule::EOI {
                let x = Self::parse_sexpr(p);
                if matches!(x, Err(ParsingError::NothingToParse)) {
                    continue;
                }
                result.push(x?);
            }
        }
        Ok(result)
    }

    pub fn parse_str(input: &str) -> Result<Sexpr<TokInfo>, ParsingError> {
        let mut parsed = Self::parse(Rule::sexpr, input)?;
        Self::parse_sexpr(parsed.next().unwrap())
    }

    fn parse_atom(atom: Pair<Rule>) -> Result<Atom<TokInfo>, ParsingError> {
        let inner = atom.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::bool => {
                let inner = inner.into_inner().next().unwrap();
                match inner.as_rule() {
                    Rule::bool_false => Ok(Atom::Boolean(false, inner.as_span().into())),
                    Rule::bool_true => Ok(Atom::Boolean(true, inner.as_span().into())),
                    _ => Err(ParsingError::InvalidSyntax(
                        inner.as_span().into(),
                        inner.to_string(),
                        Some(
                            "The given token was matched as a boolean, but wasn't a valid boolean"
                                .to_string(),
                        ),
                    )),
                }
            }
            Rule::symbol => Ok(Atom::Symbol(
                inner.as_str().to_string(),
                inner.as_span().into(),
            )),
            Rule::string => {
                let mut string = inner.as_str().to_string();
                // trimming the quotes
                string.remove(0);
                string.remove(string.len() - 1);
                Ok(Atom::String(string, inner.as_span().into()))
            }
            Rule::float => {
                let mut string = inner.as_str().to_string();
                if string.ends_with('f') {
                    string.remove(string.len() - 1);
                }
                Ok(Atom::Float(
                    string.parse::<f64>().unwrap(),
                    inner.as_span().into(),
                ))
            }
            Rule::quoted => {
                let inner = inner.into_inner().next().unwrap();
                let span = inner.as_span();
                Ok(Atom::Quoted(
                    Box::new(Self::parse_sexpr(inner)?),
                    span.into(),
                ))
            }
            Rule::quasiquoted => {
                let inner = inner.into_inner().next().unwrap();
                let span = inner.as_span();
                Ok(Atom::QuasiQuoted(
                    Box::new(Self::parse_sexpr(inner)?),
                    span.into(),
                ))
            }
            Rule::unquoted => {
                let inner = inner.into_inner().next().unwrap();
                let span = inner.as_span();
                Ok(Atom::Unquoted(
                    Box::new(Self::parse_sexpr(inner)?),
                    span.into(),
                ))
            }
            Rule::integer => {
                let inner = inner.into_inner().next().unwrap();
                match inner.as_rule() {
                    Rule::dec_integer => Ok(Atom::Integer(
                        inner.as_str().parse().unwrap(),
                        inner.as_span().into(),
                    )),
                    Rule::hex_integer => {
                        let mut string = inner.as_str().to_string();
                        // trim "0x"
                        string.remove(0);
                        string.remove(0);
                        Ok(Atom::Integer(
                            i64::from_str_radix(&string, 16).unwrap(),
                            inner.as_span().into(),
                        ))
                    }
                    _ => Err(ParsingError::InvalidSyntax(
                        inner.as_span().into(),
                        inner.to_string(),
                        Some(
                            "The given token was matched as an integer, but wasn't a valid integer"
                                .to_string(),
                        ),
                    )),
                }
            }
            Rule::sexpr_comment => Err(ParsingError::NothingToParse),
            _ => Err(ParsingError::InvalidSyntax(
                inner.as_span().into(),
                inner.to_string(),
                Some("The given token was matched as an atom, but wasn't a valid atom".to_string()),
            )),
        }
    }

    fn pair_is_sexprcomment(pair: Pair<Rule>) -> bool {
        pair.into_inner()
            .next()
            .unwrap()
            .into_inner()
            .next()
            .unwrap()
            .as_rule()
            == Rule::sexpr_comment
    }

    fn parse_sexpr(sexp: Pair<Rule>) -> Result<Sexpr<TokInfo>, ParsingError> {
        let mut inner = sexp.into_inner();
        let sexp = inner.next().unwrap();
        match sexp.as_rule() {
            Rule::atom => {
                let span = sexp.as_span();
                let atom = Self::parse_atom(sexp)?;
                Ok(Sexpr::Atom(atom, span.into()))
            }
            Rule::slist => {
                let span = sexp.as_span();
                let inner = sexp.into_inner();
                let mut list = Vec::new();
                for pair in inner {
                    // this guards for sexpr-comments nested outside of top-level, which shouldn't
                    // throw NothingToParse
                    if !Self::pair_is_sexprcomment(pair.clone()) {
                        list.push(Self::parse_sexpr(pair)?);
                    }
                }
                Ok(Sexpr::List(list, span.into()))
            }
            _ => Err(ParsingError::InvalidSyntax(
                sexp.as_span().into(),
                sexp.to_string(),
                Some(
                    "The given token was matched as a sexpr, but wasn't a valid sexpr".to_string(),
                ),
            )),
        }
    }
}

impl From<pest::error::Error<Rule>> for ParsingError {
    fn from(e: pest::error::Error<Rule>) -> Self {
        ParsingError::Pest(e.to_string())
    }
}

#[cfg(test)]
mod sexpr_parser_tests {
    use super::SexprParser;
    use crate::{
        errors::ParsingError,
        sexpr::{Atom, Sexpr},
    };

    #[test]
    fn test_parse_slist_simple() {
        let parsed = SexprParser::parse_str("(1 2 3)").unwrap().untag();
        assert_eq!(
            parsed,
            Sexpr::List(
                vec![
                    Sexpr::Atom(Atom::Integer(1, ()), ()),
                    Sexpr::Atom(Atom::Integer(2, ()), ()),
                    Sexpr::Atom(Atom::Integer(3, ()), ()),
                ],
                ()
            )
        );
    }

    #[test]
    fn test_comment_parse() {
        let parsed = SexprParser::parse_str(
            "(1 ; a comment
            2 ; another comment (1 2 3)
            3) ; yet another comment",
        )
        .unwrap()
        .untag();
        assert_eq!(
            parsed,
            Sexpr::List(
                vec![
                    Sexpr::Atom(Atom::Integer(1, ()), ()),
                    Sexpr::Atom(Atom::Integer(2, ()), ()),
                    Sexpr::Atom(Atom::Integer(3, ()), ()),
                ],
                ()
            )
        );
    }

    #[test]
    fn test_blockcomment_parse_inline() {
        let parsed = SexprParser::parse_str(
            "(1 #| 2 |# #| 2 3|#
            2 #| aaaaaaaa |#
            3)",
        )
        .unwrap()
        .untag();
        assert_eq!(
            parsed,
            Sexpr::List(
                vec![
                    Sexpr::Atom(Atom::Integer(1, ()), ()),
                    Sexpr::Atom(Atom::Integer(2, ()), ()),
                    Sexpr::Atom(Atom::Integer(3, ()), ()),
                ],
                ()
            )
        );
    }

    #[test]
    fn test_blockcomment_parse_multiline() {
        let parsed = SexprParser::parse_str(
            "(1 #| 2 |# #| 2 3
            2  aaaaaaaa |#
            2
            3)",
        )
        .unwrap()
        .untag();
        assert_eq!(
            parsed,
            Sexpr::List(
                vec![
                    Sexpr::Atom(Atom::Integer(1, ()), ()),
                    Sexpr::Atom(Atom::Integer(2, ()), ()),
                    Sexpr::Atom(Atom::Integer(3, ()), ()),
                ],
                ()
            )
        );
    }

    #[test]
    fn test_sexprcomment_atom() {
        let parsed = SexprParser::parse_str("(#;1 1 2 #;3 3)").unwrap().untag();
        assert_eq!(
            parsed,
            Sexpr::List(
                vec![
                    Sexpr::Atom(Atom::Integer(1, ()), ()),
                    Sexpr::Atom(Atom::Integer(2, ()), ()),
                    Sexpr::Atom(Atom::Integer(3, ()), ()),
                ],
                ()
            )
        );
    }

    #[test]
    fn test_sexprcomment_slist() {
        let parsed = SexprParser::parse_str("(#;(1 5 6) 1 2 #;3 3)")
            .unwrap()
            .untag();
        assert_eq!(
            parsed,
            Sexpr::List(
                vec![
                    Sexpr::Atom(Atom::Integer(1, ()), ()),
                    Sexpr::Atom(Atom::Integer(2, ()), ()),
                    Sexpr::Atom(Atom::Integer(3, ()), ()),
                ],
                ()
            )
        );
    }

    #[test]
    fn test_sexprcomment_slist2() {
        let parsed = SexprParser::parse_str("#;(1 2 3 4)");
        assert!(matches!(parsed, Err(ParsingError::NothingToParse)));
    }

    #[test]
    fn test_parse_slist_simple_int_neg() {
        let parsed = SexprParser::parse_str("(-1 2 -3)").unwrap().untag();
        assert_eq!(
            parsed,
            Sexpr::List(
                vec![
                    Sexpr::Atom(Atom::Integer(-1, ()), ()),
                    Sexpr::Atom(Atom::Integer(2, ()), ()),
                    Sexpr::Atom(Atom::Integer(-3, ()), ()),
                ],
                ()
            )
        );
    }

    #[test]
    fn test_parse_string() {
        let parsed = SexprParser::parse_str("\"hello\"").unwrap().untag();
        assert_eq!(
            parsed,
            Sexpr::Atom(Atom::String("hello".to_string(), ()), ())
        );
    }

    #[test]
    fn test_parse_hex() {
        let parsed = SexprParser::parse_str("0xFA5c2b").unwrap().untag();
        assert_eq!(parsed, Sexpr::Atom(Atom::Integer(0xFA5C2B, ()), ()));
    }

    #[test]
    fn test_parse_string_escapes() {
        let parsed = SexprParser::parse_str(r#""a\nb\x0Fc\u{a}d\u{AbAbAb}e""#)
            .unwrap()
            .untag();
        assert_eq!(
            parsed,
            Sexpr::Atom(
                Atom::String(r#"a\nb\x0Fc\u{a}d\u{AbAbAb}e"#.to_string(), ()),
                ()
            )
        );
    }

    #[test]
    fn test_parse_string_nested() {
        let parsed = SexprParser::parse_str(r#"("a" "b" "c")"#).unwrap().untag();
        assert_eq!(
            parsed,
            Sexpr::List(
                vec![
                    Sexpr::Atom(Atom::String("a".to_string(), ()), ()),
                    Sexpr::Atom(Atom::String("b".to_string(), ()), ()),
                    Sexpr::Atom(Atom::String("c".to_string(), ()), ()),
                ],
                ()
            )
        );
    }

    #[test]
    fn test_parse_simple_atom_int() {
        let parsed = SexprParser::parse_str("1337").unwrap().untag();
        assert_eq!(parsed, Sexpr::Atom(Atom::Integer(1337, ()), ()));
    }

    #[test]
    fn test_parse_simple_atom_symbol() {
        let parsed = SexprParser::parse_str("bleh").unwrap().untag();
        assert_eq!(
            parsed,
            Sexpr::Atom(Atom::Symbol("bleh".to_string(), ()), ())
        );
    }

    #[test]
    fn test_parse_slist_sym() {
        let parsed = SexprParser::parse_str("(+ 1 2)").unwrap().untag();
        assert_eq!(
            parsed,
            Sexpr::List(
                vec![
                    Sexpr::Atom(Atom::Symbol("+".to_string(), ()), ()),
                    Sexpr::Atom(Atom::Integer(1, ()), ()),
                    Sexpr::Atom(Atom::Integer(2, ()), ()),
                ],
                ()
            )
        );
    }

    #[test]
    fn test_parse_simple_atom_bool_t() {
        let parsed = SexprParser::parse_str("#t").unwrap().untag();
        assert_eq!(parsed, Sexpr::Atom(Atom::Boolean(true, ()), ()));
    }

    #[test]
    fn test_parse_simple_atom_bool_true() {
        let parsed = SexprParser::parse_str("#true").unwrap().untag();
        assert_eq!(parsed, Sexpr::Atom(Atom::Boolean(true, ()), ()));
    }

    #[test]
    fn test_parse_simple_atom_bool_f() {
        let parsed = SexprParser::parse_str("#f").unwrap().untag();
        assert_eq!(parsed, Sexpr::Atom(Atom::Boolean(false, ()), ()));
    }

    #[test]
    fn test_parse_simple_atom_bool_false() {
        let parsed = SexprParser::parse_str("#false").unwrap().untag();
        assert_eq!(parsed, Sexpr::Atom(Atom::Boolean(false, ()), ()));
    }

    #[test]
    fn test_parse_simple_float_full() {
        let parsed = SexprParser::parse_str("232.555").unwrap().untag();
        assert_eq!(parsed, Sexpr::Atom(Atom::Float(232.555, ()), ()));
    }

    #[test]
    fn test_parse_simple_float_left_optional() {
        let parsed = SexprParser::parse_str(".555").unwrap().untag();
        assert_eq!(parsed, Sexpr::Atom(Atom::Float(0.555, ()), ()));
    }

    #[test]
    fn test_parse_simple_float_right_optional() {
        let parsed = SexprParser::parse_str("4324.").unwrap().untag();
        assert_eq!(parsed, Sexpr::Atom(Atom::Float(4324.0, ()), ()));
    }

    #[test]
    fn test_parse_simple_float_literal() {
        let parsed = SexprParser::parse_str("20f").unwrap().untag();
        assert_eq!(parsed, Sexpr::Atom(Atom::Float(20.0, ()), ()));
    }

    #[test]
    fn test_sval_symbol() {
        let parsed = SexprParser::parse_str("'b").unwrap().untag();
        assert_eq!(
            parsed,
            Sexpr::Atom(
                Atom::Quoted(
                    Box::new(Sexpr::Atom(Atom::Symbol("b".to_string(), ()), ())),
                    ()
                ),
                ()
            )
        );
    }

    #[test]
    fn test_sval_symbol_doublequote() {
        let parsed = SexprParser::parse_str("''b").unwrap().untag();
        assert_eq!(
            parsed,
            Sexpr::Atom(
                Atom::Quoted(
                    Box::new(Sexpr::Atom(
                        Atom::Quoted(
                            Box::new(Sexpr::Atom(Atom::Symbol("b".to_string(), ()), ()),),
                            ()
                        ),
                        ()
                    )),
                    ()
                ),
                ()
            )
        );
    }

    #[test]
    fn test_sval_symbol_triplequote() {
        let parsed = SexprParser::parse_str("'''b").unwrap().untag();
        assert_eq!(
            parsed,
            Sexpr::Atom(
                Atom::Quoted(
                    Box::new(Sexpr::Atom(
                        Atom::Quoted(
                            Box::new(Sexpr::Atom(
                                Atom::Quoted(
                                    Box::new(Sexpr::Atom(Atom::Symbol("b".to_string(), ()), ()),),
                                    ()
                                ),
                                ()
                            ),),
                            ()
                        ),
                        ()
                    )),
                    ()
                ),
                ()
            )
        );
    }

    #[test]
    fn test_sval_quasisymbol() {
        let parsed = SexprParser::parse_str("`b").unwrap().untag();
        assert_eq!(
            parsed,
            Sexpr::Atom(
                Atom::QuasiQuoted(
                    Box::new(Sexpr::Atom(Atom::Symbol("b".to_string(), ()), ())),
                    ()
                ),
                ()
            )
        );
    }

    #[test]
    fn test_sval_symbol_quasidoublequote() {
        let parsed = SexprParser::parse_str("``b").unwrap().untag();
        assert_eq!(
            parsed,
            Sexpr::Atom(
                Atom::QuasiQuoted(
                    Box::new(Sexpr::Atom(
                        Atom::QuasiQuoted(
                            Box::new(Sexpr::Atom(Atom::Symbol("b".to_string(), ()), ()),),
                            ()
                        ),
                        ()
                    )),
                    ()
                ),
                ()
            )
        );
    }

    #[test]
    fn test_sval_symbol_quasitriplequote() {
        let parsed = SexprParser::parse_str("```b").unwrap().untag();
        assert_eq!(
            parsed,
            Sexpr::Atom(
                Atom::QuasiQuoted(
                    Box::new(Sexpr::Atom(
                        Atom::QuasiQuoted(
                            Box::new(Sexpr::Atom(
                                Atom::QuasiQuoted(
                                    Box::new(Sexpr::Atom(Atom::Symbol("b".to_string(), ()), ()),),
                                    ()
                                ),
                                ()
                            ),),
                            ()
                        ),
                        ()
                    )),
                    ()
                ),
                ()
            )
        );
    }

    #[test]
    fn test_sval_symbol_quasitriplequotemix() {
        let parsed = SexprParser::parse_str("`'`b").unwrap().untag();
        assert_eq!(
            parsed,
            Sexpr::Atom(
                Atom::QuasiQuoted(
                    Box::new(Sexpr::Atom(
                        Atom::Quoted(
                            Box::new(Sexpr::Atom(
                                Atom::QuasiQuoted(
                                    Box::new(Sexpr::Atom(Atom::Symbol("b".to_string(), ()), ()),),
                                    ()
                                ),
                                ()
                            ),),
                            ()
                        ),
                        ()
                    )),
                    ()
                ),
                ()
            )
        );
    }

    #[test]
    fn test_sval_symbol_quasitriplequotemix_unquote() {
        let parsed = SexprParser::parse_str("`,`b").unwrap().untag();
        assert_eq!(
            parsed,
            Sexpr::Atom(
                Atom::QuasiQuoted(
                    Box::new(Sexpr::Atom(
                        Atom::Unquoted(
                            Box::new(Sexpr::Atom(
                                Atom::QuasiQuoted(
                                    Box::new(Sexpr::Atom(Atom::Symbol("b".to_string(), ()), ()),),
                                    ()
                                ),
                                ()
                            ),),
                            ()
                        ),
                        ()
                    )),
                    ()
                ),
                ()
            )
        );
    }

    #[test]
    fn test_sval_list_simple() {
        let parsed = SexprParser::parse_str("'(1 \"bla\" #false bla 'bla)")
            .unwrap()
            .untag();
        assert_eq!(
            parsed,
            Sexpr::Atom(
                Atom::Quoted(
                    Box::new(Sexpr::List(
                        vec![
                            Sexpr::Atom(Atom::Integer(1, ()), ()),
                            Sexpr::Atom(Atom::String("bla".to_string(), ()), ()),
                            Sexpr::Atom(Atom::Boolean(false, ()), ()),
                            Sexpr::Atom(Atom::Symbol("bla".to_string(), ()), ()),
                            Sexpr::Atom(
                                Atom::Quoted(
                                    Box::new(Sexpr::Atom(Atom::Symbol("bla".to_string(), ()), ())),
                                    ()
                                ),
                                ()
                            ),
                        ],
                        ()
                    )),
                    ()
                ),
                ()
            )
        );
    }

    #[test]
    fn test_empty() {
        let parsed = SexprParser::parse_str("()").unwrap().untag();
        assert_eq!(parsed, Sexpr::List(vec![], ()));
    }

    #[test]
    fn test_sval_empty() {
        let parsed = SexprParser::parse_str("'()").unwrap().untag();
        assert_eq!(
            parsed,
            Sexpr::Atom(Atom::Quoted(Box::new(Sexpr::List(vec![], ())), ()), ())
        );
    }

    #[test]
    fn test_symbol_dot() {
        let parsed = SexprParser::parse_str(".").unwrap().untag();
        assert_eq!(parsed, Sexpr::Atom(Atom::Symbol(".".to_string(), ()), ()));
    }

    #[test]
    fn test_sval_list_nested() {
        let parsed = SexprParser::parse_str("'(1 '(bla '(bla bla) 'fufu #true) 'faf)")
            .unwrap()
            .untag();
        assert_eq!(
            parsed,
            Sexpr::Atom(
                Atom::Quoted(
                    Box::new(Sexpr::List(
                        vec![
                            Sexpr::Atom(Atom::Integer(1, ()), ()),
                            Sexpr::Atom(
                                Atom::Quoted(
                                    Box::new(Sexpr::List(
                                        vec![
                                            Sexpr::Atom(Atom::Symbol("bla".to_string(), ()), ()),
                                            Sexpr::Atom(
                                                Atom::Quoted(
                                                    Box::new(Sexpr::List(
                                                        vec![
                                                            Sexpr::Atom(
                                                                Atom::Symbol("bla".to_string(), ()),
                                                                ()
                                                            ),
                                                            Sexpr::Atom(
                                                                Atom::Symbol("bla".to_string(), ()),
                                                                ()
                                                            ),
                                                        ],
                                                        ()
                                                    )),
                                                    ()
                                                ),
                                                ()
                                            ),
                                            Sexpr::Atom(
                                                Atom::Quoted(
                                                    Box::new(Sexpr::Atom(
                                                        Atom::Symbol("fufu".to_string(), ()),
                                                        ()
                                                    )),
                                                    ()
                                                ),
                                                ()
                                            ),
                                            Sexpr::Atom(Atom::Boolean(true, ()), ()),
                                        ],
                                        ()
                                    )),
                                    ()
                                ),
                                ()
                            ),
                            Sexpr::Atom(
                                Atom::Quoted(
                                    Box::new(Sexpr::Atom(Atom::Symbol("faf".to_string(), ()), ())),
                                    ()
                                ),
                                ()
                            ),
                        ],
                        ()
                    )),
                    ()
                ),
                ()
            )
        );
    }

    #[test]
    fn racket_optional_see_how_parses() {
        let parsed = SexprParser::parse_str("(bla #:when #true)")
            .unwrap()
            .untag();
        assert_eq!(
            parsed,
            Sexpr::List(
                vec![
                    Sexpr::Atom(Atom::Symbol("bla".to_string(), ()), ()),
                    Sexpr::Atom(Atom::Symbol("#:when".to_string(), ()), ()),
                    Sexpr::Atom(Atom::Boolean(true, ()), ()),
                ],
                ()
            )
        );
    }

    #[test]
    fn test_balanced_parens1() {
        let parsed = SexprParser::parse_str("[1 (2 {1 2 3}) 3]");
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_unbalanced_parens1() {
        let parsed = SexprParser::parse_str("(bla (bla)");
        assert!(parsed.is_err());
    }

    #[test]
    fn test_unbalanced_parens2() {
        let parsed = SexprParser::parse_str("(bla [bla)]");
        assert!(parsed.is_err());
    }

    #[test]
    fn test_parse_nested() {
        let parsed = SexprParser::parse_str("(1 (2 3 4) (5 (2 3)))")
            .unwrap()
            .untag();
        assert_eq!(
            parsed,
            Sexpr::List(
                vec![
                    Sexpr::Atom(Atom::Integer(1, ()), ()),
                    Sexpr::List(
                        vec![
                            Sexpr::Atom(Atom::Integer(2, ()), ()),
                            Sexpr::Atom(Atom::Integer(3, ()), ()),
                            Sexpr::Atom(Atom::Integer(4, ()), ())
                        ],
                        ()
                    ),
                    Sexpr::List(
                        vec![
                            Sexpr::Atom(Atom::Integer(5, ()), ()),
                            Sexpr::List(
                                vec![
                                    Sexpr::Atom(Atom::Integer(2, ()), ()),
                                    Sexpr::Atom(Atom::Integer(3, ()), ()),
                                ],
                                ()
                            ),
                        ],
                        ()
                    ),
                ],
                ()
            )
        );
    }
}
