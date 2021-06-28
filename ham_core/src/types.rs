#[derive(Clone, Debug)]
pub struct Token {
    pub ast_type: usize,
    pub value: String,
    pub line: usize,
}

impl Token {
    pub fn new(ast_type: usize, value: String, line: usize) -> Self {
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
