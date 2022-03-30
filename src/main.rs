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

#[cfg(test)]
mod test_main {

    use crate::parser;
    use crate::translator;

    macro_rules! assert_convert {
        ($markdown:expr, $html:expr) => {
            assert_eq!(
                translator::translate(parser::parse_markdown($markdown).unwrap().1),
                String::from($html)
            );
        };
    }

    #[test]
    fn test_convert() {
        assert_convert!("# h1\n", "<h1>h1</h1>");
        assert_convert!("## h2\n", "<h2>h2</h2>");
        assert_convert!("- a\n- b\n- c\n", "<ul><li>a</li><li>b</li><li>c</li></ul>");
    }

    #[test]
    fn test_examples_full() {
        use std::fs::read_to_string;
        let content = read_to_string("./examples/full.md").unwrap();
        let expected = read_to_string("./examples/full.html").unwrap();
        assert_convert!(content.as_str(), expected.as_str());
    }
}
