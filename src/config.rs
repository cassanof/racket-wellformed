use std::{str::FromStr, vec};

use crate::{
    errors::ParsingError,
    sexpr::{Atom, Sexpr},
    sexpr_parser::SexprParser,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    // Hashlang and reader, can be either or both, but not all none.
    pub lang: (Option<String>, Option<String>),
    // List of definitions
    pub defs: Vec<String>,
}

impl Config {
    pub fn is_same_lang(&self, lang: &str) -> bool {
        // yeah, this is a bit ugly, but it's the easiest way to do it
        self.lang.0.as_ref().unwrap_or(&"no hashlang!".to_string()) == lang
            || self.lang.1.as_ref().unwrap_or(&"no reader!".to_string()) == lang
    }

    // gets the lang of this config, preferring hashlang over reader.
    pub fn get_lang(&self) -> &str {
        if let Some(lang) = &self.lang.0 {
            lang
        } else {
            self.lang.1.as_ref().unwrap()
        }
    }
}

impl FromStr for Config {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sexp = SexprParser::parse_str(s)?;
        let mut config = Self {
            lang: (None, None),
            defs: vec![],
        };
        let res: Option<()> = match sexp {
            Sexpr::List(toplevel, _) => match &toplevel[..] {
                [Sexpr::List(lang, _), Sexpr::List(defs, _)] => match (&lang[..], &defs[..]) {
                    (
                        // one day i will make a match macro for sexprs
                        [Sexpr::Atom(Atom::Symbol(lang_s, _), _), Sexpr::Atom(hashlang, _), Sexpr::Atom(hashreader, _)],
                        [Sexpr::Atom(Atom::Symbol(defs_s, _), _), Sexpr::Atom(Atom::Quoted(defs_l, _), _)],
                    ) if lang_s == "lang" && defs_s == "defs" => {
                        if let Atom::String(hashlang, _) = hashlang {
                            config.lang.0 = Some(hashlang.to_string());
                        }
                        if let Atom::String(hashreader, _) = hashreader {
                            config.lang.1 = Some(hashreader.to_string());
                        }
                        if let (None, None) = config.lang {
                            None
                        } else {
                            match &**defs_l {
                                Sexpr::List(defs, _) => {
                                    for def in defs {
                                        if let Sexpr::Atom(Atom::Symbol(sym, _), _) = def {
                                            config.defs.push(sym.to_string());
                                        }
                                    }
                                    Some(())
                                }
                                _ => None,
                            }
                        }
                    }
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        };
        match res {
            Some(_) => Ok(config),
            None => Err(ParsingError::BadWellformedConfig),
        }
    }
}

#[cfg(test)]
mod config_tests {
    use std::{fs, str::FromStr};

    use super::Config;

    #[test]
    pub fn parse_cfg() {
        let file = fs::read_to_string("./testfiles/test1-style.cfg").unwrap();
        let config = Config::from_str(&file).unwrap();
        assert_eq!(
            config,
            Config {
                lang: (
                    Some("htdp/bsl".to_string()),
                    Some("htdp-beginner-reader.ss".to_string())
                ),
                defs: vec!["my-func".to_string()]
            }
        )
    }
}
