//! Functions and methods for interfacing CKAN types with SQL

use super::*;
use rusqlite::{*, types::{ToSqlOutput, FromSql, FromSqlResult}};

macro_rules! default_blob_sql_impl(
	($t:ty) => (
		impl ToSql for $t {
			fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
				let bin = bincode::serialize(self).map_err(|e| rusqlite::Error::ToSqlConversionFailure(e))?;
				Ok(ToSqlOutput::Owned(rusqlite::types::Value::Blob(bin)))
			}
		}

		impl FromSql for $t {
			fn column_result(value: rusqlite::types::ValueRef<'_>) -> FromSqlResult<Self> {
				let t: $t = bincode::deserialize(value.as_blob()?).map_err(|e| rusqlite::types::FromSqlError::Other(e))?;
				FromSqlResult::Ok(t)
			}
		}
	)
);

impl ToSql for ReleaseStatus {
	fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
		Ok(ToSqlOutput::from(match self {
			ReleaseStatus::Stable => 0u8,
			ReleaseStatus::Testing => 1,
			ReleaseStatus::Development => 2,
		}))
	}
}

default_blob_sql_impl!(ModVersion);