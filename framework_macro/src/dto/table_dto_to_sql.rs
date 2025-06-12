/// 通过表格实体类生成sql的关键信息
pub struct TableDtoToSql {
    // (id列名,列类型)
    pub id_column: (String,String),

    // 例如,sys_role_sys_auths
    pub table_name: String,

    // 例如,SysRoleSysAuths
    pub clazz_name: String,

    // (表格的列名数组,列类型)
    pub column_name: Vec<(String,String)>,
}
