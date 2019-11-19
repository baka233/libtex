use std::iter::Iterator;
use std::fmt::{self, Display};

#[derive(Debug, Eq, Clone, PartialEq, Ord, PartialOrd, Hash)]
pub enum Token {
    Terminal(String),
    NonTerminal(String),
    Epsilon,
}

impl Display for Token {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Terminal(s) => write!(f, "{}", s),
            Token::NonTerminal(s) => write!(f, "{}", s),
            Token::Epsilon => write!(f, "ε"),
        }
    }
}

impl Token {
    pub fn parse_token(s : &str) -> TokenStreamer {
        TokenStreamer::new(s)
    }

    pub fn is_non_terminal(&self) -> bool {
        match *self {
            Token::NonTerminal(_) => true,
            _ => false, 
        }
    }

    pub fn is_terminal(&self) -> bool {
        match *self {
            Token::Terminal(_) => true,
            _ => false, 
        }
    }

    pub fn is_epsilon(&self) -> bool {
        match *self {
            Token::Epsilon => true,
            _ => false, 
        }
    }

}


pub struct TokenStreamer<'a> {
    s : &'a str,
    pos : usize,    
}

impl<'a> TokenStreamer<'a> {
    fn new(s : &'a str) -> Self {
        TokenStreamer {
            s,
            pos : 0,
        } 
    }
}

impl<'a> Iterator for TokenStreamer<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        let slice = self.s.get(self.pos..self.s.len());
        if slice.is_none() || self.pos >= self.s.len() {
            return None;
        }
        let slice = slice.unwrap();


        self.pos+= 1;

        // 简单实现
        let ch = slice.chars().next().unwrap();
        match ch {
            'A'..='Z' => Some(Token::NonTerminal(ch.to_string())),
            'ε' => Some(Token::Epsilon),
            _ => Some(Token::Terminal(ch.to_string())),
        }
    }
}
