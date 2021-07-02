use crate::{
    primitive_values::primitive_base::PrimitiveValueBase,
    utils::Ops,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub ast_type: Ops,
    pub value: String,
    pub line: usize,
}

impl Token {
    pub fn new(ast_type: Ops, value: String, line: usize) -> Self {
        Self {
            ast_type,
            value,
            line,
        }
    }
}

pub type LinesList = Vec<Vec<String>>;
pub type TokensList = Vec<Token>;
pub type IndexedTokenList = Vec<(usize, Token)>;

// Boxed primitive value
pub type BoxedPrimitiveValue = Box<dyn PrimitiveValueBase>;
