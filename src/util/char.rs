use crate::util::scan_iter::Newline;
use crate::util::scan_iter::CmpType;

impl Newline for char {
    fn is_newline(&self) -> bool {
        *self == '\n'
    }
}

impl CmpType for char {
    fn cmp_type(&self, other: &char) -> bool {
        *self == *other
    }
}

pub fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

pub fn is_alpha_or_underscore(c: char) -> bool {
    c == '_' || (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z')
}

pub fn is_alpha_numeric_or_underscore(c: char) -> bool {
    is_alpha_or_underscore(c) || is_digit(c)
}
