use crate::common::token::TokenKind;
use std::fmt::Display;

#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub enum Types {
    Void, // type-promotion order
    Char,
    Int,
    Long,
    Pointer(Box<Types>),
}
impl Types {
    pub fn into_vec() -> Vec<TokenKind> {
        vec![
            TokenKind::Char,
            TokenKind::Int,
            TokenKind::Void,
            TokenKind::Long,
        ]
    }
    // returns type-size in bytes
    pub fn size(&self) -> usize {
        match self {
            Types::Void => 0,
            Types::Char => 1,
            Types::Int => 4,
            Types::Pointer(_) | Types::Long => 8,
        }
    }
    pub fn reg_suffix(&self) -> &str {
        match self {
            Types::Void => unreachable!(),
            Types::Char => "b",
            Types::Int => "d",
            Types::Pointer(_) | Types::Long => "",
        }
    }
    pub fn suffix(&self) -> &str {
        match self {
            Types::Void => unreachable!(),
            Types::Char => "b",
            Types::Int => "l",
            Types::Pointer(_) | Types::Long => "q",
        }
    }
    pub fn return_reg(&self) -> &str {
        match self {
            Types::Void => unreachable!(),
            Types::Char => "%al",
            Types::Int => "%eax",
            Types::Pointer(_) | Types::Long => "%rax",
        }
    }
    pub fn pointer_to(&mut self) {
        *self = Types::Pointer(Box::new(self.clone()));
    }
    pub fn deref_at(&self) -> Option<Types> {
        match self {
            Types::Pointer(inner) => Some(*inner.clone()),
            _ => None,
        }
    }
}
impl Display for Types {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Types::Void => "void".to_string(),
                Types::Char => "char".to_string(),
                Types::Int => "int".to_string(),
                Types::Long => "long".to_string(),
                Types::Pointer(inside) => format!("{}*", inside),
            }
        )
    }
}
