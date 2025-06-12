use crate::attr_util;
use framework_utils::pro_time_util;
use proc_macro::TokenStream;
extern crate proc_macro;

pub fn lock(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr_map = attr_util::attr_to_map(attr.to_string());
    let input_str = input.to_string();
    let pre_str_index = input_str.find("{").unwrap();
    let lst_str_index = input_str.rfind("}").unwrap();
    let pre_str = &input_str[0..pre_str_index + 1];
    let lst_str = &input_str[pre_str_index + 1..lst_str_index];
    let key = attr_map.get("key").expect("redis lock key not a empty!");
    let lock_expire_time_option = attr_map.get("lock_expire_time");
    let wait_time_option = attr_map.get("wait_time");
    let mut lock_expire_time = pro_time_util::Millisecond::_3_MINUTE;
    if let Some(time_str) = lock_expire_time_option {
        lock_expire_time = time_str.parse().expect("lock_expire_time not a number!");
    }
    let mut wait_time = pro_time_util::Millisecond::_3_SECOND;
    if let Some(time_str) = wait_time_option {
        wait_time = time_str.parse().expect("wait_time not a number!");
    }
    let ret_str = format!(
        r#"
        {} 
        let ret = pro_redis_lock_util::full_lock_wraper({}, pro_snowflake_util::next_id_str(), {}, {}, async {{
            {} 
        }}).await;
        ret.unwrap()
        }}
    "#,
        pre_str,
        key,
        wait_time,
        lock_expire_time,
        lst_str
    );
    let mut ret_token_stream = TokenStream::new();
    ret_token_stream.extend(ret_str.parse::<TokenStream>().unwrap());
    ret_token_stream
}

pub fn get(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr_map = attr_util::attr_to_map(attr.to_string());
    let input_str = input.to_string();
    let pre_str_index = input_str.find("{").unwrap();
    let lst_str_index = input_str.rfind("}").unwrap();
    let pre_str = &input_str[0..pre_str_index + 1];
    let lst_str = &input_str[pre_str_index + 1..lst_str_index];
    let key = attr_map.get("key").expect("redis get key not a empty!");
    let expire_time_option = attr_map.get("expire_time");
    let mut expire_time = pro_time_util::Millisecond::_3_MINUTE;
    if let Some(time_str) = expire_time_option {
        expire_time = time_str.parse().expect("expire_time not a number!");
    }
    let ret_str = format!(
    r#"
        {} 
        let ret = pro_redis_util::kv_get_cached({}, async {{
            let ret_data = async {{
                {}
            }}.await;
            return Some(ret_data);
        }}, {}).await;

        ret.unwrap()
        }}
    "#        
        ,
        pre_str,
        key,
        lst_str,
        expire_time
    );
    let mut ret_token_stream = TokenStream::new();
    ret_token_stream.extend(ret_str.parse::<TokenStream>().unwrap());
    ret_token_stream
}

pub fn evict(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr_map = attr_util::attr_to_map(attr.to_string());
    let input_str = input.to_string();
    let pre_str_index = input_str.find("{").unwrap();
    let lst_str_index = input_str.rfind("}").unwrap();
    let pre_str = &input_str[0..pre_str_index + 1];
    let lst_str = &input_str[pre_str_index + 1..lst_str_index];
    let key = attr_map.get("key").expect("redis get key not a empty!");
    let ret_str = format!(
    r#"
        {} 
        pro_redis_util::del({});
        {}
        }}
    "#        
        ,
        pre_str,
        key,
        lst_str,
    );
    let mut ret_token_stream = TokenStream::new();
    ret_token_stream.extend(ret_str.parse::<TokenStream>().unwrap());
    ret_token_stream
}
