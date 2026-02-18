// Table schemas and validation

use std::collections::HashMap;
use once_cell::sync::Lazy;
use serde::Serialize;
use crate::error::ValidationError;

/// Represents the type of a database column
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnType {
    Int,
    String,
    DateTime,
    Json,
}

/// Specification for a single column in a table
#[derive(Debug, Clone)]
pub struct ColumnSpec {
    pub nullable: bool,
    pub column_type: ColumnType,
}

impl ColumnSpec {
    /// Create a new non-nullable column specification
    pub fn new(column_type: ColumnType) -> Self {
        Self {
            nullable: false,
            column_type,
        }
    }

    /// Create a new nullable column specification
    pub fn nullable(column_type: ColumnType) -> Self {
        Self {
            nullable: true,
            column_type,
        }
    }
}

/// Schema definition for a database table
#[derive(Debug, Clone)]
pub struct TableSchema {
    pub columns: HashMap<String, ColumnSpec>,
}

impl TableSchema {
    /// Create a new table schema
    pub fn new() -> Self {
        Self {
            columns: HashMap::new(),
        }
    }

    /// Add a column to the schema
    pub fn add_column(mut self, name: &str, spec: ColumnSpec) -> Self {
        self.columns.insert(name.to_string(), spec);
        self
    }

    /// Get the schema for a specific table by name
    pub fn for_table(table: &str) -> Option<&'static TableSchema> {
        SCHEMAS.get(table)
    }

    /// Validate that a value matches the schema
    /// 
    /// This method validates that:
    /// - All non-nullable columns are present
    /// - Non-nullable columns do not have null values
    /// - Column types match expected types
    /// - Nullable columns can be absent
    /// 
    /// # Requirements
    /// - 10.2: Check for missing non-nullable columns
    /// - 10.4: Validate column types match expected types
    /// - 10.6: Allow nullable columns to be absent
    pub fn validate<T: Serialize>(&self, value: &T) -> Result<(), ValidationError> {
        // Serialize the value to a JSON Value to inspect its structure
        let json_value = serde_json::to_value(value)
            .map_err(|e| ValidationError::InvalidRule(format!("Failed to serialize value: {}", e)))?;

        // Ensure the value is an object
        let obj = json_value.as_object()
            .ok_or_else(|| ValidationError::InvalidRule("Value must be an object".to_string()))?;

        // Check all non-nullable columns are present and have correct types
        for (col_name, col_spec) in &self.columns {
            match obj.get(col_name) {
                Some(value) => {
                    // Column is present, check if it's null
                    if value.is_null() {
                        if !col_spec.nullable {
                            return Err(ValidationError::NullValue(col_name.clone()));
                        }
                    } else {
                        // Column has a value, validate its type
                        self.validate_column_type(col_name, value, &col_spec.column_type)?;
                    }
                }
                None => {
                    // Column is missing
                    if !col_spec.nullable {
                        return Err(ValidationError::MissingColumn(col_name.clone()));
                    }
                    // Nullable columns can be absent
                }
            }
        }

        Ok(())
    }

    /// Validate that a JSON value matches the expected column type
    fn validate_column_type(
        &self,
        col_name: &str,
        value: &serde_json::Value,
        expected_type: &ColumnType,
    ) -> Result<(), ValidationError> {
        let matches = match expected_type {
            ColumnType::Int => value.is_i64() || value.is_u64(),
            ColumnType::String => value.is_string(),
            ColumnType::DateTime => value.is_string(), // DateTime is serialized as string
            ColumnType::Json => true, // JSON can be any valid JSON value
        };

        if !matches {
            return Err(ValidationError::TypeMismatch {
                column: col_name.to_string(),
                expected: format!("{:?}", expected_type),
            });
        }

        Ok(())
    }
}

// Static schemas for all tables
static SCHEMAS: Lazy<HashMap<&'static str, TableSchema>> = Lazy::new(|| {
    let mut schemas = HashMap::new();

    // logs table schema
    schemas.insert(
        "logs",
        TableSchema::new()
            .add_column("id", ColumnSpec::new(ColumnType::Int))
            .add_column("created_at", ColumnSpec::new(ColumnType::DateTime))
            .add_column("user", ColumnSpec::nullable(ColumnType::String))
            .add_column("network", ColumnSpec::nullable(ColumnType::String))
            .add_column("window", ColumnSpec::new(ColumnType::String))
            .add_column("type", ColumnSpec::new(ColumnType::String))
            .add_column("nick", ColumnSpec::nullable(ColumnType::String))
            .add_column("message", ColumnSpec::nullable(ColumnType::String)),
    );

    // logs_queue table schema (same as logs)
    schemas.insert(
        "logs_queue",
        TableSchema::new()
            .add_column("id", ColumnSpec::new(ColumnType::Int))
            .add_column("created_at", ColumnSpec::new(ColumnType::DateTime))
            .add_column("user", ColumnSpec::nullable(ColumnType::String))
            .add_column("network", ColumnSpec::nullable(ColumnType::String))
            .add_column("window", ColumnSpec::new(ColumnType::String))
            .add_column("type", ColumnSpec::new(ColumnType::String))
            .add_column("nick", ColumnSpec::nullable(ColumnType::String))
            .add_column("message", ColumnSpec::nullable(ColumnType::String)),
    );

    // logs_id_track table schema
    schemas.insert(
        "logs_id_track",
        TableSchema::new()
            .add_column("id", ColumnSpec::new(ColumnType::Int))
            .add_column("tid", ColumnSpec::new(ColumnType::Int)),
    );

    // push table schema (logs with recipient)
    schemas.insert(
        "push",
        TableSchema::new()
            .add_column("id", ColumnSpec::new(ColumnType::Int))
            .add_column("user", ColumnSpec::nullable(ColumnType::String))
            .add_column("network", ColumnSpec::nullable(ColumnType::String))
            .add_column("window", ColumnSpec::new(ColumnType::String))
            .add_column("type", ColumnSpec::new(ColumnType::String))
            .add_column("nick", ColumnSpec::nullable(ColumnType::String))
            .add_column("message", ColumnSpec::nullable(ColumnType::String))
            .add_column("recipient", ColumnSpec::new(ColumnType::String)),
    );

    // event_log table schema (same as push)
    schemas.insert(
        "event_log",
        TableSchema::new()
            .add_column("id", ColumnSpec::new(ColumnType::Int))
            .add_column("user", ColumnSpec::nullable(ColumnType::String))
            .add_column("network", ColumnSpec::nullable(ColumnType::String))
            .add_column("window", ColumnSpec::new(ColumnType::String))
            .add_column("type", ColumnSpec::new(ColumnType::String))
            .add_column("nick", ColumnSpec::nullable(ColumnType::String))
            .add_column("message", ColumnSpec::nullable(ColumnType::String))
            .add_column("recipient", ColumnSpec::new(ColumnType::String)),
    );

    // users table schema
    schemas.insert(
        "users",
        TableSchema::new()
            .add_column("nickname", ColumnSpec::new(ColumnType::String))
            .add_column("telegram_chat_id", ColumnSpec::nullable(ColumnType::Int))
            .add_column("hotwords", ColumnSpec::nullable(ColumnType::Json)),
    );

    // pm_table schema
    schemas.insert(
        "pm_table",
        TableSchema::new()
            .add_column("window", ColumnSpec::new(ColumnType::String))
            .add_column("nick", ColumnSpec::new(ColumnType::String)),
    );

    schemas
});
