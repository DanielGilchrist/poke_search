use std::fmt;

const DEFAULT_CAPACITY: usize = 1500;

pub struct Builder(String);

impl Builder {
    pub fn new(capacity: usize) -> Self {
        Self(String::with_capacity(capacity))
    }

    pub fn empty() -> Self {
        Self::new(0)
    }

    pub fn append<T: Appendable>(&mut self, str: T) {
        self.0.push_str(str.to_appendable());
    }

    pub fn append_c(&mut self, c: char) {
        self.0.push(c);
    }

    pub fn print(&self) {
        println!("{self}");
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new(DEFAULT_CAPACITY)
    }
}

impl fmt::Display for Builder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait Appendable {
    fn to_appendable(&self) -> &str;
}

impl Appendable for String {
    fn to_appendable(&self) -> &str {
        self.as_str()
    }
}

impl Appendable for &str {
    fn to_appendable(&self) -> &str {
        self
    }
}
