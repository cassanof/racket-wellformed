use crate::sexpr::{Sexpr, TokInfo};

pub struct Program {
    pub hashlang: String,
    pub body: Vec<Sexpr<TokInfo>>,
}
