pub mod generator;
pub mod lexer;
pub mod parser;
mod utils;

#[cfg(test)]
mod tests {
    use crate::generator;
    use crate::parser;
    #[test]
    fn it_works() {
        let mut parser = parser::Parser::new(
            "<view wx:for=\"{{list}}\">
                    hello {{item}}!
                    <text wx:if=\"{{a}}\">a</text>
                    <text wx:elseif=\"{{b}}\">b</text>
                    <text wx:else />
                </view>",
        );

        let res = parser.parse_all();
        match res {
            Ok(ast) => {
                let mut gen = generator::Generator::new(ast);
                let code = gen.generate_fre();
                assert_eq!(code, "<>{list.map((item)=><View>hello {item}! {a?<Text>a</Text>:b?<Text>b</Text>:true?<Text/>:null}</View>)}</>".to_string());
            }
            Err(err) => {
                panic!("{:?}", err);
            }
        }
    }
}
