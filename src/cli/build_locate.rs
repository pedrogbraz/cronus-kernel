//! Map AST-level names (page routes, section types, entity and field names)
//! back to 1-based source positions, so every build diagnostic has a real
//! `line`/`col` even when the validator that produced it only knows names.

use crate::parser::{tokenize, Token, TokenKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub line: usize,
    pub col: usize,
    /// Width in chars (0 = point location).
    pub len: usize,
}

impl Pos {
    /// Start of file — the fallback when nothing better is known.
    pub const START: Pos = Pos {
        line: 1,
        col: 1,
        len: 0,
    };

    fn of(t: &Token) -> Pos {
        Pos {
            line: t.line,
            col: t.col,
            len: t.width(),
        }
    }

    fn at_or_after(&self, other: Pos) -> bool {
        (self.line, self.col) >= (other.line, other.col)
    }
}

struct PageLoc {
    route: String,
    route_pos: Pos,
    sections: Vec<(String, Pos)>,
}

pub struct SourceIndex {
    tokens: Vec<Token>,
    pages: Vec<PageLoc>,
    entities: Vec<(String, Pos)>,
    constitution: Option<Pos>,
    app: Option<Pos>,
}

impl SourceIndex {
    pub fn new(source: &str) -> Self {
        let tokens: Vec<Token> = tokenize(source)
            .into_iter()
            .filter(|t| t.kind != TokenKind::Eof)
            .collect();
        let mut pages: Vec<PageLoc> = Vec::new();
        let mut entities = Vec::new();
        let mut constitution = None;
        let mut app = None;
        for (i, t) in tokens.iter().enumerate() {
            if t.kind != TokenKind::Keyword {
                continue;
            }
            let next = tokens.get(i + 1);
            match t.value.as_str() {
                "page" => {
                    if let Some(n) = next.filter(|n| n.kind == TokenKind::StringLit) {
                        pages.push(PageLoc {
                            route: n.value.clone(),
                            route_pos: Pos::of(n),
                            sections: Vec::new(),
                        });
                    }
                }
                "section" => {
                    if let (Some(n), Some(p)) = (next, pages.last_mut()) {
                        p.sections.push((n.value.clone(), Pos::of(n)));
                    }
                }
                "entity" => {
                    // `entity Name {` / `entity Name shared {` — not `auth { entity User }`
                    let opens_block = tokens
                        .get(i + 2)
                        .is_some_and(|b| b.kind == TokenKind::LBrace || b.value == "shared");
                    if let Some(n) = next.filter(|_| opens_block) {
                        entities.push((n.value.clone(), Pos::of(n)));
                    }
                }
                "constitution" if constitution.is_none() => constitution = Some(Pos::of(t)),
                "app" if app.is_none() => app = Some(Pos::of(t)),
                _ => {}
            }
        }
        SourceIndex {
            tokens,
            pages,
            entities,
            constitution,
            app,
        }
    }

    /// Route string of the `nth` (0-based) page declared with `route`.
    pub fn page(&self, route: &str, nth: usize) -> Option<Pos> {
        self.pages
            .iter()
            .filter(|p| p.route == route)
            .nth(nth)
            .map(|p| p.route_pos)
    }

    /// Type token of the `nth` section of `section_type` inside that page.
    pub fn section(
        &self,
        route: &str,
        page_nth: usize,
        section_type: &str,
        nth: usize,
    ) -> Option<Pos> {
        self.pages
            .iter()
            .filter(|p| p.route == route)
            .nth(page_nth)?
            .sections
            .iter()
            .filter(|(t, _)| t == section_type)
            .nth(nth)
            .map(|(_, pos)| *pos)
    }

    /// Name token of an `entity Name { ... }` declaration.
    pub fn entity(&self, name: &str) -> Option<Pos> {
        self.entities
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, p)| *p)
    }

    /// Every `page "<route>"` in the token stream, in source order, regardless
    /// of how the parser nested it.
    pub fn declared_pages(&self) -> impl Iterator<Item = (&str, Pos)> {
        self.pages.iter().map(|p| (p.route.as_str(), p.route_pos))
    }

    /// Every `entity Name {` declaration in source order.
    pub fn declared_entities(&self) -> impl Iterator<Item = (&str, Pos)> {
        self.entities.iter().map(|(n, p)| (n.as_str(), *p))
    }

    /// `constitution` keyword, else `app` keyword.
    pub fn constitution(&self) -> Option<Pos> {
        self.constitution.or(self.app)
    }

    /// First occurrence of `word` at or after `from`: a whole token, the
    /// key/value of a `key:value` pair, or text inside a string literal.
    pub fn word(&self, word: &str, from: Option<Pos>) -> Option<Pos> {
        if word.is_empty() {
            return None;
        }
        let after = |t: &&Token| from.is_none_or(|f| Pos::of(t).at_or_after(f));
        let len = word.chars().count();
        let exact = self.tokens.iter().filter(after).find_map(|t| {
            if t.value == word && t.kind != TokenKind::StringLit {
                return Some(Pos::of(t));
            }
            if t.kind == TokenKind::ColonPair {
                let (k, v) = t.value.split_once(':')?;
                if k == word {
                    return Some(Pos { len, ..Pos::of(t) });
                }
                if v.trim_matches('"') == word {
                    let offset = t.value.find(word)?;
                    return Some(offset_pos(t, &t.value, offset, len, 0));
                }
            }
            None
        });
        exact.or_else(|| {
            self.tokens
                .iter()
                .filter(after)
                .filter(|t| t.kind == TokenKind::StringLit)
                .find_map(|t| {
                    let offset = t.value.find(word)?;
                    // +1 skips the opening quote
                    Some(offset_pos(t, &t.value, offset, len, 1))
                })
        })
    }
}

fn offset_pos(t: &Token, text: &str, byte_offset: usize, len: usize, extra: usize) -> Pos {
    Pos {
        line: t.line,
        col: t.col + extra + text[..byte_offset].chars().count(),
        len,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = "app \"T\" {\n  port 5175\n}\nauth {\n  entity User\n}\nentity User {\n  name string!\n}\npage \"/\" {\n  section kpi { bind User { aggregate count } }\n  section table { columns \"name, email\" }\n  section kpi { bind User { aggregate sum field:name } }\n}\npage \"/about\" {\n  section hero { }\n}\n";

    #[test]
    fn locates_pages_sections_and_entities() {
        let idx = SourceIndex::new(SRC);
        assert_eq!(
            idx.page("/about", 0),
            Some(Pos {
                line: 15,
                col: 6,
                len: 8
            })
        );
        assert_eq!(
            idx.section("/", 0, "kpi", 1).map(|p| (p.line, p.col)),
            Some((13, 11))
        );
        assert_eq!(
            idx.section("/", 0, "table", 0).map(|p| (p.line, p.col)),
            Some((12, 11))
        );
        assert_eq!(idx.section("/about", 0, "kpi", 0), None);
        // the auth block's `entity User` is not the declaration
        assert_eq!(idx.entity("User").map(|p| (p.line, p.col)), Some((7, 8)));
        assert_eq!(idx.constitution().map(|p| p.line), Some(1));
    }

    #[test]
    fn locates_words_in_colon_pairs_and_string_literals() {
        let idx = SourceIndex::new(SRC);
        let from = idx.section("/", 0, "kpi", 1);
        // `field:name` on line 13 → value part
        assert_eq!(
            idx.word("name", from),
            Some(Pos {
                line: 13,
                col: 49,
                len: 4
            })
        );
        // inside `columns "name, email"`
        assert_eq!(
            idx.word("email", idx.section("/", 0, "table", 0)),
            Some(Pos {
                line: 12,
                col: 34,
                len: 5
            })
        );
        assert_eq!(idx.word("nope", None), None);
    }
}
