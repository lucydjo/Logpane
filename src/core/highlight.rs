//! Lightweight syntax marking for note text.
//!
//! Notes stay plain text: nothing is rewritten or hidden. This module only classifies byte ranges
//! so the editor can tint timestamps, tags, links, inline code and a few line prefixes. It is pure
//! and independent from egui.

/// What a range of text is, as far as the editor tint is concerned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Plain,
    /// Leading `HH:MM` and its dash, as inserted by the timestamp shortcut.
    Stamp,
    /// `#tag`
    Tag,
    /// `http://` or `https://` address.
    Link,
    /// `` `inline code` ``, backticks included.
    Code,
    /// A `# heading` line.
    Heading,
    /// A `> quote` line.
    Quote,
    /// `- ` or `* ` list marker.
    Bullet,
    /// `- [ ]` marker of an open task.
    TodoOpen,
    /// A whole `- [x] done` line.
    TodoDone,
}

/// Half-open byte range `[start, end)` of the source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub kind: Kind,
}

/// Classify `text`. The spans are contiguous, ordered and cover the whole string.
pub fn spans(text: &str) -> Vec<Span> {
    let mut out = Vec::new();
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let content = line.strip_suffix('\n').unwrap_or(line);
        let content = content.strip_suffix('\r').unwrap_or(content);
        mark_line(content, offset, &mut out);
        push(
            &mut out,
            offset + content.len(),
            offset + line.len(),
            Kind::Plain,
        );
        offset += line.len();
    }
    out
}

fn push(out: &mut Vec<Span>, start: usize, end: usize, kind: Kind) {
    if start >= end {
        return;
    }
    if let Some(last) = out.last_mut() {
        if last.kind == kind && last.end == start {
            last.end = end;
            return;
        }
    }
    out.push(Span { start, end, kind });
}

fn mark_line(line: &str, base: usize, out: &mut Vec<Span>) {
    let indent = line.len() - line.trim_start_matches([' ', '\t']).len();
    let rest = &line[indent..];
    push(out, base, base + indent, Kind::Plain);
    let start = base + indent;

    if is_heading(rest) {
        push(out, start, start + rest.len(), Kind::Heading);
        return;
    }
    if rest == ">" || rest.starts_with("> ") {
        push(out, start, start + rest.len(), Kind::Quote);
        return;
    }
    for done in ["- [x] ", "- [X] ", "* [x] ", "* [X] "] {
        if rest.starts_with(done) {
            push(out, start, start + rest.len(), Kind::TodoDone);
            return;
        }
    }
    for open in ["- [ ] ", "* [ ] "] {
        if rest.starts_with(open) {
            let marker = open.len() - 1;
            push(out, start, start + marker, Kind::TodoOpen);
            inline(&rest[marker..], start + marker, out);
            return;
        }
    }
    if rest.starts_with("- ") || rest.starts_with("* ") {
        push(out, start, start + 1, Kind::Bullet);
        inline(&rest[1..], start + 1, out);
        return;
    }
    let stamp = stamp_len(rest);
    if stamp > 0 {
        push(out, start, start + stamp, Kind::Stamp);
        inline(&rest[stamp..], start + stamp, out);
        return;
    }
    inline(rest, start, out);
}

fn is_heading(rest: &str) -> bool {
    let hashes = rest.bytes().take_while(|byte| *byte == b'#').count();
    (1..=3).contains(&hashes) && rest[hashes..].starts_with(' ') && rest.len() > hashes + 1
}

/// Length of a leading `HH:MM`, plus a following ` — ` or ` - ` when present.
fn stamp_len(rest: &str) -> usize {
    let bytes = rest.as_bytes();
    if bytes.len() < 5 || bytes[2] != b':' {
        return 0;
    }
    let digit = |index: usize| bytes[index].is_ascii_digit();
    if !(digit(0) && digit(1) && digit(3) && digit(4)) {
        return 0;
    }
    let hours = (bytes[0] - b'0') * 10 + (bytes[1] - b'0');
    let minutes = (bytes[3] - b'0') * 10 + (bytes[4] - b'0');
    if hours > 23 || minutes > 59 {
        return 0;
    }
    let tail = &rest[5..];
    if tail.is_empty() {
        return 5;
    }
    for dash in [" — ", " – ", " - "] {
        if tail.starts_with(dash) {
            // The trailing space stays plain so the caret after a stamp is not tinted.
            return 5 + dash.len() - 1;
        }
    }
    if tail.starts_with(' ') {
        5
    } else {
        0
    }
}

fn inline(text: &str, base: usize, out: &mut Vec<Span>) {
    let mut plain_from = 0;
    let mut index = 0;
    let mut previous: Option<char> = None;
    while index < text.len() {
        let here = &text[index..];
        let Some(ch) = here.chars().next() else {
            break;
        };
        let matched = match ch {
            '`' => here[1..]
                .find('`')
                .filter(|close| *close > 0)
                .map(|close| (close + 2, Kind::Code)),
            'h' if boundary(previous) => link_len(here).map(|len| (len, Kind::Link)),
            '#' if boundary(previous) => tag_len(here).map(|len| (len, Kind::Tag)),
            _ => None,
        };
        match matched {
            Some((len, kind)) => {
                push(out, base + plain_from, base + index, Kind::Plain);
                push(out, base + index, base + index + len, kind);
                previous = here[..len].chars().next_back();
                index += len;
                plain_from = index;
            }
            None => {
                previous = Some(ch);
                index += ch.len_utf8();
            }
        }
    }
    push(out, base + plain_from, base + text.len(), Kind::Plain);
}

fn boundary(previous: Option<char>) -> bool {
    previous.is_none_or(|ch| ch.is_whitespace() || matches!(ch, '(' | '[' | '<' | '"' | '\''))
}

fn link_len(here: &str) -> Option<usize> {
    let scheme = ["https://", "http://"]
        .into_iter()
        .find(|scheme| here.starts_with(scheme))?;
    let end = here.find(char::is_whitespace).unwrap_or(here.len());
    let trimmed = here[..end].trim_end_matches(['.', ',', ';', ':', '!', '?', ')', ']', '>']);
    (trimmed.len() > scheme.len()).then_some(trimmed.len())
}

fn tag_len(here: &str) -> Option<usize> {
    let mut chars = here.char_indices().skip(1);
    let (_, first) = chars.next()?;
    if !first.is_alphabetic() {
        return None;
    }
    let mut end = 1 + first.len_utf8();
    for (offset, ch) in chars {
        if ch.is_alphanumeric() || matches!(ch, '_' | '-' | '/') {
            end = offset + ch.len_utf8();
        } else {
            break;
        }
    }
    Some(end)
}

/// Every `#tag` of a text, lowercased, in order of first appearance.
pub fn tags(text: &str) -> Vec<String> {
    let mut seen = Vec::new();
    for span in spans(text) {
        if span.kind == Kind::Tag {
            let tag = text[span.start + 1..span.end].to_lowercase();
            if !seen.contains(&tag) {
                seen.push(tag);
            }
        }
    }
    seen
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(text: &str) -> Vec<(&str, Kind)> {
        spans(text)
            .into_iter()
            .map(|span| (&text[span.start..span.end], span.kind))
            .collect()
    }

    #[test]
    fn spans_cover_the_text_exactly() {
        for text in [
            "",
            "plain",
            "09:15 — run `cargo test` see https://example.org/a. #perf\n\n- [ ] todo\n- [x] done\n> quote\n# Title\r\n  - item é👍",
            "`unterminated and #not\n#",
            "``",
        ] {
            let spans = spans(text);
            let mut cursor = 0;
            for span in &spans {
                assert_eq!(span.start, cursor, "gap in {text:?}");
                assert!(span.end > span.start);
                assert!(text.is_char_boundary(span.start) && text.is_char_boundary(span.end));
                cursor = span.end;
            }
            assert_eq!(cursor, text.len(), "uncovered tail in {text:?}");
        }
    }

    #[test]
    fn leading_timestamp_is_marked_with_its_dash() {
        assert_eq!(
            kinds("09:15 — baseline"),
            vec![("09:15 —", Kind::Stamp), (" baseline", Kind::Plain)]
        );
        assert_eq!(kinds("23:59"), vec![("23:59", Kind::Stamp)]);
    }

    #[test]
    fn impossible_or_embedded_times_stay_plain() {
        assert_eq!(kinds("25:00 — no"), vec![("25:00 — no", Kind::Plain)]);
        assert_eq!(kinds("at 09:15 — no"), vec![("at 09:15 — no", Kind::Plain)]);
        assert_eq!(kinds("09:15:30"), vec![("09:15:30", Kind::Plain)]);
    }

    #[test]
    fn tags_links_and_code_are_found_inline() {
        assert_eq!(
            kinds("see https://example.org/x, `a b` #perf/p95 ok"),
            vec![
                ("see ", Kind::Plain),
                ("https://example.org/x", Kind::Link),
                (", ", Kind::Plain),
                ("`a b`", Kind::Code),
                (" ", Kind::Plain),
                ("#perf/p95", Kind::Tag),
                (" ok", Kind::Plain),
            ]
        );
    }

    #[test]
    fn a_hash_inside_a_word_or_before_a_digit_is_not_a_tag() {
        assert_eq!(
            kinds("issue#12 #42 c#"),
            vec![("issue#12 #42 c#", Kind::Plain)]
        );
    }

    #[test]
    fn line_prefixes_are_recognised() {
        assert_eq!(kinds("# Title"), vec![("# Title", Kind::Heading)]);
        assert_eq!(kinds("> said"), vec![("> said", Kind::Quote)]);
        assert_eq!(kinds("- [x] done"), vec![("- [x] done", Kind::TodoDone)]);
        assert_eq!(
            kinds("- [ ] next"),
            vec![("- [ ]", Kind::TodoOpen), (" next", Kind::Plain)]
        );
        assert_eq!(
            kinds("  - item"),
            vec![
                ("  ", Kind::Plain),
                ("-", Kind::Bullet),
                (" item", Kind::Plain)
            ]
        );
        assert_eq!(kinds("#nospace"), vec![("#nospace", Kind::Tag)]);
    }

    #[test]
    fn tags_are_collected_once_lowercased() {
        assert_eq!(tags("#Perf and #perf then #ui"), vec!["perf", "ui"]);
    }
}
