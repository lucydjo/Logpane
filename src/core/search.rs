//! In-memory search over note titles and body text.
//!
//! Matching is case-insensitive and diacritic-insensitive, which is what French note titles
//! need in practice ("préférence" must match "preference").

use std::cmp::Ordering;

use uuid::Uuid;

use super::models::Note;

/// Fold one character for comparison: lowercase plus common Latin diacritics removal.
fn fold_char(ch: char, out: &mut String) {
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
            return;
        }
    };
    out.push_str(folded);
}

/// Fold a string for comparison: lowercase plus French diacritics removal.
pub fn normalize(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        fold_char(ch, &mut out);
    }
    out
}

/// Locate the first match of `query` in `text`, with the same folding as [`search`].
///
/// Returns a half-open range of **character** indices into the original text, so the caller can
/// highlight what the user actually typed even when folding changed the length ("œ" → "oe").
pub fn find(text: &str, query: &str) -> Option<(usize, usize)> {
    let query = normalize(query.trim());
    if query.is_empty() {
        return None;
    }
    let mut folded = String::with_capacity(text.len());
    // For every folded byte, the index of the original character it came from.
    let mut origin: Vec<usize> = Vec::with_capacity(text.len());
    for (index, ch) in text.chars().enumerate() {
        let before = folded.len();
        fold_char(ch, &mut folded);
        origin.resize(origin.len() + (folded.len() - before), index);
    }
    let at = folded.find(&query)?;
    let first = *origin.get(at)?;
    let last = *origin.get(at + query.len() - 1)?;
    Some((first, last + 1))
}

/// A one-line excerpt around a match, with the matched range in character indices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snippet {
    pub text: String,
    pub start: usize,
    pub end: usize,
}

/// Excerpt of the note body around the first match of `query`, at most `max_chars` wide.
pub fn snippet(note: &Note, query: &str, max_chars: usize) -> Option<Snippet> {
    const LEAD: usize = 8;
    for text in note.blocks.iter().filter_map(|block| block.as_text()) {
        let Some((start, end)) = find(text, query) else {
            continue;
        };
        let from = start.saturating_sub(LEAD);
        let cut = from > 0;
        let budget = max_chars.saturating_sub(usize::from(cut)).max(end - from);
        let mut out = String::new();
        if cut {
            out.push('…');
        }
        let total = text.chars().count();
        for ch in text.chars().skip(from).take(budget) {
            out.push(if ch.is_whitespace() { ' ' } else { ch });
        }
        if from + budget < total {
            out.push('…');
        }
        let shift = usize::from(cut);
        return Some(Snippet {
            text: out,
            start: start - from + shift,
            end: end - from + shift,
        });
    }
    None
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

    #[test]
    fn find_returns_character_ranges_of_the_original_text() {
        assert_eq!(find("Préférence", "ference"), Some((3, 10)));
        assert_eq!(find("une ŒUVRE", "oeuvre"), Some((4, 9)));
        assert_eq!(find("une ŒUVRE", "euv"), Some((4, 7)));
        assert_eq!(find("abc", "zzz"), None);
        assert_eq!(find("abc", "  "), None);
    }

    #[test]
    fn snippet_is_centred_on_the_match_and_bounded() {
        let body = format!(
            "{} needle in\nthe haystack {}",
            "a".repeat(200),
            "b".repeat(200)
        );
        let note = note_with("t", &body);
        let snippet = snippet(&note, "NEEDLE", 30).expect("match");
        let chars: Vec<char> = snippet.text.chars().collect();
        assert!(chars.len() <= 32);
        let matched: String = chars[snippet.start..snippet.end].iter().collect();
        assert_eq!(matched, "needle");
        assert!(snippet.text.starts_with('…') && snippet.text.ends_with('…'));
        assert!(!snippet.text.contains('\n'));
    }

    #[test]
    fn snippet_is_none_without_a_body_match() {
        let note = note_with("needle", "nothing here");
        assert!(snippet(&note, "needle", 30).is_none());
    }
}
