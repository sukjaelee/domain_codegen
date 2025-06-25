use crate::parser::TableSchema;
use std::collections::HashMap;
use std::{fs, path::Path};
use tera::{Context, Tera};

/// Generates all domain, controller, and common modules based on the provided table schema.
pub fn generate_code(
    schema: &[TableSchema],
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    for table in schema {
        // domain
        generate_model(table, output_dir)?;
        generate_repository(table, output_dir)?;
        generate_service(table, output_dir)?;

        // dto, routes, handlers, services, queries, tests
        generate_dto(table, output_dir)?;

        // api
        generate_routes(table, output_dir)?;
        generate_handlers(table, output_dir)?;

        // infra
        generate_impl_service(table, output_dir)?;
        generate_impl_repository(table, output_dir)?;

        // tests
        generate_tests(table, output_dir)?;
    }

    // common
    // After all domains are generated, generate src/domains.rs
    generate_domains(schema, output_dir)?;
    // feature
    generate_feature(schema, output_dir)?;
    generate_app(schema, output_dir)?;
    generate_app_state(schema, output_dir)?;
    generate_bootstrap(schema, output_dir)?;

    Ok(())
}

/// Generates the `common/bootstrap.rs` file, wiring services into AppState.
pub fn generate_bootstrap(
    schemas: &[TableSchema],
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let tera = Tera::new("templates/**/*")?;
    let mut context = Context::new();

    let modules = schemas
        .iter()
        .map(|s| {
            let mut map = std::collections::HashMap::new();
            map.insert("module", s.module_name.clone());
            map.insert("struct_name", s.struct_name.clone());
            map
        })
        .collect::<Vec<_>>();
    context.insert("modules", &modules);

    let bootstrap_code = tera.render("bootstrap.tera", &context)?;
    let bootstrap_dir = Path::new(output_dir).join("common");
    fs::create_dir_all(&bootstrap_dir)?;
    fs::write(bootstrap_dir.join("bootstrap.rs"), bootstrap_code)?;

    Ok(())
}

/// Generates the `common/app_state.rs` struct for holding application state.
pub fn generate_app_state(
    schemas: &[TableSchema],
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let tera = Tera::new("templates/**/*")?;
    let mut context = Context::new();

    let modules = schemas
        .iter()
        .map(|s| s.module_name.clone())
        .collect::<Vec<_>>();
    context.insert("modules", &modules);

    let app_state_code = tera.render("app_state.tera", &context)?;
    let app_state_dir = Path::new(output_dir).join("common");
    fs::create_dir_all(&app_state_dir)?;
    fs::write(app_state_dir.join("app_state.rs"), app_state_code)?;

    Ok(())
}

/// Generates the `app.rs` file wiring routes and Swagger docs.
pub fn generate_app(
    schemas: &[TableSchema],
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let tera = Tera::new("templates/**/*")?;
    let mut context = Context::new();

    let modules = schemas
        .iter()
        .map(|s| s.module_name.clone())
        .collect::<Vec<_>>();
    context.insert("modules", &modules);

    let app_code = tera.render("app.tera", &context)?;
    fs::write(Path::new(output_dir).join("app.rs"), app_code)?;

    Ok(())
}

/// Generates the `domains.rs` file exposing modules.
pub fn generate_domains(
    schemas: &[TableSchema],
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let tera = Tera::new("templates/**/*")?;
    let mut context = Context::new();

    let modules = schemas
        .iter()
        .map(|s| s.module_name.clone())
        .collect::<Vec<_>>();
    context.insert("modules", &modules);

    let domains_code = tera.render("domains.tera", &context)?;
    fs::write(Path::new(output_dir).join("domains.rs"), domains_code)?;

    Ok(())
}

pub fn generate_feature(
    schemas: &[TableSchema],
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let tera = Tera::new("templates/**/*")?;

    let mut context = Context::new();

    let schema = &schemas[0]; // Assuming we only generate one feature per module
    // Insert basic context values
    context.insert("struct_name", &schema.struct_name);
    context.insert("module_name", &schema.module_name);

    let feature_code = tera.render("feature.tera", &context)?;

    let feature_file_name = format!("{}.rs", schema.module_name.to_lowercase());
    fs::write(Path::new(output_dir).join(feature_file_name), feature_code)?;

    Ok(())
}

/// Converts a snake_case string to PascalCase.
#[allow(non_snake_case)]
pub fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut c = part.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}

/// Maps SQL column types to equivalent Rust types.
fn map_sql_type(sql_type: &str) -> &'static str {
    let ty = sql_type.to_lowercase();

    match ty.as_str() {
        s if s.starts_with("char") || s.starts_with("varchar") || s.ends_with("text") => "String",
        "uuid" => "uuid::Uuid",
        "bool" | "boolean" => "bool",
        "tinyint" => "i8",
        "smallint" => "i16",
        "mediumint" | "int" | "integer" => "i32",
        "bigint" => "i64",
        "decimal" | "numeric" => "f64",
        "float" => "f32",
        "double" => "f64",
        "date" => "time::Date",
        "datetime" | "timestamp" | "timestamptz" => "DateTime<Utc>", // <-- NO Option here
        "time" => "String",
        "year" => "i16",
        s if s.starts_with("enum") || s.starts_with("set") => "String",
        s if s.starts_with("binary") || s.starts_with("varbinary") => "Vec<u8>",
        s if s.ends_with("blob") => "Vec<u8>",
        "json" => "serde_json::Value",
        _ => "String",
    }
}

/// Generates the `domain/model.rs` file for the table schema.
fn generate_model(
    schema: &TableSchema,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let domain_dir = Path::new(output_dir)
        .join("domains")
        .join(&schema.module_name)
        .join("domain");
    fs::create_dir_all(&domain_dir)?;

    let tera = Tera::new("templates/**/*")?;

    let mut context = Context::new();
    context.insert("table_name", &schema.table_name);
    context.insert("struct_name", &schema.struct_name);

    let fields = schema
        .columns
        .iter()
        .map(|col| {
            let mut map = HashMap::new();
            // Determine Rust type, using DateTime<Utc> for timestamps
            let base_type = col.sql_type.to_lowercase();
            let rust_type = if base_type == "timestamp"
                || base_type == "timestamptz"
                || base_type == "datetime"
            {
                "DateTime<Utc>".to_string()
            } else {
                map_sql_type(&col.sql_type).to_string()
            };

            // Wrap in Option<> if the column is nullable
            let final_type = if col.is_nullable {
                format!("Option<{}>", rust_type)
            } else {
                rust_type.clone()
            };

            map.insert("rust_type", final_type);
            map.insert("name", col.name.clone());
            map
        })
        .collect::<Vec<_>>();

    context.insert("fields", &fields);

    // If any field uses DateTime<Utc>, ensure the template sees it
    context.insert("use_chrono", &true);

    let model_code = tera.render("model.tera", &context)?;

    fs::write(domain_dir.join("model.rs"), model_code)?;
    Ok(())
}

/// Generates the `dto.rs` file for the table schema.
pub fn generate_dto(
    schema: &TableSchema,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let struct_name = &schema.struct_name;

    // Determine the path where the DTO file will be written
    let dto_path = Path::new(output_dir)
        .join("domains")
        .join(&schema.module_name)
        .join("dto");

    fs::create_dir_all(&dto_path)?;

    let tera = Tera::new("templates/**/*")?;
    let mut context = Context::new();

    // Insert the struct_name, module_name into context
    context.insert("struct_name", struct_name);
    context.insert("module_name", &schema.module_name);

    // Retrieve rule-based field lists or fallback to empty vectors
    let create_skip = schema.create_special_fields.as_deref().unwrap_or(&[]);
    let update_skip = schema.update_special_fields.as_deref().unwrap_or(&[]);
    let always_include = schema.always_include_in_dto.as_deref().unwrap_or(&[]);

    // Build 'fields' context: includes name, type, datetime and optional flags
    let fields = schema
        .columns
        .iter()
        .map(|col| {
            let mut map = HashMap::new();
            let sql_lower = col.sql_type.to_lowercase();

            // Map SQL datetime types to DateTime<Utc>, others via map_sql_type
            let base_type = if sql_lower == "timestamp"
                || sql_lower == "timestamptz"
                || sql_lower == "datetime"
            {
                "DateTime<Utc>".to_string()
            } else {
                map_sql_type(&col.sql_type).to_string()
            };

            // Wrap in Option<> if the column is nullable
            let ty = if col.is_nullable {
                format!("Option<{}>", base_type)
            } else {
                base_type.clone()
            };

            map.insert("name", col.name.clone());
            map.insert("ty", ty);
            // Flag indicating whether this is a datetime column
            let is_dt =
                (sql_lower == "timestamp" || sql_lower == "timestamptz" || sql_lower == "datetime")
                    .to_string();
            map.insert("is_datetime", is_dt);
            // Flag indicating whether this field is optional
            map.insert("is_optional", col.is_nullable.to_string());
            map
        })
        .collect::<Vec<_>>();
    context.insert("fields", &fields);

    // Build 'create_fields' context: exclude rule-based skip fields
    let create_fields = schema
        .columns
        .iter()
        .filter(|col| !create_skip.contains(&col.name))
        .map(|col| {
            let mut map = HashMap::new();
            let sql_lower = col.sql_type.to_lowercase();
            let base_type = if sql_lower == "timestamp"
                || sql_lower == "timestamptz"
                || sql_lower == "datetime"
            {
                "DateTime<Utc>".to_string()
            } else {
                map_sql_type(&col.sql_type).to_string()
            };
            let ty = if col.is_nullable {
                format!("Option<{}>", base_type)
            } else {
                base_type.clone()
            };
            map.insert("name", col.name.clone());
            map.insert("ty", ty);
            let is_dt =
                (sql_lower == "timestamp" || sql_lower == "timestamptz" || sql_lower == "datetime")
                    .to_string();
            map.insert("is_datetime", is_dt);
            map.insert("is_optional", col.is_nullable.to_string());
            map
        })
        .collect::<Vec<_>>();
    context.insert("create_fields", &create_fields);

    // Build 'update_fields' context: exclude rule-based skip fields; always include those in always_include without Option<>
    let update_fields = schema
        .columns
        .iter()
        .filter(|col| !update_skip.contains(&col.name))
        .map(|col| {
            let mut map = HashMap::new();
            let sql_lower = col.sql_type.to_lowercase();
            let base_type = if sql_lower == "timestamp"
                || sql_lower == "timestamptz"
                || sql_lower == "datetime"
            {
                "DateTime<Utc>".to_string()
            } else {
                map_sql_type(&col.sql_type).to_string()
            };
            // If always include, use base_type; else if not skip, use Option<>
            let ty = if always_include.contains(&col.name) {
                base_type.clone()
            } else {
                format!("Option<{}>", base_type)
            };
            map.insert("name", col.name.clone());
            map.insert("ty", ty);
            let is_dt =
                (sql_lower == "timestamp" || sql_lower == "timestamptz" || sql_lower == "datetime")
                    .to_string();
            map.insert("is_datetime", is_dt);
            let is_opt = (!always_include.contains(&col.name)).to_string();
            map.insert("is_optional", is_opt);
            map
        })
        .collect::<Vec<_>>();
    context.insert("update_fields", &update_fields);

    // Render the template and write to file
    let dto_code = tera.render("dto.tera", &context)?;
    let dto_file_name = format!("{}_dto.rs", struct_name.to_lowercase());

    fs::write(dto_path.join(dto_file_name), dto_code)?;

    Ok(())
}

/// Generates the `domain/repository.rs` file for the table schema.
fn generate_repository(
    schema: &TableSchema,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let domain_dir = Path::new(output_dir)
        .join("domains")
        .join(&schema.module_name)
        .join("domain");
    fs::create_dir_all(&domain_dir)?;

    let tera = Tera::new("templates/**/*")?;
    let mut context = Context::new();

    context.insert("struct_name", &schema.struct_name);
    context.insert("table_name", &schema.table_name);
    context.insert("module_name", &schema.module_name);

    let repository_code = tera.render("repository.tera", &context)?;
    fs::write(domain_dir.join("repository.rs"), repository_code)?;

    Ok(())
}

/// Generates the `domain/service.rs` file for the table schema.
fn generate_service(
    schema: &TableSchema,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let domain_dir = Path::new(output_dir)
        .join("domains")
        .join(&schema.module_name)
        .join("domain");
    fs::create_dir_all(&domain_dir)?;

    let tera = Tera::new("templates/**/*")?;
    let mut context = Context::new();

    context.insert("struct_name", &schema.struct_name);
    // Insert module_name instead of table_name for service template
    context.insert("module_name", &schema.module_name);

    let service_code = tera.render("service.tera", &context)?;
    fs::write(domain_dir.join("service.rs"), service_code)?;

    Ok(())
}

/// Generates the `routes.rs` file for the table schema.
fn generate_routes(
    schema: &TableSchema,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let base_dir = Path::new(output_dir)
        .join("domains")
        .join(&schema.module_name)
        .join("api");

    fs::create_dir_all(&base_dir)?;

    let tera = Tera::new("templates/**/*")?;
    let mut context = Context::new();

    context.insert("struct_name", &schema.struct_name);
    context.insert("module_name", &schema.module_name);

    let routes_code = tera.render("routes.tera", &context)?;
    fs::write(base_dir.join("routes.rs"), routes_code)?;

    Ok(())
}

/// Generates the `handlers.rs` file for the table schema.
fn generate_handlers(
    schema: &TableSchema,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let base_dir = Path::new(output_dir)
        .join("domains")
        .join(&schema.module_name)
        .join("api");

    fs::create_dir_all(&base_dir)?;

    let tera = Tera::new("templates/**/*")?;
    let mut context = Context::new();

    context.insert("struct_name", &schema.struct_name);
    context.insert("module_name", &schema.module_name);

    let handlers_code = tera.render("handlers.tera", &context)?;
    fs::write(base_dir.join("handlers.rs"), handlers_code)?;

    Ok(())
}

/// Generates the `services.rs` file for the table schema.
fn generate_impl_service(
    schema: &TableSchema,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let base_dir = Path::new(output_dir)
        .join("domains")
        .join(&schema.module_name)
        .join("infra");

    fs::create_dir_all(&base_dir)?;

    let tera = Tera::new("templates/**/*")?;
    let mut context = Context::new();

    context.insert("struct_name", &schema.struct_name);
    context.insert("module_name", &schema.module_name);

    let impl_service_code = tera.render("impl_service.tera", &context)?;

    fs::write(base_dir.join("impl_service.rs"), impl_service_code)?;

    Ok(())
}

/// Generates the `queries.rs` file for the table schema.
fn generate_impl_repository(
    schema: &TableSchema,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let base_dir = Path::new(output_dir)
        .join("domains")
        .join(&schema.module_name)
        .join("infra");
    fs::create_dir_all(&base_dir)?;

    let tera = Tera::new("templates/**/*")?;
    let mut context = Context::new();

    // Insert basic context values
    context.insert("struct_name", &schema.struct_name);
    context.insert("module_name", &schema.module_name);
    context.insert("table_name", &schema.table_name);

    // Prepare select_fields: list of column names
    let select_fields = schema
        .columns
        .iter()
        .map(|col| col.name.clone())
        .collect::<Vec<_>>();
    context.insert("select_fields", &select_fields);

    // Retrieve rule-based update skip list or fallback to empty slice
    let update_skip = schema.update_special_fields.as_deref().unwrap_or(&[]);

    // Prepare insert_fields: exclude id, created_at, and modified_at
    let insert_fields = schema
        .columns
        .iter()
        .filter(|col| !(col.name == "id" || col.name == "created_at" || col.name == "modified_at"))
        .map(|col| {
            let mut map = HashMap::new();
            map.insert("name", col.name.clone());
            // Flag indicating datetime columns for create binding
            let is_dt = (col.sql_type.to_lowercase() == "timestamp"
                || col.sql_type.to_lowercase() == "timestamptz"
                || col.sql_type.to_lowercase() == "datetime")
                .to_string();
            map.insert("is_datetime", is_dt);
            map
        })
        .collect::<Vec<_>>();
    context.insert("insert_fields", &insert_fields);

    // Prepare update_fields: filter out any column listed in update_skip
    let update_fields = schema
        .columns
        .iter()
        .filter(|col| !update_skip.contains(&col.name))
        .map(|col| {
            let mut map = HashMap::new();
            let is_dt = (col.sql_type.to_lowercase() == "timestamp"
                || col.sql_type.to_lowercase() == "timestamptz"
                || col.sql_type.to_lowercase() == "datetime")
                .to_string();
            // For update, columns in update_special_fields are skipped entirely
            // For modified_by, always include as non-Option; for others, wrap in Option
            let ty = if col.name == "modified_by" {
                // Use base type; Tera will detect datetime if needed
                if is_dt == "true" {
                    "DateTime<Utc>".to_string()
                } else {
                    map_sql_type(&col.sql_type).to_string()
                }
            } else {
                // Option<...>
                if is_dt == "true" {
                    format!("Option<DateTime<Utc>>")
                } else {
                    format!("Option<{}>", map_sql_type(&col.sql_type))
                }
            };
            map.insert("name", col.name.clone());
            map.insert("is_datetime", is_dt);
            map.insert("is_optional", (col.name != "modified_by").to_string());
            map.insert("ty", ty);
            map
        })
        .collect::<Vec<_>>();
    context.insert("update_fields", &update_fields);

    // Render and write file
    let impl_repository_code = tera.render("impl_repository.tera", &context)?;
    fs::write(base_dir.join("impl_repository.rs"), impl_repository_code)?;
    Ok(())
}

/// Generates the `test_{module_name}_routes.rs` file under the tests directory.
fn generate_tests(
    schema: &TableSchema,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let tests_dir = Path::new(output_dir).parent().unwrap().join("tests");
    fs::create_dir_all(&tests_dir)?;

    let tera = Tera::new("templates/**/*")?;
    let mut context = Context::new();
    context.insert("struct_name", &schema.struct_name);
    context.insert("module_name", &schema.module_name);

    // Prepare create_fields: exclude id, created_at, modified_at
    let create_fields = schema
        .columns
        .iter()
        .filter(|col| !(col.name == "id" || col.name == "created_at" || col.name == "modified_at"))
        .map(|col| col.name.clone())
        .collect::<Vec<_>>();
    context.insert("create_fields", &create_fields);

    // Prepare update_fields: exclude id, created_at, created_by? (Keep modified_by required)
    let update_fields = schema
        .columns
        .iter()
        .filter(|col| !(col.name == "id" || col.name == "created_at"))
        .map(|col| col.name.clone())
        .collect::<Vec<_>>();
    context.insert("update_fields", &update_fields);

    let test_code = tera.render("test_routes.tera", &context)?;
    let file_name = format!("test_{}_routes.rs", &schema.module_name);
    fs::write(tests_dir.join(file_name), test_code)?;
    Ok(())
}
