use std::fmt;

const DEFAULT_CAPACITY: usize = 1500;

pub struct Builder(String);

impl Builder {
    pub fn new(capacity: usize) -> Self {
        Self(String::with_capacity(capacity))
    }

    pub fn append<T: Appendable>(&mut self, str: T) {
        str.append_self_to(&mut self.0);
    }

    pub fn appendln<T: Appendable>(&mut self, str: T) {
        self.append(str);
        self.newline();
    }

    pub fn newline(&mut self) {
        self.append('\n');
    }

    pub fn print(&self) {
        println!("{self}");
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn pop(&mut self) -> Option<char> {
        self.0.pop()
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new(DEFAULT_CAPACITY)
    }
}

impl fmt::Display for Builder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.trim_end())
    }
}

pub trait Appendable {
    fn append_self_to(&self, string: &mut String);
}

impl Appendable for Builder {
    fn append_self_to(&self, string: &mut String) {
        string.push_str(&self.0);
    }
}

impl Appendable for char {
    fn append_self_to(&self, string: &mut String) {
        string.push(*self);
    }
}

impl Appendable for String {
    fn append_self_to(&self, string: &mut String) {
        string.push_str(self);
    }
}

impl Appendable for &String {
    fn append_self_to(&self, string: &mut String) {
        string.push_str(self);
    }
}

impl Appendable for &str {
    fn append_self_to(&self, string: &mut String) {
        string.push_str(self)
    }
}
