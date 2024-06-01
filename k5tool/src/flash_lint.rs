use crate::common::{FLASH_MAX, RAM_MAX, RAM_START};

/// Possible sanity checks the user can chose to ignore.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, clap::ValueEnum)]
pub enum Ignores {
    /// Ignore the stack pointer check.
    Stack,
    /// Ignore the entry point check.
    Entry,
    /// Ignore the flash size.
    Size,
}

pub fn check(
    data: &[u8],
    info: &crate::binformat::BinaryInfo,
    ignores: &[Ignores],
) -> anyhow::Result<()> {
    info.report();

    let mut fail = false;
    if !ignores.contains(&Ignores::Stack)
        && (info.stack_top <= RAM_START || info.stack_top > RAM_START + RAM_MAX)
    {
        eprintln!("FAIL: Stack pointer is not inside RAM.");
        fail = true;
    }

    if !ignores.contains(&Ignores::Entry) && info.entry_point >= FLASH_MAX {
        eprintln!("FAIL: Entry point is not inside flash.");
        fail = true;
    }

    if !ignores.contains(&Ignores::Size) && data.len() > FLASH_MAX {
        eprintln!("FAIL: Image is larger than available flash.");
        fail = true;
    }

    if fail {
        anyhow::bail!("Firmware failed at least one lint.")
    }

    Ok(())
}
