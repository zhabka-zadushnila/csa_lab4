pub mod assembler;
pub mod lexer;
pub mod parser;

pub use assembler::Assembler;
pub use lexer::Lexer;
pub use parser::Parser;

pub struct CompileResult {
    pub binary: Vec<u8>,
    pub data_size: u32,
    pub assembly: String,
    pub ast_debug: String,
    pub num_functions: usize,
    pub num_statements: usize,
}

pub fn compile(source: &str) -> Result<CompileResult, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.parse().map_err(|e| format!("Lexical error: {}", e))?;

    let mut parser = Parser::new(tokens);
    let program = parser
        .parse_program()
        .map_err(|e| format!("Parse error: {}", e))?;

    let num_functions = program.functions.len();
    let num_statements = program.statements.len();
    let ast_debug = program.format_ast();

    let mut asm = Assembler::new();
    asm.compile_program(&program);
    let (binary, assembly, data_size) = asm.output_results();

    Ok(CompileResult {
        binary,
        data_size,
        assembly,
        ast_debug,
        num_functions,
        num_statements,
    })
}
