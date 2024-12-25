use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
};

use crate::stats::LineSep;

pub(crate) struct Fixer {
    file_name: String,
    to: Vec<u8>,
}

impl Fixer {
    pub(crate) fn new(file_name: String, to: LineSep) -> Fixer {
        let to = match to {
            LineSep::Lf => vec![b'\n'],
            LineSep::CrLf => vec![b'\r', b'\n'],
            LineSep::Cr => vec![b'\r'],
        };

        Fixer { file_name, to }
    }

    pub(crate) fn fix(&mut self) {
        let file = File::options().read(true).write(true).open(&self.file_name);
        if file.is_err() {
            println!("Could not open file: {}", self.file_name);
            return;
        }

        let mut file = file.unwrap();
        let mut buf = Vec::new();
        let mut write_buf = Vec::new();
        let bytes_read = file.read_to_end(&mut buf);

        if bytes_read.is_err() {
            println!("Could not read file: {}", self.file_name);
            return;
        }

        let bytes_read = bytes_read.unwrap();
        let mut next_read_head = 0;

        loop {
            if next_read_head == bytes_read {
                break;
            }

            next_read_head = match buf[next_read_head] {
                b'\r' => {
                    // LL(1) to see if the next byte is '\n'
                    if next_read_head < bytes_read && buf[next_read_head] == b'\n' {
                        write_buf.extend(&self.to);
                        next_read_head + 2
                    } else {
                        next_read_head + 1
                    }
                }
                b'\n' => {
                    write_buf.extend(&self.to);
                    next_read_head + 1
                }
                any_other_byte => {
                    write_buf.push(any_other_byte);
                    next_read_head + 1
                }
            }
        }

        let seeked = file.seek(SeekFrom::Start(0));
        if seeked.is_err() {
            println!("Could not seek to start of file: {}", self.file_name);
            return;
        }

        let written = file.write_all(&write_buf);
        if written.is_err() {
            println!("Could not write to file: {}", self.file_name);
            println!("Error: {:?}", written.err());
        }
    }
}
