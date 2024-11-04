/// 将驼峰命名法的字符串转换为蛇壳命名法
///
/// # 参数
/// - `camel_name`：要转换的驼峰命名法字符串，可以是任何实现了 `Into<String>` 的类型
///
/// # 返回值
/// - 返回转换后的蛇壳命名法字符串
#[allow(warnings)]
pub fn camel_to_snake(camel_name: impl Into<String>) -> String {
    let camel_name = camel_name.into();
    let mut snake_name = String::new();
    for (i, c) in camel_name.chars().enumerate() {
        if i > 0 && c.is_uppercase() {
            snake_name.push('_');
        }
        snake_name.push(c.to_ascii_lowercase());
    }
    snake_name
}

/// 将蛇壳命名法的字符串转换为驼峰命名法
///
/// # 参数
/// - `snake_name`：要转换的蛇壳命名法字符串，可以是任何实现了 `Into<String>` 的类型
///
/// # 返回值
/// - 返回转换后的驼峰命名法字符串
#[allow(warnings)]
pub fn snake_to_camel(snake_name: impl Into<String>) -> String {
    let snake_name = snake_name.into();
    let mut parts = snake_name.split('_');
    let mut camel_name = String::new();
    if let Some(first_part) = parts.next() {
        camel_name.push_str(first_part);
    }
    for part in parts {
        if !part.is_empty() {
            camel_name.push_str(&part[0..1].to_uppercase());
            camel_name.push_str(&part[1..]);
        }
    }
    camel_name
}

/// 将蛇壳命名法的字符串转换为大驼峰命名法（PascalCase）
///
/// # 参数
/// - `snake_name`：要转换的蛇壳命名法字符串
///
/// # 返回值
/// - 返回转换后的大驼峰命名法字符串
#[allow(warnings)]
pub fn snake_to_big_camel(snake_name: impl Into<String>) -> String {
    let snake_name = snake_name.into();
    let camel_name = snake_to_camel(snake_name);
    if let Some(first_char) = camel_name.chars().next() {
        let mut pascal_name = first_char.to_uppercase().to_string();
        pascal_name.push_str(&camel_name[1..]);
        return pascal_name;
    }
    String::new()
}
