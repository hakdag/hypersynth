use std::sync::Arc;

use sqlx::{Postgres, Transaction};
use tokio::sync::Mutex;

/// Request-scoped transaction handle.
///
/// The audit middleware opens one Postgres transaction per HTTP request,
/// stamps actor/context GUCs onto it, and stores this handle in the
/// request extensions. Route handlers extract it via the `Tx` extractor
/// and run all of their queries through it so that the row-change
/// triggers see consistent actor context for the entire request.
///
/// The inner `Option` is taken by the middleware at the end of the
/// request to either commit or roll back. Once taken, further handler
/// queries will see `None` and must surface an internal error.
#[derive(Clone)]
pub struct Tx(pub Arc<Mutex<Option<Transaction<'static, Postgres>>>>);

impl Tx {
    pub fn new(tx: Transaction<'static, Postgres>) -> Self {
        Self(Arc::new(Mutex::new(Some(tx))))
    }
}
