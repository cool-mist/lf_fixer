use argh::FromArgs;
use filters::FileFilter;
use stats::{FileNames, FileStats, FileStatsAggregate};
mod filters;
mod stats;

fn main() {
    let args: Args = argh::from_env();

    let filter = match args.ext {
        Some(ext) => FileFilter::extension(&ext),
        None => FileFilter::None,
    };

    let stats = FileNames::generate(args.dir, filter, args.recursive)
        .filter_map(FileStats::generate)
        .fold(FileStatsAggregate::new(), FileStatsAggregate::fold);
    stats.print_table();
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
}
