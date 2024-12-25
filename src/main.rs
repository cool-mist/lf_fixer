use argh::FromArgs;
use fixer::Fixer;
use iter::FileNames;
use stats::{FileStats, FileStatsAggregate, LineSep};
mod filters;
mod fixer;
mod iter;
mod stats;

/// Line endings fixer tool
#[derive(FromArgs)]
struct Args {
    /// directory to search for files in
    #[argh(option, short = 'd')]
    dir: Option<String>,

    /// specific file to evaluate. All other options are ignored
    #[argh(positional)]
    file_name: Option<String>,

    /// extension to filter files by : Ex: -e .txt
    #[argh(option, short = 'e')]
    ext: Option<String>,

    /// recursively traverse directories
    #[argh(switch, short = 'r')]
    recursive: bool,

    /// normalize line endings
    #[argh(option, short = 'n', from_str_fn(parse_norm_option))]
    normalize: Option<NormalizeOption>,
}

enum NormalizeOption {
    /// normalize to the most frequent line ending
    /// across all files matching the filter.
    MostFrequent,

    /// normalize to lf
    Lf,

    /// normalize to crlf
    CrLf,

    /// normalize to cr
    Cr,
}

fn main() {
    let args: Args = argh::from_env();

    let stats = FileNames::new(&args)
        .filter_map(FileStats::generate)
        .fold(FileStatsAggregate::new(), FileStatsAggregate::fold);
    stats.print_table();

    if let Some(normalize_option) = &args.normalize {
        let target = match normalize_option {
            NormalizeOption::Lf => LineSep::Lf,
            NormalizeOption::CrLf => LineSep::CrLf,
            NormalizeOption::Cr => LineSep::Cr,
            NormalizeOption::MostFrequent => match stats.max() {
                Some(max) => max,
                None => {
                    println!("Target line ending could not be determined. Skipping normalization");
                    return;
                }
            },
        };

        FileNames::new(&args)
            .map(|file_name| Fixer::new(file_name, target.clone()))
            .for_each(|mut fixer| fixer.fix());
    }
}

fn parse_norm_option(s: &str) -> Result<NormalizeOption, String> {
    match s {
        "lf" => Ok(NormalizeOption::Lf),
        "crlf" => Ok(NormalizeOption::CrLf),
        "cr" => Ok(NormalizeOption::Cr),
        "any" => Ok(NormalizeOption::MostFrequent),
        _ => Err("Invalid line ending".to_string()),
    }
}
