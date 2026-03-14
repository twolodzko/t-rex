use clap::Parser;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read, Write},
    path::PathBuf,
};
use t_rex::Regex;

#[derive(Parser, Debug)]
struct Args {
    /// Regular expression
    #[arg(allow_hyphen_values = true)]
    regex: String,

    /// Only a count of selected lines is written to standard output
    #[arg(short, long)]
    count: bool,

    /// Perform case-insensitive matching
    #[arg(short, long)]
    ignore_case: bool,

    /// Display dot graph of the NFA for the regular expression and exit
    #[arg(long)]
    graph: bool,

    /// Files to process
    #[arg(trailing_var_arg(true))]
    file: Vec<PathBuf>,
}

macro_rules! error {
    ( $msg:expr ) => {{
        eprintln!("{}", $msg);
        std::process::exit(2);
    }};
}

macro_rules! print {
    ( $out:expr, $msg:expr ) => {
        if let Err(err) = writeln!($out, "{}", $msg) {
            eprintln!("{}", err);
        }
    };
}

fn main() {
    let args = Args::parse();
    let mut inp: BufReader<Box<dyn Read>>;
    let mut out = io::stdout().lock();

    let regex = match Regex::new(&args.regex, args.ignore_case) {
        Ok(regex) => regex,
        Err(err) => error!(err),
    };

    if args.graph {
        print!(out, regex.graph());
        return;
    }

    let mut count = 0;
    if args.file.is_empty() {
        inp = BufReader::new(Box::new(io::stdin()));
        count = process(&regex, &args, inp, &mut out);
    } else {
        for path in &args.file {
            inp = match File::open(path) {
                Ok(file) => BufReader::new(Box::new(file)),
                Err(err) => error!(err),
            };
            count += process(&regex, &args, inp, &mut out);
        }
    }

    if args.count {
        print!(out, count)
    }
    if count == 0 {
        // like grep, report if any match was found
        std::process::exit(1)
    }
}

fn process(
    regex: &Regex,
    args: &Args,
    inp: BufReader<Box<dyn Read>>,
    out: &mut io::StdoutLock,
) -> usize {
    let mut count = 0;
    for line in inp.lines() {
        match line {
            Ok(line) => {
                if regex.is_match(&line) {
                    count += 1;
                    if !args.count {
                        print!(out, line)
                    }
                }
            }
            Err(err) => error!(err),
        }
    }
    count
}
