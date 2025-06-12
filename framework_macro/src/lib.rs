mod control;
mod job_handle;
mod pro_json_ser_der;
mod redis_mq;
mod table_handle;
mod ws;
mod redis;
mod attr_util;
mod dto;
mod utils;
use quote::ToTokens;
use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};
extern crate proc_macro;

/**
 * 加在类上,为类中的字段生成,以类的字段名加FIELD_为名,字段名为数据值,例如
 * pub struct QueryDto<T> {
 *     pub page: i64,
 *     pub limit: i64,
 *     pub entity: T,
 * }
 * 
 * impl<T> QueryDto<T> {
 *     pub const FIELD_PAGE: &str = QueryDto::<String>::FIELD_PAGE;
 *     pub const FIELD_LIMIT: &str = QueryDto::<String>::FIELD_LIMIT;
 *     pub const FIELD_ENTITY: &str = "entity";
 * }
 * 
 */
#[proc_macro_derive(FieldNameConstants)]
pub fn derive_field_name_constants(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let name = format_ident!("{}", name);
    let generics = input.generics;
    let (impl_generics, _ty_generics, _where_clause) = generics.split_for_impl();
    let data = input.data;
    let mut field_arr = Vec::new();
    match data {
        Data::Struct(s) => {
            for f in &s.fields {
                let attr_name = f.ident.to_token_stream();
                let attr_name_str = attr_name.to_string();
                let field_name = format_ident!("FIELD_{}", attr_name_str.clone().to_uppercase());
                let field_val = format!("{}", attr_name.clone());
                let fd = quote! {
                    pub const #field_name: &str = #field_val;
                };
                field_arr.push(fd);
            }
        }
        Data::Enum(_) => {
        }
        Data::Union(_) => {
        }
    };
    let ret_ts = quote! {
        impl #impl_generics #name #impl_generics {
            
            #(#field_arr)*

        }
    };
    let ret_str = ret_ts.to_string();
    let mut ret_token_stream = TokenStream::new();
    ret_token_stream.extend(ret_str.parse::<TokenStream>().unwrap());
    ret_token_stream
}

#[proc_macro_attribute]
pub fn pro_json_ser_der(_: TokenStream, input: TokenStream) -> TokenStream {
    pro_json_ser_der::handle(input)
}

#[proc_macro_attribute]
pub fn id(_: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[proc_macro_attribute]
pub fn control(attr: TokenStream, input: TokenStream) -> TokenStream {
    control::control_handle(attr.to_string().replace("\"", ""), input)
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
    job_handle::job(input)
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
    table_handle::table(attr, input)
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

// 可以写代码,动态生成key锁的,redis锁注解
// #[redis_lock(key = "你好:".to_owned() +  map.get("aaa").unwrap())]
#[proc_macro_attribute]
pub fn redis_lock(attr: TokenStream, input: TokenStream) -> proc_macro::TokenStream {
    // ... 解析参数，生成代码
    redis::redis::lock(attr, input)
}

// 可以写代码,动态生成key锁的,redis锁注解
// #[redis_get(key = "你好:".to_owned() +  map.get("aaa").unwrap())]
#[proc_macro_attribute]
pub fn redis_get(attr: TokenStream, input: TokenStream) -> proc_macro::TokenStream {
    // ... 解析参数，生成代码
    redis::redis::get(attr, input)
}

// 可以写代码,动态生成key锁的,redis锁注解
// #[redis_evict(key = "你好:".to_owned() +  map.get("aaa").unwrap())]
#[proc_macro_attribute]
pub fn redis_evict(attr: TokenStream, input: TokenStream) -> proc_macro::TokenStream {
    // ... 解析参数，生成代码
    redis::redis::evict(attr, input)
}

#[proc_macro_attribute]
pub fn ws(_: TokenStream, input: TokenStream) -> TokenStream {
    ws::ws_handle(input)
}

#[proc_macro_attribute]
pub fn add_ws_fun(_: TokenStream, input: TokenStream) -> TokenStream {
    ws::add_ws_fun(input)
}

#[proc_macro_attribute]
pub fn ws_fun(_: TokenStream, item: TokenStream) -> TokenStream {
    item
}
