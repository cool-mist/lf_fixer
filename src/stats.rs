use std::fmt::Display;
use std::fs::{self, File, ReadDir};
use std::io::Read;

use crate::filters::FileFilter;

pub(crate) struct FileStatsAggregate {
    stats: Vec<FileStats>,
    crlf: usize,
    cr: usize,
    lf: usize,
    max: Option<LineSep>,
    lines: usize,
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

pub(crate) struct FileStats {
    name: String,
    lines: usize,
    crlf: usize,
    cr: usize,
    lf: usize,
    max: Option<LineSep>,
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

        let mut buf = Vec::new();

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
        } else {
            self.max = Some(LineSep::Cr);
        }
    }
}

#[derive(Clone)]
pub(crate) enum LineSep {
    Lf,
    CrLf,
    Cr,
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

pub(crate) struct FileNames {
    cur_dir: Option<String>,
    filter: FileFilter,
    all_dirs: Vec<String>,
    cur_dir_files: Vec<String>,
    recursive: bool,
}

impl FileNames {
    pub(crate) fn generate(dir: Option<String>, pattern: FileFilter, recursive: bool) -> FileNames {
        let cur_dir = match dir {
            Some(dir) => dir,
            None => "./".to_string(),
        };

        FileNames {
            cur_dir: Some(cur_dir),
            filter: pattern,
            all_dirs: Vec::new(),
            cur_dir_files: Vec::new(),
            recursive,
        }
    }

    fn get_next_dir(&mut self) -> Option<ReadDir> {
        loop {
            if let Some(dir) = self.all_dirs.pop() {
                let entries = fs::read_dir(&dir);
                if entries.is_err() {
                    println!("Could not read dir: {}", dir);
                    continue;
                }

                return Some(entries.unwrap());
            }

            return None;
        }
    }

    fn populate_files(&mut self) -> bool {
        if self.cur_dir_files.len() > 0 {
            return true;
        }

        let next_dir = self.get_next_dir();
        if next_dir.is_none() {
            return false;
        }

        let next_dir = next_dir.unwrap();
        for entry in next_dir {
            if entry.is_err() {
                println!(
                    "Could not read file {}, skipping",
                    entry.unwrap().path().display()
                );
                continue;
            }

            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                if self.recursive {
                    self.all_dirs.push(path.to_str().unwrap().to_string());
                }
            } else if path.is_file() {
                if self.filter.apply(path.to_str().unwrap()) {
                    self.cur_dir_files.push(path.to_str().unwrap().to_string());
                }
            } else {
                println!("Skipping {}, unknown file type", path.display());
            }
        }

        true
    }
}

impl Iterator for FileNames {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(dir) = self.cur_dir.take() {
            self.all_dirs.push(dir);
        }

        if let Some(file) = self.cur_dir_files.pop() {
            return Some(file);
        }

        loop {
            let populated = self.populate_files();
            if !populated {
                return None;
            }

            if self.cur_dir_files.len() > 0 {
                return Some(self.cur_dir_files.pop().unwrap());
            }
        }
    }
}
