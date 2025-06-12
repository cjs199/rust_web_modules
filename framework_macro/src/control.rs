use lazy_static::lazy_static;
use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Mutex;
use syn::{parse_macro_input, ImplItem, ItemImpl, PathSegment, Type};

use crate::attr_util;

lazy_static! {
    static ref GET_ROUTE_MAP: Mutex<HashMap<String, (String, String, String)>> =
        Mutex::new(HashMap::new());
    static ref POST_ROUTE_MAP: Mutex<HashMap<String, (String, String, String)>> =
        Mutex::new(HashMap::new());
    static ref DELETE_ROUTE_MAP: Mutex<HashMap<String, (String, String, String)>> =
        Mutex::new(HashMap::new());
    static ref ANONYMOUS_FN: Mutex<Vec<String>> = Mutex::new(Vec::new());
    pub static ref ROUTE_MAP: Mutex<Vec<HashMap<String, String>>> = Mutex::new(Vec::new());
}

// 使用路由启动
pub fn add_route_handle() -> TokenStream {
    let anonymous_fn = ANONYMOUS_FN.lock().unwrap();
    let mut route = Vec::new();
    let mut anonymous_route = Vec::new();

    // 提取请求信息返回
    let mut route_map: std::sync::MutexGuard<'_, Vec<HashMap<String, String>>> =
        ROUTE_MAP.lock().unwrap();

    // 遍历get请求组装
    for (k, (clazz, method_name, desc)) in GET_ROUTE_MAP.lock().unwrap().iter() {
        let clazz = format_ident!("{}", clazz);
        let method_name = format_ident!("{}", method_name);
        if anonymous_fn.contains(&format!("{}{}{}", clazz.clone(), "::", method_name.clone())) {
            // 是匿名接口
            anonymous_route.push(quote! {
                    let anonymous_app = anonymous_app.route(#k, get(#clazz::#method_name));
            });
        } else {
            route.push(quote! {
                    let app = app.route(#k, get(#clazz::#method_name));
            });
        }
        // 组装请求的介绍
        let mut get = HashMap::new();
        get.insert("method".to_string(), "GET".to_string());
        get.insert("url".to_string(), k.to_string());
        get.insert("desc".to_string(), desc.to_string());
        route_map.push(get);
    }
    
    // 遍历post请求组装
    for (k, (clazz, method_name, desc)) in POST_ROUTE_MAP.lock().unwrap().iter() {
        let clazz = format_ident!("{}", clazz);
        let method_name = format_ident!("{}", method_name);
        if anonymous_fn.contains(&format!("{}{}{}", clazz.clone(), "::", method_name.clone())) {
            // 是匿名接口
            anonymous_route.push(quote! {
                    let anonymous_app = anonymous_app.route(#k, post(#clazz::#method_name));
            });
        } else {
            route.push(quote! {
                    let app = app.route(#k, post(#clazz::#method_name));
            });
        }
        // 组装请求的介绍
        let mut post = HashMap::new();
        post.insert("method".to_string(), "POST".to_string());
        post.insert("url".to_string(), k.to_string());
        post.insert("desc".to_string(), desc.to_string());
        route_map.push(post);
    }
    
    // 遍历delete请求组装
    for (k, (clazz, method_name, desc)) in DELETE_ROUTE_MAP.lock().unwrap().iter() {
        let clazz = format_ident!("{}", clazz);
        let method_name = format_ident!("{}", method_name);
        if anonymous_fn.contains(&format!("{}{}{}", clazz.clone(), "::", method_name.clone())) {
            // 是匿名接口
            anonymous_route.push(quote! {
                    let anonymous_app = anonymous_app.route(#k, delete(#clazz::#method_name));
            });
        } else {
            route.push(quote! {
                    let app = app.route(#k, delete(#clazz::#method_name));
            });
        }
        // 组装请求的介绍
        let mut delete = HashMap::new();
        delete.insert("method".to_string(), "DELETE".to_string());
        delete.insert("url".to_string(), k.to_string());
        delete.insert("desc".to_string(), desc.to_string());
        route_map.push(delete);
    }

    let vec = route_map.clone();
    let route_str = serde_json::to_string(&vec).unwrap();

    let token_stream = quote! {

        fn ordinary_route() -> Router {
            let app = Router::new();
            #(#route)*
            let app = app.layer(middleware::from_fn(login_authorization::middleware));
            // 合并匿名路由
            app
        }

        fn anonymous_route() -> Router {
            let anonymous_app = Router::new();
            #(#anonymous_route)*
            anonymous_app
        }

        pub fn get_route() -> Vec<HashMap<String, String>> {
            let get_route = #route_str.to_string();
            let mut ret: Vec<HashMap<String, String>> = pro_json_util::str_to_object(&get_route).unwrap();
            let server_url = env::var("server_address").expect("服务器 url 初始化失败");
            for item in &mut ret {
                let url = item.get("url");
                let v = format!("http://{}{}", server_url, url.unwrap());
                item.insert("url".to_string(), v);
            }
            ret
        }

    };
    token_stream.into()
}

// 添加路由
pub fn control_handle(control_str: String, item: TokenStream) -> TokenStream {
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
                    if ["get", "post", "delete"].contains(&ident_str.as_str()) {
                        let mut attr_str = attr.meta.to_token_stream().to_string();
                        attr_str = attr_util::get_attr_str(&ident_str, attr_str);
                        let attr_map = attr_util::attr_to_map(attr_str);
                        let url_option = attr_map.get("url");
                        if let Some(url) = url_option {
                            let mut begin_control_str = control_str.clone();
                            if begin_control_str.ends_with("/") {
                                begin_control_str = begin_control_str[0..begin_control_str.len() - 1].to_string();
                            }
                            let mut end_control_str = attr_util::trim_begin_end_quotes(url);
                            if end_control_str.starts_with("/") {
                                end_control_str = end_control_str[1..end_control_str.len()].to_string();
                            }
                            let url = begin_control_str + "/" + &end_control_str;
                            let mut desc = "".to_string();
                            if let Some(desc_str) = attr_map.get("desc") {
                                desc = attr_util::trim_begin_end_quotes(desc_str);
                            }
                            let mut req_route_map = match ident_str.as_str() {
                                "get" => GET_ROUTE_MAP.lock().unwrap(),
                                "post" => POST_ROUTE_MAP.lock().unwrap(),
                                "delete" => DELETE_ROUTE_MAP.lock().unwrap(),
                                _ => panic!("异常类型 : {}", ident_str),
                            };
                            req_route_map.insert(url, (clazz.clone(), method_name.clone(), desc));
                        } else {
                            println!(
                                " {} {}没有设置url:{},attr_map:{:?} ",
                                clazz.clone(),
                                method_name.clone(),
                                ident_str,
                                attr_map
                            );
                        }
                    }
                    if String::from("pro_anonymous").eq(&ident_str) {
                        ANONYMOUS_FN
                            .lock()
                            .unwrap()
                            .push(format!("{}{}{}", clazz, "::", method_name));
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
            pub fn init_control() {
            }
        }
    };
    expanded.into()
}
