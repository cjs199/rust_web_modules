use lazy_static::lazy_static;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Mutex;
use syn::{parse_macro_input, ImplItem, ItemImpl, LitStr, PathSegment, Type};


lazy_static! {
    static ref GET_ROUTE_MAP: Mutex<HashMap<String, (String, String)>> = Mutex::new(HashMap::new());
    static ref POST_ROUTE_MAP: Mutex<HashMap<String, (String, String)>> =
        Mutex::new(HashMap::new());
    static ref DELETE_ROUTE_MAP: Mutex<HashMap<String, (String, String)>> =
        Mutex::new(HashMap::new());
    static ref ANONYMOUS_FN: Mutex<Vec<String>> = Mutex::new(Vec::new());

    pub static ref ROUTE_MAP: Mutex<Vec<HashMap<String, String>>> =
        Mutex::new(Vec::new());

}

// 使用路由启动
pub fn add_route_handle() -> TokenStream {
    let anonymous_fn = ANONYMOUS_FN.lock().unwrap();
    let mut route = Vec::new();
    let mut anonymous_route = Vec::new();
    for (k, (clazz, method_name)) in GET_ROUTE_MAP.lock().unwrap().iter() {
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
    }
    for (k, (clazz, method_name)) in POST_ROUTE_MAP.lock().unwrap().iter() {
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
    }
    for (k, (clazz, method_name)) in DELETE_ROUTE_MAP.lock().unwrap().iter() {
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
    }

    let get_mutex = GET_ROUTE_MAP.lock().unwrap();
    let get_keys = get_mutex.keys();
    let post_mutex = POST_ROUTE_MAP.lock().unwrap();
    let post_keys = post_mutex.keys();

    let delete_mutex = DELETE_ROUTE_MAP.lock().unwrap();
    let delete_keys = delete_mutex.keys();
    
    let mut route_map: std::sync::MutexGuard<'_, Vec<HashMap<String, String>>> = ROUTE_MAP.lock().unwrap();

    for key in get_keys {
        let mut get = HashMap::new();
        get.insert("method".to_string(), "GET".to_string());
        get.insert("url".to_string(), key.to_string());
        route_map.push(get);
    }
    for key in post_keys {
        let mut post = HashMap::new();
        post.insert("method".to_string(), "POST".to_string());
        post.insert("url".to_string(), key.to_string());
        route_map.push(post);
    }
    for key in delete_keys {
        let mut delete = HashMap::new();
        delete.insert("method".to_string(), "DELETE".to_string());
        delete.insert("url".to_string(), key.to_string());
        route_map.push(delete);
    }


    let vec = route_map.clone();
    let route_str = serde_json::to_string(&vec).unwrap();

    let token_stream = quote! {

        fn ordinary_route() -> Router {
            let app = Router::new();
            #(#route)*
            let app = app.layer(middleware::from_fn(layer_util::login_authorization));
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
            let server_url = env::var("server_address").expect("db url,初始化db失败");
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
pub fn control_handle(item: TokenStream) -> TokenStream {
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
                    if String::from("get").eq(&ident_str) {
                        let value = Some(attr.parse_args::<LitStr>());
                        if let Some(lit_str) = value {
                            let get_path = lit_str.unwrap().value();
                            GET_ROUTE_MAP
                                .lock()
                                .unwrap()
                                .insert(get_path.to_string(), (clazz.clone(), method_name.clone()));
                        }
                    }
                    if String::from("post").eq(&ident_str) {
                        let value = Some(attr.parse_args::<LitStr>());
                        if let Some(lit_str) = value {
                            let post_path = lit_str.unwrap().value();
                            POST_ROUTE_MAP.lock().unwrap().insert(
                                post_path.to_string(),
                                (clazz.clone(), method_name.clone()),
                            );
                        }
                    }
                    if String::from("delete").eq(&ident_str) {
                        let value = Some(attr.parse_args::<LitStr>());
                        if let Some(lit_str) = value {
                            let post_path = lit_str.unwrap().value();
                            DELETE_ROUTE_MAP.lock().unwrap().insert(
                                post_path.to_string(),
                                (clazz.clone(), method_name.clone()),
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
