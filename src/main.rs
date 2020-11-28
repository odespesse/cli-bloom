use clap::{App, Arg};
use cli_bloom::FsIndex;

fn main() {
    let matches = App::new("cli-bloom")
                   .version("1.0")
                   .about("A command line app to manage a bloom index.")
                   .arg(Arg::with_name("source")
                        .short("s")
                        .long("source")
                        .help("Path to the file or directory to index")
                        .takes_value(true))
                   .arg(Arg::with_name("restore")
                        .short("r")
                        .long("restore")
                        .help("Path to an index dump file")
                        .takes_value(true))
                   .arg(Arg::with_name("dump")
                        .short("d")
                        .long("dump")
                        .help("Path to dump the current index")
                        .takes_value(true))
                   .get_matches();

    let mut index = match matches.value_of("restore") {
        Some(restore_file) => FsIndex::restore(restore_file),
        None => FsIndex::new(0.00001)
    };
    if let Some(source) = matches.value_of("source") {
        index.ingest(source);
    }
    if let Some(dump_file) = matches.value_of("dump") {
        index.dump(dump_file);
    }
}

