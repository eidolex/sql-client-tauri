use crate::db::{ColumnDefinition, DatabaseError, Filter, IndexDefinition, QueryResult, Sort};
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait DatabaseProvider: Send + Sync {
    async fn list_databases(&self) -> Result<Vec<String>, DatabaseError>;
    async fn list_tables(&self) -> Result<Vec<String>, DatabaseError>;
    async fn get_table_data(
        &self,
        table_name: String,
        limit: i64,
        offset: i64,
        filters: Vec<Filter>,
        sorts: Vec<Sort>,
    ) -> Result<QueryResult, DatabaseError>;
    async fn get_table_structure(
        &self,
        table_name: String,
    ) -> Result<Vec<ColumnDefinition>, DatabaseError>;
    async fn get_table_indexes(
        &self,
        table_name: String,
    ) -> Result<Vec<IndexDefinition>, DatabaseError>;
    async fn execute_query(&self, query: String) -> Result<QueryResult, DatabaseError>;
    async fn get_database_schema(&self) -> Result<HashMap<String, Vec<String>>, DatabaseError>;
    async fn close(&self);
}
