use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::ops::DerefMut;
use syn::punctuated::Punctuated;
use syn::{ parse_macro_input, Expr, ImplItem, ItemImpl, Lit,  Meta, PathSegment, Token, Type,};

pub fn job_handle(item: TokenStream) -> TokenStream {
    let item_clone = item.clone();
    let mut item_impl = parse_macro_input!(item_clone as ItemImpl);
    let type_ = item_impl.self_ty.deref_mut().clone();
    let clazz;
    if let Type::Path(path) = type_ {
        let last_segment: &PathSegment = path.path.segments.last().unwrap();
        clazz = last_segment.ident.to_string();
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
                    if String::from("redis_lock_job") == ident_str || String::from("interval_job") == ident_str {
                        match attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) {
                            Ok(nested)=>{
                                let mut job_name = String::from("");
                                let mut interval_millis = String::from("");
                                for meta in &nested {
                                    let mut attr_name = String::from("");
                                    if let Some(data) = meta.path().get_ident() {
                                        attr_name = data.to_string();
                                    }
                                    if let Meta::NameValue(name_value) = meta {
                                        if let Expr::Lit(ref expr_lit) = name_value.value {
                                            match expr_lit.lit {
                                                Lit::Str(ref lit_str) => {
                                                    if String::from("job_name") == attr_name {
                                                        job_name = String::from(lit_str.value());
                                                    } else {
                                                        panic!("解析异常");
                                                    }
                                                }
                                                Lit::Int(ref lit_int) => {
                                                    if String::from("interval_millis") == attr_name {
                                                        interval_millis = String::from(lit_int.to_string());
                                                    } else {
                                                        panic!("解析异常");
                                                    }
                                                }
                                                _ => panic!("解析异常"),
                                            }
                                        }
                                    }
                                }
                                vec_fn.push((
                                    job_name.clone(),
                                    interval_millis,
                                    method_name.clone(),
                                    ident_str
                                ));
                            },
                            Err(_) =>{
                                panic!("解析异常");
                            }
                        }
                    }
                }
            }
        }
    }
    // 在这里处理获取到的函数信息，例如打印函数名
    let clazz_ident = format_ident!("{}", clazz);

    let job_fn = vec_fn.iter().map(|(job_name, lit_int ,method_name ,ident_str)| {
        let exec_ident = format_ident!("{}", method_name);
        let job_lock_name = format!("job_lock_{}", job_name);
        quote! {
                {
                    info!("开始执行定时任务:{}",#job_name);
                    let timer_task = TimerTask::new();
                    let interval_millis: i64 = #lit_int.parse::<i64>().expect("解析_interval_millis为数字异常");
                    timer_task.start(interval_millis, || async {
                        if String::from("interval_job") == #ident_str {
                            #clazz_ident::#exec_ident().await;
                        } else {
                            let next_id_str = IdInstance::next_id().to_string();
                            let acquire_lock = pro_redis_util::acquire_lock_wait_and_expire(#job_lock_name, &next_id_str,
                                0,
                                pro_time_util::Millisecond::_3_MINUTE.clone(),).await;
                            if acquire_lock {
                                let get_current_milliseconds = pro_time_util::get_current_milliseconds();
                                let job_key = "rust_job_time_map";
                                let redis_time_option: Option<i64> = pro_redis_util::map_get(job_key, #job_name).await;
                                match redis_time_option {
                                    Some(redis_time) => {
                                        if get_current_milliseconds > redis_time {
                                            #clazz_ident::#exec_ident().await;
                                            pro_redis_util::map_put(job_key, #job_name, get_current_milliseconds).await;
                                        } 
                                    }
                                    None => {
                                        info!("{}没有找到时间,初始化执行",#job_name);
                                        #clazz_ident::#exec_ident().await;
                                        pro_redis_util::map_put(job_key, #job_name, get_current_milliseconds).await;
                                    }
                                }
                                pro_redis_util::release_lock(#job_lock_name, &next_id_str).await;
                            }
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
