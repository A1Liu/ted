mod ast;
mod checker;
mod errors;
mod interp;
mod parser;
mod print_format;
mod types;

pub use ast::*;
pub use checker::*;
pub use errors::*;
pub use interp::*;
pub use parser::*;
pub use print_format::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::*;
    use core::fmt::Write;

    #[test]
    fn procedures() {
        run_on_file("procedures.liu");
    }

    #[test]
    fn simple() {
        run_on_file("simple.liu");
    }

    fn run_on_file(name: &str) {
        let mut path = "tests/".to_string();
        path.push_str(name);

        let buf = expect(std::fs::read_to_string(&path));
        let text = &buf;

        let mut files = FileDb::new();

        files.add(name, text);

        let out = match run_on_file_err(text) {
            Ok(out) => out,
            Err(e) => {
                let mut out = String::new();

                expect(e.render(&files, &mut out));

                eprintln!("{}\n", out);
                panic!("{:?}", e);
            }
        };

        println!("{}", out);

        assert_eq!(&*out, "12 37\n12\n");

        // panic!("viewing");
    }

    fn run_on_file_err(text: &str) -> Result<String, Error> {
        let mut table = StringTable::new();

        let data = lex(&mut table, 0, text)?;

        let ast = parse(&table, 0, data)?;

        let env = check_ast(&ast)?;

        let mut out = String::new();
        interpret(&ast, &env, &mut out);

        return Ok(out);
    }
}
