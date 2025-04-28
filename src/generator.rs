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
        generate_dto(table, output_dir)?;
        generate_repository(table, output_dir)?;
        generate_service(table, output_dir)?;

        // Create mod.rs for domain
        let base = Path::new(output_dir).join(&table.module_name);
        fs::create_dir_all(base.join("domain"))?;

        fs::write(
            base.join("domain/mod.rs"),
            "pub mod model;\npub mod repository;\npub mod service;\n",
        )?;

        // domain's super
        generate_routes(table, output_dir)?;
        generate_handlers(table, output_dir)?;
        generate_services(table, output_dir)?;
        generate_queries(table, output_dir)?;

        fs::write(
            base.join("mod.rs"),
            "pub mod domain;\n\
             pub mod dto;\n\
             pub mod handlers;\n\
             pub mod queries;\n\
             pub mod routes;\n\
             pub mod services;\n",
        )?;
    }

    // common
    // After all domains are generated, generate src/lib.rs
    generate_lib(schema, output_dir)?;
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

/// Generates the `lib.rs` file exposing modules.
pub fn generate_lib(
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

    let lib_code = tera.render("lib.tera", &context)?;
    fs::write(Path::new(output_dir).join("lib.rs"), lib_code)?;

    Ok(())
}

/// Converts a snake_case string to PascalCase.
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
        "datetime" | "timestamp" => "time::OffsetDateTime", // <-- NO Option here
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
            let rust_type = map_sql_type(&col.sql_type).to_string();

            if col.is_nullable {
                map.insert("rust_type", format!("Option<{}>", rust_type));
            } else {
                map.insert("rust_type", rust_type);
            }
            map.insert("name", col.name.clone());
            map
        })
        .collect::<Vec<_>>();

    context.insert("fields", &fields);

    let struct_code = tera.render("model.tera", &context)?;

    let model_path = domain_dir.join("model.rs");
    fs::write(model_path, struct_code)?;

    Ok(())
}

/// Generates the `dto.rs` file for the table schema.
pub fn generate_dto(
    schema: &TableSchema,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let dto_path = Path::new(output_dir)
        .join(&schema.module_name)
        .join("dto.rs");

    let tera = Tera::new("templates/**/*")?;
    let mut context = Context::new();

    let struct_name = &schema.struct_name;
    context.insert("struct_name", struct_name);

    let fields = schema
        .columns
        .iter()
        .map(|col| {
            let mut map = HashMap::new();
            let base_type = map_sql_type(&col.sql_type);
            let rust_type = if col.is_nullable {
                format!("Option<{}>", base_type)
            } else {
                base_type.to_string()
            };

            let is_optional = rust_type.starts_with("Option<");

            map.insert("name", col.name.clone());
            map.insert("ty", rust_type);
            map.insert("is_optional", is_optional.to_string());
            let is_datetime = col.sql_type.to_lowercase() == "timestamp"
                || col.sql_type.to_lowercase() == "datetime";
            map.insert("is_datetime", is_datetime.to_string());
            map
        })
        .collect::<Vec<_>>();
    context.insert("fields", &fields);

    let create_fields = schema
        .columns
        .iter()
        .filter(|col| {
            let skip = schema.create_special_fields.as_deref().unwrap_or(&[]);
            let always_include = schema.always_include_in_dto.as_deref().unwrap_or(&[]);
            !skip.iter().any(|field| field == &col.name) || always_include.contains(&col.name)
        })
        .map(|col| {
            let mut map = HashMap::new();
            let base_type = map_sql_type(&col.sql_type);
            let rust_type = if col.is_nullable {
                format!("Option<{}>", base_type)
            } else {
                base_type.to_string()
            };

            map.insert("name", col.name.clone());
            map.insert("ty", rust_type);
            let is_datetime = col.sql_type.to_lowercase() == "timestamp"
                || col.sql_type.to_lowercase() == "datetime";
            map.insert("is_datetime", is_datetime.to_string());
            let is_optional = base_type.starts_with("Option<") || col.is_nullable;
            map.insert("is_optional", is_optional.to_string());
            map
        })
        .collect::<Vec<_>>();
    context.insert("create_fields", &create_fields);

    let update_fields = schema
        .columns
        .iter()
        .filter(|col| {
            let skip = schema.update_special_fields.as_deref().unwrap_or(&[]);
            let always_include = schema.always_include_in_dto.as_deref().unwrap_or(&[]);
            !skip.iter().any(|field| field == &col.name) || always_include.contains(&col.name)
        })
        .map(|col| {
            let mut map = HashMap::new();
            let base_type = map_sql_type(&col.sql_type);
            let always_include = schema.always_include_in_dto.as_deref().unwrap_or(&[]);

            let rust_type = if always_include.contains(&col.name) {
                base_type.to_string()
            } else {
                format!("Option<{}>", base_type)
            };

            map.insert("name", col.name.clone());
            map.insert("ty", rust_type);
            map.insert(
                "always_include",
                (always_include.contains(&col.name)).to_string(),
            );
            let is_datetime = col.sql_type.to_lowercase() == "timestamp"
                || col.sql_type.to_lowercase() == "datetime";
            map.insert("is_datetime", is_datetime.to_string());
            let is_optional = !always_include.contains(&col.name);
            map.insert("is_optional", is_optional.to_string());
            map
        })
        .collect::<Vec<_>>();
    context.insert("update_fields", &update_fields);

    // no enums insertion anymore

    let dto_code = tera.render("dto.tera", &context)?;
    fs::write(dto_path, dto_code)?;

    Ok(())
}

/// Generates the `domain/repository.rs` file for the table schema.
fn generate_repository(
    schema: &TableSchema,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let domain_dir = Path::new(output_dir)
        .join(&schema.module_name)
        .join("domain");
    fs::create_dir_all(&domain_dir)?;

    let tera = Tera::new("templates/**/*")?;
    let mut context = Context::new();

    context.insert("struct_name", &schema.struct_name);
    context.insert("table_name", &schema.table_name);
    context.insert("module_name", &schema.module_name);

    let repository_code = tera.render("repository.tera", &context)?;

    let repository_path = domain_dir.join("repository.rs");
    fs::write(repository_path, repository_code)?;

    Ok(())
}

/// Generates the `domain/service.rs` file for the table schema.
fn generate_service(
    schema: &TableSchema,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let domain_dir = Path::new(output_dir)
        .join(&schema.module_name)
        .join("domain");
    fs::create_dir_all(&domain_dir)?;

    let tera = Tera::new("templates/**/*")?;
    let mut context = Context::new();

    context.insert("struct_name", &schema.struct_name);
    // Insert module_name instead of table_name for service template
    context.insert("module_name", &schema.module_name);

    let service_code = tera.render("service.tera", &context)?;

    let service_path = domain_dir.join("service.rs");
    fs::write(service_path, service_code)?;

    Ok(())
}

/// Generates the `routes.rs` file for the table schema.
fn generate_routes(
    schema: &TableSchema,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let base_dir = Path::new(output_dir).join(&schema.module_name);
    fs::create_dir_all(&base_dir)?;

    let tera = Tera::new("templates/**/*")?;
    let mut context = Context::new();

    context.insert("struct_name", &schema.struct_name);
    context.insert("module_name", &schema.module_name);

    let routes_code = tera.render("routes.tera", &context)?;

    let routes_path = base_dir.join("routes.rs");
    fs::write(routes_path, routes_code)?;

    Ok(())
}

/// Generates the `handlers.rs` file for the table schema.
fn generate_handlers(
    schema: &TableSchema,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let base_dir = Path::new(output_dir).join(&schema.module_name);
    fs::create_dir_all(&base_dir)?;

    let tera = Tera::new("templates/**/*")?;
    let mut context = Context::new();

    context.insert("struct_name", &schema.struct_name);
    context.insert("module_name", &schema.module_name);

    let handlers_code = tera.render("handlers.tera", &context)?;

    let handlers_path = base_dir.join("handlers.rs");
    fs::write(handlers_path, handlers_code)?;

    Ok(())
}

/// Generates the `services.rs` file for the table schema.
fn generate_services(
    schema: &TableSchema,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let base_dir = Path::new(output_dir).join(&schema.module_name);
    fs::create_dir_all(&base_dir)?;

    let tera = Tera::new("templates/**/*")?;
    let mut context = Context::new();

    context.insert("struct_name", &schema.struct_name);
    context.insert("module_name", &schema.module_name);

    let services_code = tera.render("services.tera", &context)?;

    let services_path = base_dir.join("services.rs");
    fs::write(services_path, services_code)?;

    Ok(())
}

/// Generates the `queries.rs` file for the table schema.
fn generate_queries(
    schema: &TableSchema,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let base_dir = Path::new(output_dir).join(&schema.module_name);
    fs::create_dir_all(&base_dir)?;

    let tera = Tera::new("templates/**/*")?;
    let mut context = Context::new();

    context.insert("struct_name", &schema.struct_name);
    context.insert("module_name", &schema.module_name);
    context.insert("table_name", &schema.table_name);

    // Prepare select_fields with only names (for backward compatibility)
    let select_fields = schema
        .columns
        .iter()
        .map(|col| col.name.clone())
        .collect::<Vec<_>>();
    context.insert("select_fields", &select_fields);

    // Prepare insert_fields as a list of maps containing name
    let insert_fields = schema
        .columns
        .iter()
        .filter(|col| {
            let skip = schema.create_special_fields.as_deref().unwrap_or(&[]);
            !skip.iter().any(|field| field == &col.name)
        })
        .map(|col| {
            let mut map = HashMap::new();
            map.insert("name", col.name.clone());
            map
        })
        .collect::<Vec<_>>();
    context.insert("insert_fields", &insert_fields);

    // Prepare update_fields with is_datetime and correct always_include_in_dto handling for optionality
    let update_fields = schema
        .columns
        .iter()
        .filter(|col| {
            let skip = schema.update_special_fields.as_deref().unwrap_or(&[]);
            !skip.iter().any(|field| field == &col.name)
        })
        .map(|col| {
            let mut map = HashMap::new();
            let always_include = schema.always_include_in_dto.as_deref().unwrap_or(&[]);

            let is_optional = !always_include.contains(&col.name);
            let is_datetime = col.sql_type.to_lowercase() == "timestamp"
                || col.sql_type.to_lowercase() == "datetime";

            map.insert("name", col.name.clone());
            map.insert("is_optional", is_optional.to_string());
            map.insert("is_datetime", is_datetime.to_string());
            map
        })
        .collect::<Vec<_>>();
    context.insert("update_fields", &update_fields);

    let queries_code = tera.render("queries.tera", &context)?;

    fs::write(base_dir.join("queries.rs"), queries_code)?;

    Ok(())
}
