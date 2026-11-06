//! In-memory search over note titles and body text.
//!
//! Matching is case-insensitive and diacritic-insensitive, which is what French note titles
//! need in practice ("préférence" must match "preference").

use std::cmp::Ordering;

use uuid::Uuid;

use super::models::Note;

/// Fold a string for comparison: lowercase plus French diacritics removal.
pub fn normalize(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        let lower = ch.to_lowercase().next().unwrap_or(ch);
        let folded: &str = match lower {
            'à' | 'â' | 'ä' | 'á' | 'ã' | 'å' | 'ā' => "a",
            'ç' | 'ć' | 'č' => "c",
            'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ė' => "e",
            'ì' | 'í' | 'î' | 'ï' | 'ī' => "i",
            'ñ' | 'ń' => "n",
            'ò' | 'ó' | 'ô' | 'ö' | 'õ' | 'ø' | 'ō' => "o",
            'ù' | 'ú' | 'û' | 'ü' | 'ū' => "u",
            'ý' | 'ÿ' => "y",
            'œ' => "oe",
            'æ' => "ae",
            'ß' => "ss",
            other => {
                out.push(other);
                continue;
            }
        };
        out.push_str(folded);
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHit {
    pub note_id: Uuid,
    pub score: u8,
}

/// Rank notes matching `query`. Empty query returns every note in library order.
pub fn search(notes: &[Note], query: &str) -> Vec<SearchHit> {
    let query = normalize(query.trim());
    if query.is_empty() {
        return notes
            .iter()
            .map(|note| SearchHit {
                note_id: note.id,
                score: 1,
            })
            .collect();
    }

    let mut hits: Vec<SearchHit> = notes
        .iter()
        .filter_map(|note| {
            let title = normalize(&note.title);
            let body = normalize(&note.plain_text());
            if title.contains(&query) {
                Some(SearchHit {
                    note_id: note.id,
                    score: 3,
                })
            } else if body.contains(&query) {
                Some(SearchHit {
                    note_id: note.id,
                    score: 1,
                })
            } else {
                None
            }
        })
        .collect();

    hits.sort_by(|a, b| match b.score.cmp(&a.score) {
        Ordering::Equal => Ordering::Equal,
        other => other,
    });
    hits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::NoteBlock;

    fn note_with(title: &str, body: &str) -> Note {
        let mut note = Note::new().with_title(title);
        note.blocks = vec![NoteBlock::text(body)];
        note
    }

    #[test]
    fn normalize_removes_french_accents() {
        assert_eq!(normalize("Préférence, garçon !"), "preference, garcon !");
        assert_eq!(normalize("ŒUVRE"), "oeuvre");
    }

    #[test]
    fn title_matches_rank_above_body() {
        let a = note_with("Test GPT", "rien");
        let b = note_with("Autre", "un test gpt ici");
        let hits = search(&[a.clone(), b.clone()], "gpt");
        assert_eq!(hits[0].note_id, a.id);
        assert_eq!(hits[1].note_id, b.id);
    }

    #[test]
    fn search_is_accent_insensitive() {
        let note = note_with("Préférence", "");
        let hits = search(std::slice::from_ref(&note), "preference");
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn empty_query_returns_all() {
        let a = note_with("a", "");
        let b = note_with("b", "");
        assert_eq!(search(&[a, b], "  ").len(), 2);
    }
}
