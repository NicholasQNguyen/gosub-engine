use crate::css3::node::{Node, NodeType};
use crate::css3::tokenizer::TokenType;
use crate::css3::{Css3, Error};

impl Css3<'_> {
    pub fn parse_combinator(&mut self) -> Result<Node, Error> {
        log::trace!("parse_combinator");
        let t = self.consume_any()?;

        let name = match t.token_type {
            TokenType::Whitespace => " ".to_string(),
            TokenType::Delim('+') => t.to_string(),
            TokenType::Delim('>') => t.to_string(),
            TokenType::Delim('~') => t.to_string(),
            TokenType::Delim('/') => {
                let tn1 = self.tokenizer.lookahead(1);
                let tn2 = self.tokenizer.lookahead(2);
                if tn1.token_type == TokenType::Ident("deep".to_string())
                    && tn2.token_type == TokenType::Delim('/')
                {
                    "/deep/".to_string()
                } else {
                    return Err(Error::new(
                        format!("Unexpected token {:?}", tn1),
                        self.tokenizer.current_location().clone(),
                    ));
                }
            }
            _ => {
                return Err(Error::new(
                    format!("Unexpected token {:?}", t),
                    self.tokenizer.current_location().clone(),
                ));
            }
        };

        self.consume_whitespace_comments();

        Ok(Node::new(NodeType::Combinator { value: name }, t.location))
    }
}
