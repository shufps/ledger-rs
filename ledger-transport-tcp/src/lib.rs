mod errors;

use ledger_apdu::{APDUAnswer, APDUCommand};

use std::io::{Read, Write};
use std::net::TcpStream;

pub use errors::LedgerTCPError;

pub struct TransportTCP {
    url: String,
}

impl TransportTCP {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            url: format!("{}:{}", host, port),
        }
    }

    pub async fn exchange(&self, command: &APDUCommand) -> Result<APDUAnswer, LedgerTCPError> {
        let raw_command = command.serialize();

        let mut stream = TcpStream::connect(&self.url).map_err(|_| LedgerTCPError::ConnectError)?;

        log::debug!("successfully connected to server {}", &self.url);

        fn wrap_err(
            raw_command: &Vec<u8>,
            stream: &mut TcpStream,
        ) -> Result<APDUAnswer, std::io::Error> {
            // store length as 32bit big endian into array
            let send_length_bytes = (raw_command.len() as u32).to_be_bytes();

            // first send number of bytes
            stream.write(&send_length_bytes[..])?;

            // then send bytes
            stream.write(&raw_command[..])?;

            let mut rcv_length_bytes = [0u8; 4];

            // first read number of bytes
            stream.read_exact(&mut rcv_length_bytes)?;

            // convert bytes to big endian (+2 for return code)
            let rcv_length = u32::from_be_bytes(rcv_length_bytes) + 2;

            let mut buf = vec![0u8; rcv_length as usize];

            stream.read_exact(&mut buf)?;
            Ok(APDUAnswer::from_answer(buf))
        }

        wrap_err(&raw_command, &mut stream).map_err(|_| LedgerTCPError::InnerError)
    }
}
