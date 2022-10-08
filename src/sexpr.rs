use pest::Span;

#[derive(PartialEq, Debug, Clone)]
/// Represents a decorated S-expression value with extended type-assigned atoms.
pub enum Sexpr<D> {
    Atom(Atom<D>, D),
    List(Vec<Sexpr<D>>, D),
}

#[derive(PartialEq, Debug, Clone)]
/// Represents the atoms of the S-expression, decorated as well.
pub enum Atom<D> {
    Symbol(String, D),
    // Additional types for commodity
    String(String, D),
    Integer(i64, D),
    Float(f64, D),
    Boolean(bool, D),
    Quoted(Box<Sexpr<D>>, D), // this is a quoted sexpr value, like 'abc or '(1 "bla" #t)
    QuasiQuoted(Box<Sexpr<D>>, D), // this is a quasiquoted sexpr value, like `abc or `(1 "bla" #t)
    Unquoted(Box<Sexpr<D>>, D), // this is an unquoted sexpr value, like ,abc or ,(1 "bla" #t)
}

impl<'a, D: 'a> Sexpr<D> {
    pub fn get_decorator(&'a self) -> &'a D {
        match self {
            Sexpr::Atom(_, span) | Sexpr::List(_, span) => span,
        }
    }
}

impl<'a, D: 'a> Atom<D> {
    pub fn get_decorator(&'a self) -> &'a D {
        match self {
            Atom::Symbol(_, p)
            | Atom::String(_, p)
            | Atom::Integer(_, p)
            | Atom::Float(_, p)
            | Atom::Boolean(_, p)
            | Atom::Quoted(_, p)
            | Atom::QuasiQuoted(_, p)
            | Atom::Unquoted(_, p) => p,
        }
    }
}

impl<D> std::fmt::Display for Sexpr<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sexpr::Atom(atom, _) => {
                write!(f, "{}", atom)
            }
            Sexpr::List(list, _) => {
                write!(f, "(")?;
                for (i, item) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, ")")
            }
        }
    }
}

impl<D> std::fmt::Display for Atom<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Atom::Symbol(s, _) => write!(f, "{}", s),
            Atom::String(s, _) => write!(f, "\"{}\"", s),
            Atom::Integer(i, _) => write!(f, "{}", i),
            Atom::Float(fl, _) => write!(f, "{}", fl),
            Atom::Boolean(b, _) => {
                if *b {
                    write!(f, "#t")
                } else {
                    write!(f, "#f")
                }
            }
            Atom::Quoted(q, _) => write!(f, "'{}", q),
            Atom::QuasiQuoted(q, _) => write!(f, "`{}", q),
            Atom::Unquoted(q, _) => write!(f, ",{}", q),
        }
    }
}

impl<T> Sexpr<T> {
    /// Produces an untagged (unit decorated) S-expression value from the given value.
    pub fn untag(self) -> Sexpr<()> {
        match self {
            Sexpr::Atom(a, _) => Sexpr::Atom(a.untag(), ()),
            Sexpr::List(l, _) => Sexpr::List(l.into_iter().map(|x| x.untag()).collect(), ()),
        }
    }
}

impl<T> Atom<T> {
    /// Produces an untagged (unit decorated) Atom value from the given value.
    pub fn untag(self) -> Atom<()> {
        match self {
            Atom::Symbol(s, _) => Atom::Symbol(s, ()),
            Atom::String(s, _) => Atom::String(s, ()),
            Atom::Integer(i, _) => Atom::Integer(i, ()),
            Atom::Float(f, _) => Atom::Float(f, ()),
            Atom::Boolean(b, _) => Atom::Boolean(b, ()),
            Atom::Quoted(s, _) => Atom::Quoted(Box::new(s.untag()), ()),
            Atom::QuasiQuoted(s, _) => Atom::QuasiQuoted(Box::new(s.untag()), ()),
            Atom::Unquoted(s, _) => Atom::Unquoted(Box::new(s.untag()), ()),
        }
    }
}

/// Information on the positioning a token.
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct TokInfo {
    pub string: String,
    pub start: (usize, usize),
    pub end: (usize, usize),
}

impl<'a> From<Span<'a>> for TokInfo {
    fn from(s: Span<'a>) -> Self {
        TokInfo {
            string: s.as_str().to_string(),
            start: s.start_pos().line_col(),
            end: s.end_pos().line_col(),
        }
    }
}

impl std::fmt::Display for TokInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f, // NOTE: "token" is deliberately lowercase
            "token starting at line {}, column {} and ending at line {}, column {}:",
            self.start.0,
            self.start.1,
            self.end.0,
            self.end.1
        )?;
        write!(f, "{}", self.string)?;
        Ok(())
    }
}
