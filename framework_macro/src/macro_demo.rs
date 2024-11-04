// use proc_macro::TokenStream;
// use quote::{format_ident, quote};
// use syn::{parse::Parse, Token};
// use syn::{parse::ParseStream, parse_macro_input, ItemFn, Lit, LitInt, LitStr, Meta, MetaList};
// use syn::{ImplItem, ItemImpl};
// extern crate proc_macro;



// #[derive(Debug)]
// struct JobFn {
//     job_name: LitStr,
//     interval_millis: LitInt,
// }

// mod kw {
//     use syn::custom_keyword;
//     custom_keyword!(job_name);
//     custom_keyword!(interval_millis);
// }

// impl Parse for JobFn {
//     fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
//         let (mut job_name, mut interval_millis) = (None, None);
//         loop {
//             let lookahead = input.lookahead1();
//             if lookahead.peek(kw::job_name) {
//                 input.parse::<kw::job_name>()?;
//                 input.parse::<Token![=]>()?;
//                 job_name = Some(input.parse::<LitStr>()?);
//             } else if lookahead.peek(kw::interval_millis) {
//                 input.parse::<kw::interval_millis>()?;
//                 input.parse::<Token![=]>()?;
//                 interval_millis = Some(input.parse::<LitInt>()?);
//             } else {
//                 return Err(input.error("invalid argument"));
//             }
//             if let Err(_) = input.parse::<Token![,]>() {
//                 break;
//             }
//         }
//         match (job_name, interval_millis) {
//             (Some(job_name), Some(interval_millis)) => Ok(Self {
//                 job_name,
//                 interval_millis,
//             }),
//             _ => Err(input.error("missing some argument")),
//         }
//     }
// }

// // 扫描函数,将函数信息映射到 JobFn中
// pub fn test_job(attr: TokenStream, item: TokenStream) -> TokenStream {
//     let job_fn = parse_macro_input!(attr as JobFn);
//     let job_name = job_fn.job_name.value();
//     let lit_int = job_fn.interval_millis.base10_parse::<u64>().unwrap();
//     let input = parse_macro_input!(item as ItemFn);
//     let func_name = &input.sig.ident;
//     let job_func_name = format_ident!("job_{}", func_name);
//     let expanded = quote! {
//         fn #job_func_name(){
//             println!("开始执行定时任务:{}",#job_name);
//             let timer_task = TimerTask::new();
//             timer_task.start(#lit_int, || {
//                 #func_name();
//             });
//         }
//         #input
//     };
//     expanded.into()
// }

// // demo 扫描 impl 获取函数信息
// pub fn test_scan_functions(_: TokenStream, item: TokenStream) -> TokenStream {
//     println!("实现过程:{}", item.clone().to_string());
//     let item = parse_macro_input!(item as ItemImpl);
//     let mut functions = Vec::new();
//     for i in &item.items {
//         if let ImplItem::Fn(method) = i {
//             if let ImplItem::Fn(method) = i {
//                 for attr in &method.attrs {
//                     if let Some(ident) = attr.path().get_ident() {
//                         println!("ident: {}", ident);
//                         let value = Some(attr.parse_args::<LitStr>());
//                         if let Some(lit_str) = value {
//                             println!("Value: {}", lit_str.unwrap().value());
//                         }
//                     } else {
//                     }
//                 }
//                 functions.push(method.sig.ident.clone());
//             }
//             functions.push(method.sig.ident.clone());
//         }
//     }
//     // 在这里处理获取到的函数信息，例如打印函数名
//     println!("Functions in this impl: {:?}", functions);
//     let expanded = quote! {
//         #item
//     };
//     expanded.into()
// }

