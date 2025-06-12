extern crate proc_macro;
use quote::ToTokens;
use syn::{Data, DeriveInput};

use crate::dto::table_dto_to_sql::TableDtoToSql;

/// 通过表格实体类对象,
/// 创建dto实体类,
/// 用于生成sql
pub fn create_table_dto_to_sql(
    table_name: String, // sys_role_sys_auths
    input: DeriveInput,
) -> TableDtoToSql {
    let data = input.data;
    let mut column_name = Vec::new();
    let mut id_column = (String::from(""),String::from(""));
    match data {
        Data::Struct(s) => {
            // 遍历成员
            for f in s.fields {
                let attr_name = f.ident.to_token_stream();
                let attr_name_str = attr_name.to_string();
                let attr_ty = f.ty.to_token_stream().to_string();
                column_name.push((attr_name_str.clone(),attr_ty.clone()));
                // 找到id,进行初始化
                for attr in f.attrs.clone() {
                    if let Some(ident) = attr.path().get_ident() {
                        let ident_str = ident.to_string();
                        if String::from("id") == ident_str {
                            id_column.0 = attr_name_str.clone();
                            id_column.1 = attr_ty.clone();
                        }
                    }
                }
            }
        }
        _ => (),
    }
    // 获取结构名称
    let clazz_name = input.ident.to_token_stream().to_string();
    TableDtoToSql {
        id_column: id_column,
        table_name: table_name,
        clazz_name: clazz_name,
        column_name: column_name,
    }
}
