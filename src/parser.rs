use sqlparser::dialect::MySqlDialect;
use sqlparser::parser::Parser;
use std::fs;

use crate::generator::to_pascal_case;

pub struct TableColumn {
    pub name: String,
    pub sql_type: String,
    pub is_nullable: bool,
}

pub struct TableSchema {
    pub table_name: String,
    pub columns: Vec<TableColumn>,
    pub module_name: String,
    pub struct_name: String,
    pub create_special_fields: Option<Vec<String>>,
    pub update_special_fields: Option<Vec<String>>,
    pub always_include_in_dto: Option<Vec<String>>,
}

pub fn parse_sql_file(
    path: &str,
    rules: Option<&serde_json::Value>,
) -> Result<TableSchema, Box<dyn std::error::Error>> {
    let sql = fs::read_to_string(path)?;
    let dialect = MySqlDialect {};
    let statements = Parser::parse_sql(&dialect, &sql)?;

    for stmt in statements {
        if let sqlparser::ast::Statement::CreateTable(create_table) = stmt {
            let table_name = create_table.name.to_string();

            let mut create_special_fields = None;
            let mut update_special_fields = None;
            let mut always_include_in_dto = None;

            if let Some(rules) = rules {
                if let Some(rule) = rules.get(&table_name) {
                    if let Some(fields) =
                        rule.get("create_special_fields").and_then(|v| v.as_array())
                    {
                        create_special_fields = Some(
                            fields
                                .iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect(),
                        );
                    }
                    if let Some(fields) =
                        rule.get("update_special_fields").and_then(|v| v.as_array())
                    {
                        update_special_fields = Some(
                            fields
                                .iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect(),
                        );
                    }
                    if let Some(always) =
                        rule.get("always_include_in_dto").and_then(|v| v.as_array())
                    {
                        always_include_in_dto = Some(
                            always
                                .iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect(),
                        );
                    }
                }
            }

            let parsed_columns = create_table
                .columns
                .iter()
                .map(|col| {
                    let is_nullable = !col
                        .options
                        .iter()
                        .any(|opt| matches!(opt.option, sqlparser::ast::ColumnOption::NotNull));
                    TableColumn {
                        name: col.name.value.clone(),
                        sql_type: col.data_type.to_string(),
                        is_nullable,
                    }
                })
                .collect();

            return Ok(TableSchema {
                table_name: table_name.clone(),
                columns: parsed_columns,
                module_name: table_name.clone(),
                struct_name: to_pascal_case(&table_name),
                create_special_fields,
                update_special_fields,
                always_include_in_dto,
            });
        }
    }

    Err("No valid CREATE TABLE statement found".into())
}
