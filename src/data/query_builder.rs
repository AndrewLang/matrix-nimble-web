use std::marker::PhantomData;

use crate::data::paging::PageRequest;
use crate::data::provider::DataResult;
use crate::data::query::{
    Aggregate, AggregateFunction, Filter, FilterOperator, GroupBy, Join, JoinOn, Query, Select,
    Sort, SortDirection, Value,
};
use crate::data::repository::Repository;
use crate::entity::entity::Entity;
use serde::de::DeserializeOwned;

pub struct QueryBuilder<E: Entity> {
    query: Query<E>,
    _marker: PhantomData<E>,
}

impl<E: Entity> QueryBuilder<E> {
    pub fn new() -> Self {
        Self {
            query: Query::new(),
            _marker: PhantomData,
        }
    }

    pub fn filter(mut self, field: &str, operator: FilterOperator, value: Value) -> Self {
        self.query.filters.push(Filter {
            field: field.to_string(),
            operator,
            value,
        });
        self
    }

    pub fn distinct(mut self) -> Self {
        self.query.distinct = true;
        self
    }

    pub fn distinct_by(mut self, column: &str) -> Self {
        self.query.distinct = true;
        self.query.distinct_by = Some(column.to_string());
        self
    }

    pub fn select(mut self, expression: &str) -> Self {
        self.query.select.push(Select {
            expression: expression.to_string(),
            alias: None,
        });
        self
    }

    pub fn select_as(mut self, expression: &str, alias: &str) -> Self {
        self.query.select.push(Select {
            expression: expression.to_string(),
            alias: Some(alias.to_string()),
        });
        self
    }

    pub fn sort_asc(mut self, field: &str) -> Self {
        self.query.sorting.push(Sort {
            field: field.to_string(),
            direction: SortDirection::Asc,
        });
        self
    }

    pub fn sort_desc(mut self, field: &str) -> Self {
        self.query.sorting.push(Sort {
            field: field.to_string(),
            direction: SortDirection::Desc,
        });
        self
    }

    pub fn join<Other: Entity>(mut self, local_field: &str, foreign_field: &str) -> Self {
        let on = JoinOn {
            left: local_field.to_string(),
            operator: FilterOperator::Eq,
            right: foreign_field.to_string(),
        };
        self.query.joins.push(Join {
            entity_name: Other::name().to_string(),
            alias: None,
            on: vec![on],
        });
        self
    }

    pub fn group_by<I, S>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let fields = fields.into_iter().map(Into::into).collect();
        let aggregates = self
            .query
            .group_by
            .take()
            .map(|group| group.aggregates)
            .unwrap_or_default();
        self.query.group_by = Some(GroupBy { fields, aggregates });
        self
    }

    pub fn count(mut self) -> Self {
        self.ensure_group_by().aggregates.push(Aggregate {
            function: AggregateFunction::Count,
            field: "*".to_string(),
            alias: None,
        });
        self
    }

    pub fn sum(mut self, field: &str) -> Self {
        self.ensure_group_by().aggregates.push(Aggregate {
            function: AggregateFunction::Sum,
            field: field.to_string(),
            alias: None,
        });
        self
    }

    pub fn page(mut self, page: u32, page_size: u32) -> Self {
        self.query.paging = Some(PageRequest::new(page, page_size));
        self
    }

    pub fn build(self) -> Query<E> {
        self.query
    }

    pub async fn query<T>(
        repository: &Repository<E>,
        sql: &str,
        params: &[Value],
    ) -> DataResult<Vec<T>>
    where
        T: DeserializeOwned + Send + Sync + 'static,
    {
        repository.raw_query::<T>(sql, params).await
    }

    fn ensure_group_by(&mut self) -> &mut GroupBy {
        if self.query.group_by.is_none() {
            self.query.group_by = Some(GroupBy {
                fields: Vec::new(),
                aggregates: Vec::new(),
            });
        }
        self.query.group_by.as_mut().expect("group by exists")
    }
}
