#[derive(Clone, Debug)]
pub struct Token {
    pub ast_type: usize,
    pub value: String,
}

pub type LinesList = Vec<Vec<String>>;
pub type TokensList = Vec<Token>;

pub type IndexedTokenList = Vec<(usize, Token)>;
