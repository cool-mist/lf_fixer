use std::fmt::Display;
use std::fs::File;
use std::io::Read;

pub(crate) struct FileStats {
    name: String,
    lines: usize,
    crlf: usize,
    cr: usize,
    lf: usize,
    max: Option<LineSep>,
}

pub(crate) struct FileStatsAggregate {
    stats: Vec<FileStats>,
    crlf: usize,
    cr: usize,
    lf: usize,
    max: Option<LineSep>,
    lines: usize,
}

#[derive(Clone)]
pub(crate) enum LineSep {
    Lf,
    CrLf,
    Cr,
}

impl FileStats {
    fn new(name: &str) -> FileStats {
        FileStats {
            name: name.to_string(),
            lines: 0,
            crlf: 0,
            cr: 0,
            lf: 0,
            max: None,
        }
    }

    pub(crate) fn generate(file_name: String) -> Option<FileStats> {
        let file = File::open(&file_name);
        if file.is_err() {
            println!("Could not open file: {}", file_name);
            return None;
        }
        let mut file = file.unwrap();
        let mut buf = Vec::with_capacity(1024 * 1024);

        let bytes_read = file.read_to_end(&mut buf);
        if bytes_read.is_err() {
            println!("Could not read file: {}", file_name);
            return None;
        }

        let bytes_read = bytes_read.unwrap();

        let mut stats = FileStats::new(&file_name);
        let mut i = 0;
        loop {
            if i == bytes_read {
                break Some(stats);
            }

            let byte = buf[i];
            if byte == b'\r' {
                if i + 1 < bytes_read && buf[i + 1] == b'\n' {
                    stats.update(LineSep::CrLf);
                    i = i + 1;
                } else {
                    stats.update(LineSep::Cr);
                }
            } else if byte == b'\n' {
                stats.update(LineSep::Lf);
            }

            i = i + 1;
        }
    }

    fn update(&mut self, line: LineSep) {
        self.lines += 1;
        match line {
            LineSep::CrLf => {
                self.crlf += 1;
            }
            LineSep::Cr => {
                self.cr += 1;
            }
            LineSep::Lf => {
                self.lf += 1;
            }
        }

        if (self.crlf > self.lf) && (self.crlf > self.cr) {
            self.max = Some(LineSep::CrLf);
        } else if (self.lf > self.cr) && (self.lf > self.crlf) {
            self.max = Some(LineSep::Lf);
        } else if (self.cr > self.lf) && (self.cr > self.crlf) {
            self.max = Some(LineSep::Cr);
        } else {
            self.max = None;
        }
    }
}

impl FileStatsAggregate {
    pub(crate) fn new() -> Self {
        FileStatsAggregate {
            stats: Vec::new(),
            crlf: 0,
            cr: 0,
            lf: 0,
            max: None,
            lines: 0,
        }
    }

    pub(crate) fn fold(mut accumulator: FileStatsAggregate, stat: FileStats) -> FileStatsAggregate {
        accumulator.crlf += stat.crlf;
        accumulator.cr += stat.cr;
        accumulator.lf += stat.lf;
        accumulator.lines += stat.lines;

        if (accumulator.crlf > accumulator.lf) && (accumulator.crlf > accumulator.cr) {
            accumulator.max = Some(LineSep::CrLf);
        } else if (accumulator.lf > accumulator.cr) && (accumulator.lf > accumulator.crlf) {
            accumulator.max = Some(LineSep::Lf);
        } else if (accumulator.cr > accumulator.crlf) && (accumulator.cr > accumulator.lf) {
            accumulator.max = Some(LineSep::Cr);
        } else {
            accumulator.max = None;
        }

        accumulator.stats.push(stat);

        accumulator
    }

    pub(crate) fn max(&self) -> Option<LineSep> {
        self.max.clone()
    }

    pub(crate) fn print_table(&self) {
        println!(
            "{:<4} | {:<4} | {:<4} | {:<4} | {:<4} | {}",
            "#", "CRLF", "CR", "LF", "Max", "File"
        );

        println!("=============================================");

        for stat in &self.stats {
            let max = match &stat.max {
                Some(max) => max.to_string(),
                None => "".to_string(),
            };

            println!(
                "{:<4} | {:<4} | {:<4} | {:<4} | {:<4} | {}",
                stat.lines, stat.crlf, stat.cr, stat.lf, max, stat.name
            );
        }

        println!("---------------------------------------------");
        let max = match &self.max {
            Some(max) => max.to_string(),
            None => "".to_string(),
        };
        println!(
            "{:<4} | {:<4} | {:<4} | {:<4} | {:<4} | -",
            self.lines, self.crlf, self.cr, self.lf, max,
        );
    }
}

impl Display for LineSep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LineSep::Lf => write!(f, "LF"),
            LineSep::CrLf => write!(f, "CRLF"),
            LineSep::Cr => write!(f, "CR"),
        }
    }
}
