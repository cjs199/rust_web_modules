use std::fs::File;
use std::io::prelude::*;
use std::fs::OpenOptions; 

#[allow(warnings)]
pub fn write_all(path: impl Into<String>, content: impl Into<String>) {
    // 创建一个新文件或者打开一个已存在的文件进行写入
    let mut file = File::create(path.into()).unwrap();
    // 写入一些内容到文件
    let content: String = content.into();
    file.write_all(content.as_bytes()).unwrap();
}

#[allow(warnings)]
pub fn append_all(path: impl Into<String>, content: impl Into<String>) {
    // 打开一个文件进行追加写入，如果文件不存在则创建
    let mut file = OpenOptions::new()
        .create(true) // 如果文件不存在，则创建它
        .append(true)  // 以追加模式打开
        .open(path.into())
        .unwrap(); // 如果出错则 panic

    // 写入一些内容到文件
    let mut  content: String = content.into();
    content.push('\n'); // 添加换行符
    file.write_all(content.as_bytes()).unwrap(); // 写入内容，如果出错则 panic
}
