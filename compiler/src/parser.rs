use crate::lexer::{Token, TokenKind};

#[derive(Debug)]
pub enum Type {
    I32,
    StringType,
    Array(Box<Type>, usize),
}

#[derive(Debug)]
pub struct Program {
    pub functions: Vec<FunctionDecl>,
    pub statements: Vec<Statement>,
}

use std::fmt::Write;

impl Program {
    pub fn format_ast(&self) -> String {
        let mut out = String::new();
        writeln!(out, "Program").unwrap();
        for stmt in &self.statements {
            fmt_stmt(&mut out, stmt, 1);
        }
        for func in &self.functions {
            fmt_func(&mut out, func, 1);
        }
        out
    }
}

fn indent(out: &mut String, level: usize) {
    for _ in 0..level {
        out.push_str("  ");
    }
}

fn fmt_type(typ: &Type) -> String {
    match typ {
        Type::I32 => "i32".into(),
        Type::StringType => "String".into(),
        Type::Array(inner, size) => format!("[{}; {}]", fmt_type(inner), size),
    }
}

fn fmt_binary_op(op: &BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "Add",
        BinaryOp::Sub => "Sub",
        BinaryOp::Mul => "Mul",
        BinaryOp::Div => "Div",
        BinaryOp::Rem => "Rem",
        BinaryOp::Eq => "Eq",
        BinaryOp::NotEq => "NotEq",
        BinaryOp::Less => "Less",
        BinaryOp::LessEq => "LessEq",
        BinaryOp::Greater => "Greater",
        BinaryOp::GreaterEq => "GreaterEq",
        BinaryOp::And => "And",
        BinaryOp::Or => "Or",
        BinaryOp::BitAnd => "BitAnd",
        BinaryOp::BitOr => "BitOr",
        BinaryOp::Xor => "Xor",
    }
}

fn fmt_unary_op(op: &UnaryOp) -> &'static str {
    match op {
        UnaryOp::Neg => "Neg",
        UnaryOp::Not => "Not",
        UnaryOp::BitNot => "BitNot",
    }
}

fn fmt_expr(out: &mut String, expr: &Expr, level: usize) {
    match expr {
        Expr::Number(n) => {
            indent(out, level);
            writeln!(out, "Number {}", n).unwrap();
        }
        Expr::String(s) => {
            indent(out, level);
            out.push_str("String \"");
            for c in s.chars() {
                match c {
                    '\n' => out.push_str("\\n"),
                    '\t' => out.push_str("\\t"),
                    '\\' => out.push_str("\\\\"),
                    '"' => out.push_str("\\\""),
                    c => out.push(c),
                }
            }
            writeln!(out, "\"").unwrap();
        }
        Expr::Variable(name) => {
            indent(out, level);
            writeln!(out, "Variable {}", name).unwrap();
        }
        Expr::UnaryOp(op, inner) => {
            indent(out, level);
            writeln!(out, "{}", fmt_unary_op(op)).unwrap();
            fmt_expr(out, inner, level + 1);
        }
        Expr::BinaryOp(left, op, right) => {
            indent(out, level);
            writeln!(out, "{}", fmt_binary_op(op)).unwrap();
            fmt_expr(out, left, level + 1);
            fmt_expr(out, right, level + 1);
        }
        Expr::FunctionCall(name, args) => {
            indent(out, level);
            writeln!(out, "FunctionCall {}", name).unwrap();
            for arg in args {
                fmt_expr(out, arg, level + 1);
            }
        }
        Expr::ArrayInit(elems) => {
            indent(out, level);
            writeln!(out, "ArrayInit").unwrap();
            for elem in elems {
                fmt_expr(out, elem, level + 1);
            }
        }
        Expr::ArrayAccess(name, index) => {
            indent(out, level);
            writeln!(out, "ArrayAccess {}", name).unwrap();
            fmt_expr(out, index, level + 1);
        }
        Expr::Grouping(inner) => {
            fmt_expr(out, inner, level);
        }
    }
}

fn fmt_block(out: &mut String, block: &Block, level: usize) {
    indent(out, level);
    writeln!(out, "Body").unwrap();
    for stmt in &block.statements {
        fmt_stmt(out, stmt, level + 1);
    }
}

fn fmt_stmt(out: &mut String, stmt: &Statement, level: usize) {
    match stmt {
        Statement::Let(name, typ, expr) => {
            indent(out, level);
            writeln!(out, "Let {} : {}", name, fmt_type(typ)).unwrap();
            fmt_expr(out, expr, level + 1);
        }
        Statement::Assign(name, index, expr) => {
            indent(out, level);
            if let Some(idx) = index {
                writeln!(out, "Assign {}[]", name).unwrap();
                fmt_expr(out, idx, level + 1);
            } else {
                writeln!(out, "Assign {}", name).unwrap();
            }
            fmt_expr(out, expr, level + 1);
        }
        Statement::If(cond, then_block, else_block) => {
            indent(out, level);
            writeln!(out, "If").unwrap();
            indent(out, level + 1);
            writeln!(out, "Cond").unwrap();
            fmt_expr(out, cond, level + 2);
            indent(out, level + 1);
            writeln!(out, "Then").unwrap();
            fmt_block(out, then_block, level + 2);
            if let Some(eb) = else_block {
                indent(out, level + 1);
                writeln!(out, "Else").unwrap();
                fmt_block(out, eb, level + 2);
            }
        }
        Statement::While(cond, body) => {
            indent(out, level);
            writeln!(out, "While").unwrap();
            indent(out, level + 1);
            writeln!(out, "Cond").unwrap();
            fmt_expr(out, cond, level + 2);
            fmt_block(out, body, level + 1);
        }
        Statement::Return(expr_opt) => {
            indent(out, level);
            if let Some(expr) = expr_opt {
                writeln!(out, "Return").unwrap();
                fmt_expr(out, expr, level + 1);
            } else {
                writeln!(out, "Return").unwrap();
            }
        }
        Statement::Expr(expr) => {
            indent(out, level);
            writeln!(out, "ExprStmt").unwrap();
            fmt_expr(out, expr, level + 1);
        }
    }
}

fn fmt_func(out: &mut String, func: &FunctionDecl, level: usize) {
    indent(out, level);
    let params_str: Vec<String> = func
        .params
        .iter()
        .map(|p| format!("{}: {}", p.name, fmt_type(&p.typ)))
        .collect();
    if let Some(ret) = &func.return_type {
        writeln!(
            out,
            "FunctionDecl {} ({}) -> {}",
            func.name,
            params_str.join(", "),
            fmt_type(ret)
        )
        .unwrap();
    } else {
        writeln!(
            out,
            "FunctionDecl {} ({})",
            func.name,
            params_str.join(", ")
        )
        .unwrap();
    }
    fmt_block(out, &func.body, level + 1);
}

#[derive(Debug)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Block,
}

#[derive(Debug)]
pub struct Param {
    pub name: String,
    pub typ: Type,
}

#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    Let(String, Type, Expr),
    Assign(String, Option<Box<Expr>>, Expr),
    If(Expr, Block, Option<Block>),
    While(Expr, Block),
    Return(Option<Expr>),
    Expr(Expr),
}

#[derive(Debug)]
pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
}

#[derive(Debug)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    NotEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    And,
    Or,
    BitAnd,
    BitOr,
    Xor,
}

#[derive(Debug)]
pub enum Expr {
    Number(i32),
    String(String),
    Variable(String),
    UnaryOp(UnaryOp, Box<Expr>),
    BinaryOp(Box<Expr>, BinaryOp, Box<Expr>),
    FunctionCall(String, Vec<Expr>),
    ArrayInit(Vec<Expr>),
    ArrayAccess(String, Box<Expr>),
    Grouping(Box<Expr>),
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut functions = Vec::new();
        let mut statements = Vec::new();

        while !self.is_at_end() {
            match &self.peek().kind {
                TokenKind::FnDecl => {
                    functions.push(self.parse_function_decl()?);
                }
                TokenKind::Let => {
                    statements.push(self.parse_let_stmt()?);
                }
                TokenKind::EOF => break,
                other => {
                    let tok = self.peek();
                    return Err(format!(
                        "Expected function or declaration, got {:?} at {}:{}",
                        other, tok.line, tok.col
                    ));
                }
            }
        }

        Ok(Program {
            functions,
            statements,
        })
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) -> &Token {
        let t = &self.tokens[self.pos];
        self.pos += 1;
        t
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.tokens[self.pos].kind == *kind
    }

    fn expect(&mut self, kind: TokenKind) -> Result<(), String> {
        if self.check(&kind) {
            self.advance();
            Ok(())
        } else {
            let tok = self.peek();
            Err(format!(
                "Expected {:?}, got {:?} at {}:{}",
                kind, tok.kind, tok.line, tok.col
            ))
        }
    }

    fn is_at_end(&self) -> bool {
        self.tokens[self.pos].kind == TokenKind::EOF
    }

    fn parse_function_decl(&mut self) -> Result<FunctionDecl, String> {
        self.expect(TokenKind::FnDecl)?;
        let name = self.expect_identifier()?;
        self.expect(TokenKind::RoundBracketOpen)?;
        let params = if self.check(&TokenKind::RoundBracketClose) {
            Vec::new()
        } else {
            self.parse_param_list()?
        };
        self.expect(TokenKind::RoundBracketClose)?;
        let return_type = if self.check(&TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        let body = self.parse_block()?;
        Ok(FunctionDecl {
            name,
            params,
            return_type,
            body,
        })
    }

    fn expect_identifier(&mut self) -> Result<String, String> {
        let tok = self.advance();
        match &tok.kind {
            TokenKind::Variable(s) => Ok(s.clone()),
            other => Err(format!(
                "Expected identifier, got {:?} at {}:{}",
                other, tok.line, tok.col
            )),
        }
    }

    fn parse_param_list(&mut self) -> Result<Vec<Param>, String> {
        let mut params = Vec::new();
        params.push(self.parse_param()?);
        while self.check(&TokenKind::Comma) {
            self.advance();
            params.push(self.parse_param()?);
        }
        Ok(params)
    }

    fn parse_param(&mut self) -> Result<Param, String> {
        let name = self.expect_identifier()?;
        self.expect(TokenKind::Colon)?;
        let typ = self.parse_type()?;
        Ok(Param { name, typ })
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        match &self.peek().kind {
            TokenKind::I32Decl => {
                self.advance();
                Ok(Type::I32)
            }
            TokenKind::StringType => {
                self.advance();
                Ok(Type::StringType)
            }
            TokenKind::SquareBracketOpen => {
                self.advance();
                let inner = self.parse_type()?;
                self.expect(TokenKind::Semicolon)?;
                let tok = self.advance();
                let size = match &tok.kind {
                    TokenKind::Number(n) => *n,
                    other => {
                        return Err(format!(
                            "Expected number for array size, got {:?} at {}:{}",
                            other, tok.line, tok.col
                        ));
                    }
                };
                self.expect(TokenKind::SquareBracketClose)?;
                Ok(Type::Array(Box::new(inner), size as usize))
            }
            other => {
                let tok = self.peek();
                Err(format!(
                    "Expected type (i32, String, or [type; N]), got {:?} at {}:{}",
                    other, tok.line, tok.col
                ))
            }
        }
    }

    fn parse_block(&mut self) -> Result<Block, String> {
        self.expect(TokenKind::CurlyBracketOpen)?;
        let mut statements = Vec::new();
        while !self.check(&TokenKind::CurlyBracketClose) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        self.expect(TokenKind::CurlyBracketClose)?;
        Ok(Block { statements })
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match &self.peek().kind {
            TokenKind::Let => self.parse_let_stmt(),
            TokenKind::Return => self.parse_return_stmt(),
            TokenKind::If => self.parse_if_stmt(),
            TokenKind::While => self.parse_while_stmt(),
            TokenKind::Variable(_) => self.parse_assign_or_expr_stmt(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_let_stmt(&mut self) -> Result<Statement, String> {
        self.expect(TokenKind::Let)?;
        let name = self.expect_identifier()?;
        self.expect(TokenKind::Colon)?;
        let typ = self.parse_type()?;
        self.expect(TokenKind::Assign)?;
        let value = self.parse_expr()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Statement::Let(name, typ, value))
    }

    fn parse_return_stmt(&mut self) -> Result<Statement, String> {
        self.expect(TokenKind::Return)?;
        let expr = if self.check(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expr()?)
        };
        self.expect(TokenKind::Semicolon)?;
        Ok(Statement::Return(expr))
    }

    fn parse_if_stmt(&mut self) -> Result<Statement, String> {
        self.expect(TokenKind::If)?;
        let cond = self.parse_expr()?;
        let then_block = self.parse_block()?;
        let else_block = if self.check(&TokenKind::Else) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };
        Ok(Statement::If(cond, then_block, else_block))
    }

    fn parse_while_stmt(&mut self) -> Result<Statement, String> {
        self.expect(TokenKind::While)?;
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Statement::While(cond, body))
    }

    fn parse_expr_stmt(&mut self) -> Result<Statement, String> {
        let expr = self.parse_expr()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Statement::Expr(expr))
    }

    fn parse_assign_or_expr_stmt(&mut self) -> Result<Statement, String> {
        let name = match &self.peek().kind {
            TokenKind::Variable(s) => s.clone(),
            _ => return self.parse_expr_stmt(),
        };

        if self.pos + 1 < self.tokens.len() {
            let next_kind = &self.tokens[self.pos + 1].kind;
            match next_kind {
                TokenKind::Assign => {
                    self.advance();
                    self.advance();
                    let value = self.parse_expr()?;
                    self.expect(TokenKind::Semicolon)?;
                    return Ok(Statement::Assign(name, None, value));
                }
                TokenKind::SquareBracketOpen => {
                    let save = self.pos;
                    self.advance();
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect(TokenKind::SquareBracketClose)?;
                    if self.check(&TokenKind::Assign) {
                        self.advance();
                        let value = self.parse_expr()?;
                        self.expect(TokenKind::Semicolon)?;
                        return Ok(Statement::Assign(name, Some(Box::new(index)), value));
                    }
                    self.pos = save;
                }
                _ => {}
            }
        }

        self.parse_expr_stmt()
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_logical_and()?;
        while self.check(&TokenKind::OpOr) {
            self.advance();
            let right = self.parse_logical_and()?;
            left = Expr::BinaryOp(Box::new(left), BinaryOp::Or, Box::new(right));
        }
        Ok(left)
    }

    fn parse_logical_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_bit_or()?;
        while self.check(&TokenKind::OpAnd) {
            self.advance();
            let right = self.parse_bit_or()?;
            left = Expr::BinaryOp(Box::new(left), BinaryOp::And, Box::new(right));
        }
        Ok(left)
    }

    fn parse_bit_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_bit_xor()?;
        while self.check(&TokenKind::OpBitOr) {
            self.advance();
            let right = self.parse_bit_xor()?;
            left = Expr::BinaryOp(Box::new(left), BinaryOp::BitOr, Box::new(right));
        }
        Ok(left)
    }

    fn parse_bit_xor(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_bit_and()?;
        while self.check(&TokenKind::OpXor) {
            self.advance();
            let right = self.parse_bit_and()?;
            left = Expr::BinaryOp(Box::new(left), BinaryOp::Xor, Box::new(right));
        }
        Ok(left)
    }

    fn parse_bit_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_equality()?;
        while self.check(&TokenKind::OpBitAnd) {
            self.advance();
            let right = self.parse_equality()?;
            left = Expr::BinaryOp(Box::new(left), BinaryOp::BitAnd, Box::new(right));
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_relational()?;
        loop {
            let op = match &self.peek().kind {
                TokenKind::OpEq => BinaryOp::Eq,
                TokenKind::OpNotEq => BinaryOp::NotEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_relational()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_relational(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match &self.peek().kind {
                TokenKind::OpLessThan => BinaryOp::Less,
                TokenKind::OpLessOrEq => BinaryOp::LessEq,
                TokenKind::OpGreaterThan => BinaryOp::Greater,
                TokenKind::OpGreaterOrEq => BinaryOp::GreaterEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match &self.peek().kind {
                TokenKind::OpAdd => BinaryOp::Add,
                TokenKind::OpSub => BinaryOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match &self.peek().kind {
                TokenKind::OpMul => BinaryOp::Mul,
                TokenKind::OpDiv => BinaryOp::Div,
                TokenKind::OpRem => BinaryOp::Rem,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        match &self.peek().kind {
            TokenKind::OpSub => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp(UnaryOp::Neg, Box::new(expr)))
            }
            TokenKind::OpNot => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp(UnaryOp::Not, Box::new(expr)))
            }
            TokenKind::OpBitNot => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp(UnaryOp::BitNot, Box::new(expr)))
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let kind = self.peek().kind.clone();
        let (line, col) = (self.peek().line, self.peek().col);
        match &kind {
            TokenKind::Number(n) => {
                self.advance();
                Ok(Expr::Number(*n))
            }
            TokenKind::String(s) => {
                self.advance();
                Ok(Expr::String(s.clone()))
            }
            TokenKind::Variable(_) | TokenKind::Out | TokenKind::In => {
                let name = match &kind {
                    TokenKind::Variable(s) => s.clone(),
                    TokenKind::Out => "out".to_string(),
                    _ => "in".to_string(),
                };
                self.advance();
                if self.check(&TokenKind::RoundBracketOpen) {
                    self.parse_function_call(name)
                } else if self.check(&TokenKind::SquareBracketOpen) {
                    self.parse_array_access(name)
                } else {
                    Ok(Expr::Variable(name))
                }
            }
            TokenKind::RoundBracketOpen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RoundBracketClose)?;
                Ok(Expr::Grouping(Box::new(expr)))
            }
            TokenKind::SquareBracketOpen => self.parse_array_init(),
            other => Err(format!(
                "Unexpected token {:?} in expression at {}:{}",
                other, line, col
            )),
        }
    }

    fn parse_function_call(&mut self, name: String) -> Result<Expr, String> {
        self.advance();
        let mut args = Vec::new();
        if !self.check(&TokenKind::RoundBracketClose) {
            args.push(self.parse_expr()?);
            while self.check(&TokenKind::Comma) {
                self.advance();
                args.push(self.parse_expr()?);
            }
        }
        self.expect(TokenKind::RoundBracketClose)?;
        Ok(Expr::FunctionCall(name, args))
    }

    fn parse_array_access(&mut self, name: String) -> Result<Expr, String> {
        self.advance();
        let index = self.parse_expr()?;
        self.expect(TokenKind::SquareBracketClose)?;
        Ok(Expr::ArrayAccess(name, Box::new(index)))
    }

    fn parse_array_init(&mut self) -> Result<Expr, String> {
        self.advance();
        let mut elems = Vec::new();
        if !self.check(&TokenKind::SquareBracketClose) {
            elems.push(self.parse_expr()?);
            while self.check(&TokenKind::Comma) {
                self.advance();
                elems.push(self.parse_expr()?);
            }
        }
        self.expect(TokenKind::SquareBracketClose)?;
        Ok(Expr::ArrayInit(elems))
    }
}
