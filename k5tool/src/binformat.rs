use elf::endian::AnyEndian;
use elf::ElfBytes;

use k5lib::pack::{PackedFirmware, UnpackedFirmware};
use k5lib::Version;

/// Binary formats accepted by this tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, clap::ValueEnum)]
pub enum BinaryFormat {
    /// A flat binary.
    Raw,
    /// An packed, obfuscated binary, with a version number and CRC.
    Packed,
    /// An ELF binary.
    Elf,

    /// Automatically decide between Elf and Packed.
    ///
    /// We will never automatically decide raw, because raw images
    /// can't be positively identified.
    Auto,
}

/// Miscellaneous info about a firmware image.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BinaryInfo {
    pub version: Option<Version>,
    pub flash_len: usize,
    pub stack_top: usize,
    pub stack_bottom: Option<usize>,
    pub entry_point: usize,
    pub ram_len: Option<usize>,
}

impl BinaryInfo {
    fn from_image(data: &[u8], version: Option<Version>) -> anyhow::Result<Self> {
        let flash_len = data.len();
        // initial stack pointer is the first u32 in cortex-m0
        let stack_top = crate::common::read_le_u32(&data[0..])
            .ok_or(anyhow::anyhow!("could not read initial stack pointer"))?;
        // next is entry point
        let entry_point = crate::common::read_le_u32(&data[4..])
            .ok_or(anyhow::anyhow!("could not read entry_point"))?;

        Ok(Self {
            version,
            flash_len,
            stack_top: stack_top as usize,
            stack_bottom: None,
            entry_point: entry_point as usize,
            ram_len: None,
        })
    }

    fn override_version(&mut self, version: Option<Version>) {
        if version.is_some() {
            self.version = version;
        }
    }

    pub fn report(&self) {
        const NAMEWIDTH: usize = 8;
        const AMTWIDTH: usize = 9;

        if let Some(ref version) = self.version {
            if let Ok(v) = version.as_str() {
                println!("{:>width$}: {}", "Version", v, width = NAMEWIDTH,);
            } else {
                println!(
                    "{:>width$}: {:x?}",
                    "Version",
                    version.as_bytes(),
                    width = NAMEWIDTH
                )
            }
        }

        fn fmtbytes(amt: usize, width: usize) -> String {
            // alas, these don't obey format alignment
            let mut amtf = format!("{}", indicatif::BinaryBytes(amt as u64));
            if amtf.len() < width {
                amtf = " ".repeat(width - amtf.len()) + &amtf;
            }

            amtf
        }

        fn bar(name: &str, amt: usize, max: usize) {
            println!(
                "{:>width$}: {} / {} {} {:>3.0}%",
                name,
                fmtbytes(amt, AMTWIDTH),
                fmtbytes(max, AMTWIDTH),
                crate::common::size_bar(amt, max),
                100.0 * (amt as f32) / (max as f32),
                width = NAMEWIDTH,
            );
        }

        bar("Flash", self.flash_len, crate::common::FLASH_MAX);
        if let Some(ram_len) = self.ram_len {
            bar("RAM", ram_len, crate::common::RAM_MAX);
        }

        if let Some(stack_bottom) = self.stack_bottom {
            let stack_size = self.stack_top - stack_bottom;
            println!(
                "{:>width$}: {}",
                "Stack",
                fmtbytes(stack_size, AMTWIDTH),
                width = NAMEWIDTH
            );
        }
    }
}

pub fn read_firmware<P>(
    path: P,
    format: BinaryFormat,
    version: Option<Version>,
) -> anyhow::Result<(UnpackedFirmware, BinaryInfo)>
where
    P: AsRef<std::path::Path>,
{
    let data = std::fs::read(path)?;
    read_firmware_from(&data, format, version)
}

pub fn read_firmware_from(
    data: &[u8],
    format: BinaryFormat,
    version: Option<Version>,
) -> anyhow::Result<(UnpackedFirmware, BinaryInfo)> {
    match format {
        BinaryFormat::Raw => Ok((
            UnpackedFirmware::new_cloned(data),
            BinaryInfo::from_image(data, None)?,
        )),
        BinaryFormat::Packed => {
            let packed = PackedFirmware::new_cloned(data)?;
            let (unpacked, fileversion) = packed.unpack()?;
            let mut info = BinaryInfo::from_image(&unpacked, Some(fileversion))?;
            info.override_version(version);
            Ok((unpacked, info))
        }
        BinaryFormat::Elf => {
            let elf = ElfBytes::minimal_parse(data)
                .map_err(|_| anyhow::anyhow!("could not open ELF image"))?;
            let (unpacked, mut info) = flatten_elf(elf)?;
            info.override_version(version);
            Ok((unpacked, info))
        }
        BinaryFormat::Auto => {
            for tryfmt in &[BinaryFormat::Elf, BinaryFormat::Packed] {
                if let Ok(r) = read_firmware_from(data, *tryfmt, version.clone()) {
                    return Ok(r);
                }
            }
            anyhow::bail!("could not detect binary image format");
        }
    }
}

pub fn flatten_elf(elf: ElfBytes<AnyEndian>) -> anyhow::Result<(UnpackedFirmware, BinaryInfo)> {
    // search for a VERSION symbol, which stores a pointer to the version
    let version_ptr_offset = (|| {
        let (symbols, symstr) = elf.symbol_table().ok()??;
        let (sections, secstr) = elf.section_headers_with_strtab().ok()?;
        let sections = sections?;
        let secstr = secstr?;

        for sym in symbols {
            let name = symstr.get(sym.st_name as usize).ok();
            let section = sections
                .get(sym.st_shndx as usize)
                .ok()
                .and_then(|sec| secstr.get(sec.sh_name as usize).ok());

            if let (Some(name), Some(section)) = (name, section) {
                if name != "VERSION" {
                    continue;
                }

                // sanity check, in case something else makes a VERSION sym
                if section != ".text" && section != ".rodata" {
                    continue;
                }

                return Some(sym.st_value as usize);
            }
        }

        None
    })();

    // flatten the binary
    let mut flat = Vec::new();
    if let Some(segments) = elf.segments() {
        for segment in segments {
            if segment.p_type != elf::abi::PT_LOAD {
                continue;
            }
            if segment.p_filesz == 0 {
                continue;
            }

            // sanity check
            let big_size = 0x10000;
            if segment.p_filesz >= big_size
                || segment.p_memsz >= big_size
                || segment.p_paddr >= big_size
            {
                anyhow::bail!("ELF file segments too large");
            }

            let data = elf.segment_data(&segment)?;
            let start = segment.p_paddr as usize;
            let end = start + segment.p_memsz as usize;

            if end > flat.len() {
                flat.resize(end, 0)
            }

            // be careful, if data.len() != memsz
            let len = data.len().min(segment.p_memsz as usize);
            flat[start..start + len].copy_from_slice(&data[..len]);
        }
    }

    // ok, now extract the version if we can
    let version = version_ptr_offset.and_then(|start| {
        let start = crate::common::read_le_u32(&flat[start..])? as usize;
        let end = flat.len().min(start + k5lib::VERSION_LEN);

        let version = std::ffi::CStr::from_bytes_until_nul(&flat[start..end]).ok()?;

        Version::from_c_str(version).ok().and_then(|v| {
            // sanity check -- is the version utf-8 and at least 1 char
            if let Ok(s) = v.as_str() {
                if s.len() > 0 {
                    return Some(v);
                }
            }
            None
        })
    });

    let mut info = BinaryInfo::from_image(&flat, version)?;

    // count up ram used and figure out where the stack ends
    let mut ram_len = 0;
    let mut stack_bottom = crate::common::RAM_START;
    if let Some(segments) = elf.segments() {
        for segment in segments {
            if segment.p_type != elf::abi::PT_LOAD {
                continue;
            }

            if segment.p_vaddr < crate::common::RAM_START as u64 {
                // not ram
                continue;
            }

            ram_len += segment.p_memsz as usize;

            let end = (segment.p_vaddr + segment.p_memsz) as usize;
            if end <= info.stack_top && end > stack_bottom {
                stack_bottom = end;
            }
        }
    }

    info.ram_len = Some(ram_len);
    info.stack_bottom = Some(stack_bottom);

    Ok((UnpackedFirmware::new(flat), info))
}
