mod generator;
mod parser;

use generator::to_pascal_case;

use clap::Parser;
use serde_json::Value;

/// CLI tool for generating domain code from SQL schema
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Input path to domain.sql
    #[arg(short, long, default_value = "gen/domain.sql")]
    input: String,

    /// Output folder path
    #[arg(short, long, default_value = "gen/src")]
    output: String,

    /// Input path to domain.rules.json
    #[arg(long, default_value = "gen/domain.rules.json")]
    rules: String,
}

fn main() {
    let args = Args::parse();
    std::fs::create_dir_all(&args.output).expect("Failed to create output directory");

    let rules_text =
        std::fs::read_to_string(&args.rules).expect("Failed to read domain.rules.json");
    let rules: Value =
        serde_json::from_str(&rules_text).expect("Failed to parse domain.rules.json");

    let mut schema =
        parser::parse_sql_file(&args.input, Some(&rules)).expect("Failed to parse domain.sql");

    if let Some(rule) = rules.get(&schema.table_name) {
        if let Some(module_name) = rule.get("module_name").and_then(|v| v.as_str()) {
            schema.module_name = module_name.to_string();
        }
        if let Some(struct_name) = rule.get("struct_name").and_then(|v| v.as_str()) {
            schema.struct_name = struct_name.to_string();
        }
    } else {
        schema.module_name = schema.table_name.clone();
        schema.struct_name = to_pascal_case(&schema.table_name);
    }

    generator::generate_code(&[schema], &args.output).expect("Code generation failed");
}
