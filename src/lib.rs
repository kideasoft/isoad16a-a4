//! A platform agnostic driver to interface with the 16 channel 4-20mA AD module

use serialport::SerialPort;
use std::str;

#[derive(Debug)]
pub enum Error {
    SerialError(serialport::Error),
    IoError(std::io::Error),
    DecodeError,
    ResponseError,
    StatusError,
    ChanError,
}

pub struct Ad {
    addr: u8,
    port: Box<dyn SerialPort>,
}

impl Ad {
    pub fn new(port_path: &str, addr: u8, baud_rate: u32) -> Result<Self, serialport::Error> {
        let mut port = serialport::open(port_path)?;
        port.set_baud_rate(baud_rate)?;
        Ok(Ad {
            // port_path,
            addr,
            // baud_rate,
            port,
        })
    }

    fn decode(&self, s: &str) -> Result<[f32; 16], Error> {
        if s.len() != 114 {
            return Err(Error::DecodeError);
        }

        match s.chars().nth(0) {
            Some(c) => {
                if c != '>' {
                    return Err(Error::DecodeError);
                }
            }
            None => return Err(Error::DecodeError),
        }

        let mut r = [0.0; 16];
        for i in 0..16 {
            let v = s[1 + i * 7..1 + (i + 1) * 7]
                .parse::<f32>()
                .map_err(|_| Error::DecodeError)?;
            r[i] = v;
        }

        Ok(r)
    }

    fn decode_chan(&self, s: &str) -> Result<f32, Error> {
        if s.len() != 9 {
            return Err(Error::DecodeError);
        }

        match s.chars().nth(0) {
            Some(c) => {
                if c != '>' {
                    return Err(Error::DecodeError);
                }
            }
            None => return Err(Error::DecodeError),
        }

        let v = s[1..8].parse::<f32>().map_err(|_| Error::DecodeError)?;

        Ok(v)
    }

    pub fn get_all(&mut self) -> Result<[f32; 16], Error> {
        let addr = self.addr.to_string();

        let cmd = match addr.len() {
            1 => ["#0", &addr, "\r"].concat(),
            2 => ["#", &addr, "\r"].concat(),
            _ => return Err(Error::DecodeError),
        };

        self.port
            .write(cmd.as_bytes())
            .map_err(|e| Error::IoError(e))?;

        std::thread::sleep(std::time::Duration::from_millis(2000));

        let mut buf = [0_u8; 1024];
        let n = self.port.read(&mut buf).map_err(|e| Error::IoError(e))?;

        // DEBUG:
        // println!(
        //     "{} bytes, receive: {}",
        //     n,
        //     str::from_utf8(&buf[..n]).unwrap()
        // );

        self.decode(str::from_utf8(&buf[..n]).map_err(|_| Error::DecodeError)?)
    }

    pub fn get_chan(&mut self, chan: u8) -> Result<f32, Error> {
        if chan > 15 {
            return Err(Error::ChanError);
        }
        let chan = chan.to_string();
        let chan = match chan.len() {
            1 => ["0", &chan].concat(),
            2 => chan,
            _ => return Err(Error::ChanError),
        };

        let addr = self.addr.to_string();

        let cmd = match addr.len() {
            1 => ["#0", &addr, &chan, "\r"].concat(),
            2 => ["#", &addr, &chan, "\r"].concat(),
            _ => return Err(Error::DecodeError),
        };

        self.port
            .write(cmd.as_bytes())
            .map_err(|e| Error::IoError(e))?;

        std::thread::sleep(std::time::Duration::from_millis(100));

        let mut buf = [0_u8; 1024];
        let n = self.port.read(&mut buf).map_err(|e| Error::IoError(e))?;

        // DEBUG:
        // println!(
        //     "{} bytes, receive: {}",
        //     n,
        //     str::from_utf8(&buf[..n]).unwrap()
        // );

        let reply = str::from_utf8(&buf[..n]).map_err(|_| Error::DecodeError)?;

        self.decode_chan(reply)
    }
}
