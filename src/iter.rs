use std::fs::{self, ReadDir};

use crate::{filters::FileFilter, Args};

pub(crate) struct SingleFileName {
    file_name: Option<String>,
}

pub(crate) struct MultipleFileNames {
    cur_dir: Option<String>,
    filter: FileFilter,
    all_dirs: Vec<String>,
    cur_dir_files: Vec<String>,
    recursive: bool,
}

pub(crate) enum FileNames {
    Single(SingleFileName),
    Multiple(MultipleFileNames),
}

impl SingleFileName {
    fn new(file_name: String) -> SingleFileName {
        SingleFileName {
            file_name: Some(file_name),
        }
    }
}

impl MultipleFileNames {
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

impl Iterator for MultipleFileNames {
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

impl FileNames {
    pub(crate) fn new(args: &Args) -> FileNames {
        if let Some(file_name) = &args.file_name {
            return FileNames::Single(SingleFileName::new(file_name.to_string()));
        }

        let cur_dir = match &args.dir {
            Some(dir) => dir.to_string(),
            None => "./".to_string(),
        };

        let filter = match &args.ext {
            Some(ext) => FileFilter::extension(&ext),
            None => FileFilter::None,
        };

        FileNames::Multiple(MultipleFileNames {
            cur_dir: Some(cur_dir),
            filter,
            all_dirs: Vec::new(),
            cur_dir_files: Vec::new(),
            recursive: args.recursive,
        })
    }
}

impl Iterator for FileNames {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            FileNames::Single(file_name) => file_name.file_name.take(),
            FileNames::Multiple(file_names) => file_names.next(),
        }
    }
}
