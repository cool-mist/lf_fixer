use std::{
    fs::File,
    io::{Read, Write},
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
        let file = File::options().read(true).open(&self.file_name);
        if file.is_err() {
            println!("Could not open file: {}", self.file_name);
            return;
        }

        let mut file = file.unwrap();
        let mut read_buf = Vec::with_capacity(1024 * 1024);
        let mut write_buf = Vec::with_capacity(1024 * 1024);
        let bytes_read = file.read_to_end(&mut read_buf);

        if bytes_read.is_err() {
            println!("Could not read file: {}", self.file_name);
            return;
        }

        let bytes_read = bytes_read.unwrap();
        let mut read_head: usize = 0;
        let mut write_head: usize = 0;

        loop {
            if read_head == bytes_read {
                break;
            }

            let byte = read_buf[read_head];
            read_head = match byte {
                b'\r' => {
                    self.normalize_ending(&mut write_buf, &mut write_head);
                    let lf_index = read_head + 1;
                    // LL(1) to see if the next byte is '\n'
                    if lf_index < bytes_read && read_buf[lf_index] == b'\n' {
                        lf_index + 1
                    } else {
                        read_head + 1
                    }
                }
                b'\n' => {
                    self.normalize_ending(&mut write_buf, &mut write_head);
                    read_head + 1
                }
                _ => {
                    write_buf.push(byte);
                    write_head = write_head + 1;
                    read_head + 1
                }
            }
        }

        drop(file);

        let file = File::options()
            .write(true)
            .truncate(true)
            .open(&self.file_name);
        if file.is_err() {
            println!("Could not open file: {}", self.file_name);
            return;
        }
        let mut file = file.unwrap();
        let written = file.write_all(&write_buf[0..write_head]);
        if written.is_err() {
            println!(
                "Could not write to file: {} {:?}",
                self.file_name,
                written.err()
            );
            return;
        }

        println!("Fixed line endings in file: {}", self.file_name);
    }

    fn normalize_ending(&self, write_buf: &mut Vec<u8>, write_head: &mut usize) {
        for &b in &self.to {
            write_buf.push(b);
            *write_head += 1;
        }
    }
}
