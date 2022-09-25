//! Functions and methods for interfacing CKAN types with SQL

use super::*;

impl rusqlite::ToSql for ReleaseStatus {
	fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
		Ok(rusqlite::types::ToSqlOutput::from(match self {
			ReleaseStatus::Stable => 0u8,
			ReleaseStatus::Testing => 1,
			ReleaseStatus::Development => 2,
		}))
	}
}