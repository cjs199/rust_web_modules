use std::collections::HashMap;

// 传入       url = "/sys/v1/sql/list_read_only", desc = "123"
// 返回 map  {"desc": "\"123\"", "url": "\"/sys/v1/sql/list_read_only\""}
pub fn attr_to_map(attr_str: impl Into<String>) -> HashMap<String, String> {
    let mut attr_str:String = attr_str.into();
    attr_str = attr_str.trim().to_string();
    let mut ret = HashMap::new();
    let attr_split = attr_str.split(",");
    for attr in attr_split {
        let kv: Vec<&str> = attr.split("=").collect();
        ret.insert(
            kv[0].to_string().trim().to_owned(),
            kv[1].to_string().trim().to_owned(),
        );
    }
    ret

}

// 去除字符串开头和结尾的双引号
//
// 该函数接受一个可以转换为字符串类型的参数，并移除其中开头和结尾的双引号。
//
// # 参数
//
// * `ret_str`: 可以转换为字符串类型的参数。
//
// # 示例
//
// ```rust
// let result = trim_begin_end_quotes("\"hello, world!\"");
// assert_eq!(result, "hello, world!");
// ```
pub fn trim_begin_end_quotes(ret_str: impl Into<String>) -> String {
    let mut ret_str: String = ret_str.into();
    ret_str = ret_str.trim().to_string();
    let mut begin = 0;
    let mut end = ret_str.len();
    if ret_str.starts_with(r#"""#) {
        begin = 1;
    }
    if ret_str.ends_with(r#"""#) {
        end = end - 1;
    }
    let str = ret_str[begin..end].to_string();
    return str;
}

// 提取注解中的字符串
// 例如 redis_mq_pub("sys_dict_update_pub") -> "sys_dict_update_pub"
pub fn get_attr_str(attr_name: impl Into<String>, attr_str: impl Into<String>) -> String {
    // 将注解名格式化
    let mut attr_name: String = attr_name.into();
    attr_name = attr_name.trim().to_string();
    // 将注解字符串格式化
    let mut attr_str: String = attr_str.into();
    attr_str = attr_str.trim().to_string();
    // 将注解字符串去掉注解
    attr_str = attr_str[attr_name.len()..attr_str.len()].to_string().trim().to_string();
    let ret_str = &attr_str[1..attr_str.len() - 1];
    ret_str.to_string()
}
