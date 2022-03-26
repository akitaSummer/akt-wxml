use crate::lexer::{LexerError, TokenKind};
use crate::parser::Node;
use crate::utils::*;
use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq)]
pub struct Generator {
    pub ast: Node,
    pub conditions: Vec<String>,
}

pub fn handle_attribute(
    // 处理wxml特殊属性，tap, bind, wx:
    name: String,
    value: String,
    mut code: String,
    directs: &mut VecDeque<(String, String)>,
) -> String {
    let prop = wried_prop(name);
    let expression = parse_expression(value);

    match prop.as_str() {
        "wx:key" => code = format!("{} {}=\"{}\"", code, "key", expression),
        "wx:if" | "wx:elseif" | "wx:else" => {
            directs.push_back((prop, expression));
        }
        "wx:for" => directs.push_front((prop, expression)), // wx:for 最后执行
        _ => code = format!("{} {}=\"{}\"", code, prop, expression),
    }

    code
}

impl Generator {
    pub fn new(ast: Node) -> Self {
        Self {
            ast,
            conditions: vec![],
        }
    }

    pub fn generate_directs(
        &mut self,
        directs: VecDeque<(String, String)>,
        mut code: String,
    ) -> String {
        // 将directs的逻辑转成jsx
        let d = match self.conditions.last() {
            // 查看最后一个逻辑
            Some(d) => d.clone(),
            None => "".to_string(),
        };
        let len = directs.len();
        for direct in directs {
            match direct.0.as_str() {
                "wx:if" => {
                    if d == "" || d == "else" {
                        self.conditions.push("if".to_string());
                    }
                    code = format!("{{{}?{}:", direct.1, code);
                }
                "wx:elseif" => {
                    if d == "if" {
                        self.conditions.push("elseif".to_string());
                    }
                    code = format!("{}?{}:", direct.1, code);
                }
                "wx:else" => {
                    if d == "if" || d == "elseif" {
                        self.conditions.push("else".to_string());
                    }
                    code = format!("{}?{}:null}}", direct.1, code);
                }
                "wx:for" => {
                    code = format!("{{{}.map((item)=>{})}}", direct.1, code);
                    if len > 0 {
                        code = format!("<>{}</>", code)
                    }
                }
                _ => {}
            }
        }
        return code;
    }

    pub fn generate_node(&mut self, node: Node) -> String {
        let token = node.token;
        let mut directs = VecDeque::new(); // 存储wx:if等wx:开头相关逻辑
        let mut code = "".to_string();

        match token.kind {
            TokenKind::OpenTag(name) => {
                let tag = camel_case(name);
                code = format!("{}<{}", code, tag);

                for attr in token.attributes.unwrap() {
                    if let TokenKind::Attribute(name, value) = attr.kind {
                        code = handle_attribute(name, value, code, &mut directs);
                    }
                }
                code += ">";

                for child in node.children.unwrap() {
                    let str = self.generate_node(child);
                    if str == "" {
                        println!("{:#?}", self.conditions)
                    }
                    code = format!("{}{}", code, str);
                }
                code = format!("{}</{}>", code, tag);
            }

            TokenKind::SelfCloseTag(name) => {
                let tag = first_upper(name);
                code = format!("{}<{}", code, tag);
                for attr in token.attributes.unwrap() {
                    if let TokenKind::Attribute(name, value) = attr.kind {
                        code = handle_attribute(name, value, code, &mut directs);
                    }
                }
                code += "/>";
            }

            TokenKind::Text(text) => {
                let expression = parse_expression_text(text);
                code = format!("{}{}", code, expression);
            }

            _ => {}
        }

        let c = self.generate_directs(directs, code); // 逻辑转换
        c
    }

    pub fn generate_fre(&mut self) -> String {
        let root = self.ast.clone();
        return self.generate_node(root);
    }
}
