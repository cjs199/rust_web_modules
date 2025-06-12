use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use std::ops::DerefMut;
use syn::{ parse_macro_input, ImplItem, ItemImpl, Type,};

use crate::attr_util;

pub fn job(item: TokenStream) -> TokenStream {
    let item_clone = item.clone();
    let mut item_impl = parse_macro_input!(item_clone as ItemImpl);
    let type_ = item_impl.self_ty.deref_mut().clone();
    let clazz;
    if let Type::Path(path) = type_ {
        let last_segment_option = path.path.segments.last();
        if let Some(last_segment ) = last_segment_option{
            clazz = last_segment.ident.to_string();
        }else{
            panic!("获取类解析异常");
        }
    } else {
        panic!("解析异常");
    }
    let mut vec_fn = Vec::new();
    for i in &item_impl.items {
        if let ImplItem::Fn(method) = i {
            let method_name = method.sig.ident.to_string();
            for attr in method.attrs.clone() {
                if let Some(ident) = attr.path().get_ident() {
                    let ident_str = ident.to_string();
                    if ["redis_lock_job", "interval_job"].contains(&ident_str.as_str()) {
                        let attr_str_1 = attr.meta.to_token_stream().to_string();
                        let attr_str = attr_util::get_attr_str(&ident_str,&attr_str_1);
                        let attr_map = attr_util::attr_to_map(&attr_str);
                        let interval_millis_option = attr_map.get("interval_millis").clone();
                        let interval_millis;
                        if let Some(_interval_millis) = interval_millis_option {
                            interval_millis = _interval_millis.clone();
                        }else {
                            panic!("获取时间解析异常 {:?} ." , attr_map);
                        }
                        let job_name_option = attr_map.get("job_name");
                        let job_name;
                        if let Some(_job_name) = job_name_option {
                            job_name = _job_name[1.._job_name.len()-1].to_string();
                        }else{
                            println!(" job_name_option {:?} ",job_name_option);
                            println!(" attr_map {:?} ",attr_map);
                            println!(" attr_str {:?} ",attr_str);
                            println!(" attr_str_1 {:?} ",attr_str_1);
                            panic!("获取任务名解析异常 {:?} ." , attr_map);
                        }
                       
                        vec_fn.push((
                            job_name,
                            interval_millis,
                            method_name.clone(),
                            ident_str.clone()
                        ));
                    }
                }
            }
        }
    }
    // 在这里处理获取到的函数信息，例如打印函数名
    let clazz_ident = format_ident!("{}", clazz);

    let job_fn = vec_fn.iter().map(|(job_name, interval_millis ,method_name ,ident_str)| {
        let exec_ident = format_ident!("{}", method_name);
        let job_lock_name = format!("job_lock_{}", job_name);
        quote! {
                {
                    info!("开始执行定时任务:{}",#job_name);
                    let interval_millis: u64 = #interval_millis.parse::<u64>().expect("解析_interval_millis为数字异常");
                    pro_thread_util::fiber(async move {

                        loop {
                            if String::from("interval_job") == #ident_str {
                                #clazz_ident::#exec_ident().await;
                            } else {
                                let next_id_str = pro_snowflake_util::next_id_str();
                                let acquire_lock = pro_redis_lock_util::acquire_lock_wait_and_expire(#job_lock_name, &next_id_str, 0,
                                    pro_time_util::Millisecond::_3_MINUTE.clone(),).await;
                                if acquire_lock {
                                    let get_current_milliseconds = pro_time_util::get_current_milliseconds();
                                    let job_key = "rust_job_time_map";
                                    let redis_time_option: Option<i64> = pro_redis_util::map_get(job_key, #job_name);
                                    match redis_time_option {
                                        Some(redis_time) => {
                                            if get_current_milliseconds > redis_time {
                                                #clazz_ident::#exec_ident().await;
                                                pro_redis_util::map_put(job_key, #job_name, get_current_milliseconds);
                                            } 
                                        }
                                        None => {
                                            info!("{}没有找到时间,初始化执行",#job_name);
                                            #clazz_ident::#exec_ident().await;
                                            pro_redis_util::map_put(job_key, #job_name, get_current_milliseconds);
                                        }
                                    }
                                    pro_redis_lock_util::release_lock(#job_lock_name, &next_id_str).await;
                                }
                            }
    
                            let mut interval = interval_at(
                                Instant::now() + Duration::from_millis(interval_millis),
                                Duration::from_millis(interval_millis),
                            );
                            interval.tick().await;
                        }

                    });
                };
        }
    });

    let expanded = quote! {
        #item_impl
        impl #clazz_ident {
            pub fn init_job() {
                #(#job_fn)*
            }
        }
    };
    expanded.into()
}
