use crate::lexer::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub token: Token,
    pub children: Option<Vec<Node>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Parser {
    pub lexer: Lexer,
}

impl Parser {
    pub fn new(code: &str) -> Self {
        Parser {
            lexer: Lexer::new(code.to_string()),
        }
    }

    pub fn read_token(&mut self) -> Result<Token, LexerError> {
        // 每调用一次，给与一个token
        if self.lexer.index < self.lexer.tokens.len() {
            let index = self.lexer.index;
            self.lexer.index += 1;
            Ok(self.lexer.tokens[index].clone())
        } else {
            Err(LexerError::END)
        }
    }

    pub fn peek_token(&mut self, index: usize) -> Result<Token, LexerError> {
        // 获取特定的token
        let index_in_tokens = self.lexer.index + index;
        if index_in_tokens < self.lexer.tokens.len() {
            Ok(self.lexer.tokens[index_in_tokens].clone())
        } else {
            Err(LexerError::END)
        }
    }

    pub fn read_child(&mut self) -> Result<Node, LexerError> {
        let current = self.read_token()?;
        let mut children = vec![];

        match &current.kind {
            TokenKind::OpenTag(_) => {
                // <Text>需要遍历其所有子标签
                loop {
                    let next = self.peek_token(0);
                    match next {
                        Ok(n) => {
                            match n.kind {
                                TokenKind::CloseTag(_) => {
                                    // 结束
                                    self.read_child()?;
                                    break;
                                }
                                TokenKind::Comment(_) => {
                                    self.read_child()?;
                                }
                                _ => {
                                    let node = self.read_child()?;
                                    children.push(node);
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }
                return Ok(Node {
                    token: current,
                    children: Some(children),
                });
            }
            TokenKind::CloseTag(_) => {
                // </Text>
                return Ok(Node {
                    token: current,
                    children: None,
                });
            }
            TokenKind::SelfCloseTag(_) => {
                // <Text/>
                return Ok(Node {
                    token: current,
                    children: None,
                });
            }
            TokenKind::Text(_) => {
                return Ok(Node {
                    token: current,
                    children: None,
                })
            }
            TokenKind::Comment(_) => {
                return self.read_child();
            }
            _ => Err(LexerError::END),
        }
    }

    pub fn read_node(&mut self) -> Result<Node, LexerError> {
        // 读取一个标签节点
        return self.read_child();
    }

    pub fn parse_all(&mut self) -> Result<Node, LexerError> {
        self.lexer.tokenize_all()?;
        return self.read_node();
    }
}
