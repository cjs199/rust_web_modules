use std::fs::File;
use std::io::prelude::*;

pub fn write_all(path: impl Into<String>, content: impl Into<String>) {
    // 创建一个新文件或者打开一个已存在的文件进行写入
    let mut file = File::create(path.into()).unwrap();
    // 写入一些内容到文件
    let content:String = content.into();
    file.write_all(content.as_bytes()).unwrap();
}
