use crate::database_provider::DatabaseProvider;
use crate::db::{ColumnDefinition, DatabaseError, Filter, IndexDefinition, QueryResult, Sort};
use async_trait::async_trait;
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{Column, Pool, Postgres, Row, TypeInfo};
use std::collections::HashMap;

pub struct PostgresProvider {
    pool: Pool<Postgres>,
}

impl PostgresProvider {
    pub async fn new(connection_string: &str) -> Result<Self, DatabaseError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(connection_string)
            .await
            .map_err(DatabaseError::from)?;
        Ok(Self { pool })
    }
}

fn row_to_values(row: PgRow) -> Vec<serde_json::Value> {
    let mut values = Vec::new();
    for col in row.columns() {
        let col_name = col.name();
        let type_info = col.type_info();
        let type_name = type_info.name();

        let value = match type_name {
            "BOOL" => {
                let val: Option<bool> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "INT2" | "INT4" | "INT" => {
                let val: Option<i32> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "INT8" | "BIGINT" => {
                let val: Option<i64> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "FLOAT4" | "REAL" => {
                let val: Option<f32> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "FLOAT8" | "DOUBLE PRECISION" => {
                let val: Option<f64> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "VARCHAR" | "TEXT" | "BPCHAR" | "NAME" | "UNKNOWN" => {
                let val: Option<String> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "TIMESTAMP" | "Timestamp" => {
                let val: Option<chrono::NaiveDateTime> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "TIMESTAMPTZ" | "Timestamptz" => {
                let val: Option<chrono::DateTime<chrono::Utc>> =
                    row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "UUID" | "Uuid" => {
                let val: Option<uuid::Uuid> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "JSON" | "JSONB" => {
                let val: Option<serde_json::Value> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "INET" | "inet" | "CIDR" | "cidr" => {
                let val: Option<ipnetwork::IpNetwork> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "DATE" | "Date" => {
                let val: Option<chrono::NaiveDate> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "TIME" | "Time" => {
                let val: Option<chrono::NaiveTime> = row.try_get(col_name).unwrap_or(None);
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
impl DatabaseProvider for PostgresProvider {
    async fn list_databases(&self) -> Result<Vec<String>, DatabaseError> {
        let rows = sqlx::query("SELECT datname FROM pg_database WHERE datistemplate = false;")
            .fetch_all(&self.pool)
            .await
            .map_err(DatabaseError::from)?;

        let databases: Vec<String> = rows.iter().map(|row| row.get("datname")).collect();
        Ok(databases)
    }

    async fn list_tables(&self) -> Result<Vec<String>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public';",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::from)?;

        let tables: Vec<String> = rows
            .iter()
            .map(|row: &PgRow| row.get("table_name"))
            .collect();
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
        // Get column types for casting
        let type_rows = sqlx::query(
            "SELECT column_name, udt_name 
             FROM information_schema.columns 
             WHERE table_name = $1",
        )
        .bind(&table_name)
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::from)?;

        let mut column_types = HashMap::new();
        for row in type_rows {
            let column_name: String = row.get("column_name");
            let data_type: String = row.get("udt_name");
            column_types.insert(column_name, data_type);
        }

        let mut where_clauses = Vec::new();
        let mut query_params = Vec::new();
        let mut param_index = 1;

        for filter in &filters {
            if !filter
                .field
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_')
            {
                continue;
            }

            let field = format!("\"{}\"", filter.field);
            let col_type = column_types
                .get(&filter.field)
                .map(|s| s.as_str())
                .unwrap_or("text");

            let cast_suffix = match col_type {
                "uuid" => "::uuid",
                "date" => "::date",
                "timestamp" | "timestamptz" => match col_type {
                    "timestamp" => "::timestamp",
                    "timestamptz" => "::timestamptz",
                    _ => "",
                },
                "bool" => "::boolean",
                _ => "",
            };

            match filter.operator.as_str() {
                "=" | ">=" | "<=" | ">" | "<" => {
                    where_clauses.push(format!(
                        "{} {} ${}{}",
                        field, filter.operator, param_index, cast_suffix
                    ));
                    query_params.push(filter.value.clone());
                    param_index += 1;
                }
                "contain" => {
                    where_clauses.push(format!("{}::text ILIKE ${}", field, param_index));
                    query_params.push(format!("%{}%", filter.value));
                    param_index += 1;
                }
                "start with" => {
                    where_clauses.push(format!("{}::text ILIKE ${}", field, param_index));
                    query_params.push(format!("{}%", filter.value));
                    param_index += 1;
                }
                "end with" => {
                    where_clauses.push(format!("{}::text ILIKE ${}", field, param_index));
                    query_params.push(format!("%{}", filter.value));
                    param_index += 1;
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
            order_clauses.push(format!("\"{}\" {}", sort.field, direction));
        }

        let order_sql = if order_clauses.is_empty() {
            String::new()
        } else {
            format!("ORDER BY {}", order_clauses.join(", "))
        };

        let count_query = format!("SELECT COUNT(*) FROM \"{}\" {}", table_name, where_sql);
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
            "SELECT * FROM \"{}\" {} {} LIMIT ${} OFFSET ${}",
            table_name,
            where_sql,
            order_sql,
            param_index,
            param_index + 1
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
                a.attname AS column_name,
                format_type(a.atttypid, a.atttypmod) AS data_type,
                CASE WHEN a.attnotnull THEN 'NO' ELSE 'YES' END AS is_nullable,
                pg_get_expr(d.adbin, d.adrelid) AS column_default,
                col_description(a.attrelid, a.attnum) AS comment,
                (
                    SELECT 
                        confrelid::regclass::text || '(' || a2.attname || ')'
                    FROM pg_constraint c
                    JOIN pg_attribute a2 ON a2.attnum = c.confkey[1] AND a2.attrelid = c.confrelid
                    WHERE c.conrelid = a.attrelid 
                      AND c.contype = 'f' 
                      AND c.conkey[1] = a.attnum
                    LIMIT 1
                ) AS foreign_key
            FROM pg_attribute a
            LEFT JOIN pg_attrdef d ON d.adrelid = a.attrelid AND d.adnum = a.attnum
            WHERE a.attrelid = $1::regclass
              AND a.attnum > 0 
              AND NOT a.attisdropped
            ORDER BY a.attnum;
        ";

        let table_oid_str = if table_name.contains('.') {
            table_name.clone()
        } else {
            format!("public.\"{}\"", table_name)
        };

        let rows = sqlx::query(query)
            .bind(&table_oid_str)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DatabaseError {
                message: format!("Failed to get structure for {}: {}", table_name, e),
            })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(ColumnDefinition {
                column_name: row.get("column_name"),
                data_type: row.get("data_type"),
                is_nullable: row.get("is_nullable"),
                column_default: row.get("column_default"),
                comment: row.get("comment"),
                foreign_key: row.get("foreign_key"),
            });
        }

        Ok(results)
    }

    async fn get_table_indexes(
        &self,
        table_name: String,
    ) -> Result<Vec<IndexDefinition>, DatabaseError> {
        let table_oid_str = if table_name.contains('.') {
            table_name.clone()
        } else {
            format!("public.\"{}\"", table_name)
        };

        let query = "
            SELECT 
                i.relname AS index_name,
                am.amname AS index_algorithm,
                ix.indisunique AS is_unique,
                ix.indisprimary AS is_primary,
                pg_get_indexdef(ix.indexrelid, 0, true) AS full_def,
                pg_get_expr(ix.indpred, ix.indrelid) AS condition,
                obj_description(i.oid, 'pg_class') AS comment,
                (
                    SELECT string_agg(a.attname, ', ' ORDER BY array_position(ix.indkey, a.attnum))
                    FROM pg_attribute a
                    WHERE a.attrelid = ix.indrelid AND a.attnum = ANY(ix.indkey)
                ) as column_names
            FROM pg_index ix
            JOIN pg_class i ON i.oid = ix.indexrelid
            JOIN pg_am am ON am.oid = i.relam
            WHERE ix.indrelid = $1::regclass
            ORDER BY ix.indisprimary DESC, i.relname;
        ";

        let rows = sqlx::query(query)
            .bind(&table_oid_str)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DatabaseError {
                message: format!("Failed to get indexes for {}: {}", table_name, e),
            })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(IndexDefinition {
                index_name: row.get("index_name"),
                index_algorithm: row.get("index_algorithm"),
                is_unique: row.get("is_unique"),
                is_primary: row.get("is_primary"),
                column_names: row.get("column_names"),
                condition: row.get("condition"),
                comment: row.get("comment"),
            });
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
            "SELECT table_name, column_name FROM information_schema.columns WHERE table_schema = 'public';",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::from)?;

        let mut schema: HashMap<String, Vec<String>> = HashMap::new();

        for row in rows {
            let table_name: String = row.get("table_name");
            let column_name: String = row.get("column_name");

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
