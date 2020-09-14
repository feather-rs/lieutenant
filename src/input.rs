#[derive(Debug, Clone, Copy)]
pub struct Input<'a> {
    ptr: &'a str,
}

impl<'a> Input<'a> {
    pub fn new(ptr: &'a str) -> Self {
        Self { ptr }
    }

    pub fn take(&mut self, n: usize) -> &'a str {
        let n = self
            .ptr
            .char_indices()
            .skip(n)
            .next()
            .map(|(i, _)| i)
            .unwrap_or(0);
        let head = &self.ptr[..n];
        self.ptr = &self.ptr[n..];
        head
    }

    /// Advances the pointer by the number of trimed spaces. 
    pub fn trim_start(&mut self) {
        self.ptr = self.ptr.trim_start();
    }

    pub fn take_bytes(&mut self, n: usize) -> Option<&'a str> {
        let head = self.ptr.get(..n)?;
        self.ptr = &self.ptr[n..];
        Some(head)
    }

    /// Advances the pointer until the given pattern has been reached, returning
    /// the consumed characters.
    #[inline]
    pub fn advance_until<'b>(&'b mut self, pat: &str) -> &'a str {
        let head = self.ptr.split(pat).next().unwrap_or("");
        self.ptr = &self.ptr[(head.len() + pat.len()).min(self.ptr.len())..];
        head
    }

    /// Advances until the end of input, returning all
    /// consumed characters.
    #[inline]
    pub fn advance_to_end(&mut self) -> &'a str {
        let head = self.ptr;
        self.ptr = &self.ptr[self.ptr.len()..];
        head
    }

    /// Returns the number of remaining characters to read.
    #[inline]
    pub fn len(&self) -> usize {
        self.ptr.len()
    }

    /// Returns whether there are no more characters to read.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'i> From<&'i str> for Input<'i> {
    fn from(val: &'i str) -> Self {
        Input { ptr: val }
    }
}
