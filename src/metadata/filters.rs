use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum MetadataFilter {
    Eq(Value),
    Ne(Value),
    Gt(Value),
    Gte(Value),
    Lt(Value),
    Lte(Value),
    Like(String),
    In(Vec<Value>),
    Between { lower: Value, upper: Value },
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetadataFilterExpr {
    Field {
        field: String,
        filter: MetadataFilter,
    },
    Exists {
        sql: String,
        params: Vec<Value>,
    },
    Custom {
        sql: String,
        params: Vec<Value>,
    },
    Raw(String),
    And(Vec<MetadataFilterExpr>),
    Or(Vec<MetadataFilterExpr>),
    Not(Box<MetadataFilterExpr>),
}

impl MetadataFilterExpr {
    pub fn eq(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Field {
            field: field.into(),
            filter: MetadataFilter::Eq(value.into()),
        }
    }

    pub fn ne(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Field {
            field: field.into(),
            filter: MetadataFilter::Ne(value.into()),
        }
    }

    pub fn gt(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Field {
            field: field.into(),
            filter: MetadataFilter::Gt(value.into()),
        }
    }

    pub fn gte(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Field {
            field: field.into(),
            filter: MetadataFilter::Gte(value.into()),
        }
    }

    pub fn lt(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Field {
            field: field.into(),
            filter: MetadataFilter::Lt(value.into()),
        }
    }

    pub fn lte(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Field {
            field: field.into(),
            filter: MetadataFilter::Lte(value.into()),
        }
    }

    pub fn like(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Field {
            field: field.into(),
            filter: MetadataFilter::Like(value.into()),
        }
    }

    pub fn in_list(field: impl Into<String>, values: Vec<Value>) -> Self {
        Self::Field {
            field: field.into(),
            filter: MetadataFilter::In(values),
        }
    }

    pub fn between(
        field: impl Into<String>,
        lower: impl Into<Value>,
        upper: impl Into<Value>,
    ) -> Self {
        Self::Field {
            field: field.into(),
            filter: MetadataFilter::Between {
                lower: lower.into(),
                upper: upper.into(),
            },
        }
    }

    pub fn is_null(field: impl Into<String>) -> Self {
        Self::Field {
            field: field.into(),
            filter: MetadataFilter::IsNull,
        }
    }

    pub fn is_not_null(field: impl Into<String>) -> Self {
        Self::Field {
            field: field.into(),
            filter: MetadataFilter::IsNotNull,
        }
    }

    pub fn exists(sql: impl Into<String>, params: Vec<Value>) -> Self {
        Self::Exists {
            sql: sql.into(),
            params,
        }
    }

    pub fn custom(sql: impl Into<String>, params: Vec<Value>) -> Self {
        Self::Custom {
            sql: sql.into(),
            params,
        }
    }

    pub fn raw(sql: impl Into<String>) -> Self {
        Self::Raw(sql.into())
    }

    pub fn and(filters: Vec<MetadataFilterExpr>) -> Self {
        Self::And(filters)
    }

    pub fn or(filters: Vec<MetadataFilterExpr>) -> Self {
        Self::Or(filters)
    }

    pub fn not(filter: MetadataFilterExpr) -> Self {
        Self::Not(Box::new(filter))
    }
}
