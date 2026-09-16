// ══════════════════════════════════════════════════
// TOKENIZER
// ══════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TokenKind {
    Keyword,
    Identifier,
    StringLit,
    Number,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    LParen,
    RParen,
    Arrow,
    LeftArrow,
    ColonPair,
    Plus,
    Comma,
    Price,
    Method,
    Path,
    EnvRef,
    Operator,
    Pipe,
    DocComment,
    Eof,
}

#[derive(Debug, Clone)]
pub(crate) struct Token {
    pub kind: TokenKind,
    pub value: String,
    /// 1-based source line.
    pub line: usize,
    /// 1-based column (in chars) of the token's first character.
    pub col: usize,
}

impl Token {
    /// Width of the token in source chars (string literals include their quotes).
    pub fn width(&self) -> usize {
        match self.kind {
            TokenKind::StringLit => self.value.chars().count() + 2,
            TokenKind::Eof => 0,
            _ => self.value.chars().count(),
        }
    }
}

pub(crate) const KEYWORDS: &[&str] = &[
    "app",
    "entity",
    "api",
    "page",
    "style",
    "service",
    "section",
    "import",
    "compose",
    "use",
    "merge",
    "on",
    "worker",
    "component",
    "middleware",
    "env",
    "test",
    "webhook",
    "constitution",
    "must",
    "never",
    "transition",
    "deploy",
];

pub(crate) const METHODS: &[&str] = &["GET", "POST", "PATCH", "PUT", "DELETE"];

pub(crate) fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-' || c == '.'
}

pub(crate) fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    for (line_idx, line) in lines.iter().enumerate() {
        let line_num = line_idx + 1;
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // skip whitespace
            if chars[i] == ' ' || chars[i] == '\t' {
                i += 1;
                continue;
            }
            let tok_start = i;

            // doc-comment: /// preserved in AST
            if i + 2 < chars.len() && chars[i] == '/' && chars[i + 1] == '/' && chars[i + 2] == '/'
            {
                let doc_text: String = chars[i + 3..].iter().collect();
                tokens.push(Token {
                    kind: TokenKind::DocComment,
                    value: doc_text.trim().to_string(),
                    line: line_num,
                    col: tok_start + 1,
                });
                break;
            }

            // hex color literal: #abc123 (before comment check)
            if chars[i] == '#' && i + 1 < chars.len() && chars[i + 1].is_ascii_hexdigit() {
                let start = i;
                i += 1;
                while i < chars.len() && chars[i].is_ascii_hexdigit() {
                    i += 1;
                }
                let value: String = chars[start..i].iter().collect();
                tokens.push(Token {
                    kind: TokenKind::Identifier,
                    value,
                    line: line_num,
                    col: tok_start + 1,
                });
                continue;
            }

            // comment (discarded)
            if chars[i] == '#' {
                break;
            }

            // string literal
            if chars[i] == '"' {
                let start = i;
                i += 1;
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\' {
                        i += 1;
                    }
                    i += 1;
                }
                if i < chars.len() {
                    i += 1;
                } // closing quote (guard against unclosed string)
                let raw: String = chars[start..i].iter().collect();
                let value = if raw.len() >= 2 {
                    raw[1..raw.len() - 1].to_string()
                } else {
                    String::new()
                };
                tokens.push(Token {
                    kind: TokenKind::StringLit,
                    value,
                    line: line_num,
                    col: tok_start + 1,
                });
                continue;
            }

            // braces/brackets/comma/plus
            match chars[i] {
                '{' => {
                    tokens.push(Token {
                        kind: TokenKind::LBrace,
                        value: "{".into(),
                        line: line_num,
                        col: tok_start + 1,
                    });
                    i += 1;
                    continue;
                }
                '}' => {
                    tokens.push(Token {
                        kind: TokenKind::RBrace,
                        value: "}".into(),
                        line: line_num,
                        col: tok_start + 1,
                    });
                    i += 1;
                    continue;
                }
                '[' => {
                    tokens.push(Token {
                        kind: TokenKind::LBracket,
                        value: "[".into(),
                        line: line_num,
                        col: tok_start + 1,
                    });
                    i += 1;
                    continue;
                }
                ']' => {
                    tokens.push(Token {
                        kind: TokenKind::RBracket,
                        value: "]".into(),
                        line: line_num,
                        col: tok_start + 1,
                    });
                    i += 1;
                    continue;
                }
                ',' => {
                    tokens.push(Token {
                        kind: TokenKind::Comma,
                        value: ",".into(),
                        line: line_num,
                        col: tok_start + 1,
                    });
                    i += 1;
                    continue;
                }
                '(' => {
                    tokens.push(Token {
                        kind: TokenKind::LParen,
                        value: "(".into(),
                        line: line_num,
                        col: tok_start + 1,
                    });
                    i += 1;
                    continue;
                }
                ')' => {
                    tokens.push(Token {
                        kind: TokenKind::RParen,
                        value: ")".into(),
                        line: line_num,
                        col: tok_start + 1,
                    });
                    i += 1;
                    continue;
                }
                '+' => {
                    tokens.push(Token {
                        kind: TokenKind::Plus,
                        value: "+".into(),
                        line: line_num,
                        col: tok_start + 1,
                    });
                    i += 1;
                    continue;
                }
                '|' => {
                    tokens.push(Token {
                        kind: TokenKind::Pipe,
                        value: "|".into(),
                        line: line_num,
                        col: tok_start + 1,
                    });
                    i += 1;
                    continue;
                }
                _ => {}
            }

            // comparison and assignment operators: ==, =, !=, >=, <=, >, <
            if chars[i] == '=' && i + 1 < chars.len() && chars[i + 1] == '=' {
                tokens.push(Token {
                    kind: TokenKind::Operator,
                    value: "==".into(),
                    line: line_num,
                    col: tok_start + 1,
                });
                i += 2;
                continue;
            }
            if chars[i] == '=' {
                tokens.push(Token {
                    kind: TokenKind::Operator,
                    value: "=".into(),
                    line: line_num,
                    col: tok_start + 1,
                });
                i += 1;
                continue;
            }
            if chars[i] == '!' {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token {
                        kind: TokenKind::Operator,
                        value: "!=".into(),
                        line: line_num,
                        col: tok_start + 1,
                    });
                    i += 2;
                    continue;
                }
                // Standalone ! emitted as Identifier (required shorthand in fields)
                tokens.push(Token {
                    kind: TokenKind::Identifier,
                    value: "!".into(),
                    line: line_num,
                    col: tok_start + 1,
                });
                i += 1;
                continue;
            }
            if chars[i] == '>' && i + 1 < chars.len() && chars[i + 1] == '=' {
                tokens.push(Token {
                    kind: TokenKind::Operator,
                    value: ">=".into(),
                    line: line_num,
                    col: tok_start + 1,
                });
                i += 2;
                continue;
            }
            if chars[i] == '<' && i + 1 < chars.len() && chars[i + 1] == '-' {
                tokens.push(Token {
                    kind: TokenKind::LeftArrow,
                    value: "<-".into(),
                    line: line_num,
                    col: tok_start + 1,
                });
                i += 2;
                continue;
            }
            if chars[i] == '<' && i + 1 < chars.len() && chars[i + 1] == '=' {
                tokens.push(Token {
                    kind: TokenKind::Operator,
                    value: "<=".into(),
                    line: line_num,
                    col: tok_start + 1,
                });
                i += 2;
                continue;
            }
            if chars[i] == '>' {
                tokens.push(Token {
                    kind: TokenKind::Operator,
                    value: ">".into(),
                    line: line_num,
                    col: tok_start + 1,
                });
                i += 1;
                continue;
            }
            if chars[i] == '<' {
                tokens.push(Token {
                    kind: TokenKind::Operator,
                    value: "<".into(),
                    line: line_num,
                    col: tok_start + 1,
                });
                i += 1;
                continue;
            }

            // arrow ->
            if chars[i] == '-' && i + 1 < chars.len() && chars[i + 1] == '>' {
                tokens.push(Token {
                    kind: TokenKind::Arrow,
                    value: "->".into(),
                    line: line_num,
                    col: tok_start + 1,
                });
                i += 2;
                continue;
            }

            // price: $29/mo
            if chars[i] == '$' {
                let start = i;
                i += 1;
                while i < chars.len()
                    && chars[i] != ' '
                    && chars[i] != '\t'
                    && chars[i] != '['
                    && chars[i] != ']'
                {
                    i += 1;
                }
                let value: String = chars[start..i].iter().collect();
                tokens.push(Token {
                    kind: TokenKind::Price,
                    value,
                    line: line_num,
                    col: tok_start + 1,
                });
                continue;
            }

            // path or word
            if chars[i] == '/' || is_word_char(chars[i]) {
                let start = i;

                // path starts with /
                if chars[i] == '/' {
                    i += 1;
                    while i < chars.len()
                        && !chars[i].is_whitespace()
                        && chars[i] != '{'
                        && chars[i] != '}'
                        && chars[i] != '['
                        && chars[i] != ']'
                    {
                        i += 1;
                    }
                    let value: String = chars[start..i].iter().collect();
                    tokens.push(Token {
                        kind: TokenKind::Path,
                        value,
                        line: line_num,
                        col: tok_start + 1,
                    });
                    continue;
                }

                // read full word
                while i < chars.len()
                    && !chars[i].is_whitespace()
                    && chars[i] != '{'
                    && chars[i] != '}'
                    && chars[i] != '['
                    && chars[i] != ']'
                    && chars[i] != ','
                    && chars[i] != '+'
                    && chars[i] != ')'
                {
                    if chars[i] == '(' {
                        // Check if this is env(...) or role(...) — consume as part of word
                        let word_so_far: String = chars[start..i].iter().collect();
                        if word_so_far == "env"
                            || word_so_far == "role"
                            || word_so_far == "sum"
                            || word_so_far == "count"
                            || word_so_far.ends_with(":role")
                            || word_so_far.ends_with(":env")
                        {
                            i += 1;
                            while i < chars.len() && chars[i] != ')' {
                                i += 1;
                            }
                            if i < chars.len() {
                                i += 1;
                            }
                            continue;
                        }
                        // Otherwise, stop the word here — ( will be tokenized as LParen
                        break;
                    }
                    if chars[i] == '-' && i + 1 < chars.len() && chars[i + 1] == '>' {
                        break;
                    }
                    if chars[i] == '<' && i + 1 < chars.len() && chars[i + 1] == '-' {
                        break;
                    }
                    // When a word contains a quoted value (e.g. value:"12,842"),
                    // consume the entire quoted string including commas and spaces
                    if chars[i] == '"' {
                        i += 1; // opening quote
                        while i < chars.len() && chars[i] != '"' {
                            if chars[i] == '\\' {
                                i += 1;
                            } // skip escaped chars
                            i += 1;
                        }
                        if i < chars.len() {
                            i += 1;
                        } // closing quote
                        break; // end of word after closing quote
                    }
                    i += 1;
                }

                let word: String = chars[start..i].iter().collect();
                if word.is_empty() {
                    i += 1;
                    continue;
                }

                // classify
                if METHODS.contains(&word.as_str()) {
                    tokens.push(Token {
                        kind: TokenKind::Method,
                        value: word,
                        line: line_num,
                        col: tok_start + 1,
                    });
                } else if word.starts_with("env(") && word.ends_with(')') {
                    tokens.push(Token {
                        kind: TokenKind::EnvRef,
                        value: word,
                        line: line_num,
                        col: tok_start + 1,
                    });
                } else if word.contains(':') && !word.starts_with('/') {
                    tokens.push(Token {
                        kind: TokenKind::ColonPair,
                        value: word,
                        line: line_num,
                        col: tok_start + 1,
                    });
                } else if word.starts_with('/') {
                    tokens.push(Token {
                        kind: TokenKind::Path,
                        value: word,
                        line: line_num,
                        col: tok_start + 1,
                    });
                } else if word.chars().all(|c| c.is_ascii_digit()) {
                    tokens.push(Token {
                        kind: TokenKind::Number,
                        value: word,
                        line: line_num,
                        col: tok_start + 1,
                    });
                } else if KEYWORDS.contains(&word.as_str()) {
                    tokens.push(Token {
                        kind: TokenKind::Keyword,
                        value: word,
                        line: line_num,
                        col: tok_start + 1,
                    });
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Identifier,
                        value: word,
                        line: line_num,
                        col: tok_start + 1,
                    });
                }
                continue;
            }

            i += 1;
        }
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        value: String::new(),
        line: lines.len().max(1),
        col: lines.last().map(|l| l.chars().count() + 1).unwrap_or(1),
    });
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_carry_line_and_column() {
        let src = "entity Task {\n  title  \"x y\"  -> User\n\tqty number!\n}";
        let toks = tokenize(src);
        let find = |v: &str| toks.iter().find(|t| t.value == v).unwrap().clone();
        assert_eq!((find("entity").line, find("entity").col), (1, 1));
        assert_eq!((find("Task").line, find("Task").col), (1, 8));
        assert_eq!((find("{").line, find("{").col), (1, 13));
        assert_eq!((find("title").line, find("title").col), (2, 3));
        let s = find("x y");
        assert_eq!((s.line, s.col, s.width()), (2, 10, 5));
        assert_eq!((find("->").line, find("->").col), (2, 17));
        assert_eq!((find("User").line, find("User").col), (2, 20));
        // tab counts as one column
        assert_eq!((find("qty").line, find("qty").col), (3, 2));
        assert_eq!((find("number!").line, find("number!").col), (3, 6));
        let eof = toks.last().unwrap();
        assert_eq!(eof.kind, TokenKind::Eof);
        assert_eq!((eof.line, eof.col), (4, 2));
    }

    #[test]
    fn eof_of_empty_source_is_line_one() {
        let toks = tokenize("");
        assert_eq!((toks[0].line, toks[0].col), (1, 1));
    }
}
