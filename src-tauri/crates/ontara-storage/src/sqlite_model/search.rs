//! Full-text search over the index the migrations keep.

use ontara_core::{EntityId, TextHit};

use super::SqliteModel;
use crate::error::StorageError;

/// Words an excerpt shows around a match.
const EXCERPT_WORDS: i32 = 14;

/// Weights of the indexed columns, in table order: names weigh most, long prose least. The two
/// unindexed leading columns take a weight each so the rest line up.
const COLUMN_WEIGHTS: &str = "0, 0, 12.0, 8.0, 5.0, 3.0, 1.0, 1.5, 1.0";

/// Turns what somebody typed into an index query: every word must appear, the last one may be
/// a prefix, and nothing they type is read as query syntax.
fn expression(query: &str) -> Option<String> {
    let words: Vec<&str> = query
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect();
    let (last, rest) = words.split_last()?;
    let mut parts: Vec<String> = rest.iter().map(|word| format!("\"{word}\"")).collect();
    parts.push(format!("\"{last}\"*"));
    Some(parts.join(" "))
}

impl SqliteModel<'_> {
    pub(super) fn search(
        &self,
        query: &str,
        include_archived: bool,
        limit: usize,
    ) -> Result<Vec<TextHit>, StorageError> {
        let Some(expression) = expression(query) else {
            return Ok(Vec::new());
        };
        let sql = format!(
            "SELECT entity_id,
                    snippet(search_index, -1, char(2), char(3), '…', {EXCERPT_WORDS}),
                    bm25(search_index, {COLUMN_WEIGHTS}) AS rank
             FROM search_index
             WHERE search_index MATCH ?1 AND (?2 OR archived = 0)
             ORDER BY rank, entity_id
             LIMIT ?3"
        );
        let mut statement = self.connection.prepare_cached(&sql)?;
        let mut rows = statement.query(rusqlite::params![
            expression,
            include_archived,
            i64::try_from(limit).unwrap_or(i64::MAX)
        ])?;
        let mut hits = Vec::new();
        while let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            hits.push(TextHit {
                entity: id
                    .parse::<EntityId>()
                    .map_err(|error| StorageError::CorruptRecord {
                        table: "search_index",
                        id: id.clone(),
                        reason: error.to_string(),
                    })?,
                excerpt: row.get(1)?,
                rank: row.get(2)?,
            });
        }
        Ok(hits)
    }
}

#[cfg(test)]
mod tests {
    use ontara_core::{MATCH_END, MATCH_START};

    use super::*;

    #[test]
    fn what_is_typed_never_becomes_query_syntax() {
        assert_eq!(expression("  "), None);
        assert_eq!(expression("Ayla"), Some("\"Ayla\"*".to_owned()));
        assert_eq!(
            expression("the \"drowned\" light-house OR NEAR("),
            Some("\"the\" \"drowned\" \"light\" \"house\" \"OR\" \"NEAR\"*".to_owned())
        );
    }

    #[test]
    fn markers_are_control_characters_that_prose_does_not_use() {
        assert!(MATCH_START.is_control() && MATCH_END.is_control());
    }
}
