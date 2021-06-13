#[derive(Clone, Debug)]
pub struct Token {
    pub ast_type: i32,
    pub value: String,
}

pub type LinesList = Vec<Vec<String>>;
pub type TokensList = Vec<Token>;
