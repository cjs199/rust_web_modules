use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::ops::DerefMut;
use syn::FnArg::Typed;
use syn::{parse_macro_input, ImplItem, ItemImpl, LitStr, PathSegment, Type};

use crate::redis_mq_que_model::RedisMqModel;

// 添加路由
pub fn handle(item: TokenStream) -> TokenStream {
    let item_clone = item.clone();
    let mut item_impl = parse_macro_input!(item_clone as ItemImpl);
    let type_ = item_impl.self_ty.deref_mut().clone();
    let clazz_ident;
    if let Type::Path(path) = type_ {
        let last_segment: &PathSegment = path.path.segments.last().unwrap();
        clazz_ident = last_segment.ident.clone();
    } else {
        panic!("Unexpected type format");
    }

    let mut mq_fn = Vec::new();

    for i in &item_impl.items {
        if let ImplItem::Fn(method) = i {
            let method_ident = method.sig.ident.clone();
            for attr in &method.attrs {
                if let Some(ident) = attr.path().get_ident() {
                    let ident_str = ident.to_string();
                    if String::from("redis_mq_pub").eq(&ident_str)
                        || String::from("redis_mq_que").eq(&ident_str)
                    {
                        let mut arg_ident = format_ident!("String");
                        let first_fn_arg = method.sig.inputs.first().unwrap();
                        if let Typed(arg_path_type) = first_fn_arg {
                            if let Type::Path(path_type) = *arg_path_type.ty.clone() {
                                if let Some(path_segment) = path_type.path.segments.first() {
                                    if let syn::PathArguments::None = path_segment.arguments {
                                        let ident = &path_segment.ident;
                                        arg_ident = ident.clone();
                                    }
                                }
                            }
                        }
                        if String::from("redis_mq_pub").eq(&ident_str) {
                            let value = Some(attr.parse_args::<LitStr>());
                            if let Some(lit_str) = value {
                                let mq_path = lit_str.unwrap().value();
                                mq_fn.push( quote! {
                                    tokio::task::spawn(async {
                                        {
                                            let mut con = pro_redis_util::REDIS_POOL.get().unwrap();
                                            // 订阅频道
                                            let mut pubsub = con.as_pubsub();
                                            pubsub
                                            .subscribe(pro_redis_mq_msg_util::get_msg_pub_key(#mq_path))
                                            .unwrap();
                                            loop {
                                                // 订阅频道
                                                let get_message = pubsub.get_message();
                                                if let Ok(msg) = get_message {
                                                    let get_payload: Result<String, redis::RedisError> = msg.get_payload();
                                                    if let Ok(payload) = get_payload {
                                                        let str_to_object_result: Result<#arg_ident, serde_json::Error> = pro_json_util::str_to_object(payload.as_str());
                                                        if let Ok(str_to_object) = str_to_object_result {
                                                            #clazz_ident::#method_ident(str_to_object).await;
                                                        } else {
                                                            info!("{}消息反序列化异常:{}",#mq_path,payload.as_str());
                                                        }

                                                    }
                                                }
                                            }
                                        }
                                    });
                                });
                            }
                        }
                        if String::from("redis_mq_que").eq(&ident_str) {
                            let redis_mq_model: RedisMqModel = attr.parse_args().unwrap();
                            let base_que_str = redis_mq_model.que.value();
                            let group_str = redis_mq_model.group.value();
                            mq_fn.push( quote! {
                                tokio::task::spawn(async {
                                    {
                                        let que_str = pro_redis_mq_msg_util::get_msg_que_key(#base_que_str);
                                        // 判断队列是否存在
                                        // 创建group
                                        pro_redis_util::streams_xgroup_create_mkstream(&que_str, #group_str).await;
                                        loop {
                                            let message_option = pro_redis_util::streams_xread_group(
                                                &que_str, 
                                                #group_str,
                                                pro_snowflake_util::next_id_str(),
                                            ).await;
                                            if let Some(message) = message_option {
                                                let keys = message.keys;
                                                for item in keys {
                                                    let msg_ids  = item.ids;
                                                    for msg in msg_ids {
                                                        let map = msg.map;
                                                        let payload_value = map.get( "payload").unwrap();
                                                        if let Data(payload) = payload_value {


                                                            let payload = String::from_utf8(payload.clone()).unwrap();
                                                            let str_to_object_result: Result<#arg_ident, serde_json::Error> = pro_json_util::str_to_object(payload.as_str());


                                                            if let Ok(str_to_object) = str_to_object_result {
                                                                #clazz_ident::#method_ident(str_to_object).await;
                                                            } else {
                                                                info!("{}消息反序列化异常",#base_que_str);
                                                            }
                                                        }
                                                        let id = msg.id;
                                                        let _ = pro_redis_util::streams_xack(&que_str, #group_str, id.clone()).await;
                                                        let _ = pro_redis_util::streams_xdel(&que_str, id).await;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                });
                            });
                        }
                    }
                }
            }
        }
    }
    // 在这里处理获取到的函数信息，例如打印函数名
    let expanded = quote! {
        #item_impl
        impl #clazz_ident {
            pub fn init_mq() {
                #(#mq_fn)*
            }
        }
    };
    expanded.into()
}
