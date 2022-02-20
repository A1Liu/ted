pub mod ast;
pub mod checker;
pub mod errors;
pub mod parser;
pub mod print_format;
pub mod types;

#[cfg(test)]
mod tests {
    use super::checker::*;
    use super::errors::*;
    use super::parser::*;
    use super::types::*;
    use crate::util::*;
    use core::fmt::Write;

    // eventually use src/bin

    #[test]
    fn test_parser() {
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

        let printed = format!("{:#?}", ast.block);

        println!("{}", printed);

        // panic!("viewing");
    }
}
