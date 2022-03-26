pub fn first_upper(s: String) -> String {
    // 首字母大写
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub fn camel_case(s: String) -> String {
    // 驼峰生成
    let arr: Vec<&str> = s.split("-").collect();
    let mut out = "".to_string();
    for s in arr {
        out = format!("{}{}", out, first_upper(s.to_string()));
    }
    out
}

pub fn wried_prop(p: String) -> String {
    // 更新wxml特殊属性
    if p.starts_with("bind") {
        let n = p.replace("bind", "");
        return format!(
            "on{}",
            match n.as_str() {
                "tap" => "click".to_string(),
                "confirm" => "keydown".to_string(),
                _ => n,
            }
        );
    } else {
        p
    }
}

pub fn parse_expression(e: String) -> String {
    // 解析表达式，未做
    return e.replace("{{", "").replace("}}", "");
}

pub fn parse_expression_text(e: String) -> String {
    // 解析表达式，未做
    let mut out = "".to_string();
    let mut once = true;
    let text = e.replace("{{", "{").replace("}}", "}").replace("\n", "");
    for s in text.chars() {
        // 替换 \s
        if s == ' ' {
            if once == true {
                once = false;
                out.push(s);
            }
        } else {
            once = true;
            out.push(s)
        }
    }
    return out;
}