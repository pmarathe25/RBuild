#[derive(Debug,PartialEq)]
pub(crate) enum Token {
    Ident(String),
    Num(usize),
    // Keywords
    Path,
    Deps,
    Run,
    Always,
    // Special Types
    Tag
}

pub(crate) const ESCAPE: char = '\\';

impl Token {
    // Tries to match ident with a keyword or tag, otherwise returns an Ident.
    pub(crate) fn lookup(ident: &str) -> Token {
        match ident {
            "path" => return Token::Path,
            "deps" => return Token::Deps,
            "run" =>  return Token::Run,
            "always" =>  return Token::Always,
            _ => {
                if let Ok(num) = ident.parse::<usize>() {
                    return Token::Num(num);
                }
                return Token::Ident(String::from(ident));
            },
        }
    }
}
