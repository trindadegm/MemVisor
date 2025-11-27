use crate::dap::DapError;
use crate::dap::message::ProtocolMessage;
use std::io::{BufRead, Write};
use std::sync::mpsc::SyncSender;

pub struct DapMessenger<TWriter> {
    writer: TWriter,
    trace_enabled: bool,
}
impl<TWriter> DapMessenger<TWriter>
where
    TWriter: Write,
{
    pub fn new<TReader>(reader: TReader, writer: TWriter, tx: SyncSender<ProtocolMessage>) -> Self
    where
        TReader: BufRead + Send + 'static,
    {
        let trace_enabled = std::env::var("MEMVISOR_TRACE_DAP")
            .map(|v| v != "0")
            .unwrap_or(false);

        let _worker = std::thread::spawn(move || {
            let mut read_buf = String::new();
            let mut json_scratchpad = Vec::new();
            let mut reader = reader;

            loop {
                let res =
                    Self::worker_receive_message(&mut reader, &mut read_buf, &mut json_scratchpad, trace_enabled);
                if let Ok(msg) = res {
                    if let Err(e) = tx.send(msg) {
                        log::error!("Channel broken: {e}");
                        break; // Out of the loop
                    }
                } else {
                    let msg_err = res.unwrap_err();
                    log::error!("Receive message error: {msg_err}");
                    break; // Out of the loop
                }
            }

            log::info!("DAP messenger quitting");
        });

        if trace_enabled {
            log::info!("MEMVISOR_TRACE_DAP enabled");
        }
        Self {
            writer,
            trace_enabled,
        }
    }

    fn worker_receive_message<R: BufRead>(
        reader: &mut R,
        read_buf: &mut String,
        json_scratchpad: &mut Vec<u8>,
        trace_enabled: bool,
    ) -> Result<ProtocolMessage, DapError> {
        read_buf.clear();
        reader.read_line(read_buf)?;

        let content_length;
        if let Some((header, value)) = read_buf.split_once(" ") {
            match header.trim() {
                "Content-Length:" => {
                    let value = value.trim();
                    let value_num: usize = value
                        .parse()
                        .map_err(|_| DapError::InvalidContentLength(value.into()))?;
                    content_length = value_num;
                }
                _ => {
                    return Err(DapError::BadMessageHeader(read_buf.clone()));
                }
            }
        } else {
            return Err(DapError::BadMessageHeader(read_buf.clone()));
        }

        // Discard next line
        reader.read_line(read_buf)?;

        json_scratchpad.resize(content_length, 0);
        reader.read_exact(&mut json_scratchpad[..])?;

        let json_str = std::str::from_utf8(&json_scratchpad[..])?;
        if trace_enabled {
            println!("RECEIVED: {json_str}");
        }

        let message = serde_json::from_str(json_str)?;

        Ok(message)
    }

    pub fn send_message(&mut self, msg: &str) -> Result<(), DapError> {
        let encoded = format!(
            "Content-Length: {msg_length}\r\n\r\n{msg}",
            msg_length = msg.len()
        );

        if self.trace_enabled {
            println!("SENDING: {msg}");
        }

        self.writer.write_all(encoded.as_bytes())?;
        self.writer.flush()?;

        Ok(())
    }
}
