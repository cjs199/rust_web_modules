use framework_utils::pro_collection_util;
use framework_utils::pro_str_util;
use proc_macro::TokenStream;
extern crate proc_macro;
use crate::utils::table_dto_to_sql_util;
use quote::format_ident;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

const CONST_AND: &str = "AND";
const CONST_OR: &str = "OR";

pub fn table(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input_str = input.to_string();

    let input = parse_macro_input!(input as DeriveInput);

    let create_table_dto_to_sql =
        table_dto_to_sql_util::create_table_dto_to_sql(attr.to_string(), input);

    let table_name_str = create_table_dto_to_sql.table_name;

    let table_name_ident = format_ident!("{}", table_name_str);

    // 获取结构名称
    let clazz_name_str = create_table_dto_to_sql.clazz_name;

    let clazz_name_ident = format_ident!("{}", clazz_name_str);

    let clazz_name_query_ident = format_ident!("{}SqlQuery", clazz_name_str);

    let mut lombok_data_fun = quote! {};

    let mut fields = quote! {};
    
    // 提取列名
    let column_name_arr = pro_collection_util::collect_field_values(&create_table_dto_to_sql.column_name, |field| field.0.clone());

    // 新增更新监控列
    let mut insert_monitor_column = Vec::new();

    let mut update_monitor_column = Vec::new();

    // 初始化id
    let id_column = create_table_dto_to_sql.id_column;
    let id_column_name = id_column.0;
    let id_column_name_ident = format_ident!("{}", id_column_name);
    let mut version_lock = false;

    
    // 像select等查询时选择的列
    let select_column_str =
        pro_str_util::format_and_join(&column_name_arr, |field| format!("`{}`", field), ",");

    // 更新时设置的列sql
    let update_set_sql =
        pro_str_util::format_and_join(&column_name_arr, |field| format!("`{}`=?", field), ",");

    // 设置插入时val的数据
    let insert_sql = pro_str_util::format_and_join(&column_name_arr, |_| String::from("?"), ",");

    // 插入更新时,绑定实体类的字段
    let mut bind_sql = Vec::new();

    // where条件,非空字段绑定
    let mut bind_not_none_where = Vec::new();

    // 插入更新时,绑定实体类的字段
    let mut bind_not_none_val = Vec::new();


    for column in create_table_dto_to_sql.column_name {
        let column_name = column.0;
        let column_type = column.1;

        let column_name_ident = format_ident!("{}", column_name.clone());
        let mut column_type_ident = proc_macro2::TokenStream::new();
        column_type_ident.extend(column_type.parse::<proc_macro2::TokenStream>().unwrap());

        {
            // 创建时间,更新时间,版本控制等等
            match column_name.as_str() {
                "create_by" => {
                    insert_monitor_column.push(quote! {
                            if let None = #table_name_ident.#id_column_name_ident {
                                #table_name_ident.create_by = Some(pro_base_security_util::get_login_user_id());
                            }
                        });
                }
                "create_time" => {
                    insert_monitor_column.push(quote! {
                        if let None = #table_name_ident.create_time {
                            #table_name_ident.create_time = Some(Utc::now());
                        }
                    });
                }
                "update_by" => {
                    insert_monitor_column.push(quote! {
                            if let None = #table_name_ident.update_by {
                                #table_name_ident.update_by = Some(pro_base_security_util::get_login_user_id());
                            }
                        });
                    update_monitor_column.push(quote! {
                            #table_name_ident.update_by = Some(pro_base_security_util::get_login_user_id());
                        });
                }
                "update_time" => {
                    insert_monitor_column.push(quote! {
                        if let None = #table_name_ident.update_time {
                            #table_name_ident.update_time = Some(Utc::now());
                        }
                    });
                    update_monitor_column.push(quote! {
                        #table_name_ident.update_time = Some(Utc::now());
                    });
                }
                "version" => {
                    let col_mon = quote! {
                        if let Some(v) = #table_name_ident.version {
                            #table_name_ident.version = Some(v + 1);
                        } else {
                            #table_name_ident.version = Some(1);
                        }
                    };
                    insert_monitor_column.push(col_mon.clone());
                    update_monitor_column.push(col_mon.clone());
                    version_lock = true;
                }
                _ => (),
            }
        }

        {
            // 拼接get set等函数名称
            let get_name = format_ident!("get_{}", column_name.clone());
            let set_name = format_ident!("set_{}", column_name.clone());

            let t = quote! {
                pub fn #get_name(&self)->&#column_type_ident{
                    &self.#column_name_ident
                }
                pub fn #set_name(&mut self, val:#column_type_ident){
                    self.#column_name_ident = val
                }
            };

            // 循环列,拼接对应的get set方法
            lombok_data_fun = quote! {
                #lombok_data_fun
                #t
            };

            let field_name = format_ident!("FIELD_{}", column_name.to_uppercase());
            let field_val = format!("{}", column_name.clone());

            let fd = quote! {
                pub const #field_name: &str = #field_val;
            };

            fields = quote! {
                #fields
               #fd
            };
        }
    
        {
            // 新增,更新,where条件等绑定参数设置
            bind_sql.push(quote! {.bind(#table_name_ident.#column_name_ident)});
            bind_not_none_where.push(quote! {
                if let Some(v) = &#table_name_ident.#column_name_ident {
                    not_none_where.push(format!("`{}`=?", #column_name));
                }
            });
            bind_not_none_val.push(quote! {
                if let Some(v) = &#table_name_ident.#column_name_ident {
                    query_as = query_as.bind(v);
                }
            });
        }
    
    }

    // 如果id是None,那么插入和更新时,需要将id初始化
    insert_monitor_column.push(quote! {
        if let None = #table_name_ident.#id_column_name_ident {
            #table_name_ident.#id_column_name_ident = Some(pro_snowflake_util::next_id());
        }
    });


    // 非None时绑定
    let bind_code = quote! {
        if let Some(b_v) = boxed.downcast_ref::<String>() {
            query_as = query_as.bind(b_v);
        } else if let Some(b_v) = boxed.downcast_ref::<&str>() {
            query_as = query_as.bind(b_v);
        } else if let Some(b_v) = boxed.downcast_ref::<char>() {
            query_as = query_as.bind(b_v.to_string());
        } else if let Some(b_v) = boxed.downcast_ref::<i32>() {
            query_as = query_as.bind(b_v);
        } else if let Some(b_v) = boxed.downcast_ref::<f32>() {
            query_as = query_as.bind(b_v);
        } else if let Some(b_v) = boxed.downcast_ref::<bool>() {
            query_as = query_as.bind(b_v);
        } else if let Some(b_v) = boxed.downcast_ref::<i8>() {
            query_as = query_as.bind(b_v);
        } else if let Some(b_v) = boxed.downcast_ref::<i16>() {
            query_as = query_as.bind(b_v);
        } else if let Some(b_v) = boxed.downcast_ref::<i64>() {
            query_as = query_as.bind(b_v);
        } else if let Some(b_v) = boxed.downcast_ref::<u8>() {
            query_as = query_as.bind(b_v);
        } else if let Some(b_v) = boxed.downcast_ref::<u16>() {
            query_as = query_as.bind(b_v);
        } else if let Some(b_v) = boxed.downcast_ref::<u32>() {
            query_as = query_as.bind(b_v);
        } else if let Some(b_v) = boxed.downcast_ref::<u64>() {
            query_as = query_as.bind(b_v);
        } else if let Some(b_v) = boxed.downcast_ref::<Vec<String>>() {
            for value in b_v {
                query_as = query_as.bind(value);
            }
        } else if let Some(b_v) = boxed.downcast_ref::<Vec<&str>>() {
            for value in b_v {
                query_as = query_as.bind(value);
            }
        } else if let Some(b_v) = boxed.downcast_ref::<Vec<char>>() {
            for value in b_v {
                query_as = query_as.bind(value.to_string());
            }
        } else if let Some(b_v) = boxed.downcast_ref::<Vec<i32>>() {
            for value in b_v {
                query_as = query_as.bind(value);
            }
        } else if let Some(b_v) = boxed.downcast_ref::<Vec<f32>>() {
            for value in b_v {
                query_as = query_as.bind(value);
            }
        } else if let Some(b_v) = boxed.downcast_ref::<Vec<bool>>() {
            for value in b_v {
                query_as = query_as.bind(value);
            }
        } else if let Some(b_v) = boxed.downcast_ref::<Vec<i8>>() {
            for value in b_v {
                query_as = query_as.bind(value);
            }
        } else if let Some(b_v) = boxed.downcast_ref::<Vec<i16>>() {
            for value in b_v {
                query_as = query_as.bind(value);
            }
        } else if let Some(b_v) = boxed.downcast_ref::<Vec<i64>>() {
            for value in b_v {
                query_as = query_as.bind(value);
            }
        } else if let Some(b_v) = boxed.downcast_ref::<Vec<u8>>() {
            for value in b_v {
                query_as = query_as.bind(value);
            }
        } else if let Some(b_v) = boxed.downcast_ref::<Vec<u16>>() {
            for value in b_v {
                query_as = query_as.bind(value);
            }
        } else if let Some(b_v) = boxed.downcast_ref::<Vec<u32>>() {
            for value in b_v {
                query_as = query_as.bind(value);
            }
        } else if let Some(b_v) = boxed.downcast_ref::<Vec<u64>>() {
            for value in b_v {
                query_as = query_as.bind(value);
            }
        } else {
            panic!("没有实现处理的异常类型！,或者你传入了 Option，没有 unwrap?");
        }
    };

    // 绑定函数时,匹配获取函数真实类型
    let bind_fn = quote! {
        for boxed in &self.set_bind_value {
            #bind_code
        }
        for boxed in &self.where_bind_value {
            #bind_code
        }
        for boxed in &self.having_bind_value {
            #bind_code
        }
    };

    let fetch_all_sql = format!("SELECT * FROM `{}`", table_name_str);

    let insert_sql = format!(
        "INSERT INTO `{}` ({}) VALUES ({})",
        table_name_str, select_column_str, insert_sql
    );

    let mut version_bind_begin = quote! {};
    let mut version_bind_end = quote! {};

    let update_sql = {
        if version_lock {
            version_bind_begin = quote! {
                let version = #table_name_ident.version.clone();
            };

            version_bind_end = quote! {
                .bind(version)
            };
            format!(
                "UPDATE `{}` SET {} WHERE `{}` = ? AND version = ?",
                table_name_str, update_set_sql, id_column_name
            )
        } else {
            format!(
                "UPDATE `{}` SET {} WHERE `{}` = ?",
                table_name_str, update_set_sql, id_column_name
            )
        }
    };
    let expanded = quote! {

        impl #clazz_name_ident {

            #lombok_data_fun

            #fields

            pub const TABLE_NAME: &str = #table_name_str;

            pub fn get_table_name()->String{
                #clazz_name_ident::TABLE_NAME.to_string()
            }

            pub fn clone(&self) -> #clazz_name_ident {
                let object_to_str = pro_json_util::object_to_str(&self);
                pro_json_util::str_to_object(&object_to_str).unwrap()
            }

        }

        pub struct #clazz_name_query_ident {
            select_columns: Vec<String>,
            set_columns: Vec<String>,
            set_bind_value: Vec<Box<dyn Any + Send + Sync>>,
            where_columns: Vec<String>,
            where_bind_value: Vec<Box<dyn Any + Send + Sync>>,
            page: i64,
            limit: i64,
            group_by: Vec<String>,
            having_columns: Vec<String>,
            having_bind_value: Vec<Box<dyn Any + Send + Sync>>,
            order_by_columns: Vec<String>,
        }

        impl #clazz_name_query_ident {

            pub fn new() -> #clazz_name_query_ident {
                #clazz_name_query_ident {
                    select_columns: Vec::new(),
                    set_columns: Vec::new(),
                    set_bind_value: Vec::new(),
                    where_columns: Vec::new(),
                    where_bind_value: Vec::new(),
                    page: -1,
                    limit: -1,
                    group_by: Vec::new(),
                    having_columns: Vec::new(),
                    having_bind_value: Vec::new(),
                    order_by_columns: Vec::new(),
                }
            }

            // SELECT column1, column2
            // FROM table_name
            // WHERE condition
            // GROUP BY column3
            // HAVING group_condition
            // ORDER BY column4
            // LIMIT n OFFSET m;
            pub fn get_select_sql(&mut self) -> String {
                let temp_column_str = &self.select_columns.join(",");
                let temp_column_str = temp_column_str.trim();
                let column_str;
                if temp_column_str.len() == 0 {
                    column_str=  #select_column_str;
                } else {
                    column_str = temp_column_str;
                }
                let where_columns_len = self.where_columns.len();
                let mut sql;
                if where_columns_len == 0 {
                    sql = format!("SELECT {} FROM `{}` ", column_str, #table_name_str );
                } else {
                    if let Some(last) = self.where_columns.last_mut() {
                        if matches!(last.as_str(), #CONST_AND | #CONST_OR) {
                            *last = String::from("");
                        }
                    }
                    let where_str = &self.where_columns.join(" ");
                    sql = format!("SELECT {} FROM `{}` WHERE {}", column_str, #table_name_str, where_str );
                }
                if self.group_by.len() > 0 {
                    sql = format!(" {} GROUP BY {} ", sql, &self.group_by.join(","));
                }
                let having_columns_len = self.having_columns.len();
                if having_columns_len > 0 {
                    if let Some(last) = self.having_columns.last_mut() {
                        if matches!(last.as_str(), #CONST_AND | #CONST_OR) {
                            *last = String::from("");
                        }
                    }
                    sql = format!(" {} HAVING {} ", sql, &self.having_columns.join(" "));
                }
                let order_by_columns_len = self.order_by_columns.len();
                if order_by_columns_len > 0 {
                    sql = format!(" {} ORDER BY {} ", sql, &self.order_by_columns.join(","));
                }
                if self.limit > 0 {
                    sql = format!(" {} LIMIT {} ", sql, self.limit);
                }
                if self.page > 0 {
                    let offset = (self.page - 1) * self.limit;
                    sql = format!(" {} OFFSET {} ", sql, offset );
                }
                sql
            }

            pub fn group_by<T: Into<String>>(&mut self, group_by_columns: Vec<T>) -> &mut Self {
                for item in group_by_columns {
                    let item = pro_str_util::camel_to_snake(item);
                    let item = item.trim();
                    self.group_by.push(format!("`{}`", item));
                }
                self
            }

            pub fn order_by<T: Into<String>>(&mut self, order_by_column: T,sort: Sort) -> &mut Self {
                let item = pro_str_util::camel_to_snake(order_by_column);
                let item = item.trim();
                self.order_by_columns.push(format!("`{}` {}", item , sort));
                self
            }

            pub fn select<T: Into<String>>(&mut self, select_columns: Vec<T>) -> &mut Self {
                for item in select_columns {
                    let item = pro_str_util::camel_to_snake(item);
                    self.select_columns.push(format!("`{}`", item.trim()));
                }
                self
            }

            pub async fn find_all<F, T>(&mut self,f: F) -> Result<Vec<T>, sqlx::Error>
                where
                F: FnMut(MySqlRow) -> T + Send,
                T: Unpin + Send,
            {
                let sql = self.get_select_sql();
                let mut query_as = sqlx::query(&sql);
                #bind_fn
                // 这里,因为 query_as 被map以后,返回类型发生了改变,必须用 let 重新接收一下
                let query_as = query_as.map(f);
                query_as.fetch_all(DB_ONCE_LOCK.get()).await
            }

            pub async fn find_all_to_entity(&mut self) -> Result<Vec<#clazz_name_ident>, sqlx::Error> {
                let sql = self.get_select_sql();
                let mut query_as = sqlx::query_as::<_, #clazz_name_ident>(&sql);
                #bind_fn
                // 这里,因为 query_as 被map以后,返回类型发生了改变,必须用 let 重新接收一下
                query_as.fetch_all(DB_ONCE_LOCK.get()).await
            }

            pub async fn find_one<F, T>(&mut self,f: F) -> Result<T, sqlx::Error>
            where
            F: FnMut(MySqlRow) -> T + Send,
            T: Unpin + Send,
            {
                let sql = self.get_select_sql();
                let mut query_as = sqlx::query(&sql);
                #bind_fn
                // 这里,因为 query_as 被map以后,返回类型发生了改变,必须用 let 重新接收一下
                let query_as = query_as.map(f);
                query_as.fetch_one(DB_ONCE_LOCK.get()).await
            }

            pub async fn find_one_to_entity(&mut self) -> Result<#clazz_name_ident, sqlx::Error> {
                let sql = self.get_select_sql();
                let mut query_as = sqlx::query_as::<_, #clazz_name_ident>(&sql);
                #bind_fn
                // 这里,因为 query_as 被map以后,返回类型发生了改变,必须用 let 重新接收一下
                query_as.fetch_one(DB_ONCE_LOCK.get()).await
            }

            pub fn page(&mut self, page: i64) -> &mut Self {
                self.page = page;
                self
            }

            pub fn limit(&mut self, limit: i64) -> &mut Self {
                self.limit = limit;
                self
            }

            pub async fn find_paged_result_to_entity(&mut self) -> PageResult<#clazz_name_ident>   {
                let len = self.where_columns.len();
                let count:i64 = {
                    // 搜索count
                    let sql;
                    if len == 0 {
                        sql = format!("SELECT COUNT(*) FROM `{}` ", #table_name_str );
                    } else {
                        if let Some(last) = self.where_columns.last_mut() {
                            if matches!(last.as_str(), #CONST_AND | #CONST_OR) {
                                *last = String::from("");
                            }
                        }
                        let where_str = &self.where_columns.join(" ");
                        sql = format!("SELECT COUNT(*) FROM `{}` WHERE {}", #table_name_str, where_str );
                    }
                    let mut query_as = sqlx::query_scalar(&sql);
                    #bind_fn
                    // 这里,因为 query_as 被map以后,返回类型发生了改变,必须用 let 重新接收一下
                    query_as.fetch_one(DB_ONCE_LOCK.get()).await.unwrap()
                };
                if count == 0 {
                    PageResult {
                        content: Vec::new(),
                        totalElements: count,
                        page: self.page
                    }
                } else {
                    let sql = self.get_select_sql();
                    let mut query_as = sqlx::query_as::<_, #clazz_name_ident>(&sql);
                    #bind_fn
                    // 这里,因为 query_as 被map以后,返回类型发生了改变,必须用 let 重新接收一下
                    let items: Vec<#clazz_name_ident> = query_as.fetch_all(DB_ONCE_LOCK.get()).await.unwrap();
                    PageResult {
                        content: items,
                        totalElements: count,
                        page: self.page,
                    }
                }
            }

            pub async fn update(&mut self) -> MySqlQueryResult {
                let set_str = &self.set_columns.join(" ");
                let len = self.where_columns.len();
                let sql;
                if len == 0 {
                    sql = format!("UPDATE `{}` SET {} ", #table_name_str, set_str);
                } else {
                    let where_str = &self.where_columns.join(" ");
                    sql = format!("UPDATE `{}` SET {} WHERE {} ", #table_name_str, set_str, where_str);
                }
                let mut query_as = sqlx::query(&sql);
                #bind_fn
                query_as.execute(DB_ONCE_LOCK.get()).await.unwrap()
            }

            pub fn set<T: Into<String>>(&mut self, column: T, boxed: Box<dyn Any + Send + Sync>) -> &mut Self {
                let column = pro_str_util::camel_to_snake(column);
                self.set_columns.push(format!("`{}`=?", column.trim()));
                self.set_bind_value.push(boxed);
                self
            }

            pub fn where_<T: Into<String>>(
                &mut self,
                column: T,
                condition: Condition,
                boxed: Box<dyn Any + Send + Sync>
            ) -> &mut Self {
                let condition_str = match condition {
                    // 大于
                    Condition::gt => ">",
                    // 小于
                    Condition::lt => "<",
                    // 等于
                    Condition::eq => "=",
                    // 不等于
                    Condition::ne => "!=",
                    Condition::like => "LIKE",
                    Condition::In => "IN",
                };
                let column = pro_str_util::camel_to_snake(column);
                let column = column.trim();
                if Condition::In == condition {
                    let vec_len = pro_collection_util::get_box_vec_len(&boxed);
                    if vec_len == 0 {
                        info!("{} IN 条件集合是空",column);
                        self.where_columns.push(String::from("FALSE"));
                    }else{
                        let mut val_col = Vec::new();
                        for item in 0..vec_len {
                            val_col.push("?");
                        }
                        self.where_columns.push(format!("`{}` {} ({}) ", column, condition_str , val_col.join(",")));
                        self.where_bind_value.push(boxed);
                    }
                } else if Condition::like == condition {
                    self.where_columns.push(format!("`{}` {} '%{}%'", column, condition_str, pro_collection_util::box_to_string(boxed)));
                } else {
                    self.where_columns.push(format!("`{}`{}?", column, condition_str));
                    self.where_bind_value.push(boxed);
                }
                self
            }

            pub fn having<T: Into<String>>(
                &mut self,
                column: T,
                condition: Condition,
                boxed: Box<dyn Any + Send + Sync>
            ) -> &mut Self {
                let condition_str = match condition {
                    // 大于
                    Condition::gt => ">",
                    // 小于
                    Condition::lt => "<",
                    // 等于
                    Condition::eq => "=",
                    // 不等于
                    Condition::ne => "!=",
                    Condition::like => "LIKE",
                    Condition::In => "IN",
                };
                let column = pro_str_util::camel_to_snake(column);
                let column = column.trim();
                if Condition::In == condition {
                    let vec_len = pro_collection_util::get_box_vec_len(&boxed);
                    let mut val_col = Vec::new();
                    for item in 0..vec_len {
                        val_col.push("?");
                    }
                    self.having_columns.push(format!("`{}` {} ({}) ", column, condition_str , val_col.join(",")));
                    self.having_bind_value.push(boxed);
                } else if Condition::like == condition {
                    self.having_columns.push(format!("`{}` {} '%{}%'", column, condition_str, pro_collection_util::box_to_string(boxed)));
                } else {
                    self.having_columns.push(format!("`{}`{}?", column, condition_str));
                    self.having_bind_value.push(boxed);
                }
                self
            }

            pub fn having_and(&mut self) -> &mut Self {
                self.having_columns.push(String::from(#CONST_AND));
                self
            }

            pub fn having_or(&mut self) -> &mut Self {
                self.having_columns.push(String::from(#CONST_OR));
                self
            }

            pub fn where_is_null<T: Into<String>>(
                &mut self,
                column: T
            ) -> &mut Self {
                let column_name = pro_str_util::camel_to_snake(column);
                self.where_columns.push(format!("`{}`  IS NULL ", column_name.trim()));
                self
            }

            pub fn having_is_null<T: Into<String>>(
                &mut self,
                column: T
            ) -> &mut Self {
                let column_name = pro_str_util::camel_to_snake(column);
                self.where_columns.push(format!("`{}`  IS NULL ", column_name.trim()));
                self
            }

            pub fn and(&mut self) -> &mut Self {
                self.where_columns.push(String::from(#CONST_AND));
                self
            }

            pub fn or(&mut self) -> &mut Self {
                self.where_columns.push(String::from(#CONST_OR));
                self
            }

            pub async fn direct_find_all_to_entity(#table_name_ident: #clazz_name_ident) -> Result<Vec<#clazz_name_ident>, sqlx::Error> {
                let mut not_none_where = Vec::new();
                #(#bind_not_none_where)*
                let sql;
                if not_none_where.len() == 0{
                     sql = format!(
                        "SELECT {} FROM `{}`", #select_column_str,  #table_name_str
                    );
                } else {
                     sql = format!(
                        "SELECT {} FROM `{}` WHERE {}" , #select_column_str , #table_name_str, not_none_where.join(" and")
                    );
                }
                let mut query_as = sqlx::query_as::<_, #clazz_name_ident>(&sql);
                #(#bind_not_none_val)*
                query_as.fetch_all(DB_ONCE_LOCK.get()).await
            }

            // 这是一个异步函数，用于获取指定表的分页查询结果，并以自定义的分页结果结构体形式返回。
            pub async fn direct_find_paged_result(#table_name_ident: #clazz_name_ident, page: i64, limit:i64) -> PageResult<#clazz_name_ident>   {
                #clazz_name_query_ident::direct_find_sorted_paged_result(#table_name_ident, #id_column_name, Sort::Desc, page, limit).await
            }

            pub async fn direct_find_sorted_paged_result (#table_name_ident: #clazz_name_ident, order_by_column: impl Into<String>,sort: Sort, page: i64, limit:i64) -> PageResult<#clazz_name_ident>   {
                // 克隆传入的实体对象，可能是为了在后续操作中不影响原始对象。
                let clone_entity = #table_name_ident.clone();
                // 调用另一个异步函数获取指定表的记录总数。
                let direct_count = #clazz_name_query_ident::direct_count(#table_name_ident).await;
                if(direct_count == 0){
                    PageResult {
                        content: Vec::new(),
                        totalElements: direct_count,
                        page: page,
                    }
                }else{
                    // 调用另一个异步函数进行分页查询，获取一页的数据。
                    let direct_find_by_entity_and_page = #clazz_name_query_ident::direct_find_entities_with_sort_and_pagination(clone_entity,order_by_column,sort, page, limit).await.unwrap();
                    // 将分页查询结果封装到自定义的分页结果结构体中，并转换为 JSON 格式返回。
                    PageResult {
                        content: direct_find_by_entity_and_page,
                        totalElements: direct_count,
                        page: page,
                    }
                }
            }

            // 计算指定表的记录总数的异步函数。
            // 如果有非空的条件，将根据这些条件进行计数，否则对整个表进行计数。
            pub async fn direct_count(#table_name_ident: #clazz_name_ident) -> i64 {
                // 创建一个可变的空向量，用于存储非空条件。
                let mut not_none_where = Vec::new();
                #(#bind_not_none_where)*
                // 根据是否有非空条件来构建不同的 SQL 查询语句。
                let sql;
                if not_none_where.len() == 0{
                    sql = format!(
                        "SELECT COUNT(*) FROM `{}`",  #table_name_str
                    );
                } else {
                    sql = format!(
                        "SELECT COUNT(*) FROM `{}` WHERE {}", #table_name_str, not_none_where.join(" and")
                    );
                }
                // 创建一个可变的查询对象，用于执行计数查询。
                let mut query_as = sqlx::query_scalar::<_, i64>(&sql);
                #(#bind_not_none_val)*
                // 执行查询并获取结果，如果有错误则直接 panic。
                query_as.fetch_one(DB_ONCE_LOCK.get()).await.unwrap()
            }

            // 根据条件和分页参数查找指定表中的记录，并返回结果列表的异步函数。
            // 如果有非空的条件，将根据这些条件进行查询，否则对整个表进行查询。
            // page从1开始,但是经过转换以后数据库中是从0开始,所以开始时减1
            pub async fn direct_find_entities_by_page(#table_name_ident: #clazz_name_ident, page: i64, limit: i64) -> Result<Vec<#clazz_name_ident>, sqlx::Error> {
                #clazz_name_query_ident::direct_find_entities_with_sort_and_pagination(#table_name_ident, #id_column_name, Sort::Desc, page, limit).await
            }

            pub async fn direct_find_entities_with_sort_and_pagination(#table_name_ident: #clazz_name_ident, order_by_column: impl Into<String>,sort: Sort, page: i64, limit: i64) -> Result<Vec<#clazz_name_ident>, sqlx::Error> {
                // 创建一个可变的空向量，用于存储非空条件。
                let order_by_column = pro_str_util::camel_to_snake(order_by_column);
                let order_by_column = order_by_column.trim();
                let mut not_none_where = Vec::new();
                #(#bind_not_none_where)*
                // 根据是否有非空条件来构建不同的 SQL 查询语句，并结合分页参数。
                let sql;
                let page = page - 1;
                let offset = page * limit;
                if not_none_where.len() == 0{
                    sql = format!(
                        "SELECT {} FROM `{}` ORDER BY `{}` {} LIMIT {} OFFSET {}", #select_column_str, #table_name_str, order_by_column, sort, limit, offset
                    );
                } else {
                    sql = format!(
                        "SELECT {} FROM `{}` WHERE {} ORDER BY `{}` {} LIMIT {} OFFSET {}", #select_column_str, #table_name_str, not_none_where.join(" and"), order_by_column, sort, limit, offset
                    );
                }
                // 创建一个可变的查询对象，用于执行查询操作。
                let mut query_as = sqlx::query_as::<_, #clazz_name_ident>(&sql);
                #(#bind_not_none_val)*
                // 执行查询并获取所有结果，如果有错误则直接 panic。
                query_as.fetch_all(DB_ONCE_LOCK.get()).await
            }

            pub async fn direct_find_all() -> Result<Vec<#clazz_name_ident>, sqlx::Error> {
                sqlx::query_as::<_, #clazz_name_ident>(#fetch_all_sql).fetch_all(DB_ONCE_LOCK.get()).await
            }

            pub async fn direct_find_by_id(boxed: Box<dyn Any + Send + Sync>) -> Result<#clazz_name_ident, sqlx::Error> {
                #clazz_name_query_ident::direct_find_one_by_column(#id_column_name,boxed).await
            }

            pub async fn direct_find_by_id_vec(boxed: Box<dyn Any + Send + Sync>) -> Result<Vec<#clazz_name_ident>, sqlx::Error> {
                #clazz_name_query_ident::direct_find_all_by_condition(#id_column_name,Condition::In,boxed).await
            }

            pub async fn direct_find_one_by_column<T: Into<String>>(column: T, boxed: Box<dyn Any + Send + Sync>) -> Result<#clazz_name_ident, sqlx::Error> {
                let column_name = pro_str_util::camel_to_snake(column);
                let sql = format!("SELECT {} FROM `{}` WHERE `{}` = ? ", #select_column_str, #table_name_str, column_name);
                let mut query_as = sqlx::query_as::<_, #clazz_name_ident>(&sql);
                #bind_code
                query_as.fetch_one(DB_ONCE_LOCK.get()).await
            }

            pub async fn direct_find_all_by_column<T: Into<String>>(column: T, boxed: Box<dyn Any + Send + Sync>) -> Result<Vec<#clazz_name_ident>, sqlx::Error> {
                let column_name = pro_str_util::camel_to_snake(column);
                let sql = format!("SELECT {} FROM `{}` WHERE `{}` = ? ", #select_column_str, #table_name_str, column_name);
                let mut query_as = sqlx::query_as::<_, #clazz_name_ident>(&sql);
                #bind_code
                query_as.fetch_all(DB_ONCE_LOCK.get()).await
            }

            pub async fn direct_find_all_by_condition<T: Into<String>>(column: T,condition: Condition, boxed: Box<dyn Any + Send + Sync>) -> Result<Vec<#clazz_name_ident>, sqlx::Error> {
                let condition_str = match condition {
                    // 大于
                    Condition::gt => ">",
                    // 小于
                    Condition::lt => "<",
                    // 等于
                    Condition::eq => "=",
                    // 不等于
                    Condition::ne => "!=",
                    Condition::like => "LIKE",
                    Condition::In => "IN",
                };
                let column_name = pro_str_util::camel_to_snake(column);
                if Condition::In == condition {
                    let vec_len = pro_collection_util::get_box_vec_len(&boxed);
                    if vec_len == 0 {
                        return Ok(vec!());
                    }
                    let mut val_col = Vec::new();
                    for item in 0..vec_len {
                        val_col.push("?");
                    }
                    let sql = format!("SELECT {} FROM `{}` WHERE `{}` {} ( {} ) ", #select_column_str, #table_name_str, column_name, condition_str , val_col.join(","));
                    let mut query_as = sqlx::query_as::<_, #clazz_name_ident>(&sql);
                    #bind_code
                    query_as.fetch_all(DB_ONCE_LOCK.get()).await
                } else if Condition::like == condition {
                    let sql = format!("SELECT {} FROM `{}` WHERE `{}` {} '%{}%' ", #select_column_str, #table_name_str, column_name, condition_str, pro_collection_util::box_to_string(boxed));
                    let mut query_as = sqlx::query_as::<_, #clazz_name_ident>(&sql);
                    query_as.fetch_all(DB_ONCE_LOCK.get()).await

                } else {
                    let sql = format!("SELECT {} FROM `{}` WHERE `{}` {} ? ", #select_column_str, #table_name_str, column_name, condition_str);
                    let mut query_as = sqlx::query_as::<_, #clazz_name_ident>(&sql);
                    #bind_code
                    query_as.fetch_all(DB_ONCE_LOCK.get()).await
                }
            }

            pub async fn direct_delete_all() -> MySqlQueryResult {
                let sql = format!("DELETE FROM `{}`", #table_name_str);
                let mut query_as = sqlx::query(&sql);
                query_as.execute(DB_ONCE_LOCK.get()).await.unwrap()
            }

            pub async fn direct_delete_by_column<T: Into<String>>(column: T, boxed: Box<dyn Any + Send + Sync>) -> MySqlQueryResult {
                let column_name = pro_str_util::camel_to_snake(column);
                let sql = format!("DELETE FROM `{}` WHERE `{}` = ? ", #table_name_str, column_name);
                let mut query_as = sqlx::query(&sql);
                #bind_code
                query_as.execute(DB_ONCE_LOCK.get()).await.unwrap()
            }

            pub async fn direct_delete_by_condition<T: Into<String>>(column: T,condition: Condition, boxed: Box<dyn Any + Send + Sync>) -> MySqlQueryResult {
                let condition_str = match condition {
                    // 大于
                    Condition::gt => ">",
                    // 小于
                    Condition::lt => "<",
                    // 等于
                    Condition::eq => "=",
                    // 不等于
                    Condition::ne => "!=",
                    Condition::like => "LIKE",
                    Condition::In => "IN",
                };
                let column_name = pro_str_util::camel_to_snake(column);
                let column_name = column_name.trim();
                if Condition::In == condition {
                    let vec_len = pro_collection_util::get_box_vec_len(&boxed);
                    if vec_len == 0 {
                        return <MySqlQueryResult as std::default::Default>::default();
                    }
                    let mut val_col = Vec::new();
                    for item in 0..vec_len {
                        val_col.push("?");
                    }
                    let sql = format!("DELETE FROM `{}` WHERE `{}` {} ( {} ) ", #table_name_str, column_name, condition_str , val_col.join(","));
                    let mut query_as = sqlx::query(&sql);
                    #bind_code
                    query_as.execute(DB_ONCE_LOCK.get()).await.unwrap()
                } else if Condition::like == condition {
                    let sql = format!("DELETE FROM `{}` WHERE `{}` {} '%{}%' ", #table_name_str, column_name, condition_str, pro_collection_util::box_to_string(boxed));
                    let mut query_as = sqlx::query(&sql);
                    query_as.execute(DB_ONCE_LOCK.get()).await.unwrap()
                } else {
                    let sql = format!("DELETE FROM `{}` WHERE `{}` {} ? ", #table_name_str, column_name, condition_str );
                    let mut query_as = sqlx::query(&sql);
                    #bind_code
                    query_as.execute(DB_ONCE_LOCK.get()).await.unwrap()
                }
            }

            pub async fn direct_insert(mut #table_name_ident: #clazz_name_ident) ->  MySqlQueryResult {
                #clazz_name_query_ident::direct_insert_by_exec(#table_name_ident,DB_ONCE_LOCK.get()).await
            }

            pub async fn direct_insert_vec(mut entity_vec: Vec<#clazz_name_ident>) {
                let group_by_vec_size = pro_collection_util::group_by_vec_size(&entity_vec, 50);
                let get = DB_ONCE_LOCK.get();
                for group_vec in group_by_vec_size {
                    let temp_sql = String::from(#insert_sql);
                    // 生成sql
                    let index = temp_sql.find("VALUES").unwrap();
                    let mut sql_base = temp_sql[0..index + 6].to_string();
                    let value = &temp_sql[index + 6..temp_sql.len()];
                    let mut sql_vec = Vec::new();
                    for _ in 0..group_vec.len() {
                        sql_vec.push(value);
                    }
                    sql_base.push_str(&sql_vec.join(","));
                    let mut query = sqlx::query(sql_base.as_str());
                    // 绑定数据
                    for entity in group_vec {
                        let mut #table_name_ident = entity.clone();
                        #(#insert_monitor_column)*
                        query = query
                        #(#bind_sql)*
                        ;
                    }
                    query.execute(get).await.unwrap();
                }
            }

            pub async fn direct_update(mut #table_name_ident: #clazz_name_ident) ->  MySqlQueryResult   {
                #clazz_name_query_ident::direct_update_by_exec(#table_name_ident,DB_ONCE_LOCK.get()).await
            }

            pub async fn single_read_only(sql: String) -> HashMap<String, String> {
                let query_as = sqlx::query(&sql)
                    .map(|row: MySqlRow| {
                        let mut row_map = HashMap::new();
                        let len = row.len();
                        for index in 0..len {
                            let column_name = pro_sql_query_util::get_row_column_name_to_str(&row, index);
                            let column_type_name = pro_sql_query_util::get_row_column_type_name_to_str(&row, index);
                            let val = pro_sql_query_util::get_row_val_to_str(&row, index, column_type_name);
                            row_map.insert(column_name, val);
                        }
                        return row_map;
                    })
                    .fetch_one(DB_ONCE_LOCK.get())
                    .await
                    .unwrap();
                query_as
            }

            pub async fn list_read_only(sql: String) -> Vec<HashMap<String, String>> {
                let query_as: Vec<HashMap<String, String>> = sqlx::query(&sql)
                .map(|row: MySqlRow| {
                    let mut row_map = HashMap::new();
                    let len = row.len();
                    for index in 0..len {
                        let column_name = pro_sql_query_util::get_row_column_name_to_str(&row, index);
                        let column_type_name = pro_sql_query_util::get_row_column_type_name_to_str(&row, index);
                        let val = pro_sql_query_util::get_row_val_to_str(&row, index, column_type_name);
                        row_map.insert(column_name, val);
                    }
                    return row_map;
                })
                .fetch_all(DB_ONCE_LOCK.get())
                .await
                .unwrap();
                query_as
            }

            // 事务执行方法
            pub async fn tx_exec<'q, F, FnRet, RET>(mut func: F) -> Result<RET, ProException>
            where
                F: FnMut(Transaction<'q, sqlx::MySql>) -> FnRet,
                FnRet: Future<Output = (Result<RET, ProException>, Transaction<'q, sqlx::MySql>)>,
            {
                let pool = DB_ONCE_LOCK.get();
                let tx: Transaction<'q, sqlx::MySql> = pool.begin().await.unwrap();
                let (ret_result, tx) = func(tx).await;
                match ret_result {
                    Ok(_) => {
                        let tx_commit = tx.commit().await;
                        match tx_commit {
                            Ok(_) => return ret_result,
                            Err(err) => {
                                error!("事务提交异常:{}", err);
                                Err(ProException::事务提交异常)
                            }
                        }
                    }
                    Err(err) => {
                        error!("事务执行异常:{:?}", err);
                        let tx_rollback = tx.rollback().await;
                        if let Err(tx_rollback_err) = tx_rollback {
                            error!("事务回滚异常:{}", tx_rollback_err);
                            Err(ProException::事务回滚异常)
                        } else {
                            Err(ProException::事务执行异常)
                        }
                    }
                }

                // 使用demo
                // let _1 = sqlx::query("INSERT INTO `bei_you_you`.`sys_dept` (`id`, `create_by`, `create_time`, `update_by`, `update_time`, `version`, `dept_name`) VALUES (1512443351729702350, 0, '2022-04-08 22:52:49.552880', 1512445510265341685, '2022-09-02 12:44:59.539236', 2, '运维部门');")
                // .execute( &mut *tx)
                // .await;
                // let _2 = sqlx::query("INSERT INTO `bei_you_you`.`sys_dept` (`id`, `create_by`, `create_time`, `update_by`, `update_time`, `version`, `dept_name`) VALUES (1512443351729702351, 0, '2022-04-08 22:52:49.552880', 1512445510265341685, '2022-09-02 12:44:59.539236', 2, '运维部门');")
                // .execute( &mut *tx)
                // .await;
                // if let Err(_1err) = _1  {
                //     return (Err(ProException::事务执行异常),tx);
                // }
                // if let Err(_2err) = _2 {
                //     return (Err(ProException::事务执行异常),tx);
                // }

            }

            pub async fn direct_insert_by_exec<'c, EXEC>(mut #table_name_ident: #clazz_name_ident, pool: EXEC) ->  MySqlQueryResult
            where
                EXEC: Executor<'c, Database = MySql>,
            {
                #(#insert_monitor_column)*
                sqlx::query(#insert_sql)
                    #(#bind_sql)*
                    .execute(pool).await.unwrap()
            }

            // 判断更新影响行数
            // println!("{}", direct_update.rows_affected());
            // 判断是否更新成功案例
            // println!("{}", direct_update.rows_affected() > 0);
            pub async fn direct_update_by_exec<'c, EXEC>(mut #table_name_ident: #clazz_name_ident, pool: EXEC) ->  MySqlQueryResult
            where
                EXEC: Executor<'c, Database = MySql>,
            {
                #version_bind_begin
                #(#update_monitor_column)*
                sqlx::query(#update_sql)
                    #(#bind_sql)*
                    .bind(#table_name_ident.#id_column_name_ident)
                    #version_bind_end
                    .execute(pool).await.unwrap()
            }


        }

    };
    let ret_str = "#[derive(FromRow)]  ".to_owned() + &input_str + expanded.to_string().as_str();
    let mut ret_token_stream = TokenStream::new();
    ret_token_stream.extend(ret_str.parse::<TokenStream>().unwrap());
    ret_token_stream
}
