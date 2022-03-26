use std::fmt;

// 代码位置信息
#[derive(Clone, Copy, PartialEq)]
pub struct Loc {
    pub line: usize,
    pub column: usize,
    pub index: usize,
}

impl Default for Loc {
    fn default() -> Self {
        Self {
            line: 1,
            column: 1,
            index: 0,
        }
    }
}

impl fmt::Debug for Loc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Loc(line:{},column:{},index:{})",
            self.line, self.column, self.index
        )
    }
}

impl Loc {
    pub fn new(line: usize, column: usize, index: usize) -> Self {
        Self {
            line,
            column,
            index,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    OpenTag(String),           // <Text>
    CloseTag(String),          // </Text>
    SelfCloseTag(String),      // <Text/>
    Attribute(String, String), // 属性
    Text(String),
    Comment(String), // 注释
}

#[derive(Clone, Debug, PartialEq)]
pub enum LexerError {
    END,
    Expect(Loc, String),
    UnexpectedToken(Loc, String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub attributes: Option<Vec<Token>>, // 属性
    pub loc: Loc,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lexer {
    // 解析器
    pub code: String, // wxml源码
    pub loc: Loc,
    pub tokens: Vec<Token>,
    pub index: usize,
}

// 私有方法
impl Lexer {
    fn peek_char(&self) -> Result<char, LexerError> {
        // 读取字符
        self.code[self.loc.index..]
            .chars()
            .next()
            .ok_or(LexerError::END)
    }

    fn peek_chars(&self, index: usize) -> Result<char, LexerError> {
        let chars = self.code[self.loc.index..].chars().collect::<Vec<char>>();
        match chars.get(index) {
            Some(c) => Ok(*c),
            None => Ok(' '),
        }
    }

    fn take_char(&mut self) -> Result<char, LexerError> {
        // 获得字符
        let mut iter = self.code[self.loc.index..].char_indices();
        let (_, cur_char) = iter.next().ok_or(LexerError::END)?;
        let (next_index, _) = iter.next().unwrap_or((cur_char.len_utf8(), ' '));
        self.loc.index += next_index;
        self.loc.column += next_index;
        Ok(cur_char)
    }

    fn reach_the_end(&self) -> bool {
        // 是否到达本行最后
        self.loc.index >= self.code.len()
    }

    fn take_char_while<Func>(&mut self, mut func: Func) -> Result<String, LexerError>
    where
        Func: FnMut(char) -> bool,
    {
        // 循环读取
        let mut s = String::from("");
        while !self.reach_the_end() && func(self.peek_char()?) {
            s.push(self.take_char()?);
        }
        Ok(s)
    }

    fn skip_whitespace(&mut self) -> Result<(), LexerError> {
        // 跳过空格不解析
        return self.take_char_while(|c| c == ' ' || c == 't').and(Ok(()));
    }

    fn read_text(&mut self) -> Result<Token, LexerError> {
        // 读取文本
        let text = self.take_char_while(|c| c != '<')?;
        Ok(Token {
            kind: TokenKind::Text(text),
            attributes: None,
            loc: self.loc,
        })
    }

    fn read_comment(&mut self) -> Result<Token, LexerError> {
        // 读取注释
        let perfix = self.take_char_while(|c| c == '<' || c == '!' || c == '-'); // <--!
        let mut s = String::from("");
        loop {
            let char = self.peek_char()?;
            let next_char = self.peek_chars(1)?;
            let next_next_char = self.peek_chars(2)?;
            if char == '-' && next_char == '-' && next_next_char == '>' {
                // -->
                break;
            }
            s.push(self.take_char()?);
        }
        self.take_char_while(|c| c == '>' || c == '-');

        Ok(Token {
            kind: TokenKind::Comment(s),
            attributes: None,
            loc: self.loc,
        })
    }

    fn read_attributes(&mut self) -> Result<Vec<Token>, LexerError> {
        // 读取属性
        let mut attributes = vec![];

        loop {
            let char = self.peek_char()?;
            let next_char = self.peek_chars(1)?;

            if char == '>' || (char == '/' && next_char == '>') {
                break;
            }

            if char.is_whitespace() {
                self.take_char()?;
            } else {
                let name =
                    self.take_char_while(|c| c != '=' && c != ' ' && c != '>' && c != '/')?;

                let no_value = self.peek_char()? != '=';

                if no_value {
                    // required
                    attributes.push(Token {
                        kind: TokenKind::Attribute(name, "{{true}}".to_string()),
                        attributes: None,
                        loc: self.loc,
                    })
                } else {
                    // name=""
                    assert_eq!(self.take_char()?, '='); // 取出”=“
                    let quote = self.take_char()?;
                    let quote_type = if quote == '\"' { '\"' } else { '\'' };
                    let value = self.take_char_while(|c| c != quote_type)?;
                    attributes.push(Token {
                        kind: TokenKind::Attribute(name, value),
                        loc: self.loc,
                        attributes: None,
                    })
                }
                self.take_char()?;
            }
        }

        Ok(attributes)
    }

    fn read_tag(&mut self) -> Result<Token, LexerError> {
        let loc = self.loc;
        assert_eq!(self.take_char()?, '<');

        let close_start = self.peek_char()? == '/'; // </Text>

        if close_start {
            assert_eq!(self.take_char()?, '/');
        }

        let name = self.take_char_while(|c| c != '/' && c != '>' && c != ' ')?;

        let attributes = self.read_attributes()?;

        let close_end = self.peek_char()? == '/'; // <Text/>

        if close_end {
            assert_eq!(self.take_char()?, '/');
        }

        assert_eq!(self.take_char()?, '>');

        if close_start {
            Ok(Token {
                kind: TokenKind::CloseTag(name),
                attributes: None,
                loc,
            })
        } else if close_end {
            Ok(Token {
                kind: TokenKind::SelfCloseTag(name),
                attributes: Some(attributes),
                loc,
            })
        } else {
            Ok(Token {
                kind: TokenKind::OpenTag(name),
                attributes: Some(attributes),
                loc,
            })
        }
    }
}

impl Lexer {
    pub fn new(code: String) -> Self {
        Self {
            code,
            loc: Loc::default(),
            tokens: vec![],
            index: 0,
        }
    }

    pub fn tokenize(&mut self) -> Result<Token, LexerError> {
        let current = self.peek_char()?;
        match current {
            '\n' => {
                assert_eq!(self.take_char()?, '\n');
                // 换行
                self.loc.line += 1;
                self.loc.column = 0;
                return self.tokenize();
            }
            c if c.is_whitespace() => {
                // 空格
                self.skip_whitespace()?;
                return self.tokenize();
            }
            c if c != '<' => self.read_text(), // 读取文本内容
            _ => {
                let next_char = self.peek_chars(1)?;
                if next_char == '!' {
                    return self.read_comment();
                } else {
                    return self.read_tag();
                }
            }
        }
    }

    pub fn tokenize_all(&mut self) -> Result<(), LexerError> {
        loop {
            match self.tokenize() {
                Ok(token) => self.tokens.push(token),
                Err(LexerError::END) => break,
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }
}
