use ham_core::{
    get_tokens,
    types::{
        Token,
        TokensList,
    },
    utils::Ops,
};

/*
 * Make sure a sample code is properly tokenized
 */
#[test]
pub fn tokenizer_works() {
    // Sample code
    const CODE: &str = "fn x(b){ let c = b return c } x(4)";

    // Generated tokens
    let created_tokens: TokensList = get_tokens(CODE.to_string());

    // Expected tokens
    let tokens: TokensList = vec![
        Token::new(Ops::FnDef, "fn".to_string(), 1),
        Token::new(Ops::Reference, "x".to_string(), 1),
        Token::new(Ops::OpenParent, "(".to_string(), 1),
        Token::new(Ops::Reference, "b".to_string(), 1),
        Token::new(Ops::CloseParent, ")".to_string(), 1),
        Token::new(Ops::OpenBlock, "{".to_string(), 1),
        Token::new(Ops::VarDef, "let".to_string(), 1),
        Token::new(Ops::Reference, "c".to_string(), 1),
        Token::new(Ops::LeftAssign, "=".to_string(), 1),
        Token::new(Ops::Reference, "b".to_string(), 1),
        Token::new(Ops::Return, "return".to_string(), 1),
        Token::new(Ops::Reference, "c".to_string(), 1),
        Token::new(Ops::CloseBlock, "}".to_string(), 1),
        Token::new(Ops::Reference, "x".to_string(), 1),
        Token::new(Ops::OpenParent, "(".to_string(), 1),
        Token::new(Ops::Reference, "4".to_string(), 1),
        Token::new(Ops::CloseParent, ")".to_string(), 1),
    ];

    let mut all_tokens_are_ok = true;

    for (i, token) in created_tokens.iter().enumerate() {
        if *token != tokens[i] {
            all_tokens_are_ok = false;
        }
    }

    assert_eq!(true, all_tokens_are_ok);
}
