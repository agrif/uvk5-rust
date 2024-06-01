use std::io::{Read, Write};

use crate::common::{FLASH_MAX, RAM_MAX, RAM_START};

use k5lib::protocol::messages::bootloader::{
    BootloaderReady, BootloaderReadyReply, WriteFlash, WriteFlashReply, WRITE_FLASH_LEN,
    WRITE_FLASH_SESSION_ID,
};
use k5lib::Version;

// what page does the bootloader start on?
const BOOTLOADER_START_PAGE: usize = 0xf0;

#[derive(clap::Args, Debug)]
pub struct FlashOpts {
    firmware: String,
    #[arg(long)]
    version: Option<String>,
    #[arg(long, value_enum, default_value = "auto")]
    format: crate::binformat::BinaryFormat,

    #[command(flatten)]
    port: crate::common::SerialPortArgs,
    #[command(flatten)]
    debug: crate::debug::DebugClientArgs,

    #[arg(short, long)]
    yes: bool,
    /// Override built-in sanity checks to ignore errors.
    ///
    /// !!! Very Dangerous !!
    #[arg(long)]
    ignore: Vec<Ignores>,
}

/// Possible sanity checks the user can chose to ignore.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, clap::ValueEnum)]
enum Ignores {
    /// Ignore the stack pointer check.
    Stack,
    /// Ignore the entry point check.
    Entry,
    /// Ignore the flash size.
    Size,
}

impl crate::ToolRun for FlashOpts {
    fn run(&self) -> anyhow::Result<()> {
        let version = if let Some(ref v) = self.version {
            Some(Version::new_from_str(v)?)
        } else {
            None
        };

        let (unpacked, info) =
            crate::binformat::read_firmware(&self.firmware, self.format, version)?;

        let version = info.version.clone().ok_or(anyhow::anyhow!(
            "image has no version, use --version to provide one"
        ))?;

        let mut flasher = Flasher::new(self, self.port.open()?, &unpacked, version, info)?;
        flasher.check()?;
        flasher.flash()?;
        Ok(())
    }
}

struct Flasher<'a, F> {
    opts: &'a FlashOpts,
    client: crate::debug::DebugClientHost<F>,
    data: &'a [u8],
    version: Version,
    info: crate::binformat::BinaryInfo,
    session_id: u32,
}

impl<'a, F> Flasher<'a, F>
where
    F: Read + Write,
{
    fn new(
        opts: &'a FlashOpts,
        port: F,
        data: &'a [u8],
        version: Version,
        info: crate::binformat::BinaryInfo,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            opts,
            client: opts.debug.wrap_host(k5lib::ClientHost::new(port))?,
            data,
            version,
            info,
            session_id: WRITE_FLASH_SESSION_ID,
        })
    }

    fn check(&mut self) -> anyhow::Result<()> {
        if !self.opts.ignore.is_empty() {
            eprintln!(
                "You have chosen to ignore these warnings: {:#?}",
                self.opts.ignore
            );
            eprintln!("This can potentially be very dangerous and brick your radio!");
            crate::common::confirm("Are you sure you want to continue?", self.opts.yes)?;
            eprintln!();
        }

        self.info.report();

        if !self.opts.ignore.contains(&Ignores::Stack)
            && (self.info.stack_top <= RAM_START || self.info.stack_top > RAM_START + RAM_MAX)
        {
            anyhow::bail!("Stack pointer is not inside RAM.");
        }

        if !self.opts.ignore.contains(&Ignores::Entry) && self.info.entry_point >= FLASH_MAX {
            anyhow::bail!("Entry point is not inside flash.");
        }

        if !self.opts.ignore.contains(&Ignores::Size) && self.data.len() > FLASH_MAX {
            anyhow::bail!("Image is larger than available flash.");
        }

        Ok(())
    }

    fn flash(&mut self) -> anyhow::Result<()> {
        // figure out how many pages we have
        let page_size = WRITE_FLASH_LEN;
        let max_page = (self.data.len() + page_size - 1) / page_size;

        // make sure that math worked
        // we want the minimum number of pages to include all data
        assert!(max_page * page_size >= self.data.len());
        assert!(max_page * page_size - self.data.len() < page_size);

        // last sanity check
        if !self.opts.ignore.contains(&Ignores::Size) {
            // bootloader starts
            assert!(max_page <= BOOTLOADER_START_PAGE);
        }

        // wait for a bootloader ready message
        let m = loop {
            if let Some(m) = self.client.read::<BootloaderReady>()?.ok() {
                break m;
            }
        };

        // make sure the user thinks any info printed is ok
        eprintln!();
        crate::common::confirm("Continue flashing?", self.opts.yes)?;

        // report info
        eprint!("Connected to bootloader, version: ");
        if let Ok(v) = m.version.as_str() {
            eprintln!("{}", v);
        } else {
            eprintln!("{:x?}", m.version.as_bytes());
        }

        // ok, let's do this
        self.client.write(&BootloaderReadyReply {
            version: self.version.clone(),
        })?;

        let bar = crate::common::upload_bar((max_page * WRITE_FLASH_LEN) as u64);

        // we need a buffer in case one page is partially full, to add 0's
        let mut chunk = [0; WRITE_FLASH_LEN];

        for page in 0..max_page {
            // paranoia
            if !self.opts.ignore.contains(&Ignores::Size) {
                assert!(page < BOOTLOADER_START_PAGE);
            }

            // figure out the extents of this page
            let start = page * WRITE_FLASH_LEN;
            let end = self.data.len().min(start + WRITE_FLASH_LEN);
            let chunkend = end - start;

            // copy this page into chunk, zero what's left over
            chunk[..chunkend].copy_from_slice(&self.data[start..end]);
            chunk[chunkend..].fill(0);

            // write this chunk
            self.client.write(&WriteFlash {
                session_id: self.session_id,
                page: page as u16,
                max_page: max_page as u16,
                len: chunkend as u16,
                _pad: Default::default(),
                data: &chunk[..],
            })?;

            // wait for a confirmation
            let m = loop {
                if let Some(m) = self.client.read::<WriteFlashReply>()?.ok() {
                    break m;
                }
            };

            if m.error > 0 {
                anyhow::bail!("Bootloader reported error.");
            }

            if m.session_id != self.session_id || m.page != page as u16 {
                anyhow::bail!("Bootloader did not confirm write.");
            }

            bar.set_position(end as u64);
        }

        bar.finish();

        Ok(())
    }
}
