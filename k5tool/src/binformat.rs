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

pub fn read_firmware<P>(
    path: P,
    format: BinaryFormat,
    version: Option<Version>,
) -> anyhow::Result<(UnpackedFirmware, Option<Version>)>
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
) -> anyhow::Result<(UnpackedFirmware, Option<Version>)> {
    match format {
        BinaryFormat::Raw => Ok((UnpackedFirmware::new_cloned(data), version)),
        BinaryFormat::Packed => {
            let packed = PackedFirmware::new_cloned(data)?;
            let (unpacked, fileversion) = packed.unpack()?;
            Ok((unpacked, version.or(Some(fileversion))))
        }
        BinaryFormat::Elf => {
            let elf = ElfBytes::minimal_parse(data)
                .map_err(|_| anyhow::anyhow!("could not open ELF image"))?;
            let (unpacked, fileversion) = flatten_elf(elf)?;
            Ok((unpacked, version.or(fileversion)))
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

pub fn flatten_elf(
    elf: ElfBytes<AnyEndian>,
) -> anyhow::Result<(UnpackedFirmware, Option<Version>)> {
    // search for a VERSION symbol to use as the version
    let version_offset = (|| {
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

                return Some((sym.st_value as usize, sym.st_size as usize));
            }
        }

        None
    })();

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
    let version = version_offset.and_then(|(start, len)| {
        let end = start + len;
        if end <= flat.len() {
            Version::from_bytes(&flat[start..end]).ok().and_then(|v| {
                // sanity check -- is the version utf-8 and at least 1 char
                if let Ok(s) = v.as_str() {
                    if s.len() > 0 {
                        return Some(v);
                    }
                }
                None
            })
        } else {
            None
        }
    });

    Ok((UnpackedFirmware::new(flat), version))
}
