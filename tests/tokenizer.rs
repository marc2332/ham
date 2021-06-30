use ham_core::get_tokens;
use ham_core::types::{Token, TokensList};
use ham_core::utils::op_codes;

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
        Token::new(op_codes::FN_DEF, "fn".to_string(), 1),
        Token::new(op_codes::REFERENCE, "x".to_string(), 1),
        Token::new(op_codes::OPEN_PARENT, "(".to_string(), 1),
        Token::new(op_codes::REFERENCE, "b".to_string(), 1),
        Token::new(op_codes::CLOSE_PARENT, ")".to_string(), 1),
        Token::new(op_codes::OPEN_BLOCK, "{".to_string(), 1),
        Token::new(op_codes::VAR_DEF, "let".to_string(), 1),
        Token::new(op_codes::REFERENCE, "c".to_string(), 1),
        Token::new(op_codes::LEFT_ASSIGN, "=".to_string(), 1),
        Token::new(op_codes::REFERENCE, "b".to_string(), 1),
        Token::new(op_codes::RETURN, "return".to_string(), 1),
        Token::new(op_codes::REFERENCE, "c".to_string(), 1),
        Token::new(op_codes::CLOSE_BLOCK, "}".to_string(), 1),
        Token::new(op_codes::REFERENCE, "x".to_string(), 1),
        Token::new(op_codes::OPEN_PARENT, "(".to_string(), 1),
        Token::new(op_codes::REFERENCE, "4".to_string(), 1),
        Token::new(op_codes::CLOSE_PARENT, ")".to_string(), 1),
    ];

    let mut is_ok = true;

    // Terribly ugly
    for (i, token) in created_tokens.iter().enumerate() {
        if token.line != tokens[i].line {
            is_ok = false;
        }
        if token.ast_type != tokens[i].ast_type {
            is_ok = false;
        }
        if token.value != tokens[i].value {
            is_ok = false;
        }
    }

    assert_eq!(true, is_ok);
}
