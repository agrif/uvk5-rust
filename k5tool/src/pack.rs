const OBFUSCATION: [u8; 128] = [
    0x47, 0x22, 0xC0, 0x52, 0x5D, 0x57, 0x48, 0x94, 0xB1, 0x60, 0x60, 0xDB, 0x6F, 0xE3, 0x4C, 0x7C,
    0xD8, 0x4A, 0xD6, 0x8B, 0x30, 0xEC, 0x25, 0xE0, 0x4C, 0xD9, 0x00, 0x7F, 0xBF, 0xE3, 0x54, 0x05,
    0xE9, 0x3A, 0x97, 0x6B, 0xB0, 0x6E, 0x0C, 0xFB, 0xB1, 0x1A, 0xE2, 0xC9, 0xC1, 0x56, 0x47, 0xE9,
    0xBA, 0xF1, 0x42, 0xB6, 0x67, 0x5F, 0x0F, 0x96, 0xF7, 0xC9, 0x3C, 0x84, 0x1B, 0x26, 0xE1, 0x4E,
    0x3B, 0x6F, 0x66, 0xE6, 0xA0, 0x6A, 0xB0, 0xBF, 0xC6, 0xA5, 0x70, 0x3A, 0xBA, 0x18, 0x9E, 0x27,
    0x1A, 0x53, 0x5B, 0x71, 0xB1, 0x94, 0x1E, 0x18, 0xF2, 0xD6, 0x81, 0x02, 0x22, 0xFD, 0x5A, 0x28,
    0x91, 0xDB, 0xBA, 0x5D, 0x64, 0xC6, 0xFE, 0x86, 0x83, 0x9C, 0x50, 0x1C, 0x73, 0x03, 0x11, 0xD6,
    0xAF, 0x30, 0xF4, 0x2C, 0x77, 0xB2, 0x7D, 0xBB, 0x3F, 0x29, 0x28, 0x57, 0x22, 0xD6, 0x92, 0x8B,
];

const VERSION_LOC: usize = 0x2000;
pub const VERSION_LEN: usize = 16;

pub fn obfuscate_skip(data: &mut [u8], skip: usize) {
    let mut i = skip % OBFUSCATION.len();
    for x in data.iter_mut() {
        *x = *x ^ OBFUSCATION[i];
        i = (i + 1) % OBFUSCATION.len();
    }
}

pub fn obfuscate(data: &mut [u8]) {
    obfuscate_skip(data, 0);
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Version([u8; VERSION_LEN]);

impl Version {
    pub fn new_empty() -> Self {
        Self([0; VERSION_LEN])
    }

    pub fn new(data: [u8; VERSION_LEN]) -> Self {
        Self(data)
    }

    pub fn from_str(name: &str) -> anyhow::Result<Self> {
        Self::from_bytes(name.as_bytes())
    }

    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        if bytes.len() > VERSION_LEN {
            anyhow::bail!("version must be {:?} bytes or less", VERSION_LEN);
        }

        let mut data = [0; VERSION_LEN];
        for (i, b) in bytes.iter().enumerate() {
            data[i] = *b;
        }

        Ok(Self(data))
    }

    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.0).map(|s| s.trim_end_matches('\0'))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl std::fmt::Debug for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self.as_str() {
            Ok(s) => f.debug_tuple("Version").field(&s).finish(),
            Err(_) => f.debug_tuple("Version").field(&self.as_bytes()).finish(),
        }
    }
}

impl std::ops::Deref for Version {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

impl std::ops::DerefMut for Version {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_bytes()
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PackedFirmware {
    data: Vec<u8>,
}

impl PackedFirmware {
    pub fn new(data: Vec<u8>) -> anyhow::Result<Self> {
        if data.len() < VERSION_LOC + VERSION_LEN + 2 {
            anyhow::bail!(
                "packed firmware must be at least {:?} bytes",
                VERSION_LOC + VERSION_LEN + 2
            );
        }

        Ok(Self { data })
    }

    pub fn new_cloned(data: &[u8]) -> anyhow::Result<Self> {
        Self::new(data.to_owned())
    }

    pub fn check(&self) -> bool {
        // checksum is last two bytes, xmodem 16 bit, little-endian
        let checksum =
            self.data[self.data.len() - 2] as u16 | ((self.data[self.data.len() - 1] as u16) << 8);
        let body = &self.data[..self.data.len() - 2];
        let crc = crc::Crc::<u16>::new(&crc::CRC_16_XMODEM);

        checksum == crc.checksum(body)
    }

    pub fn unpack(&self) -> anyhow::Result<(UnpackedFirmware, Version)> {
        if !self.check() {
            anyhow::bail!("bad checksum on packed firmware");
        }

        Ok(self.unpack_unchecked())
    }

    pub fn unpack_unchecked(&self) -> (UnpackedFirmware, Version) {
        // obfuscate is xor -- deobfuscating is the same as obfuscating
        // last two bytes are crc, ignore those (unchecked)
        let mut deobfuscated = self.data.clone();
        deobfuscated.truncate(deobfuscated.len() - 2);
        obfuscate(&mut deobfuscated[..]);

        // splice out the 16 bytes of version at 0x2000
        let version = deobfuscated
            .splice(VERSION_LOC..VERSION_LOC + VERSION_LEN, std::iter::empty())
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap();
        (UnpackedFirmware::new(deobfuscated), Version(version))
    }
}

impl std::ops::Deref for PackedFirmware {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UnpackedFirmware {
    data: Vec<u8>,
}

impl UnpackedFirmware {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn new_cloned(data: &[u8]) -> Self {
        Self::new(data.to_owned())
    }

    pub fn pack(&self, version: Version) -> anyhow::Result<PackedFirmware> {
        let mut work = self.data.clone();
        if work.len() < VERSION_LOC {
            // pad with zeros until we reach version location
            work.resize(VERSION_LOC, 0);
        }

        // insert version
        work.splice(VERSION_LOC..VERSION_LOC, version.iter().copied());

        // obfuscate
        obfuscate(&mut work[..]);

        // shuffle on a checksum, little-endian
        let crc = crc::Crc::<u16>::new(&crc::CRC_16_XMODEM);
        let checksum = crc.checksum(&work[..]);
        work.extend(
            [(checksum & 0xff) as u8, ((checksum >> 8) & 0xff) as u8]
                .iter()
                .copied(),
        );

        PackedFirmware::new(work)
    }
}

impl std::ops::Deref for UnpackedFirmware {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
