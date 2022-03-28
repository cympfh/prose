pub mod entity;
pub mod parser;
pub mod translator;

use std::io::{self, Read};
use structopt::StructOpt;

fn read() -> String {
    let mut content = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_string(&mut content).unwrap();
    if !content.ends_with('\n') {
        content += "\n"
    }
    content
}

fn write(buf: &String) {
    println!("{}", buf);
}

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long = "debug")]
    pub debug: bool,
}

fn main() {
    let opt = Opt::from_args();
    if opt.debug {
        println!(">>> opt = {:?}", &opt);
    }
    let content = read();
    if let Ok((_, markdown)) = parser::parse_markdown(content.as_str()) {
        if opt.debug {
            println!(">>> markdown = {:?}", &markdown);
        }
        let html = translator::translate(markdown);
        write(&html);
    } else {
        eprintln!("Something critical error");
    }
}
