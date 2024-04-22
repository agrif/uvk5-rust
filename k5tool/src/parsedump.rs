#[derive(clap::Args, Debug)]
pub struct ParseDumpOpts {
    dump: String,
}

impl crate::ToolRun for ParseDumpOpts {
    fn run(&self) -> anyhow::Result<()> {
        let rawdata = std::fs::read(&self.dump)?;
        let mut raw = &rawdata[..];

        let xmodem = k5lib::protocol::crc::CrcXModem::new();
        let dummy = k5lib::protocol::crc::CrcConstant(0xffff);

        loop {
            if raw.len() < 3 {
                break;
            }
            let dir = raw[0];
            let len = ((raw[1] as u16) | ((raw[2] as u16) << 8)) as usize;
            let frameraw = &raw[3..3 + len];
            raw = &raw[3 + len..];

            let crc = if dir == 0 {
                println!("radio -> computer, {} bytes", len);
                k5lib::protocol::crc::CrcEither::Left(&dummy)
            } else {
                println!("computer -> radio, {} bytes", len);
                k5lib::protocol::crc::CrcEither::Right(&xmodem)
            };

            println!();

            match parse_frame(crc, frameraw) {
                Ok(o) => {
                    println!("{:?}", o);
                    println!();
                }
                Err(e) => {
                    println!("Unparsed frame:");
                    hexdump::hexdump(frameraw);
                    println!();
                    anyhow::bail!(e);
                }
            }
        }
        Ok(())
    }
}

fn parse_frame<C>(crc: C, data: &[u8]) -> anyhow::Result<k5lib::protocol::Message>
where
    C: k5lib::protocol::crc::CrcStyle,
{
    let (rest, frame) = k5lib::protocol::parse::framed(crc, nom::combinator::rest)(data);
    anyhow::ensure!(rest.len() == 0, "Frame parser left leftover data.");

    use k5lib::protocol::ParseResult;
    match frame {
        ParseResult::Ok(framebody) => match parse_message(framebody.clone()) {
            Ok(o) => Ok(o),
            Err(e) => {
                println!("Deobfuscated frame:");
                hexdump::hexdump(framebody.to_vec().as_ref());
                println!();
                Err(e)
            }
        },
        ParseResult::ParseErr(_f, e) => anyhow::bail!("Frame parse error: {:?}", e.code),
        ParseResult::CrcErr(f) => {
            println!("Deobfuscated frame + CRC:");
            hexdump::hexdump(f.to_vec().as_ref());
            println!();
            anyhow::bail!("CRC error.");
        }
        ParseResult::None => anyhow::bail!("Frame parser found no frames."),
    }
}

fn parse_message(
    data: k5lib::protocol::obfuscation::Deobfuscated<&[u8]>,
) -> anyhow::Result<k5lib::protocol::Message> {
    let (rest, (typ, body)) =
        k5lib::protocol::parse::message(|t| nom::combinator::map(nom::combinator::rest, move |r| (t, r)))(
            data,
        )
        .map_err(|_| anyhow::anyhow!("Message parser falied."))?;
    anyhow::ensure!(rest.len() == 0, "Message parser left leftover data.");

    println!("Message type: {:x?}", typ);

    match parse_message_body(typ, body.clone()) {
        Ok(o) => Ok(o),
        Err(e) => {
            println!("Unparsed message body:");
            hexdump::hexdump(body.to_vec().as_ref());
            println!();
            anyhow::bail!(e);
        }
    }
}

fn parse_message_body(
    typ: u16,
    body: k5lib::protocol::obfuscation::Deobfuscated<&[u8]>,
) -> anyhow::Result<k5lib::protocol::Message> {
    use k5lib::protocol::MessageParse;
    use nom::Parser;

    let (rest, msg) = k5lib::protocol::Message::parse_body(typ)
        .parse(body)
        .map_err(|_| anyhow::anyhow!("Message body parser falied."))?;
    anyhow::ensure!(rest.len() == 0, "Message body parser left leftover data.");
    Ok(msg)
}
