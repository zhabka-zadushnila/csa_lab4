use std::collections::HashMap;

use crate::parser::{BinaryOp, Block, Expr, FunctionDecl, Program, Statement, Type, UnaryOp};

const OP_LD: u8 = 0x01;
const OP_ST: u8 = 0x02;
const OP_ADD: u8 = 0x03;
const OP_SUB: u8 = 0x04;
const OP_MUL: u8 = 0x05;
const OP_DIV: u8 = 0x06;
const OP_MOD: u8 = 0x07;
const OP_CMP: u8 = 0x08;
const OP_NEG: u8 = 0x09;
const OP_NOT: u8 = 0x0A;
const OP_AND: u8 = 0x0B;
const OP_OR: u8 = 0x0C;
const OP_XOR: u8 = 0x0D;
const OP_JMP: u8 = 0x0E;
const OP_JZS: u8 = 0x0F;
const OP_JZC: u8 = 0x10;
const OP_JNS: u8 = 0x11;
const OP_JNC: u8 = 0x12;
const OP_CALL: u8 = 0x13;
const OP_RET: u8 = 0x14;
const OP_HALT: u8 = 0x15;

const TEMP_COUNT: usize = 32;
const TEMP_IDX_COUNT: usize = 8;
const TEMP_PTR_COUNT: usize = 8;

#[derive(Debug, Clone)]
enum Operand {
    Named(String),
    Immediate(i32),
    Addr(u32),
}

#[derive(Debug, Clone)]
enum Command {
    Word(Vec<i32>),
    Label(String),

    Ld(Operand),
    Ldi(Operand),
    St(Operand),
    Sti(Operand),
    Add(Operand),
    Sub(Operand),
    Mul(Operand),
    Div(Operand),
    Mod(Operand),
    Cmp(Operand),
    Neg,
    Not,
    And(Operand),
    Or(Operand),
    Xor(Operand),
    Jmp(Operand),
    Jzs(Operand),
    Jzc(Operand),
    Jns(Operand),
    Jnc(Operand),
    Call(Operand),
    Ret,
    Halt,
}

pub struct Assembler {
    data_cmds: Vec<Command>,
    code_cmds: Vec<Command>,
    symbols: HashMap<String, u32>,
    func_params: HashMap<String, Vec<String>>,
    data_addr: u32,
    label_counter: usize,
    text: String,
    binary: Vec<u8>,
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}

impl Assembler {
    pub fn new() -> Self {
        Assembler {
            data_cmds: Vec::new(),
            code_cmds: Vec::new(),
            symbols: HashMap::new(),
            func_params: HashMap::new(),
            data_addr: 5,
            label_counter: 0,
            text: String::new(),
            binary: Vec::new(),
        }
    }

    pub fn compile_program(&mut self, program: &Program) {
        self.collect_symbols(program);
        if let Some(main) = program.functions.iter().find(|f| f.name == "main") {
            self.compile_function(main);
        }
        for func in &program.functions {
            if func.name != "main" {
                self.compile_function(func);
            }
        }
        self.resolve_and_emit();
    }

    pub fn output_results(self) -> (Vec<u8>, String, u32) {
        (self.binary, self.text, self.data_addr)
    }
    pub fn output_text(&self) -> &str {
        &self.text
    }

    pub fn output_binary(&self) -> &[u8] {
        &self.binary
    }
    fn add_variable(&mut self, name: String, size: u32) -> u32 {
        let addr = self.data_addr;
        self.symbols.insert(name, addr);
        self.data_addr += size;
        addr
    }

    fn collect_symbols(&mut self, program: &Program) {
        self.data_addr = 0;

        for stmt in &program.statements {
            if let Statement::Let(name, typ, expr) = stmt {
                match typ {
                    Type::I32 => {
                        let val = eval_i32_expr(expr).unwrap_or(0);
                        self.add_variable(name.clone(), 1);
                        self.data_cmds.push(Command::Word(vec![val]));
                    }
                    Type::StringType => {
                        let content: Vec<i32> = if let Expr::String(s) = expr {
                            s.bytes()
                                .map(|b| b as i32)
                                .chain(std::iter::once(0))
                                .collect()
                        } else {
                            vec![0]
                        };
                        let size = content.len() as u32;
                        self.add_variable(name.clone(), size);
                        self.data_cmds.push(Command::Word(content));
                    }
                    Type::Array(_, size) => {
                        let vals: Vec<i32> = if let Expr::ArrayInit(elems) = expr {
                            elems
                                .iter()
                                .map(|e| eval_i32_expr(e).unwrap_or(0))
                                .collect()
                        } else {
                            vec![0; *size]
                        };
                        self.add_variable(name.clone(), *size as u32);
                        self.data_cmds.push(Command::Word(vals));
                    }
                }
            }
        }

        for func in &program.functions {
            if func.return_type.is_some() {
                let ret_name = format!("{}_ret", func.name);
                self.add_variable(ret_name, 1);
                self.data_cmds.push(Command::Word(vec![0]));
            }

            let mut param_names = Vec::new();
            for param in &func.params {
                let mangled = mangle(&param.name, &func.name);
                self.add_variable(mangled, 1);
                self.data_cmds.push(Command::Word(vec![0]));
                param_names.push(param.name.clone());
            }
            self.func_params.insert(func.name.clone(), param_names);

            self.collect_locals_in_block(&func.body, &func.name);
        }

        self.add_variable("__tmp_val".to_string(), 1);
        self.data_cmds.push(Command::Word(vec![0]));

        for i in 0..TEMP_COUNT {
            let name = format!("__tmp_{}", i);
            self.add_variable(name, 1);
            self.data_cmds.push(Command::Word(vec![0]));
        }
        for i in 0..TEMP_IDX_COUNT {
            let name = format!("__tmp_idx_{}", i);
            self.add_variable(name, 1);
            self.data_cmds.push(Command::Word(vec![0]));
        }
        for i in 0..TEMP_PTR_COUNT {
            let name = format!("__tmp_ptr_{}", i);
            self.add_variable(name, 1);
            self.data_cmds.push(Command::Word(vec![0]));
        }
    }

    fn collect_locals_in_block(&mut self, block: &Block, func: &str) {
        for stmt in &block.statements {
            match stmt {
                Statement::Let(name, typ, expr) => {
                    let mangled = mangle(name, func);
                    if self.symbols.contains_key(&mangled) {
                        continue;
                    }
                    let size = match typ {
                        Type::I32 => 1,
                        Type::StringType => {
                            if let Expr::String(s) = expr {
                                (s.len() + 1) as u32
                            } else {
                                1
                            }
                        }
                        Type::Array(_, sz) => *sz as u32,
                    };
                    self.add_variable(mangled, size);
                    self.data_cmds.push(Command::Word(vec![0; size as usize]));
                }
                Statement::If(_, then_block, else_block) => {
                    self.collect_locals_in_block(then_block, func);
                    if let Some(eb) = else_block {
                        self.collect_locals_in_block(eb, func);
                    }
                }
                Statement::While(_, body) => {
                    self.collect_locals_in_block(body, func);
                }
                _ => {}
            }
        }
    }

    fn compile_function(&mut self, func: &FunctionDecl) {
        self.code_cmds.push(Command::Label(func.name.clone()));

        self.compile_block(&func.body, &func.name, 0);

        if func.name == "main" {
            self.code_cmds.push(Command::Halt);
        } else {
            self.code_cmds.push(Command::Ret);
        }
    }

    fn compile_block(&mut self, block: &Block, func: &str, depth: usize) {
        for stmt in &block.statements {
            self.compile_statement(stmt, func, depth);
        }
    }

    fn compile_statement(&mut self, stmt: &Statement, func: &str, depth: usize) {
        match stmt {
            Statement::Let(name, typ, expr) => {
                self.compile_let_stmt(name, typ, expr, func, depth);
            }
            Statement::Assign(name, index, expr) => {
                self.compile_assign(name, index.as_deref(), expr, func, depth);
            }
            Statement::If(cond, then_block, else_block) => {
                self.compile_if(cond, then_block, else_block.as_ref(), func, depth);
            }
            Statement::While(cond, body) => {
                self.compile_while(cond, body, func, depth);
            }
            Statement::Return(expr_opt) => {
                self.compile_return(expr_opt.as_ref(), func, depth);
            }
            Statement::Expr(expr) => {
                self.compile_expr(expr, func, depth);
            }
        }
    }

    fn compile_let_stmt(&mut self, name: &str, typ: &Type, expr: &Expr, func: &str, depth: usize) {
        let mangled = mangle(name, func);

        match (typ, expr) {
            (Type::Array(_, _), Expr::ArrayInit(elems)) => {
                let base_addr = self.symbols[&mangled];
                for (i, elem) in elems.iter().enumerate() {
                    self.compile_expr(elem, func, depth);
                    self.code_cmds
                        .push(Command::St(Operand::Addr(base_addr + i as u32)));
                }
            }
            (Type::StringType, Expr::String(s)) => {
                let base_addr = self.symbols[&mangled];
                for (i, byte) in s.bytes().chain(std::iter::once(0)).enumerate() {
                    self.code_cmds
                        .push(Command::Ld(Operand::Immediate(byte as i32)));
                    self.code_cmds
                        .push(Command::St(Operand::Addr(base_addr + i as u32)));
                }
            }
            _ => {
                self.compile_expr(expr, func, depth);
                self.code_cmds.push(Command::St(Operand::Named(mangled)));
            }
        }
    }

    fn compile_assign(
        &mut self,
        name: &str,
        index: Option<&Expr>,
        expr: &Expr,
        func: &str,
        depth: usize,
    ) {
        let mangled = self.resolve_name(name, func);

        if let Some(idx) = index {
            let base_addr = self.symbols[&mangled];
            let val_tmp = "__tmp_val".to_string();
            let idx_tmp = format!("__tmp_idx_{}", depth);
            let ptr_tmp = format!("__tmp_ptr_{}", depth);

            self.compile_expr(expr, func, depth);
            self.code_cmds
                .push(Command::St(Operand::Named(val_tmp.clone())));

            self.compile_expr(idx, func, depth + 1);
            self.code_cmds
                .push(Command::St(Operand::Named(idx_tmp.clone())));
            self.code_cmds
                .push(Command::Ld(Operand::Immediate(base_addr as i32)));
            self.code_cmds.push(Command::Add(Operand::Named(idx_tmp)));
            self.code_cmds
                .push(Command::St(Operand::Named(ptr_tmp.clone())));

            self.code_cmds.push(Command::Ld(Operand::Named(val_tmp)));
            self.code_cmds.push(Command::Sti(Operand::Named(ptr_tmp)));
        } else {
            let mangled = self.resolve_name(name, func);
            self.compile_expr(expr, func, depth);
            self.code_cmds.push(Command::St(Operand::Named(mangled)));
        }
    }

    fn compile_if(
        &mut self,
        cond: &Expr,
        then_block: &Block,
        else_block: Option<&Block>,
        func: &str,
        depth: usize,
    ) {
        let else_label = self.new_label();
        let end_label = self.new_label();

        self.compile_expr(cond, func, depth);
        self.code_cmds
            .push(Command::Jzs(Operand::Named(else_label.clone())));

        self.compile_block(then_block, func, depth);
        self.code_cmds
            .push(Command::Jmp(Operand::Named(end_label.clone())));

        self.code_cmds.push(Command::Label(else_label));
        if let Some(eb) = else_block {
            self.compile_block(eb, func, depth);
        }

        self.code_cmds.push(Command::Label(end_label));
    }

    fn compile_while(&mut self, cond: &Expr, body: &Block, func: &str, depth: usize) {
        let loop_label = self.new_label();
        let end_label = self.new_label();

        self.code_cmds.push(Command::Label(loop_label.clone()));

        self.compile_expr(cond, func, depth);
        self.code_cmds
            .push(Command::Jzs(Operand::Named(end_label.clone())));

        self.compile_block(body, func, depth);
        self.code_cmds
            .push(Command::Jmp(Operand::Named(loop_label)));

        self.code_cmds.push(Command::Label(end_label));
    }

    fn compile_return(&mut self, expr_opt: Option<&Expr>, func: &str, depth: usize) {
        if let Some(expr) = expr_opt {
            self.compile_expr(expr, func, depth);
            let ret_name = format!("{}_ret", func);
            self.code_cmds.push(Command::St(Operand::Named(ret_name)));
        }
        self.code_cmds.push(Command::Ret);
    }

    fn compile_expr(&mut self, expr: &Expr, func: &str, depth: usize) {
        match expr {
            Expr::Number(n) => {
                self.code_cmds.push(Command::Ld(Operand::Immediate(*n)));
            }
            Expr::String(_) => {
                panic!("String literal in expression context is not allowed");
            }
            Expr::Variable(name) => {
                let mangled = self.resolve_name(name, func);
                self.code_cmds.push(Command::Ld(Operand::Named(mangled)));
            }
            Expr::UnaryOp(op, inner) => {
                self.compile_unary_op(op, inner, func, depth);
            }
            Expr::BinaryOp(left, op, right) => {
                self.compile_binary_op(left, op, right, func, depth);
            }
            Expr::FunctionCall(name, args) => {
                self.compile_function_call(name, args, func, depth);
            }
            Expr::ArrayInit(elems) => {
                if let Some(first) = elems.first() {
                    self.compile_expr(first, func, depth);
                }
            }
            Expr::ArrayAccess(name, index) => {
                self.compile_array_access(name, index, func, depth);
            }
            Expr::Grouping(inner) => {
                self.compile_expr(inner, func, depth);
            }
        }
    }

    fn compile_unary_op(&mut self, op: &UnaryOp, inner: &Expr, func: &str, depth: usize) {
        self.compile_expr(inner, func, depth + 1);

        match op {
            UnaryOp::Neg => {
                self.code_cmds.push(Command::Neg);
            }
            UnaryOp::Not => {
                let tmp = format!("__tmp_{}", depth);
                self.code_cmds
                    .push(Command::St(Operand::Named(tmp.clone())));
                self.code_cmds.push(Command::Ld(Operand::Immediate(0)));
                self.code_cmds.push(Command::Cmp(Operand::Named(tmp)));

                let true_label = self.new_label();
                let end_label = self.new_label();
                self.code_cmds
                    .push(Command::Jzs(Operand::Named(true_label.clone())));
                self.code_cmds.push(Command::Ld(Operand::Immediate(0)));
                self.code_cmds
                    .push(Command::Jmp(Operand::Named(end_label.clone())));
                self.code_cmds.push(Command::Label(true_label));
                self.code_cmds.push(Command::Ld(Operand::Immediate(1)));
                self.code_cmds.push(Command::Label(end_label));
            }
            UnaryOp::BitNot => {
                self.code_cmds.push(Command::Not);
            }
        }
    }

    fn compile_binary_op(
        &mut self,
        left: &Expr,
        op: &BinaryOp,
        right: &Expr,
        func: &str,
        depth: usize,
    ) {
        let tmp = format!("__tmp_{}", depth);

        match op {
            BinaryOp::And => {
                self.compile_expr(left, func, depth);
                let false_label = self.new_label();
                let end_label = self.new_label();
                self.code_cmds.push(Command::Cmp(Operand::Immediate(0)));
                self.code_cmds
                    .push(Command::Jzs(Operand::Named(false_label.clone())));
                self.compile_expr(right, func, depth);
                self.code_cmds.push(Command::Cmp(Operand::Immediate(0)));
                self.code_cmds
                    .push(Command::Jzs(Operand::Named(false_label.clone())));
                self.code_cmds.push(Command::Ld(Operand::Immediate(1)));
                self.code_cmds
                    .push(Command::Jmp(Operand::Named(end_label.clone())));
                self.code_cmds.push(Command::Label(false_label));
                self.code_cmds.push(Command::Ld(Operand::Immediate(0)));
                self.code_cmds.push(Command::Label(end_label));
            }
            BinaryOp::Or => {
                self.compile_expr(left, func, depth);
                let true_label = self.new_label();
                let end_label = self.new_label();
                self.code_cmds.push(Command::Cmp(Operand::Immediate(0)));
                self.code_cmds
                    .push(Command::Jzc(Operand::Named(true_label.clone())));
                self.compile_expr(right, func, depth);
                self.code_cmds.push(Command::Cmp(Operand::Immediate(0)));
                self.code_cmds
                    .push(Command::Jzc(Operand::Named(true_label.clone())));
                self.code_cmds.push(Command::Ld(Operand::Immediate(0)));
                self.code_cmds
                    .push(Command::Jmp(Operand::Named(end_label.clone())));
                self.code_cmds.push(Command::Label(true_label));
                self.code_cmds.push(Command::Ld(Operand::Immediate(1)));
                self.code_cmds.push(Command::Label(end_label));
            }
            _ => {
                self.compile_expr(right, func, depth + 1);
                self.code_cmds
                    .push(Command::St(Operand::Named(tmp.clone())));
                self.compile_expr(left, func, depth + 1);

                match op {
                    BinaryOp::Add => {
                        self.code_cmds.push(Command::Add(Operand::Named(tmp)));
                    }
                    BinaryOp::Sub => {
                        self.code_cmds.push(Command::Sub(Operand::Named(tmp)));
                    }
                    BinaryOp::Mul => {
                        self.code_cmds.push(Command::Mul(Operand::Named(tmp)));
                    }
                    BinaryOp::Div => {
                        self.code_cmds.push(Command::Div(Operand::Named(tmp)));
                    }
                    BinaryOp::Rem => {
                        self.code_cmds.push(Command::Mod(Operand::Named(tmp)));
                    }
                    BinaryOp::BitAnd => {
                        self.code_cmds.push(Command::And(Operand::Named(tmp)));
                    }
                    BinaryOp::BitOr => {
                        self.code_cmds.push(Command::Or(Operand::Named(tmp)));
                    }
                    BinaryOp::Xor => {
                        self.code_cmds.push(Command::Xor(Operand::Named(tmp)));
                    }
                    BinaryOp::Eq => self.compile_cmp_result(op, &tmp),
                    BinaryOp::NotEq => self.compile_cmp_result(op, &tmp),
                    BinaryOp::Less => self.compile_cmp_result(op, &tmp),
                    BinaryOp::LessEq => self.compile_cmp_result(op, &tmp),
                    BinaryOp::Greater => self.compile_cmp_result(op, &tmp),
                    BinaryOp::GreaterEq => self.compile_cmp_result(op, &tmp),
                    _ => unreachable!(),
                }
            }
        }
    }

    fn compile_cmp_result(&mut self, op: &BinaryOp, tmp: &str) {
        self.code_cmds
            .push(Command::Cmp(Operand::Named(tmp.to_string())));

        let true_label = self.new_label();
        let end_label = self.new_label();

        let (use_jz, use_jn, use_jp) = match op {
            BinaryOp::Eq => (true, false, false),
            BinaryOp::NotEq => (false, false, false),
            BinaryOp::Less => (false, true, false),
            BinaryOp::LessEq => (true, true, false),
            BinaryOp::Greater => (false, false, true),
            BinaryOp::GreaterEq => (false, false, false),
            _ => unreachable!(),
        };

        if use_jz {
            self.code_cmds
                .push(Command::Jzs(Operand::Named(true_label.clone())));
        }
        if use_jn {
            self.code_cmds
                .push(Command::Jns(Operand::Named(true_label.clone())));
        }
        if use_jp {
            self.code_cmds
                .push(Command::Jnc(Operand::Named(true_label.clone())));
        }

        match op {
            BinaryOp::Eq | BinaryOp::Less | BinaryOp::Greater => {
                self.code_cmds.push(Command::Ld(Operand::Immediate(0)));
                self.code_cmds
                    .push(Command::Jmp(Operand::Named(end_label.clone())));
                self.code_cmds.push(Command::Label(true_label));
                self.code_cmds.push(Command::Ld(Operand::Immediate(1)));
                self.code_cmds.push(Command::Label(end_label));
            }
            BinaryOp::NotEq => {
                self.code_cmds
                    .push(Command::Jzs(Operand::Named(true_label.clone())));
                self.code_cmds.push(Command::Ld(Operand::Immediate(1)));
                self.code_cmds
                    .push(Command::Jmp(Operand::Named(end_label.clone())));
                self.code_cmds.push(Command::Label(true_label));
                self.code_cmds.push(Command::Ld(Operand::Immediate(0)));
                self.code_cmds.push(Command::Label(end_label));
            }
            BinaryOp::LessEq => {
                self.code_cmds.push(Command::Ld(Operand::Immediate(0)));
                self.code_cmds
                    .push(Command::Jmp(Operand::Named(end_label.clone())));
                self.code_cmds.push(Command::Label(true_label));
                self.code_cmds.push(Command::Ld(Operand::Immediate(1)));
                self.code_cmds.push(Command::Label(end_label));
            }
            BinaryOp::GreaterEq => {
                self.code_cmds
                    .push(Command::Jns(Operand::Named(true_label.clone())));
                self.code_cmds.push(Command::Ld(Operand::Immediate(1)));
                self.code_cmds
                    .push(Command::Jmp(Operand::Named(end_label.clone())));
                self.code_cmds.push(Command::Label(true_label));
                self.code_cmds.push(Command::Ld(Operand::Immediate(0)));
                self.code_cmds.push(Command::Label(end_label));
            }
            _ => unreachable!(),
        }
    }

    fn resolve_name(&self, name: &str, func: &str) -> String {
        let mangled = mangle(name, func);
        if self.symbols.contains_key(&mangled) {
            mangled
        } else {
            name.to_string()
        }
    }

    fn compile_function_call(&mut self, name: &str, args: &[Expr], func: &str, depth: usize) {
        match name {
            "out" => {
                if let Some(arg) = args.first() {
                    self.compile_expr(arg, func, depth);
                    self.code_cmds.push(Command::St(Operand::Addr(0x1FFFFFF)));
                }
            }
            "in" => {
                self.code_cmds.push(Command::Ld(Operand::Addr(0x1FFFFFE)));
            }
            _ => {
                let param_names: Vec<String> =
                    self.func_params.get(name).cloned().unwrap_or_default();
                for (i, arg) in args.iter().enumerate() {
                    self.compile_expr(arg, func, depth);
                    let mangled = mangle(&param_names[i], name);
                    self.code_cmds.push(Command::St(Operand::Named(mangled)));
                }

                self.code_cmds
                    .push(Command::Call(Operand::Named(name.to_string())));

                let ret_name = format!("{}_ret", name);
                self.code_cmds.push(Command::Ld(Operand::Named(ret_name)));
            }
        }
    }

    fn compile_array_access(&mut self, name: &str, index: &Expr, func: &str, depth: usize) {
        let mangled = self.resolve_name(name, func);
        let base_addr = self.symbols[&mangled];
        let idx_tmp = format!("__tmp_idx_{}", depth);
        let ptr_tmp = format!("__tmp_ptr_{}", depth);

        self.compile_expr(index, func, depth + 1);
        self.code_cmds
            .push(Command::St(Operand::Named(idx_tmp.clone())));
        self.code_cmds
            .push(Command::Ld(Operand::Immediate(base_addr as i32)));
        self.code_cmds.push(Command::Add(Operand::Named(idx_tmp)));
        self.code_cmds
            .push(Command::St(Operand::Named(ptr_tmp.clone())));

        self.code_cmds.push(Command::Ldi(Operand::Named(ptr_tmp)));
    }

    fn resolve_and_emit(&mut self) {
        let data_size = self.data_addr;

        let mut resolved: HashMap<String, u32> = self.symbols.clone();

        let mut addr = data_size;
        for cmd in &self.code_cmds {
            if let Command::Label(name) = cmd {
                resolved.entry(name.clone()).or_insert(addr);
            }
            match cmd {
                Command::Label(_) => {}
                Command::Word(v) => addr += v.len() as u32,
                _ => addr += 1,
            }
        }

        self.text.push_str("\t.data\n");
        let mut addr = 0u32;
        let rev_syms: HashMap<u32, &str> =
            self.symbols.iter().map(|(k, &v)| (v, k.as_str())).collect();

        for cmd in &self.data_cmds {
            if let Command::Word(values) = cmd {
                let name = rev_syms.get(&addr).copied().unwrap_or("__data");
                self.text.push_str(&format!("{}: word ", name));
                let hex_vals: Vec<String> = values.iter().map(|v| format!("0x{:X}", v)).collect();
                self.text.push_str(&hex_vals.join(", "));
                self.text.push('\n');

                for &v in values {
                    self.binary.extend_from_slice(&(v as u32).to_le_bytes());
                }
                addr += values.len() as u32;
            }
        }

        self.text.push_str("\n\t.code\n");

        let cmds = self.code_cmds.clone();
        for cmd in &cmds {
            match cmd {
                Command::Label(name) => {
                    self.text.push_str(&format!("{}:\n", name));
                }
                Command::Word(values) => {
                    for &v in values {
                        self.binary.extend_from_slice(&(v as u32).to_le_bytes());
                    }
                }
                _ => {
                    self.emit_text_command(cmd);
                    self.emit_binary_command(cmd, &resolved);
                }
            }
        }
    }

    fn emit_text_command(&mut self, cmd: &Command) {
        let mnemonic = |op: &str, operand: &Operand| match operand {
            Operand::Immediate(v) => {
                format!("\t{} #{}\n", op, v)
            }
            Operand::Named(n) => format!("\t{} {}\n", op, n),
            Operand::Addr(a) => format!("\t{} 0x{:X}\n", op, a),
        };

        let line = match cmd {
            Command::Ld(op) => mnemonic("LD", op),
            Command::Ldi(op) => format!("\tLD [{}]\n", operand_name(op)),
            Command::St(op) => format!("\tST {}\n", operand_name(op)),
            Command::Sti(op) => format!("\tST [{}]\n", operand_name(op)),
            Command::Add(op) => mnemonic("ADD", op),
            Command::Sub(op) => mnemonic("SUB", op),
            Command::Mul(op) => mnemonic("MUL", op),
            Command::Div(op) => mnemonic("DIV", op),
            Command::Mod(op) => mnemonic("MOD", op),
            Command::Cmp(op) => mnemonic("CMP", op),
            Command::Neg => "\tNEG\n".to_string(),
            Command::Not => "\tNOT\n".to_string(),
            Command::And(op) => mnemonic("AND", op),
            Command::Or(op) => mnemonic("OR", op),
            Command::Xor(op) => mnemonic("XOR", op),
            Command::Jmp(op) => format!("\tJMP {}\n", operand_name(op)),
            Command::Jzs(op) => format!("\tJZS {}\n", operand_name(op)),
            Command::Jzc(op) => format!("\tJZC {}\n", operand_name(op)),
            Command::Jns(op) => format!("\tJNS {}\n", operand_name(op)),
            Command::Jnc(op) => format!("\tJNC {}\n", operand_name(op)),
            Command::Call(op) => format!("\tCALL {}\n", operand_name(op)),
            Command::Ret => "\tRET\n".to_string(),
            Command::Halt => "\tHALT\n".to_string(),
            _ => String::new(),
        };
        self.text.push_str(&line);
    }

    fn emit_binary_command(&mut self, cmd: &Command, resolved: &HashMap<String, u32>) {
        let (opcode, mode, operand) = match cmd {
            Command::Ld(Operand::Immediate(v)) => (OP_LD, 0b01, *v as u32),
            Command::Ld(op) => (OP_LD, 0b00, resolve_addr(op, resolved)),
            Command::Ldi(op) => (OP_LD, 0b10, resolve_addr(op, resolved)),
            Command::St(op) => (OP_ST, 0b00, resolve_addr(op, resolved)),
            Command::Sti(op) => (OP_ST, 0b10, resolve_addr(op, resolved)),
            Command::Add(Operand::Immediate(v)) => (OP_ADD, 0b01, *v as u32),
            Command::Add(op) => (OP_ADD, 0b00, resolve_addr(op, resolved)),
            Command::Sub(Operand::Immediate(v)) => (OP_SUB, 0b01, *v as u32),
            Command::Sub(op) => (OP_SUB, 0b00, resolve_addr(op, resolved)),
            Command::Mul(Operand::Immediate(v)) => (OP_MUL, 0b01, *v as u32),
            Command::Mul(op) => (OP_MUL, 0b00, resolve_addr(op, resolved)),
            Command::Div(Operand::Immediate(v)) => (OP_DIV, 0b01, *v as u32),
            Command::Div(op) => (OP_DIV, 0b00, resolve_addr(op, resolved)),
            Command::Mod(Operand::Immediate(v)) => (OP_MOD, 0b01, *v as u32),
            Command::Mod(op) => (OP_MOD, 0b00, resolve_addr(op, resolved)),
            Command::Cmp(Operand::Immediate(v)) => (OP_CMP, 0b01, *v as u32),
            Command::Cmp(op) => (OP_CMP, 0b00, resolve_addr(op, resolved)),
            Command::And(Operand::Immediate(v)) => (OP_AND, 0b01, *v as u32),
            Command::And(op) => (OP_AND, 0b00, resolve_addr(op, resolved)),
            Command::Or(Operand::Immediate(v)) => (OP_OR, 0b01, *v as u32),
            Command::Or(op) => (OP_OR, 0b00, resolve_addr(op, resolved)),
            Command::Xor(Operand::Immediate(v)) => (OP_XOR, 0b01, *v as u32),
            Command::Xor(op) => (OP_XOR, 0b00, resolve_addr(op, resolved)),
            Command::Neg => (OP_NEG, 0b11, 0),
            Command::Not => (OP_NOT, 0b11, 0),
            Command::Jmp(op) => (OP_JMP, 0b00, resolve_addr(op, resolved)),
            Command::Jzs(op) => (OP_JZS, 0b00, resolve_addr(op, resolved)),
            Command::Jzc(op) => (OP_JZC, 0b00, resolve_addr(op, resolved)),
            Command::Jns(op) => (OP_JNS, 0b00, resolve_addr(op, resolved)),
            Command::Jnc(op) => (OP_JNC, 0b00, resolve_addr(op, resolved)),
            Command::Call(op) => (OP_CALL, 0b00, resolve_addr(op, resolved)),
            Command::Ret => (OP_RET, 0b11, 0),
            Command::Halt => (OP_HALT, 0b11, 0),
            _ => return,
        };

        let instr = (opcode as u32) << 27 | (mode as u32) << 25 | (operand & 0x1FFFFFF);
        self.binary.extend_from_slice(&instr.to_le_bytes());
    }

    fn new_label(&mut self) -> String {
        let label = format!(".L{}", self.label_counter);
        self.label_counter += 1;
        label
    }
}

fn mangle(name: &str, func: &str) -> String {
    format!("{}_{}", name, func)
}

fn operand_name(op: &Operand) -> String {
    match op {
        Operand::Named(n) => n.clone(),
        Operand::Immediate(v) => format!("#{}", v),
        Operand::Addr(a) => format!("0x{:X}", a),
    }
}

fn resolve_addr(op: &Operand, resolved: &HashMap<String, u32>) -> u32 {
    match op {
        Operand::Named(n) => resolved.get(n).copied().unwrap_or(0),
        Operand::Immediate(v) => *v as u32,
        Operand::Addr(a) => *a,
    }
}

fn eval_i32_expr(expr: &Expr) -> Option<i32> {
    match expr {
        Expr::Number(n) => Some(*n),
        Expr::UnaryOp(UnaryOp::Neg, inner) => eval_i32_expr(inner).map(|v| -v),
        Expr::UnaryOp(UnaryOp::Not, inner) => eval_i32_expr(inner).map(|v| !v),
        Expr::UnaryOp(UnaryOp::BitNot, inner) => eval_i32_expr(inner).map(|v| !v),
        Expr::BinaryOp(left, op, right) => {
            let l = eval_i32_expr(left)?;
            let r = eval_i32_expr(right)?;
            match op {
                BinaryOp::Add => Some(l + r),
                BinaryOp::Sub => Some(l - r),
                BinaryOp::Mul => Some(l * r),
                BinaryOp::Div => Some(l / r),
                BinaryOp::Rem => Some(l % r),
                BinaryOp::Eq => Some((l == r) as i32),
                BinaryOp::NotEq => Some((l != r) as i32),
                BinaryOp::Less => Some((l < r) as i32),
                BinaryOp::LessEq => Some((l <= r) as i32),
                BinaryOp::Greater => Some((l > r) as i32),
                BinaryOp::GreaterEq => Some((l >= r) as i32),
                BinaryOp::And => Some((l != 0 && r != 0) as i32),
                BinaryOp::Or => Some((l != 0 || r != 0) as i32),
                BinaryOp::BitAnd => Some(l & r),
                BinaryOp::BitOr => Some(l | r),
                BinaryOp::Xor => Some(l ^ r),
            }
        }
        _ => None,
    }
}
