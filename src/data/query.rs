use chrono::{DateTime, NaiveDate, Utc};
use std::marker::PhantomData;
use uuid::Uuid;

use crate::data::paging::PageRequest;
use crate::entity::entity::Entity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterOperator {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
    In,
    NotIn,
    Like,
    Contains,
    StartsWith,
    EndsWith,
    IsNull,
    IsNotNull,
    Between,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    UInt(u64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    Date(NaiveDate),
    DateTime(DateTime<Utc>),
    List(Vec<Value>),
    Uuid(Uuid),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Filter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sort {
    pub field: String,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinOn {
    pub left: String,
    pub operator: FilterOperator,
    pub right: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Join {
    pub entity_name: String,
    pub alias: Option<String>,
    pub on: Vec<JoinOn>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregateFunction {
    Count,
    Sum,
    Avg,
    Min,
    Max,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Aggregate {
    pub function: AggregateFunction,
    pub field: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupBy {
    pub fields: Vec<String>,
    pub aggregates: Vec<Aggregate>,
}

#[derive(Debug, Clone)]
pub struct Query<E: Entity> {
    entity_name: &'static str,
    entity_plural_name: String,
    pub filters: Vec<Filter>,
    pub sorting: Vec<Sort>,
    pub joins: Vec<Join>,
    pub group_by: Option<GroupBy>,
    pub paging: Option<PageRequest>,
    _marker: PhantomData<E>,
}

impl<E: Entity> Query<E> {
    pub fn new() -> Self {
        Self {
            entity_name: E::name(),
            entity_plural_name: E::plural_name(),
            filters: Vec::new(),
            sorting: Vec::new(),
            joins: Vec::new(),
            group_by: None,
            paging: None,
            _marker: PhantomData,
        }
    }

    pub fn entity_name(&self) -> &'static str {
        self.entity_name
    }

    pub fn entity_plural_name(&self) -> &str {
        self.entity_plural_name.as_str()
    }

    pub fn with_filter(mut self, field: impl Into<String>, value: Value) -> Self {
        self.filters.push(Filter {
            field: field.into(),
            operator: FilterOperator::Eq,
            value,
        });
        self
    }

    pub fn with_paging(mut self, page: u32, page_size: u32) -> Self {
        self.paging = Some(PageRequest::new(page, page_size));
        self
    }

    pub fn with_page_size(mut self, page_size: u32) -> Self {
        let page = self.paging.map(|p| p.page).unwrap_or(1);
        self.paging = Some(PageRequest::new(page, page_size));
        self
    }
}
