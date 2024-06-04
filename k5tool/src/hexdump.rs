const WIDTH: usize = 0x10;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Line<'a, A = u16> {
    address: A,
    data: &'a [u8],
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DedupLine<'a, A = u16> {
    Data(Line<'a, A>),
    Duplicate,
}

pub fn printable(chr: u8) -> Option<char> {
    if (0x20..0x7f).contains(&chr) {
        Some(chr as char)
    } else {
        None
    }
}

pub trait FormatAddr: Sized {
    fn format_addr(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result;
    fn from_usize(n: usize) -> Option<Self>;
}

impl FormatAddr for u16 {
    fn format_addr(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:04x}", self)
    }

    fn from_usize(n: usize) -> Option<Self> {
        n.try_into().ok()
    }
}

impl<'a, A> std::fmt::Display for Line<'a, A>
where
    A: FormatAddr,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.address.format_addr(f)?;

        if self.data.is_empty() {
            return Ok(());
        }

        for i in 0..WIDTH {
            if i % 8 == 0 {
                write!(f, " ")?;
            }
            if i < self.data.len() {
                write!(f, " {:02x}", self.data[i])?;
            } else {
                write!(f, "   ")?;
            }
        }

        write!(f, "  |")?;

        for b in self.data {
            write!(f, "{}", printable(*b).unwrap_or('.'))?;
        }

        write!(f, "|")
    }
}

impl<'a, A> std::fmt::Display for DedupLine<'a, A>
where
    A: FormatAddr,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Data(line) => line.fmt(f),
            Self::Duplicate => {
                write!(f, "*")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineIter<'a, A = u16> {
    data: &'a [u8],
    next: usize,
    endline: bool,
    _phantom: std::marker::PhantomData<A>,
}

impl<'a, A> LineIter<'a, A> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            next: 0,
            endline: false,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, A> Iterator for LineIter<'a, A>
where
    A: FormatAddr,
{
    type Item = Line<'a, A>;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.next;
        if start >= self.data.len() {
            if self.endline {
                None
            } else {
                self.endline = true;
                Some(Line {
                    address: A::from_usize(self.next).expect("address too large"),
                    data: &[],
                })
            }
        } else {
            let end = (start + WIDTH).min(self.data.len());
            let part = &self.data[start..end];
            self.next = end;
            Some(Line {
                address: A::from_usize(start).expect("address too large"),
                data: part,
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DedupLineIter<'a, A = u16> {
    inner: LineIter<'a, A>,
    last: Option<&'a [u8]>,
    in_duplicate: bool,
}

impl<'a, A> DedupLineIter<'a, A> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            inner: LineIter::new(data),
            last: None,
            in_duplicate: false,
        }
    }
}

impl<'a, A> Iterator for DedupLineIter<'a, A>
where
    A: FormatAddr,
{
    type Item = DedupLine<'a, A>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(line) = self.inner.next() {
                if self.last == Some(line.data) {
                    if !self.in_duplicate {
                        self.in_duplicate = true;
                        return Some(DedupLine::Duplicate);
                    }
                } else {
                    self.last = Some(line.data);
                    self.in_duplicate = false;
                    return Some(DedupLine::Data(line));
                }
            } else {
                return None;
            }
        }
    }
}

pub fn hexdump_iter(data: &[u8]) -> DedupLineIter {
    DedupLineIter::new(data)
}

pub fn hexdump(data: &[u8]) {
    for line in hexdump_iter(data) {
        println!("{}", line);
    }
}

pub fn ehexdump(data: &[u8]) {
    for line in hexdump_iter(data) {
        eprintln!("{}", line);
    }
}

pub fn hexdump_prefix(prefix: &str, data: &[u8]) {
    for line in hexdump_iter(data) {
        println!("{}{}", prefix, line);
    }
}

pub fn ehexdump_prefix(prefix: &str, data: &[u8]) {
    for line in hexdump_iter(data) {
        eprintln!("{}{}", prefix, line);
    }
}

pub fn hexdump_format(data: &[u8]) -> String {
    let mut out = String::new();
    for line in hexdump_iter(data) {
        out += &format!("{}\n", line);
    }
    out
}
