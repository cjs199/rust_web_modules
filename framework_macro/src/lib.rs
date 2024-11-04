mod control;
mod job;
mod macro_str_util;
mod pro_file_util;
mod pro_json_ser_der;
mod redis_mq;
mod redis_mq_que_model;
mod table;
use proc_macro::TokenStream;
use syn::parse_macro_input;
use syn::DeriveInput;
use quote::quote;
extern crate proc_macro;

#[proc_macro_attribute]
pub fn pro_json_ser_der(_: TokenStream, input: TokenStream) -> TokenStream {
    pro_json_ser_der::handle(input)
}

#[proc_macro_attribute]
pub fn id(_: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[proc_macro_attribute]
pub fn control(_: TokenStream, input: TokenStream) -> TokenStream {
    control::control_handle(input)
}

#[proc_macro_attribute]
pub fn get(_: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn post(_: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn delete(_: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn add_route(_: TokenStream, _: TokenStream) -> TokenStream {
    control::add_route_handle()
}

#[proc_macro_attribute]
pub fn job(_: TokenStream, input: TokenStream) -> TokenStream {
    job::job_handle(input)
}

// 使用redis分布式锁的定时任务
#[proc_macro_attribute]
pub fn redis_lock_job(_: TokenStream, item: TokenStream) -> TokenStream {
    item
}

// 间隔执行的定时任务
#[proc_macro_attribute]
pub fn interval_job(_: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn pro_anonymous(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn table(attr: TokenStream, input: TokenStream) -> proc_macro::TokenStream {
    // ... 解析参数，生成代码
    table::table_handle(attr, input)
}

#[proc_macro_derive(SqlEnum)]
pub fn sql_enum(input: TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    
    let expanded = quote! {
        impl sqlx::Type<sqlx::MySql> for #ident {
            fn type_info() -> <sqlx::MySql as sqlx::Database>::TypeInfo {
                <str as sqlx::Type<sqlx::MySql>>::type_info()
            }
        
            fn compatible(ty: &<sqlx::MySql as sqlx::Database>::TypeInfo) -> bool {
                <str as sqlx::Type<sqlx::MySql>>::compatible(ty)
            }
        }
    };
    expanded.into()
}

#[proc_macro_attribute]
pub fn redis_mq(_: TokenStream, input: TokenStream) -> TokenStream {
    redis_mq::handle(input)
}

#[proc_macro_attribute]
pub fn redis_mq_pub(_: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn redis_mq_que(_: TokenStream, item: TokenStream) -> TokenStream {
    item
}
