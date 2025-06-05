# domain_codegen

This project provides a code generator specifically designed for the [clean_axum_demo](https://github.com/sukjaelee/clean_axum_demo) project.
It automatically generates the domain layer structure under `gen`, which you can copy and customize as needed.

```plaintext
/gen
├── src
│ │ ├── <domain>/ # Replace with: auth, user, device, file, etc.
│ │ ├── mod.rs # Module entry point
│ │ ├── domain/ # Domain logic: models, traits
│ │ │ ├── mod.rs
│ │ │ ├── model.rs
│ │ │ ├── repository.rs
│ │ │ └── service.rs
│ │ ├── handlers.rs # HTTP handlers
│ │ ├── routes.rs # Route definitions
│ │ ├── queries.rs # SQLx query logic
│ │ ├── dto.rs # Data Transfer Objects
│ │ └── service.rs # Infrastructure-layer service implementations
├── tests/
│   ├── test_<domain>_routes.rs # Integration and API tests
```

> When adding a new domain module, be sure to register it in the following files:
>
> - `src/lib.rs`
> - `src/app.rs`
> - `src/common/app_state.rs`
> - `src/common/bootstrap.rs`

## 📦 Usage

### Run with defaults:

```bash
cargo run
```

---

## 📄 How Code Generation Works

- `gen/domain.sql`:  
  Defines the SQL schema for your domain tables (e.g., `todos`, `devices`).  
  The code generator parses this file to understand table columns, types, and constraints.

- `gen/domain.rules.json`:  
  Provides generation rules for each table, including:
  - `module_name`: folder name (e.g., `todo`)
  - `struct_name`: Rust struct name (e.g., `Todo`)
  - `create_special_fields`: fields that should be excluded when generating the Create DTO and insert statements (e.g., `id`, `created_at`, `modified_at`)
  - `update_special_fields`: fields that should be excluded when generating the Update DTO and update statements (e.g., `id`, `created_at`, `created_by`, `modified_at`)
  - `always_include_in_dto`: fields that should always be required (not `Option`) in DTOs, even during update (e.g., `modified_by`)

These two files drive the entire domain code generation process automatically.

Generated Rust code will be created under the `gen/src/` directory, organized by domain module.

---

## 🧩 Tera Templates

- `templates/` folder contains `.tera` template files for generating Rust code.
- Each `.tera` file (e.g., `model.tera`, `dto.tera`, `repository.tera`) defines how a corresponding Rust file should be created.
- These templates are rendered dynamically using table schema information from `domain.sql` and `domain.rules.json`.

For more details about Tera syntax, see the [Tera crate documentation](https://docs.rs/tera/latest/tera/).

---

### 🏷️ Template Variables

Inside `.tera` files, you can use placeholders that get replaced during code generation:

- `{{ struct_name }}` → Struct name for the domain (e.g., `Todo`)
- `{{ module_name }}` → Module name (e.g., `todo`)
- `{{ table_name }}` → Database table name (e.g., `todos`)
- `{{ select_fields }}` → List of fields used in SQL `SELECT` statements
- `{{ insert_fields }}` → List of fields used in SQL `INSERT` statements

Control structures like `{% for field in fields %}` and `{% if condition %}` are used to dynamically repeat or control parts of the template output.

---

## 🛠 VS Code Tips for Tera Templates

To avoid unwanted auto-formatting of `.tera` files in VS Code, disable format-on-save:

Create or edit `.vscode/settings.json` in your project with the following:

```json
{
  "[tera]": {
    "editor.formatOnSave": false
  }
}
```
