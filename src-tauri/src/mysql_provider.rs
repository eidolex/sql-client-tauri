use crate::database_provider::DatabaseProvider;
use crate::db::{ColumnDefinition, DatabaseError, Filter, IndexDefinition, QueryResult, Sort};
use async_trait::async_trait;
use sqlx::mysql::{MySqlPoolOptions, MySqlRow};
use sqlx::{Column, MySql, Pool, Row, TypeInfo};
use std::collections::HashMap;

pub struct MysqlProvider {
    pool: Pool<MySql>,
}

impl MysqlProvider {
    pub async fn new(connection_string: &str) -> Result<Self, DatabaseError> {
        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect(connection_string)
            .await
            .map_err(DatabaseError::from)?;
        Ok(Self { pool })
    }
}

fn row_to_values(row: MySqlRow) -> Vec<serde_json::Value> {
    let mut values = Vec::new();
    for col in row.columns() {
        let col_name = col.name();
        let type_info = col.type_info();
        let type_name = type_info.name();

        let value = match type_name {
            "BOOLEAN" | "TINYINT(1)" => {
                // MySQL treats boolean as tinyint(1)
                let val: Option<i8> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val.map(|v| v != 0))
            }
            "TINYINT" | "SMALLINT" | "INT" | "INTEGER" | "MEDIUMINT" => {
                let val: Option<i32> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "BIGINT" => {
                let val: Option<i64> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "BIGINT UNSIGNED" => {
                let val: Option<u64> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "INT UNSIGNED" | "INTEGER UNSIGNED" => {
                let val: Option<u32> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "SMALLINT UNSIGNED" => {
                let val: Option<u16> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "TINYINT UNSIGNED" => {
                let val: Option<u8> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "FLOAT" => {
                let val: Option<f32> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "DOUBLE" | "REAL" => {
                let val: Option<f64> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "VARCHAR" | "CHAR" | "TEXT" | "TINYTEXT" | "MEDIUMTEXT" | "LONGTEXT" | "ENUM"
            | "SET" => {
                let val: Option<String> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "DATETIME" | "TIMESTAMP" => {
                let val: Option<chrono::NaiveDateTime> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "DATE" => {
                let val: Option<chrono::NaiveDate> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "TIME" => {
                let val: Option<chrono::NaiveTime> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "JSON" => {
                let val: Option<serde_json::Value> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            _ => {
                if let Ok(val) = row.try_get::<String, _>(col_name) {
                    serde_json::Value::String(val)
                } else {
                    serde_json::Value::String(format!("Unsupported Type: {}", type_name))
                }
            }
        };
        values.push(value);
    }
    values
}

#[async_trait]
impl DatabaseProvider for MysqlProvider {
    async fn list_databases(&self) -> Result<Vec<String>, DatabaseError> {
        let rows = sqlx::query("SHOW DATABASES;")
            .fetch_all(&self.pool)
            .await
            .map_err(DatabaseError::from)?;

        let databases: Vec<String> = rows.iter().map(|row| row.get("Database")).collect();
        Ok(databases)
    }

    async fn list_tables(&self) -> Result<Vec<String>, DatabaseError> {
        let rows = sqlx::query("SHOW TABLES;")
            .fetch_all(&self.pool)
            .await
            .map_err(DatabaseError::from)?;

        let tables: Vec<String> = rows.iter().map(|row: &MySqlRow| row.get(0)).collect();
        Ok(tables)
    }

    async fn get_table_data(
        &self,
        table_name: String,
        limit: i64,
        offset: i64,
        filters: Vec<Filter>,
        sorts: Vec<Sort>,
    ) -> Result<QueryResult, DatabaseError> {
        // MySQL doesn't need explicit casting as much as Postgres, but we might need some.
        // For now, let's try without complex type mapping for filters.

        let mut where_clauses = Vec::new();
        let mut query_params = Vec::new();
        // MySQL uses ? for parameters

        for filter in &filters {
            if !filter
                .field
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_')
            {
                continue;
            }

            let field = format!("`{}`", filter.field);

            match filter.operator.as_str() {
                "=" | ">=" | "<=" | ">" | "<" => {
                    where_clauses.push(format!("{} {} ?", field, filter.operator));
                    query_params.push(filter.value.clone());
                }
                "contain" => {
                    where_clauses.push(format!("{} LIKE ?", field));
                    query_params.push(format!("%{}%", filter.value));
                }
                "start with" => {
                    where_clauses.push(format!("{} LIKE ?", field));
                    query_params.push(format!("{}%", filter.value));
                }
                "end with" => {
                    where_clauses.push(format!("{} LIKE ?", field));
                    query_params.push(format!("%{}", filter.value));
                }
                "not null" => {
                    where_clauses.push(format!("{} IS NOT NULL", field));
                }
                "is null" => {
                    where_clauses.push(format!("{} IS NULL", field));
                }
                _ => {}
            }
        }

        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let mut order_clauses = Vec::new();
        for sort in &sorts {
            if !sort.field.chars().all(|c| c.is_alphanumeric() || c == '_') {
                continue;
            }
            let direction = if sort.order.to_uppercase() == "DESC" {
                "DESC"
            } else {
                "ASC"
            };
            order_clauses.push(format!("`{}` {}", sort.field, direction));
        }

        let order_sql = if order_clauses.is_empty() {
            String::new()
        } else {
            format!("ORDER BY {}", order_clauses.join(", "))
        };

        let count_query = format!("SELECT COUNT(*) FROM `{}` {}", table_name, where_sql);
        let mut count_q = sqlx::query_as::<_, (i64,)>(&count_query);
        for param in &query_params {
            count_q = count_q.bind(param);
        }
        let count_row = count_q
            .fetch_one(&self.pool)
            .await
            .map_err(DatabaseError::from)?;
        let total_rows = count_row.0;

        let query = format!(
            "SELECT * FROM `{}` {} {} LIMIT ? OFFSET ?",
            table_name, where_sql, order_sql
        );

        let mut q = sqlx::query(&query);
        for param in &query_params {
            q = q.bind(param);
        }
        q = q.bind(limit).bind(offset);

        let rows = q.fetch_all(&self.pool).await.map_err(DatabaseError::from)?;

        let columns = if let Some(first_row) = rows.first() {
            first_row
                .columns()
                .iter()
                .map(|col| col.name().to_string())
                .collect()
        } else {
            Vec::new()
        };

        let result_rows: Vec<Vec<serde_json::Value>> =
            rows.into_iter().map(row_to_values).collect();

        Ok(QueryResult {
            columns,
            rows: result_rows,
            total_rows: Some(total_rows),
        })
    }

    async fn get_table_structure(
        &self,
        table_name: String,
    ) -> Result<Vec<ColumnDefinition>, DatabaseError> {
        let query = "
            SELECT 
                COLUMN_NAME as column_name,
                COLUMN_TYPE as data_type,
                IS_NULLABLE as is_nullable,
                COLUMN_DEFAULT as column_default,
                COLUMN_COMMENT as comment,
                CAST(NULL AS CHAR) as foreign_key
            FROM information_schema.COLUMNS
            WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ?
            ORDER BY ORDINAL_POSITION;
        ";

        // Note: Foreign key retrieval in MySQL is a bit more complex, often requiring joining KEY_COLUMN_USAGE.
        // For simplicity in this first pass, we might skip detailed FK info or add it if easy.
        // Let's try to get basic structure first.

        let rows = sqlx::query(query)
            .bind(&table_name)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DatabaseError {
                message: format!("Failed to get structure for {}: {}", table_name, e),
            })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(ColumnDefinition {
                column_name: row.get("column_name"),
                data_type: {
                    let bytes: Vec<u8> = row.get("data_type");
                    String::from_utf8_lossy(&bytes).to_string()
                },
                is_nullable: row.get("is_nullable"),
                column_default: {
                    let bytes: Option<Vec<u8>> = row.get("column_default");
                    bytes.map(|b| String::from_utf8_lossy(&b).to_string())
                },
                comment: {
                    let bytes: Option<Vec<u8>> = row.get("comment");
                    bytes.map(|b| String::from_utf8_lossy(&b).to_string())
                },
                foreign_key: None, // TODO: Implement FK for MySQL
            });
        }

        Ok(results)
    }

    async fn get_table_indexes(
        &self,
        table_name: String,
    ) -> Result<Vec<IndexDefinition>, DatabaseError> {
        let query = format!("SHOW INDEX FROM `{}`", table_name);

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DatabaseError {
                message: format!("Failed to get indexes for {}: {}", table_name, e),
            })?;

        let mut results = Vec::new();
        // SHOW INDEX returns one row per column in index. We need to aggregate.
        // But for now, let's just return raw rows or simplify.
        // The trait expects one entry per index.

        // Group by Key_name
        let mut index_map: HashMap<String, IndexDefinition> = HashMap::new();

        for row in rows {
            let index_name: String = row.get("Key_name");
            let non_unique: i32 = row.get("Non_unique");
            let column_name: String = row.get("Column_name");
            let index_type_bytes: Vec<u8> = row.get("Index_type");
            let index_type = String::from_utf8_lossy(&index_type_bytes).to_string();
            let comment_bytes: Vec<u8> = row.get("Index_comment");
            let comment = String::from_utf8_lossy(&comment_bytes).to_string();

            let entry = index_map
                .entry(index_name.clone())
                .or_insert(IndexDefinition {
                    index_name: index_name.clone(),
                    index_algorithm: index_type,
                    is_unique: non_unique == 0,
                    is_primary: index_name == "PRIMARY",
                    column_names: String::new(),
                    condition: None,
                    comment: if comment.is_empty() {
                        None
                    } else {
                        Some(comment)
                    },
                });

            if !entry.column_names.is_empty() {
                entry.column_names.push_str(", ");
            }
            entry.column_names.push_str(&column_name);
        }

        for (_, index) in index_map {
            results.push(index);
        }

        Ok(results)
    }

    async fn execute_query(&self, query: String) -> Result<QueryResult, DatabaseError> {
        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(DatabaseError::from)?;

        let columns = if let Some(first_row) = rows.first() {
            first_row
                .columns()
                .iter()
                .map(|col| col.name().to_string())
                .collect()
        } else {
            Vec::new()
        };

        let result_rows: Vec<Vec<serde_json::Value>> =
            rows.into_iter().map(row_to_values).collect();

        Ok(QueryResult {
            columns,
            rows: result_rows,
            total_rows: None,
        })
    }

    async fn get_database_schema(&self) -> Result<HashMap<String, Vec<String>>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT TABLE_NAME, COLUMN_NAME FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = DATABASE();",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::from)?;

        let mut schema: HashMap<String, Vec<String>> = HashMap::new();

        for row in rows {
            let table_name: String = row.get("TABLE_NAME");
            let column_name: String = row.get("COLUMN_NAME");

            schema
                .entry(table_name)
                .or_insert_with(Vec::new)
                .push(column_name);
        }

        Ok(schema)
    }

    async fn close(&self) {
        self.pool.close().await;
    }
}
