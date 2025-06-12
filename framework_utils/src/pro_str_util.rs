use std::collections::HashMap;

use crate::pro_collection_util;

#[macro_export]
macro_rules! str_join {
    // 定义名为 str_join 的宏，接受任意数量的字符串表达式作为参数
    ($($str:expr),*) => {
        {
            // 创建一个新的可变字符串用于存储拼接结果
            let mut result = String::new();
            // 遍历每个传入的字符串表达式，将其转换为字符串并追加到结果中
            $(result.push_str(String::from($str).as_str());)*
            // 返回拼接后的字符串
            result
        }
    };
}

#[macro_export]
macro_rules! str_join_separator {
    // 定义名为 str_join_separator 的宏，接受一个分隔符和任意数量的字符串表达式作为参数
    ($separator:expr, $($str:expr),*) => {
        {
            // 创建一个新的可变字符串用于存储拼接结果
            let mut result = String::new();
            // 设置一个标志用于判断是否需要添加分隔符
            let mut add_separator = true;
            // 遍历每个传入的字符串表达式
            $(
                // 如果不是第一个字符串，则添加分隔符
                if!add_separator {
                    result.push_str(String::from($separator).as_str());
                }
                // 将当前字符串添加到结果中
                result.push_str(String::from($str).as_str());
                // 将标志设置为 false，表示下一次需要添加分隔符（如果有下一个字符串的话）
                add_separator = false;
            )*
            // 返回拼接后的字符串
            result
        }
    };
}

// 定义一个函数，接受两个可以转换为字符串的参数，将它们拼接在一起并返回结果
pub fn append(src: impl Into<String>, append: impl Into<String>) -> String {
    // 将第一个参数转换为字符串
    let src_: String = src.into();
    // 将第二个参数转换为字符串
    let append_: String = append.into();
    // 使用加法运算符将两个字符串拼接在一起并返回
    src_ + &append_
}

// 将驼峰命名法的字符串转换为蛇壳命名法
//
// # 参数
// - `camel_name`：要转换的驼峰命名法字符串，可以是任何实现了 `Into<String>` 的类型
//
// # 返回值
// - 返回转换后的蛇壳命名法字符串
pub fn camel_to_snake(camel_name: impl Into<String>) -> String {
    let camel_name = camel_name.into();
    // 预先分配空间
    let mut snake_name = String::with_capacity(camel_name.len() + 5);
    for (i, c) in camel_name.chars().enumerate() {
        if i > 0 && c.is_uppercase() {
            snake_name.push('_');
        }
        snake_name.push(c.to_ascii_lowercase());
    }
    snake_name
}

// 将蛇壳命名法的字符串转换为驼峰命名法
//
// # 参数
// - `snake_name`：要转换的蛇壳命名法字符串，可以是任何实现了 `Into<String>` 的类型
//
// # 返回值
// - 返回转换后的驼峰命名法字符串
pub fn snake_to_camel(snake_name: impl Into<String>) -> String {
    let snake_name = snake_name.into();
    let mut parts = snake_name.split('_');
    // 预先分配空间
    let mut camel_name = String::with_capacity(snake_name.len());
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

// 将蛇壳命名法的字符串转换为大驼峰命名法（PascalCase）
//
// # 参数
// - `snake_name`：要转换的蛇壳命名法字符串
//
// # 返回值
// - 返回转换后的大驼峰命名法字符串
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

pub fn is_not_empty(s: impl Into<String>) -> bool {
    let s = s.into();
    s.trim().len() > 0
}

pub fn is_blank(s: impl Into<String>) -> bool {
    let s: String = s.into();
    s.trim().is_empty()
}

pub fn format(template: impl Into<String>, map: &HashMap<String, String>) -> String {
    // 将模板转换为字符串并存储在 template_str 中
    let mut template_str = template.into();
    // 如果模板字符串为空，返回一个空字符串
    if template_str.is_empty() {
        return String::new();
    }
    // 如果哈希表为空，直接返回模板字符串
    if map.is_empty() {
        return template_str;
    }

    // 遍历哈希表中的键值对
    for (key, value) in map.iter() {
        // 使用键来构造占位符，并用对应的 value 替换模板中的占位符
        template_str = template_str.replace(&format!("{{{}}}", key), value);
    }
    // 返回格式化后的字符串
    template_str
}

/// 例如 ["1","2"], |a| format!("'{}'", a), "_".to_string() -> '1'_'2'
/// 格式化集合中的元素，然后使用指定的分隔符连接它们。
///
/// 该方法兼容包含 `String` 或 `&str` 类型元素的切片。
///
/// # 参数
/// * `vec`: 一个切片，包含需要格式化和连接的元素。
///          元素类型 `T` 必须实现 `AsRef<str>`。
/// * `field_fn`: 一个闭包，接收 `&T` 类型的元素引用，并返回其格式化后的 `String` 表示。
/// * `sep`: 用于连接格式化字符串的分隔符。
///
/// # 返回
/// 一个包含所有格式化并连接后的元素的 `String`。
pub fn format_and_join<T, F>(vec: &[T], field_fn: F, sep: impl Into<String>) -> String
where
    T: AsRef<str>, // 确保集合中的元素可以被视为 &str
    F: Fn(&T) -> String,
{
    // 假设 pro_collection_util::collect_field_values 已经处理了 T 到 String 的转换
    let collect_field_values = pro_collection_util::collect_field_values(vec, field_fn);
    let sep_str = sep.into();
    collect_field_values.join(&sep_str)
}
