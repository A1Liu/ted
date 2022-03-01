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

    // eventually use src/bin

    #[test]
    fn sample() {
        let mut table = StringTable::new();
        let mut files = FileDb::new();

        let text = r#"
        let a = 12
        let b = a + 12 + 13
        let c = print(a,b,)
        print(a, b)
        "#;

        if let Err(e) = files.add("data.liu", text) {
            panic!("{}", e);
        }

        let data = match lex(&mut table, 0, text) {
            Ok(data) => data,
            Err(e) => {
                let mut out = String::new();

                expect(e.render(&files, &mut out));

                eprintln!("{}\n", out);
                panic!("{:?}", e);
            }
        };

        let ast = match parse(&table, 0, data) {
            Ok(data) => data,
            Err(e) => {
                let mut out = String::new();

                expect(e.render(&files, &mut out));

                eprintln!("{}\n", out);
                panic!("{:?}", e);
            }
        };

        let env = match check_ast(&ast) {
            Ok(data) => data,
            Err(e) => {
                let mut out = String::new();

                expect(e.render(&files, &mut out));

                eprintln!("{}\n", out);
                panic!("{:?}", e);
            }
        };

        let printed = format!("{:#?}", ast.block);

        println!("{}", printed);

        // panic!("viewing");
    }
}
