use argh::FromArgs;
use filters::FileFilter;
use fixer::Fixer;
use stats::{FileNames, FileStats, FileStatsAggregate, LineSep};
mod filters;
mod fixer;
mod stats;

fn main() {
    let args: Args = argh::from_env();

    let filter = match args.ext {
        Some(ext) => FileFilter::extension(&ext),
        None => FileFilter::None,
    };

    let stats = FileNames::generate(args.dir.clone(), filter.clone(), args.recursive)
        .filter_map(FileStats::generate)
        .fold(FileStatsAggregate::new(), FileStatsAggregate::fold);
    stats.print_table();

    if args.fix {
        let target = match args.target {
            Some(target) => {
                println!("Fixing line endings to provided line ending - {:}", target);
                target
            }
            None => match stats.max() {
                Some(max) => {
                    println!("Fixing line endings to most common line ending - {:}", max);
                    max
                }
                None => {
                    panic!("No line endings found to fix");
                }
            },
        };

        FileNames::generate(args.dir, filter, args.recursive)
            .map(|file_name| Fixer::new(file_name, target.clone()))
            .for_each(|mut fixer| fixer.fix());
    }
}

/// Line endings fixer tool
#[derive(FromArgs)]
struct Args {
    /// directory to search for files in
    #[argh(positional)]
    dir: Option<String>,

    /// extension to filter files by : Ex: -e .txt
    #[argh(option, short = 'e')]
    ext: Option<String>,

    /// recursively traverse directories
    #[argh(switch, short = 'r')]
    recursive: bool,

    /// fix line endings
    #[argh(switch, short = 'f')]
    fix: bool,

    /// target line ending, applicable only with -f
    #[argh(option, short = 't', from_str_fn(parse_line_sep))]
    target: Option<LineSep>,
}

fn parse_line_sep(s: &str) -> Result<LineSep, String> {
    match s {
        "lf" => Ok(LineSep::Lf),
        "crlf" => Ok(LineSep::CrLf),
        "cr" => Ok(LineSep::Cr),
        _ => Err("Invalid line ending".to_string()),
    }
}
