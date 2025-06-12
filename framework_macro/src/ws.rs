use lazy_static::lazy_static;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Mutex;
use syn::{parse_macro_input, ImplItem, ItemImpl, LitStr, PathSegment, Type};

lazy_static! {
    static ref WS_FUN_ROUTE_MAP: Mutex<HashMap<String, (String, String)>> =
        Mutex::new(HashMap::new());
}

pub fn ws_handle(item: TokenStream) -> TokenStream {
    let item_clone = item.clone();
    let mut item_impl = parse_macro_input!(item_clone as ItemImpl);
    let type_ = item_impl.self_ty.deref_mut().clone();
    let clazz;
    let clazz_ident;
    if let Type::Path(path) = type_ {
        let last_segment: &PathSegment = path.path.segments.last().unwrap();
        clazz_ident = last_segment.ident.clone();
        clazz = last_segment.ident.to_string();
    } else {
        panic!("Unexpected type format");
    }
    for i in &item_impl.items {
        if let ImplItem::Fn(method) = i {
            let method_name = method.sig.ident.to_string();
            for attr in &method.attrs {
                if let Some(ident) = attr.path().get_ident() {
                    let ident_str = ident.to_string();
                    if String::from("ws_fun").eq(&ident_str) {
                        let value = Some(attr.parse_args::<LitStr>());
                        if let Some(lit_str) = value {
                            let get_path = lit_str.unwrap().value();
                            WS_FUN_ROUTE_MAP
                                .lock()
                                .unwrap()
                                .insert(get_path.to_string(), (clazz.clone(), method_name.clone()));
                        }
                    }
                } else {
                }
            }
        }
    }
    // 在这里处理获取到的函数信息，例如打印函数名
    let expanded = quote! {
        #item_impl
        impl #clazz_ident {
            pub fn init_ws() {
            }
        }
    };
    expanded.into()
}

pub fn add_ws_fun(_: TokenStream) -> TokenStream {
    let ret = r#"
        pub async fn add_ws_fun(netty_user_info: NettyUserInfo, req_type: ReqType){
            tokio::spawn(async move {
                let func = req_type.func.as_str();
                match func {
                    match_str
                    "ping" => PingService::ping(netty_user_info,req_type).await,
                    _ => println!("无法处理的消息: {}", func),
                }
            });
        }    
    "#;

    let mut select = Vec::new();

    for (k, (clazz, method_name)) in WS_FUN_ROUTE_MAP.lock().unwrap().iter() {
        let clazz = format_ident!("{}", clazz);
        let method_name = format_ident!("{}", method_name);
        let select_fun = format!(r#""{}" => {}::{}(netty_user_info,req_type).await,"#, k, clazz, method_name);
        select.push(select_fun);
    }

    let select_str = select.join("");
    let ret = ret.replace("match_str", &select_str);
    let mut ret_token_stream = TokenStream::new();
    ret_token_stream.extend(ret.to_string().parse::<TokenStream>().unwrap());
    ret_token_stream
}
