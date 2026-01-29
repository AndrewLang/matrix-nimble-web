use async_trait::async_trait;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::data::paging::{Page, PageRequest};
use crate::data::provider::{DataError, DataProvider, DataResult};
use crate::data::query::{
    Filter, FilterOperator, GroupBy, Join, JoinOn, Query, SortDirection, Value,
};
use crate::entity::entity::Entity;
pub mod migration;

pub trait PostgresEntity:
    Entity + for<'r> sqlx::FromRow<'r, PgRow> + Send + Sync + Unpin + 'static
{
    fn id_column() -> &'static str;
    fn id_value(id: &Self::Id) -> Value;
    fn insert_columns() -> &'static [&'static str];
    fn insert_values(&self) -> Vec<Value>;
    fn update_columns() -> &'static [&'static str];
    fn update_values(&self) -> Vec<Value>;
    fn table_columns() -> Vec<crate::data::schema::ColumnDef>;
}

pub struct PostgresProvider<E: Entity> {
    pool: PgPool,
    schema: Option<String>,
    _marker: std::marker::PhantomData<E>,
}

impl<E: Entity> PostgresProvider<E> {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            schema: None,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn with_schema(pool: PgPool, schema: &str) -> Self {
        Self {
            pool,
            schema: Some(schema.to_string()),
            _marker: std::marker::PhantomData,
        }
    }

    fn base_table(&self) -> String {
        let table = E::plural_name();
        match &self.schema {
            Some(schema) => format!("{}.{}", schema, table),
            None => table,
        }
    }

    fn join_table(&self, entity_name: &str) -> String {
        match &self.schema {
            Some(schema) => format!("{}.{}", schema, entity_name),
            None => entity_name.to_string(),
        }
    }

    pub fn build_select_sql(query: &Query<E>) -> String {
        let mut sql = String::new();
        sql.push_str("SELECT t.* FROM ");
        sql.push_str(&E::plural_name());
        sql.push_str(" t");
        Self::append_joins_sql(&mut sql, &query.joins);
        Self::append_filters_sql(&mut sql, &query.filters);
        Self::append_group_by_sql(&mut sql, query.group_by.as_ref());
        Self::append_sorting_sql(&mut sql, &query.sorting);
        Self::append_paging_sql(&mut sql, query.paging);
        sql
    }

    pub fn build_count_sql(query: &Query<E>) -> String {
        let mut sql = String::new();
        if query.group_by.is_some() {
            sql.push_str("SELECT COUNT(*) FROM (SELECT 1 FROM ");
        } else {
            sql.push_str("SELECT COUNT(*) FROM ");
        }
        sql.push_str(&E::plural_name());
        sql.push_str(" t");
        Self::append_joins_sql(&mut sql, &query.joins);
        Self::append_filters_sql(&mut sql, &query.filters);
        Self::append_group_by_sql(&mut sql, query.group_by.as_ref());
        if query.group_by.is_some() {
            sql.push_str(") AS grouped");
        }
        sql
    }

    fn operator_to_sql(operator: &FilterOperator) -> &'static str {
        match operator {
            FilterOperator::Eq => "=",
            FilterOperator::Ne => "<>",
            FilterOperator::Gt => ">",
            FilterOperator::Gte => ">=",
            FilterOperator::Lt => "<",
            FilterOperator::Lte => "<=",
            FilterOperator::Like => "LIKE",
            FilterOperator::Contains => "LIKE",
            FilterOperator::StartsWith => "LIKE",
            FilterOperator::EndsWith => "LIKE",
            FilterOperator::In => "IN",
            FilterOperator::NotIn => "NOT IN",
            FilterOperator::IsNull => "IS NULL",
            FilterOperator::IsNotNull => "IS NOT NULL",
            FilterOperator::Between => "BETWEEN",
        }
    }

    fn bind_value(builder: &mut QueryBuilder<Postgres>, value: Value) {
        match value {
            Value::Null => {
                builder.push("NULL");
            }
            Value::Bool(value) => {
                builder.push_bind(value);
            }
            Value::Int(value) => {
                builder.push_bind(value);
            }
            Value::UInt(value) => {
                builder.push_bind(value as i64);
            }
            Value::Float(value) => {
                builder.push_bind(value);
            }
            Value::String(value) => {
                builder.push_bind(value);
            }
            Value::Bytes(value) => {
                builder.push_bind(value);
            }
            Value::DateTime(value) => {
                builder.push_bind(value);
            }
            Value::List(_) => {
                builder.push("NULL");
            }
        }
    }

    fn map_sqlx_error(err: sqlx::Error) -> DataError {
        DataError::Provider(err.to_string())
    }

    fn append_joins_sql(sql: &mut String, joins: &[Join]) {
        for join in joins {
            sql.push_str(" JOIN ");
            sql.push_str(&join.entity_name);
            if let Some(alias) = &join.alias {
                sql.push(' ');
                sql.push_str(alias);
            }
            sql.push_str(" ON ");
            for (idx, on) in join.on.iter().enumerate() {
                if idx > 0 {
                    sql.push_str(" AND ");
                }
                sql.push_str(&on.left);
                sql.push(' ');
                sql.push_str(Self::operator_to_sql(&on.operator));
                sql.push(' ');
                sql.push_str(&on.right);
            }
        }
    }

    fn append_filters_sql(sql: &mut String, filters: &[Filter]) {
        if filters.is_empty() {
            return;
        }
        sql.push_str(" WHERE ");
        for (idx, filter) in filters.iter().enumerate() {
            if idx > 0 {
                sql.push_str(" AND ");
            }
            Self::append_filter_sql(sql, filter);
        }
    }

    fn append_filter_sql(sql: &mut String, filter: &Filter) {
        let field = &filter.field;
        match filter.operator {
            FilterOperator::IsNull => {
                sql.push_str(field);
                sql.push_str(" IS NULL");
                return;
            }
            FilterOperator::IsNotNull => {
                sql.push_str(field);
                sql.push_str(" IS NOT NULL");
                return;
            }
            FilterOperator::Eq | FilterOperator::Ne if matches!(filter.value, Value::Null) => {
                sql.push_str(field);
                if matches!(filter.operator, FilterOperator::Eq) {
                    sql.push_str(" IS NULL");
                } else {
                    sql.push_str(" IS NOT NULL");
                }
                return;
            }
            _ => {}
        }

        match filter.operator {
            FilterOperator::In | FilterOperator::NotIn => {
                sql.push_str(field);
                if matches!(filter.operator, FilterOperator::In) {
                    sql.push_str(" IN (");
                } else {
                    sql.push_str(" NOT IN (");
                }
                let count = match &filter.value {
                    Value::List(values) => values.len(),
                    _ => 0,
                };
                for idx in 0..count.max(1) {
                    if idx > 0 {
                        sql.push_str(", ");
                    }
                    sql.push('?');
                }
                sql.push(')');
            }
            FilterOperator::Between => {
                sql.push_str(field);
                sql.push_str(" BETWEEN ? AND ?");
            }
            FilterOperator::Contains | FilterOperator::StartsWith | FilterOperator::EndsWith => {
                sql.push_str(field);
                sql.push_str(" LIKE ?");
            }
            _ => {
                sql.push_str(field);
                sql.push(' ');
                sql.push_str(Self::operator_to_sql(&filter.operator));
                sql.push_str(" ?");
            }
        }
    }

    fn append_group_by_sql(sql: &mut String, group_by: Option<&GroupBy>) {
        let Some(group_by) = group_by else {
            return;
        };
        if group_by.fields.is_empty() {
            return;
        }
        sql.push_str(" GROUP BY ");
        for (idx, field) in group_by.fields.iter().enumerate() {
            if idx > 0 {
                sql.push_str(", ");
            }
            sql.push_str(field);
        }
    }

    fn append_sorting_sql(sql: &mut String, sorting: &[crate::data::query::Sort]) {
        if sorting.is_empty() {
            return;
        }
        sql.push_str(" ORDER BY ");
        for (idx, sort) in sorting.iter().enumerate() {
            if idx > 0 {
                sql.push_str(", ");
            }
            sql.push_str(&sort.field);
            match sort.direction {
                SortDirection::Asc => sql.push_str(" ASC"),
                SortDirection::Desc => sql.push_str(" DESC"),
            }
        }
    }

    fn append_paging_sql(sql: &mut String, paging: Option<PageRequest>) {
        let Some(paging) = paging else {
            return;
        };
        let page = paging.page.max(1);
        let page_size = paging.page_size.max(1);
        let offset = (page - 1) as u64 * page_size as u64;
        sql.push_str(" LIMIT ");
        sql.push_str(&page_size.to_string());
        sql.push_str(" OFFSET ");
        sql.push_str(&offset.to_string());
    }
}

#[async_trait]
impl<E> DataProvider<E> for PostgresProvider<E>
where
    E: PostgresEntity,
{
    async fn create(&self, entity: E) -> DataResult<E> {
        let columns = E::insert_columns();
        let values = entity.insert_values();
        if columns.len() != values.len() {
            return Err(DataError::InvalidQuery(
                "insert columns and values mismatch".to_string(),
            ));
        }

        let mut builder = QueryBuilder::<Postgres>::new("INSERT INTO ");
        builder.push(self.base_table());
        builder.push(" (");
        for (idx, col) in columns.iter().enumerate() {
            if idx > 0 {
                builder.push(", ");
            }
            builder.push(*col);
        }
        builder.push(") VALUES (");
        for (idx, value) in values.into_iter().enumerate() {
            if idx > 0 {
                builder.push(", ");
            }
            Self::bind_value(&mut builder, value);
        }
        builder.push(") RETURNING *");

        let row = builder
            .build_query_as::<E>()
            .fetch_one(&self.pool)
            .await
            .map_err(Self::map_sqlx_error)?;
        Ok(row)
    }

    async fn get(&self, id: &E::Id) -> DataResult<Option<E>> {
        let mut builder = QueryBuilder::<Postgres>::new("SELECT t.* FROM ");
        builder.push(self.base_table());
        builder.push(" t WHERE t.");
        builder.push(E::id_column());
        builder.push(" = ");
        Self::bind_value(&mut builder, E::id_value(id));

        let row = builder
            .build_query_as::<E>()
            .fetch_optional(&self.pool)
            .await;

        match &row {
            Ok(Some(_)) => log::debug!("PostgresProvider::get: found row"),
            Ok(None) => log::debug!("PostgresProvider::get: row not found"),
            Err(e) => log::error!("PostgresProvider::get: error: {}", e),
        }

        let row = row.map_err(Self::map_sqlx_error)?;
        Ok(row)
    }

    async fn update(&self, entity: E) -> DataResult<E> {
        let columns = E::update_columns();
        let values = entity.update_values();
        if columns.len() != values.len() {
            return Err(DataError::InvalidQuery(
                "update columns and values mismatch".to_string(),
            ));
        }

        let mut builder = QueryBuilder::<Postgres>::new("UPDATE ");
        builder.push(self.base_table());
        builder.push(" SET ");
        for (idx, (col, value)) in columns.iter().zip(values.into_iter()).enumerate() {
            if idx > 0 {
                builder.push(", ");
            }
            builder.push(*col);
            builder.push(" = ");
            Self::bind_value(&mut builder, value);
        }
        builder.push(" WHERE ");
        builder.push(E::id_column());
        builder.push(" = ");
        Self::bind_value(&mut builder, E::id_value(entity.id()));
        builder.push(" RETURNING *");

        let row = builder
            .build_query_as::<E>()
            .fetch_one(&self.pool)
            .await
            .map_err(Self::map_sqlx_error)?;
        Ok(row)
    }

    async fn delete(&self, id: &E::Id) -> DataResult<bool> {
        let mut builder = QueryBuilder::<Postgres>::new("DELETE FROM ");
        builder.push(self.base_table());
        builder.push(" WHERE ");
        builder.push(E::id_column());
        builder.push(" = ");
        Self::bind_value(&mut builder, E::id_value(id));

        let result = builder
            .build()
            .execute(&self.pool)
            .await
            .map_err(Self::map_sqlx_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn query(&self, query: Query<E>) -> DataResult<Page<E>> {
        let total = self.count_total(&query).await?;

        let mut builder = QueryBuilder::<Postgres>::new("SELECT t.* FROM ");
        self.append_from_and_joins(&mut builder, &query);
        self.append_filters(&mut builder, &query)?;
        self.append_group_by(&mut builder, query.group_by.as_ref());
        self.append_sorting(&mut builder, &query);
        self.append_paging(&mut builder, &query);

        let items = builder
            .build_query_as::<E>()
            .fetch_all(&self.pool)
            .await
            .map_err(Self::map_sqlx_error)?;

        let (page, page_size) = query
            .paging
            .map(|p| (p.page, p.page_size))
            .unwrap_or((1, items.len() as u32));

        Ok(Page::new(items, total, page, page_size))
    }

    async fn get_by(&self, column: &str, value: Value) -> DataResult<Option<E>> {
        let mut builder = QueryBuilder::<Postgres>::new("SELECT t.* FROM ");
        builder.push(self.base_table());
        builder.push(" t WHERE t.");
        builder.push(column);
        builder.push(" = ");
        Self::bind_value(&mut builder, value);

        let row = builder
            .build_query_as::<E>()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::map_sqlx_error)?;
        Ok(row)
    }
}

impl<E: Entity> PostgresProvider<E> {
    fn append_from_and_joins(&self, builder: &mut QueryBuilder<Postgres>, query: &Query<E>) {
        builder.push(self.base_table());
        builder.push(" t");
        for join in &query.joins {
            self.append_join(builder, join);
        }
    }

    fn append_join(&self, builder: &mut QueryBuilder<Postgres>, join: &Join) {
        builder.push(" JOIN ");
        builder.push(self.join_table(&join.entity_name));
        if let Some(alias) = &join.alias {
            builder.push(" ");
            builder.push(alias);
        }
        builder.push(" ON ");
        for (idx, on) in join.on.iter().enumerate() {
            if idx > 0 {
                builder.push(" AND ");
            }
            self.append_join_on(builder, on);
        }
    }

    fn append_join_on(&self, builder: &mut QueryBuilder<Postgres>, on: &JoinOn) {
        builder.push(&on.left);
        builder.push(" ");
        builder.push(Self::operator_to_sql(&on.operator));
        builder.push(" ");
        builder.push(&on.right);
    }

    fn append_filters(
        &self,
        builder: &mut QueryBuilder<Postgres>,
        query: &Query<E>,
    ) -> DataResult<()> {
        if query.filters.is_empty() {
            return Ok(());
        }

        builder.push(" WHERE ");
        for (idx, filter) in query.filters.iter().enumerate() {
            if idx > 0 {
                builder.push(" AND ");
            }
            self.append_filter(builder, filter)?;
        }
        Ok(())
    }

    fn append_filter(
        &self,
        builder: &mut QueryBuilder<Postgres>,
        filter: &Filter,
    ) -> DataResult<()> {
        let field = &filter.field;
        match filter.operator {
            FilterOperator::IsNull => {
                builder.push(field);
                builder.push(" IS NULL");
                return Ok(());
            }
            FilterOperator::IsNotNull => {
                builder.push(field);
                builder.push(" IS NOT NULL");
                return Ok(());
            }
            FilterOperator::Eq | FilterOperator::Ne if matches!(filter.value, Value::Null) => {
                builder.push(field);
                builder.push(if matches!(filter.operator, FilterOperator::Eq) {
                    " IS NULL"
                } else {
                    " IS NOT NULL"
                });
                return Ok(());
            }
            _ => {}
        }

        builder.push(field);
        builder.push(" ");
        match filter.operator {
            FilterOperator::In | FilterOperator::NotIn => {
                builder.push(if matches!(filter.operator, FilterOperator::In) {
                    "IN"
                } else {
                    "NOT IN"
                });
                builder.push(" (");
                let Value::List(values) = &filter.value else {
                    return Err(DataError::InvalidQuery("IN requires list".to_string()));
                };
                if values.is_empty() {
                    return Err(DataError::InvalidQuery("IN requires values".to_string()));
                }
                for (idx, value) in values.iter().cloned().enumerate() {
                    if idx > 0 {
                        builder.push(", ");
                    }
                    Self::bind_value(builder, value);
                }
                builder.push(")");
                return Ok(());
            }
            FilterOperator::Between => {
                builder.push("BETWEEN ");
                let Value::List(values) = &filter.value else {
                    return Err(DataError::InvalidQuery("BETWEEN requires list".to_string()));
                };
                if values.len() != 2 {
                    return Err(DataError::InvalidQuery(
                        "BETWEEN requires two values".to_string(),
                    ));
                }
                Self::bind_value(builder, values[0].clone());
                builder.push(" AND ");
                Self::bind_value(builder, values[1].clone());
                return Ok(());
            }
            FilterOperator::Contains => {
                builder.push("LIKE ");
                let value = match &filter.value {
                    Value::String(text) => Value::String(format!("%{}%", text)),
                    _ => {
                        return Err(DataError::InvalidQuery(
                            "CONTAINS requires string".to_string(),
                        ))
                    }
                };
                Self::bind_value(builder, value);
                return Ok(());
            }
            FilterOperator::StartsWith => {
                builder.push("LIKE ");
                let value = match &filter.value {
                    Value::String(text) => Value::String(format!("{}%", text)),
                    _ => {
                        return Err(DataError::InvalidQuery(
                            "STARTS_WITH requires string".to_string(),
                        ))
                    }
                };
                Self::bind_value(builder, value);
                return Ok(());
            }
            FilterOperator::EndsWith => {
                builder.push("LIKE ");
                let value = match &filter.value {
                    Value::String(text) => Value::String(format!("%{}", text)),
                    _ => {
                        return Err(DataError::InvalidQuery(
                            "ENDS_WITH requires string".to_string(),
                        ))
                    }
                };
                Self::bind_value(builder, value);
                return Ok(());
            }
            _ => {
                builder.push(Self::operator_to_sql(&filter.operator));
                builder.push(" ");
                Self::bind_value(builder, filter.value.clone());
                return Ok(());
            }
        }
    }

    fn append_group_by(&self, builder: &mut QueryBuilder<Postgres>, group_by: Option<&GroupBy>) {
        let Some(group_by) = group_by else {
            return;
        };
        if group_by.fields.is_empty() {
            return;
        }
        builder.push(" GROUP BY ");
        for (idx, field) in group_by.fields.iter().enumerate() {
            if idx > 0 {
                builder.push(", ");
            }
            builder.push(field);
        }
    }

    fn append_sorting(&self, builder: &mut QueryBuilder<Postgres>, query: &Query<E>) {
        if query.sorting.is_empty() {
            return;
        }
        builder.push(" ORDER BY ");
        for (idx, sort) in query.sorting.iter().enumerate() {
            if idx > 0 {
                builder.push(", ");
            }
            builder.push(&sort.field);
            builder.push(match sort.direction {
                SortDirection::Asc => " ASC",
                SortDirection::Desc => " DESC",
            });
        }
    }

    fn append_paging(&self, builder: &mut QueryBuilder<Postgres>, query: &Query<E>) {
        let Some(paging) = query.paging else {
            return;
        };
        let page = paging.page.max(1);
        let page_size = paging.page_size.max(1);
        let offset = (page - 1) as u64 * page_size as u64;
        builder.push(" LIMIT ");
        builder.push_bind(page_size as i64);
        builder.push(" OFFSET ");
        builder.push_bind(offset as i64);
    }

    async fn count_total(&self, query: &Query<E>) -> DataResult<u64> {
        let mut builder = if query.group_by.is_some() {
            QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM (SELECT 1 FROM ")
        } else {
            QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM ")
        };

        self.append_from_and_joins(&mut builder, query);
        self.append_filters(&mut builder, query)?;
        self.append_group_by(&mut builder, query.group_by.as_ref());

        if query.group_by.is_some() {
            builder.push(") AS grouped");
        }

        let total: i64 = builder
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await
            .map_err(Self::map_sqlx_error)?;
        Ok(total.max(0) as u64)
    }
}
