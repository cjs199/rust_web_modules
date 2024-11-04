use proc_macro::TokenStream;
extern crate proc_macro;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput};

fn remove_spaces(s: impl Into<String>) -> String {
    let s = s.into();
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

pub fn handle(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // 提取属性,准备重写
    // let attributes = input.attrs;
    let data = input.data;
    let mut table_field_arr = Vec::new();
    let clazz_name = input.ident.to_token_stream();
    match data {
        Data::Struct(s) => {
            // 遍历成员
            for f in s.fields {
                let attr_name = f.ident.to_token_stream();
                let attr_ty = f.ty.to_token_stream();
                let vis = f.vis.to_token_stream();

                let attr_ty_str = remove_spaces(attr_ty.to_string());

                let is_num = matches!(
                    attr_ty_str.as_str(),
                            "i8" 
                        | "i16"
                        | "i32"
                        | "i64"
                        | "i128"
                        | "u8"
                        | "u16"
                        | "u32"
                        | "u64"
                        | "u128"
                        | "isize"
                        | "usize"
                        | "f32"
                        | "f64"
                        | "Option<i8>"
                        | "Option<i16>"
                        | "Option<i32>"
                        | "Option<i64>"
                        | "Option<i128>"
                        | "Option<u8>"
                        | "Option<u16>"
                        | "Option<u32>"
                        | "Option<u64>"
                        | "Option<u128>"
                        | "Option<isize>"
                        | "Option<usize>"
                        | "Option<f32>"
                        | "Option<f64>"
                );

                if is_num {
                    if attr_ty_str.contains("Option") {
                        table_field_arr.push(quote! {
                            #[serde(default = "option_default_none", serialize_with = "option_num_ser", deserialize_with = "option_num_deser")]
                            #vis #attr_name: # attr_ty,
                        });
                    } else {
                        table_field_arr.push(quote! {
                            // 这个注解,无法将空字符串等非数字正常转换为None
                            #[serde(serialize_with = "num_ser", deserialize_with = "num_deser")]
                            #vis #attr_name: # attr_ty,
                        });
                    }
                } else {

                    if attr_ty_str.contains("Option") {
                        table_field_arr.push(quote! {
                            #[serde(default = "option_default_none", deserialize_with = "option_empty_ignore_deser")]
                            #vis #attr_name: # attr_ty,
                        });
                    } else {
                        table_field_arr.push(quote! {
                            #vis #attr_name: # attr_ty,
                        });
                    }
                }
            }
        }
        _ => (),
    }
    let ret = quote! {
        
        #[derive(Serialize, Deserialize, Debug)]
        #[serde(rename_all = "camelCase")]
        pub struct #clazz_name {
            #(#table_field_arr)*
        }

        impl #clazz_name {
            pub fn default() -> #clazz_name {
                pro_json_util::str_to_object("{}").unwrap()
            } 
        }

    };
    // ret.into()
    let mut ret_token_stream = TokenStream::new();
    ret_token_stream.extend(ret.to_string().parse::<TokenStream>().unwrap());
    ret_token_stream



}
