use std::collections::HashSet;

use crate::{
    config::Config,
    prog::Program,
    sexpr::{Atom, Sexpr, TokInfo},
};

impl Config {
    // TODO: proper errors
    pub fn check_wellformedness(&self, prog: &Program) -> Result<(), Vec<WellformedError>> {
        fn get_def(sexpr: &Sexpr<TokInfo>) -> Option<String> {
            match sexpr {
                Sexpr::List(l, _) => match &l[..] {
                    [Sexpr::Atom(Atom::Symbol(define, _), _), Sexpr::List(l, _), ..]
                        if define == "define" =>
                    {
                        match &l[..] {
                            [Sexpr::Atom(Atom::Symbol(sym, _), _), ..] => Some(sym.to_string()),
                            _ => None,
                        }
                    }
                    _ => None,
                },
                _ => None,
            }
        }

        let mut maybe_errs = Vec::new();

        let defs = prog
            .body
            .iter()
            .flat_map(get_def)
            .collect::<HashSet<String>>();

        for exp_def in self.defs.iter() {
            if !defs.contains(exp_def) {
                maybe_errs.push(WellformedError::MissingDef(exp_def.to_string()));
            }
        }

        if self.is_same_lang(&prog.hashlang) {
            maybe_errs.push(WellformedError::WrongHashlang(
                self.get_lang().to_string(),
                prog.hashlang.to_string(),
            ));
        }

        if maybe_errs.is_empty() {
            Ok(())
        } else {
            Err(maybe_errs)
        }
    }
}

#[derive(Debug)]
pub enum WellformedError {
    WrongHashlang(String, String), // expected, found
    MissingDef(String),
}

impl std::error::Error for WellformedError {}

impl std::fmt::Display for WellformedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WellformedError::WrongHashlang(exp, found) => {
                write!(
                    f,
                    "Wrong language selected: expected {}, found {}",
                    exp, found
                )
            }
            WellformedError::MissingDef(def) => write!(f, "Missing expected definition: {}", def),
        }
    }
}
