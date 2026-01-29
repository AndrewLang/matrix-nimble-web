use async_trait::async_trait;
use futures_util::TryStreamExt;
use mongodb::bson::{self, Bson, Document, Regex};
use mongodb::options::FindOptions;
use mongodb::{Collection, Database};

use crate::data::paging::Page;
use crate::data::provider::{DataError, DataProvider, DataResult};
use crate::data::query::{
    AggregateFunction, Filter, FilterOperator, GroupBy, Join, Query, SortDirection, Value,
};
use crate::entity::entity::Entity;

pub trait MongoEntity:
    Entity + serde::Serialize + serde::de::DeserializeOwned + Send + Sync
{
    fn id_field() -> &'static str;
    fn id_bson(id: &Self::Id) -> Bson;
}

pub struct MongoProvider<E: Entity> {
    database: Database,
    schema: Option<String>,
    _marker: std::marker::PhantomData<E>,
}

impl<E: Entity> MongoProvider<E> {
    pub fn new(database: Database) -> Self {
        Self {
            database,
            schema: None,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn with_schema(database: Database, schema: &str) -> Self {
        Self {
            database,
            schema: Some(schema.to_string()),
            _marker: std::marker::PhantomData,
        }
    }

    pub(crate) fn collection_name(&self) -> String {
        let base = E::plural_name();
        match &self.schema {
            Some(schema) => format!("{}.{}", schema, base),
            None => base,
        }
    }

    fn collection(&self) -> Collection<Document> {
        self.database
            .collection::<Document>(&self.collection_name())
    }

    pub fn collection_name_for_entity(schema: Option<&str>) -> String {
        let base = E::plural_name();
        match schema {
            Some(schema) => format!("{}.{}", schema, base),
            None => base,
        }
    }
}

#[async_trait]
impl<E> DataProvider<E> for MongoProvider<E>
where
    E: MongoEntity,
{
    async fn create(&self, entity: E) -> DataResult<E> {
        let collection = self.collection();
        let doc = bson::to_document(&entity).map_err(|err| DataError::Provider(err.to_string()))?;
        collection
            .insert_one(doc)
            .await
            .map_err(Self::map_mongo_error)?;

        match self.get(entity.id()).await? {
            Some(found) => Ok(found),
            None => Ok(entity),
        }
    }

    async fn get(&self, id: &E::Id) -> DataResult<Option<E>> {
        let collection = self.collection();
        let filter = Self::id_filter(id);
        let doc = collection
            .find_one(filter)
            .await
            .map_err(Self::map_mongo_error)?;
        match doc {
            Some(doc) => {
                let entity = bson::from_document::<E>(doc)
                    .map_err(|err| DataError::Provider(err.to_string()))?;
                Ok(Some(entity))
            }
            None => Ok(None),
        }
    }

    async fn update(&self, entity: E) -> DataResult<E> {
        let collection = self.collection();
        let filter = Self::id_filter(entity.id());
        let doc = bson::to_document(&entity).map_err(|err| DataError::Provider(err.to_string()))?;
        let result = collection
            .replace_one(filter, doc)
            .await
            .map_err(Self::map_mongo_error)?;
        if result.matched_count == 0 {
            return Err(DataError::NotFound);
        }
        Ok(entity)
    }

    async fn delete(&self, id: &E::Id) -> DataResult<bool> {
        let collection = self.collection();
        let filter = Self::id_filter(id);
        let result = collection
            .delete_one(filter)
            .await
            .map_err(Self::map_mongo_error)?;
        Ok(result.deleted_count > 0)
    }

    async fn query(&self, query: Query<E>) -> DataResult<Page<E>> {
        let total = self.count_total(&query).await?;

        let uses_pipeline = !query.joins.is_empty() || query.group_by.is_some();
        if uses_pipeline {
            return self.query_with_pipeline(query, total).await;
        }

        let filter = Self::build_filter_doc(&query)?;
        let options = Self::build_find_options(&query);
        let mut cursor = self
            .collection()
            .find(filter)
            .with_options(options)
            .await
            .map_err(Self::map_mongo_error)?;

        let mut items = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(Self::map_mongo_error)? {
            let entity = bson::from_document::<E>(doc)
                .map_err(|err| DataError::Provider(err.to_string()))?;
            items.push(entity);
        }

        let (page, page_size) = query
            .paging
            .map(|p| (p.page, p.page_size))
            .unwrap_or((1, items.len() as u32));

        Ok(Page::new(items, total, page, page_size))
    }

    async fn get_by(&self, column: &str, value: Value) -> DataResult<Option<E>> {
        let collection = self.collection();
        let mut filter = Document::new();
        filter.insert(column, Self::to_bson(&value)?);

        let doc = collection
            .find_one(filter)
            .await
            .map_err(Self::map_mongo_error)?;
        match doc {
            Some(doc) => {
                let entity = bson::from_document::<E>(doc)
                    .map_err(|err| DataError::Provider(err.to_string()))?;
                Ok(Some(entity))
            }
            None => Ok(None),
        }
    }
}

impl<E> MongoProvider<E>
where
    E: MongoEntity,
{
    async fn query_with_pipeline(&self, query: Query<E>, total: u64) -> DataResult<Page<E>> {
        let pipeline = Self::build_pipeline(&query)?;

        let mut cursor = self
            .collection()
            .aggregate(pipeline)
            .await
            .map_err(Self::map_mongo_error)?;

        let mut items = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(Self::map_mongo_error)? {
            let entity = bson::from_document::<E>(doc)
                .map_err(|err| DataError::Provider(err.to_string()))?;
            items.push(entity);
        }

        let (page, page_size) = query
            .paging
            .map(|p| (p.page, p.page_size))
            .unwrap_or((1, items.len() as u32));

        Ok(Page::new(items, total, page, page_size))
    }

    async fn count_total(&self, query: &Query<E>) -> DataResult<u64> {
        if query.group_by.is_none() && query.joins.is_empty() {
            let filter = Self::build_filter_doc(query)?;
            let count = self
                .collection()
                .count_documents(filter)
                .await
                .map_err(Self::map_mongo_error)?;
            return Ok(count);
        }

        let mut pipeline = Vec::new();
        if !query.filters.is_empty() {
            let filter = Self::build_filter_doc(query)?;
            let mut stage = Document::new();
            stage.insert("$match", Bson::Document(filter));
            pipeline.push(stage);
        }

        for join in &query.joins {
            pipeline.push(Self::build_lookup(join)?);
        }

        if let Some(group_by) = &query.group_by {
            pipeline.push(Self::build_group(group_by));
        }

        let mut count_stage = Document::new();
        count_stage.insert("$count", Bson::String("total".to_string()));
        pipeline.push(count_stage);

        let mut cursor = self
            .collection()
            .aggregate(pipeline)
            .await
            .map_err(Self::map_mongo_error)?;

        if let Some(doc) = cursor.try_next().await.map_err(Self::map_mongo_error)? {
            if let Some(Bson::Int64(total)) = doc.get("total") {
                return Ok(*total as u64);
            }
            if let Some(Bson::Int32(total)) = doc.get("total") {
                return Ok(*total as u64);
            }
        }

        Ok(0)
    }

    pub fn build_filter_doc(query: &Query<E>) -> DataResult<Document> {
        Self::build_filter_doc_from_filters(&query.filters)
    }

    pub fn build_sort_doc(query: &Query<E>) -> Document {
        Self::build_sort_doc_internal(query)
    }

    pub fn build_pipeline(query: &Query<E>) -> DataResult<Vec<Document>> {
        let mut pipeline = Vec::new();

        if !query.filters.is_empty() {
            let filter = Self::build_filter_doc(query)?;
            let mut stage = Document::new();
            stage.insert("$match", Bson::Document(filter));
            pipeline.push(stage);
        }

        for join in &query.joins {
            pipeline.push(Self::build_lookup(join)?);
        }

        if let Some(group_by) = &query.group_by {
            pipeline.push(Self::build_group(group_by));
        }

        if !query.sorting.is_empty() {
            pipeline.push(Self::build_sort_doc(query));
        }

        if let Some(paging) = query.paging {
            let page = paging.page.max(1);
            let page_size = paging.page_size.max(1);
            let skip = (page - 1) as i64 * page_size as i64;
            let mut skip_stage = Document::new();
            skip_stage.insert("$skip", Bson::Int64(skip));
            pipeline.push(skip_stage);

            let mut limit_stage = Document::new();
            limit_stage.insert("$limit", Bson::Int64(page_size as i64));
            pipeline.push(limit_stage);
        }

        Ok(pipeline)
    }

    fn id_filter(id: &E::Id) -> Document {
        let mut doc = Document::new();
        doc.insert(E::id_field(), E::id_bson(id));
        doc
    }

    fn build_find_options(query: &Query<E>) -> FindOptions {
        let mut options = FindOptions::default();

        if !query.sorting.is_empty() {
            let mut sort = Document::new();
            for sort_item in &query.sorting {
                let direction = match sort_item.direction {
                    SortDirection::Asc => 1,
                    SortDirection::Desc => -1,
                };
                sort.insert(sort_item.field.clone(), Bson::Int32(direction));
            }
            options.sort = Some(sort);
        }

        if let Some(paging) = query.paging {
            let page = paging.page.max(1);
            let page_size = paging.page_size.max(1);
            options.skip = Some((page - 1) as u64 * page_size as u64);
            options.limit = Some(page_size as i64);
        }

        options
    }

    fn build_filter_doc_from_filters(filters: &[Filter]) -> DataResult<Document> {
        if filters.is_empty() {
            return Ok(Document::new());
        }
        let mut clauses = Vec::new();
        for filter in filters {
            clauses.push(Self::build_filter_clause(filter)?);
        }

        if clauses.len() == 1 {
            return Ok(clauses.remove(0));
        }

        let mut doc = Document::new();
        doc.insert(
            "$and",
            Bson::Array(clauses.into_iter().map(Bson::Document).collect()),
        );
        Ok(doc)
    }

    fn build_filter_clause(filter: &Filter) -> DataResult<Document> {
        let mut doc = Document::new();
        let field = filter.field.clone();
        match filter.operator {
            FilterOperator::IsNull => {
                doc.insert(field, Bson::Null);
                return Ok(doc);
            }
            FilterOperator::IsNotNull => {
                let mut inner = Document::new();
                inner.insert("$ne", Bson::Null);
                doc.insert(field, Bson::Document(inner));
                return Ok(doc);
            }
            FilterOperator::Eq => {
                doc.insert(field, Self::to_bson(&filter.value)?);
                return Ok(doc);
            }
            FilterOperator::Ne => {
                let mut inner = Document::new();
                inner.insert("$ne", Self::to_bson(&filter.value)?);
                doc.insert(field, Bson::Document(inner));
                return Ok(doc);
            }
            FilterOperator::Gt | FilterOperator::Gte | FilterOperator::Lt | FilterOperator::Lte => {
                let mut inner = Document::new();
                let op = match filter.operator {
                    FilterOperator::Gt => "$gt",
                    FilterOperator::Gte => "$gte",
                    FilterOperator::Lt => "$lt",
                    FilterOperator::Lte => "$lte",
                    _ => "$eq",
                };
                inner.insert(op, Self::to_bson(&filter.value)?);
                doc.insert(field, Bson::Document(inner));
                return Ok(doc);
            }
            FilterOperator::In | FilterOperator::NotIn => {
                let Value::List(values) = &filter.value else {
                    return Err(DataError::InvalidQuery("IN requires list".to_string()));
                };
                let array: Vec<Bson> = values
                    .iter()
                    .map(|v| Self::to_bson(v))
                    .collect::<Result<_, _>>()?;
                let mut inner = Document::new();
                inner.insert(
                    if matches!(filter.operator, FilterOperator::In) {
                        "$in"
                    } else {
                        "$nin"
                    },
                    Bson::Array(array),
                );
                doc.insert(field, Bson::Document(inner));
                return Ok(doc);
            }
            FilterOperator::Between => {
                let Value::List(values) = &filter.value else {
                    return Err(DataError::InvalidQuery("BETWEEN requires list".to_string()));
                };
                if values.len() != 2 {
                    return Err(DataError::InvalidQuery(
                        "BETWEEN requires two values".to_string(),
                    ));
                }
                let mut inner = Document::new();
                inner.insert("$gte", Self::to_bson(&values[0])?);
                inner.insert("$lte", Self::to_bson(&values[1])?);
                doc.insert(field, Bson::Document(inner));
                return Ok(doc);
            }
            FilterOperator::Like => {
                let pattern = match &filter.value {
                    Value::String(text) => Self::sql_like_to_regex(text),
                    _ => return Err(DataError::InvalidQuery("LIKE requires string".to_string())),
                };
                doc.insert(
                    field,
                    Bson::RegularExpression(Regex {
                        pattern,
                        options: String::new(),
                    }),
                );
                return Ok(doc);
            }
            FilterOperator::Contains => {
                let Value::String(text) = &filter.value else {
                    return Err(DataError::InvalidQuery(
                        "CONTAINS requires string".to_string(),
                    ));
                };
                doc.insert(
                    field,
                    Bson::RegularExpression(Regex {
                        pattern: format!("{}", Self::regex_escape(text)),
                        options: String::new(),
                    }),
                );
                return Ok(doc);
            }
            FilterOperator::StartsWith => {
                let Value::String(text) = &filter.value else {
                    return Err(DataError::InvalidQuery(
                        "STARTS_WITH requires string".to_string(),
                    ));
                };
                doc.insert(
                    field,
                    Bson::RegularExpression(Regex {
                        pattern: format!("^{}", Self::regex_escape(text)),
                        options: String::new(),
                    }),
                );
                return Ok(doc);
            }
            FilterOperator::EndsWith => {
                let Value::String(text) = &filter.value else {
                    return Err(DataError::InvalidQuery(
                        "ENDS_WITH requires string".to_string(),
                    ));
                };
                doc.insert(
                    field,
                    Bson::RegularExpression(Regex {
                        pattern: format!("{}$", Self::regex_escape(text)),
                        options: String::new(),
                    }),
                );
                return Ok(doc);
            }
        }
    }

    fn build_lookup(join: &Join) -> DataResult<Document> {
        if join.on.len() != 1 {
            return Err(DataError::InvalidQuery(
                "join supports single ON condition".to_string(),
            ));
        }
        let on = &join.on[0];
        if !matches!(on.operator, FilterOperator::Eq) {
            return Err(DataError::InvalidQuery(
                "join supports only equality".to_string(),
            ));
        }

        let local_field = Self::strip_prefix(&on.left);
        let foreign_field = Self::strip_prefix(&on.right);

        let mut lookup = Document::new();
        lookup.insert("from", Bson::String(join.entity_name.clone()));
        lookup.insert("localField", Bson::String(local_field));
        lookup.insert("foreignField", Bson::String(foreign_field));
        lookup.insert(
            "as",
            Bson::String(
                join.alias
                    .clone()
                    .unwrap_or_else(|| join.entity_name.clone()),
            ),
        );

        let mut stage = Document::new();
        stage.insert("$lookup", Bson::Document(lookup));
        Ok(stage)
    }

    fn build_group(group_by: &GroupBy) -> Document {
        let mut group = Document::new();

        if group_by.fields.is_empty() {
            group.insert("_id", Bson::Null);
        } else {
            let mut id_doc = Document::new();
            for field in &group_by.fields {
                id_doc.insert(field, Bson::String(format!("${}", field)));
            }
            group.insert("_id", Bson::Document(id_doc));
        }

        for (idx, agg) in group_by.aggregates.iter().enumerate() {
            let key = agg.alias.clone().unwrap_or_else(|| format!("agg_{}", idx));
            let value = match agg.function {
                AggregateFunction::Count => {
                    let mut doc = Document::new();
                    doc.insert("$sum", Bson::Int32(1));
                    Bson::Document(doc)
                }
                AggregateFunction::Sum => {
                    let mut doc = Document::new();
                    doc.insert("$sum", Bson::String(format!("${}", agg.field)));
                    Bson::Document(doc)
                }
                AggregateFunction::Avg => {
                    let mut doc = Document::new();
                    doc.insert("$avg", Bson::String(format!("${}", agg.field)));
                    Bson::Document(doc)
                }
                AggregateFunction::Min => {
                    let mut doc = Document::new();
                    doc.insert("$min", Bson::String(format!("${}", agg.field)));
                    Bson::Document(doc)
                }
                AggregateFunction::Max => {
                    let mut doc = Document::new();
                    doc.insert("$max", Bson::String(format!("${}", agg.field)));
                    Bson::Document(doc)
                }
            };
            group.insert(key, value);
        }

        let mut stage = Document::new();
        stage.insert("$group", Bson::Document(group));
        stage
    }

    fn build_sort_doc_internal(query: &Query<E>) -> Document {
        let mut sort = Document::new();
        for item in &query.sorting {
            let direction = match item.direction {
                SortDirection::Asc => 1,
                SortDirection::Desc => -1,
            };
            sort.insert(item.field.clone(), Bson::Int32(direction));
        }
        let mut stage = Document::new();
        stage.insert("$sort", Bson::Document(sort));
        stage
    }

    fn to_bson(value: &Value) -> DataResult<Bson> {
        match value {
            Value::Null => Ok(Bson::Null),
            Value::Bool(value) => Ok(Bson::Boolean(*value)),
            Value::Int(value) => Ok(Bson::Int64(*value)),
            Value::UInt(value) => Ok(Bson::Int64(*value as i64)),
            Value::Float(value) => Ok(Bson::Double(*value)),
            Value::String(value) => Ok(Bson::String(value.clone())),
            Value::Bytes(value) => Ok(Bson::Binary(bson::Binary {
                subtype: bson::spec::BinarySubtype::Generic,
                bytes: value.clone(),
            })),
            Value::DateTime(value) => Ok(Bson::DateTime(bson::DateTime::from_millis(
                value.timestamp_millis(),
            ))),
            Value::List(values) => Ok(Bson::Array(
                values.iter().map(Self::to_bson).collect::<Result<_, _>>()?,
            )),
        }
    }

    fn strip_prefix(value: &str) -> String {
        value.rsplit('.').next().unwrap_or(value).to_string()
    }

    fn sql_like_to_regex(value: &str) -> String {
        let mut pattern = String::new();
        pattern.push('^');
        let mut chars = value.chars().peekable();
        while let Some(ch) = chars.next() {
            match ch {
                '%' => pattern.push_str(".*"),
                '_' => pattern.push('.'),
                other => pattern.push_str(&Self::regex_escape_char(other)),
            }
        }
        pattern.push('$');
        pattern
    }

    fn regex_escape(value: &str) -> String {
        value.chars().map(Self::regex_escape_char).collect()
    }

    fn regex_escape_char(ch: char) -> String {
        match ch {
            '.' | '+' | '*' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '^' | '$' | '|' | '\\' => {
                format!("\\{}", ch)
            }
            _ => ch.to_string(),
        }
    }

    fn map_mongo_error(err: mongodb::error::Error) -> DataError {
        DataError::Provider(err.to_string())
    }
}
