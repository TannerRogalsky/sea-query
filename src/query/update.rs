use crate::{
    backend::QueryBuilder,
    expr::*,
    prepare::*,
    query::{condition::*, OrderedStatement},
    types::*,
    value::*,
    QueryStatementBuilder, QueryStatementWriter, ReturningClause, SubQueryStatement, WithClause,
    WithQuery,
};

/// Update existing rows in the table
///
/// # Examples
///
/// ```
/// use sea_query::{tests_cfg::*, *};
///
/// let query = Query::update()
///     .table(Glyph::Table)
///     .values([(Glyph::Aspect, 1.23.into()), (Glyph::Image, "123".into())])
///     .and_where(Expr::col(Glyph::Id).eq(1))
///     .to_owned();
///
/// assert_eq!(
///     query.to_string(MysqlQueryBuilder),
///     r#"UPDATE `glyph` SET `aspect` = 1.23, `image` = '123' WHERE `id` = 1"#
/// );
/// assert_eq!(
///     query.to_string(PostgresQueryBuilder),
///     r#"UPDATE "glyph" SET "aspect" = 1.23, "image" = '123' WHERE "id" = 1"#
/// );
/// assert_eq!(
///     query.to_string(SqliteQueryBuilder),
///     r#"UPDATE "glyph" SET "aspect" = 1.23, "image" = '123' WHERE "id" = 1"#
/// );
/// ```
#[derive(Default, Debug, Clone)]
pub struct UpdateStatement {
    pub(crate) table: Option<Box<TableRef>>,
    pub(crate) values: Vec<(DynIden, Box<SimpleExpr>)>,
    pub(crate) r#where: ConditionHolder,
    pub(crate) orders: Vec<OrderExpr>,
    pub(crate) limit: Option<Value>,
    pub(crate) returning: Option<ReturningClause>,
}

impl UpdateStatement {
    /// Construct a new [`UpdateStatement`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Specify which table to update.
    ///
    /// # Examples
    ///
    /// See [`UpdateStatement::values`]
    #[allow(clippy::wrong_self_convention)]
    pub fn table<T>(&mut self, tbl_ref: T) -> &mut Self
    where
        T: IntoTableRef,
    {
        self.table = Some(Box::new(tbl_ref.into_table_ref()));
        self
    }

    /// Update column values. To set multiple column-value pairs at once.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::update()
    ///     .table(Glyph::Table)
    ///     .values([
    ///         (Glyph::Aspect, 2.1345.into()),
    ///         (Glyph::Image, "235m".into()),
    ///     ])
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"UPDATE `glyph` SET `aspect` = 2.1345, `image` = '235m'"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"UPDATE "glyph" SET "aspect" = 2.1345, "image" = '235m'"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"UPDATE "glyph" SET "aspect" = 2.1345, "image" = '235m'"#
    /// );
    /// ```
    pub fn values<T, I>(&mut self, values: I) -> &mut Self
    where
        T: IntoIden,
        I: IntoIterator<Item = (T, SimpleExpr)>,
    {
        for (k, v) in values.into_iter() {
            self.values.push((k.into_iden(), Box::new(v)));
        }
        self
    }

    /// Update column value by [`SimpleExpr`].
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::update()
    ///     .table(Glyph::Table)
    ///     .value(Glyph::Aspect, Expr::cust("60 * 24 * 24"))
    ///     .values([
    ///         (Glyph::Image, "24B0E11951B03B07F8300FD003983F03F0780060".into()),
    ///     ])
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"UPDATE `glyph` SET `aspect` = 60 * 24 * 24, `image` = '24B0E11951B03B07F8300FD003983F03F0780060'"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"UPDATE "glyph" SET "aspect" = 60 * 24 * 24, "image" = '24B0E11951B03B07F8300FD003983F03F0780060'"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"UPDATE "glyph" SET "aspect" = 60 * 24 * 24, "image" = '24B0E11951B03B07F8300FD003983F03F0780060'"#
    /// );
    /// ```
    pub fn value<C, T>(&mut self, col: C, value: T) -> &mut Self
    where
        C: IntoIden,
        T: Into<SimpleExpr>,
    {
        self.values.push((col.into_iden(), Box::new(value.into())));
        self
    }

    /// Limit number of updated rows.
    pub fn limit(&mut self, limit: u64) -> &mut Self {
        self.limit = Some(limit.into());
        self
    }

    /// RETURNING expressions.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::update()
    ///     .table(Glyph::Table)
    ///     .value(Glyph::Aspect, 2.1345)
    ///     .value(Glyph::Image, "235m")
    ///     .returning(Query::returning().columns([Glyph::Id]))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"UPDATE `glyph` SET `aspect` = 2.1345, `image` = '235m'"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"UPDATE "glyph" SET "aspect" = 2.1345, "image" = '235m' RETURNING "id""#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"UPDATE "glyph" SET "aspect" = 2.1345, "image" = '235m' RETURNING "id""#
    /// );
    /// ```
    pub fn returning(&mut self, returning: ReturningClause) -> &mut Self {
        self.returning = Some(returning);
        self
    }

    /// RETURNING expressions for a column.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::update()
    ///     .table(Glyph::Table)
    ///     .table(Glyph::Table)
    ///     .value(Glyph::Aspect, 2.1345)
    ///     .value(Glyph::Image, "235m")
    ///     .returning_col(Glyph::Id)
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"UPDATE `glyph` SET `aspect` = 2.1345, `image` = '235m'"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"UPDATE "glyph" SET "aspect" = 2.1345, "image" = '235m' RETURNING "id""#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"UPDATE "glyph" SET "aspect" = 2.1345, "image" = '235m' RETURNING "id""#
    /// );
    /// ```
    pub fn returning_col<C>(&mut self, col: C) -> &mut Self
    where
        C: IntoColumnRef,
    {
        self.returning(ReturningClause::Columns(vec![col.into_column_ref()]))
    }

    /// RETURNING expressions all columns.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::update()
    ///     .table(Glyph::Table)
    ///     .table(Glyph::Table)
    ///     .value(Glyph::Aspect, 2.1345)
    ///     .value(Glyph::Image, "235m")
    ///     .returning_all()
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"UPDATE `glyph` SET `aspect` = 2.1345, `image` = '235m'"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"UPDATE "glyph" SET "aspect" = 2.1345, "image" = '235m' RETURNING *"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"UPDATE "glyph" SET "aspect" = 2.1345, "image" = '235m' RETURNING *"#
    /// );
    /// ```
    pub fn returning_all(&mut self) -> &mut Self {
        self.returning(ReturningClause::All)
    }

    /// Create a [WithQuery] by specifying a [WithClause] to execute this query with.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, IntoCondition, IntoIden, tests_cfg::*};
    ///
    /// let select = SelectStatement::new()
    ///         .columns([Glyph::Id])
    ///         .from(Glyph::Table)
    ///         .and_where(Expr::col(Glyph::Image).like("0%"))
    ///         .to_owned();
    ///     let cte = CommonTableExpression::new()
    ///         .query(select)
    ///         .column(Glyph::Id)
    ///         .table_name(Alias::new("cte"))
    ///         .to_owned();
    ///     let with_clause = WithClause::new().cte(cte).to_owned();
    ///     let update = UpdateStatement::new()
    ///         .table(Glyph::Table)
    ///         .and_where(Expr::col(Glyph::Id).in_subquery(SelectStatement::new().column(Glyph::Id).from(Alias::new("cte")).to_owned()))
    ///         .value(Glyph::Aspect, Expr::cust("60 * 24 * 24"))
    ///         .to_owned();
    ///     let query = update.with(with_clause);
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"WITH `cte` (`id`) AS (SELECT `id` FROM `glyph` WHERE `image` LIKE '0%') UPDATE `glyph` SET `aspect` = 60 * 24 * 24 WHERE `id` IN (SELECT `id` FROM `cte`)"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"WITH "cte" ("id") AS (SELECT "id" FROM "glyph" WHERE "image" LIKE '0%') UPDATE "glyph" SET "aspect" = 60 * 24 * 24 WHERE "id" IN (SELECT "id" FROM "cte")"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"WITH "cte" ("id") AS (SELECT "id" FROM "glyph" WHERE "image" LIKE '0%') UPDATE "glyph" SET "aspect" = 60 * 24 * 24 WHERE "id" IN (SELECT "id" FROM "cte")"#
    /// );
    /// ```
    pub fn with(self, clause: WithClause) -> WithQuery {
        clause.query(self)
    }

    /// Get column values
    pub fn get_values(&self) -> &[(DynIden, Box<SimpleExpr>)] {
        &self.values
    }
}

impl QueryStatementBuilder for UpdateStatement {
    fn build_collect_any_into(&self, query_builder: &dyn QueryBuilder, sql: &mut dyn SqlWriter) {
        query_builder.prepare_update_statement(self, sql);
    }

    fn into_sub_query_statement(self) -> SubQueryStatement {
        SubQueryStatement::UpdateStatement(self)
    }
}

impl QueryStatementWriter for UpdateStatement {
    fn build_collect_into<T: QueryBuilder>(&self, query_builder: T, sql: &mut dyn SqlWriter) {
        query_builder.prepare_update_statement(self, sql);
    }
}

impl OrderedStatement for UpdateStatement {
    fn add_order_by(&mut self, order: OrderExpr) -> &mut Self {
        self.orders.push(order);
        self
    }

    fn clear_order_by(&mut self) -> &mut Self {
        self.orders = Vec::new();
        self
    }
}

impl ConditionalStatement for UpdateStatement {
    fn and_or_where(&mut self, condition: LogicalChainOper) -> &mut Self {
        self.r#where.add_and_or(condition);
        self
    }

    fn cond_where<C>(&mut self, condition: C) -> &mut Self
    where
        C: IntoCondition,
    {
        self.r#where.add_condition(condition.into_condition());
        self
    }
}
