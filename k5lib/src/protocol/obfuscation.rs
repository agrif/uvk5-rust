pub const OBFUSCATION: [u8; 16] = [
    0x16, 0x6c, 0x14, 0xe6, 0x2e, 0x91, 0x0d, 0x40, 0x21, 0x35, 0xd5, 0x40, 0x13, 0x03, 0xe9, 0x80,
];

/// Infinite deobfuscation key iterator.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Key {
    index: usize,
}

impl Key {
    pub fn new() -> Self {
        Self { index: 0 }
    }

    pub fn next(&mut self) -> u8 {
        let v = OBFUSCATION[self.index];
        self.index += 1;
        if self.index >= OBFUSCATION.len() {
            self.index = 0;
        }
        v
    }

    pub fn apply(&mut self, val: u8) -> u8 {
        val ^ self.next()
    }

    pub fn advance(&self, num: usize) -> Self {
        let index = (self.index + num) % OBFUSCATION.len();
        Self { index }
    }
}
