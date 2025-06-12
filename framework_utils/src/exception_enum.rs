use serde::{Deserialize, Serialize};

// 登录信息数据传输对象结构体
#[derive(Serialize, Deserialize, Debug)]
pub struct ProException {
    pub code: i32,
    pub message: &'static str,
}

impl ProException {

    pub const 登录已失效_请重新登录: ProException = ProException {
        code: 12,
        message: "登录已失效,请重新登录!",
    };
    pub const 请登录: ProException = ProException {
        code: 14,
        message: "请登录",
    };

    pub const 签名已过期: ProException = ProException {
        code: 39,
        message: "签名已过期!",
    };

    pub const 无权操作: ProException = ProException {
        code: 15,
        message: "无权操作!",
    };

    pub const 幂等重复: ProException = ProException {
        code: 73,
        message: "请勿频繁请求",
    };

    pub const 解密失败: ProException = ProException {
        code: 101,
        message: "解密失败",
    };

    pub const 事务提交异常: ProException = ProException {
        code: 115,
        message: "事务提交异常",
    };

    pub const 事务执行异常: ProException = ProException {
        code: 116,
        message: "事务执行异常",
    };

    pub const 事务回滚异常: ProException = ProException {
        code: 117,
        message: "事务回滚异常",
    };

    pub fn get_code(&self) -> i32 {
        self.code
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }
}
