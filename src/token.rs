#[derive(Debug,PartialEq)]
pub(crate) enum Token {
    Ident(String),
    Tag(usize),
    // Keywords
    Path,
    Deps,
    Run
}

pub(crate) const ESCAPE: char = '\\';

impl Token {
    // Tried to match ident with a keyword, otherwise returns an Ident.
    pub(crate) fn lookup(ident: &str) -> Token {
        match ident {
            "path" => return Token::Path,
            "deps" => return Token::Deps,
            "run" =>  return Token::Run,
            _ => {
                if let Ok(num) = ident.parse::<usize>() {
                    return Token::Tag(num);
                }
                return Token::Ident(String::from(ident));
            },
        }
    }
}
