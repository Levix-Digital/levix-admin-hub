//! Conversions between domain values and columns.

use hub_core::CoreResult;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::StorageError;

pub(crate) fn parse_id(value: &str) -> CoreResult<Uuid> {
    Uuid::parse_str(value)
        .map_err(|e| StorageError::Corrupt(format!("identifier '{value}': {e}")).into())
}

pub(crate) fn parse_optional_id(value: Option<String>) -> CoreResult<Option<Uuid>> {
    value.as_deref().map(parse_id).transpose()
}

pub(crate) fn parse_time(value: &str) -> CoreResult<OffsetDateTime> {
    OffsetDateTime::parse(value, &Rfc3339)
        .map_err(|e| StorageError::Corrupt(format!("time '{value}': {e}")).into())
}

pub(crate) fn parse_optional_time(value: Option<String>) -> CoreResult<Option<OffsetDateTime>> {
    value.as_deref().map(parse_time).transpose()
}

pub(crate) fn time_text(value: OffsetDateTime) -> CoreResult<String> {
    value
        .format(&Rfc3339)
        .map_err(|e| StorageError::Corrupt(format!("time {value}: {e}")).into())
}

pub(crate) fn optional_time_text(value: Option<OffsetDateTime>) -> CoreResult<Option<String>> {
    value.map(time_text).transpose()
}
