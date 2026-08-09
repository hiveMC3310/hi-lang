//! Parser for the Hi language, produces AST.

pub mod lexer;

use crate::ast::{BinOp, Block, Expr, Program, Span, Stmt, UnOp};
use crate::error::{ParseError, ParseResult};
use crate::parser::lexer::{Token, TokenKind};
use hi_common::Symbol;

pub struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Returns the current token (without advancing).
    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    /// Checks if the current token is of the expected kind.
    fn peek_if(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    /// Checks if the current token is Ident.
    fn peek_if_ident(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Ident(_))
    }

    fn peek_compound_assign_op(&self) -> Option<BinOp> {
        match self.peek().kind {
            TokenKind::PlusAssign => Some(BinOp::Add),
            TokenKind::MinusAssign => Some(BinOp::Sub),
            TokenKind::StarAssign => Some(BinOp::Mul),
            TokenKind::SlashAssign => Some(BinOp::Div),
            TokenKind::PercentAssign => Some(BinOp::Mod),
            TokenKind::CaretAssign => Some(BinOp::Pow),
            _ => None,
        }
    }

    fn is_expr_start(kind: &TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Int(_)
                | TokenKind::Float(_)
                | TokenKind::String(_)
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Ident(_)
                | TokenKind::LParen
                | TokenKind::LBracket
                | TokenKind::LBrace
                | TokenKind::Not
                | TokenKind::Minus
                | TokenKind::Plus
        )
    }

    /// Consumes a token of the expected kind, advances position.
    fn consume(&mut self, expected: TokenKind) -> ParseResult<Token> {
        let tok = self.peek().clone();
        if tok.kind == expected {
            self.pos += 1;
            Ok(tok)
        } else {
            Err(ParseError {
                message: format!("Expected {:?}, got {:?}", expected, tok.kind),
                span: tok.span,
            })
        }
    }

    /// Consumes a string literal and returns its content.
    fn consume_string(&mut self) -> ParseResult<String> {
        let tok = self.peek().clone();
        match &tok.kind {
            TokenKind::String(s) => {
                self.pos += 1;
                Ok(s.clone())
            }
            _ => Err(ParseError {
                message: format!("Expected string literal, got {:?}", tok.kind),
                span: tok.span,
            }),
        }
    }

    /// Consumes an identifier and returns its name and span.
    fn consume_ident(&mut self) -> ParseResult<(Symbol, Span)> {
        let tok = self.peek().clone();
        match &tok.kind {
            TokenKind::Ident(sym) => {
                self.pos += 1;
                Ok((*sym, tok.span))
            }
            _ => Err(ParseError {
                message: format!("Expected identifier, got {:?}", tok.kind),
                span: tok.span,
            }),
        }
    }

    // ---- Entry point ----
    pub fn parse(&mut self) -> ParseResult<Program> {
        let mut stmts = Vec::new();
        while !self.peek_if(TokenKind::Eof) {
            stmts.push(self.parse_statement()?);
        }
        Ok(Program { stmts })
    }

    // ---- Parsing statements ----
    fn parse_statement(&mut self) -> ParseResult<Stmt> {
        let tok = self.peek();
        match &tok.kind {
            TokenKind::Let => self.parse_let(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::Func => self.parse_func(),
            TokenKind::Ret => self.parse_return(),
            TokenKind::Break => self.parse_break(),
            TokenKind::Print => self.parse_print(),
            TokenKind::Input => self.parse_input(),
            TokenKind::Import => self.parse_import(),
            TokenKind::For => self.parse_for(),
            _ => {
                let left = self.parse_expression()?;

                if let Some(op) = self.peek_compound_assign_op() {
                    self.pos += 1;
                    let right = self.parse_expression()?;
                    let span = left.span().merge(&right.span());
                    match left {
                        Expr::Variable(_, _) | Expr::Index(_, _, _) => Ok(Stmt::CompoundAssign(
                            Box::new(left),
                            op,
                            Box::new(right),
                            span,
                        )),
                        _ => Err(ParseError {
                            span: left.span(),
                            message: "Invalid left-hand side for compound assignment".to_string(),
                        }),
                    }
                } else if self.peek_if(TokenKind::Assign) {
                    self.consume(TokenKind::Assign)?;
                    let right = self.parse_expression()?;
                    let span = left.span().merge(&right.span());
                    match left {
                        Expr::Variable(_, _) | Expr::Index(_, _, _) => {
                            Ok(Stmt::Assign(Box::new(left), Box::new(right), span))
                        }
                        _ => Err(ParseError {
                            span: left.span(),
                            message: "Invalid left-hand side of assignment".to_string(),
                        }),
                    }
                } else {
                    let span = left.span();
                    Ok(Stmt::Expr(left, span))
                }
            }
        }
    }

    // ---- LET name = expr ----
    fn parse_let(&mut self) -> ParseResult<Stmt> {
        let start = self.peek().span;
        self.consume(TokenKind::Let)?;
        let (name, name_span) = self.consume_ident()?;
        self.consume(TokenKind::Assign)?;
        let expr = self.parse_expression()?;
        let full_span = start.merge(&expr.span());
        Ok(Stmt::Let(name, expr, name_span, full_span))
    }

    // ---- INPUT [prompt] var ----
    fn parse_input(&mut self) -> ParseResult<Stmt> {
        let start = self.peek().span;
        self.consume(TokenKind::Input)?;
        let (prompt, var) = if self.peek_if_ident() {
            let (var, _) = self.consume_ident()?;
            (None, var)
        } else {
            let prompt = self.consume_string()?;
            let (var, _) = self.consume_ident()?;
            (Some(prompt), var)
        };
        let span = start.merge(&self.tokens[self.pos - 1].span);
        Ok(Stmt::Input(prompt, var, span))
    }

    // ---- IF cond THEN block ELSE block END ----
    fn parse_if(&mut self) -> ParseResult<Stmt> {
        let start = self.peek().span;
        self.consume(TokenKind::If)?;
        let cond = self.parse_expression()?;
        self.consume(TokenKind::Then)?;
        let then_block = self.parse_block(&[TokenKind::End, TokenKind::Else])?;
        let else_block = if self.peek_if(TokenKind::Else) {
            self.consume(TokenKind::Else)?;
            Some(self.parse_block(&[TokenKind::End])?)
        } else {
            None
        };
        self.consume(TokenKind::End)?;
        let span = start.merge(&self.tokens[self.pos - 1].span);
        Ok(Stmt::If(cond, then_block, else_block, span))
    }

    // ---- WHILE cond DO block END ----
    fn parse_while(&mut self) -> ParseResult<Stmt> {
        let start = self.peek().span;
        self.consume(TokenKind::While)?;
        let cond = self.parse_expression()?;
        self.consume(TokenKind::Do)?;
        let block = self.parse_block(&[TokenKind::End])?;
        self.consume(TokenKind::End)?;
        let span = start.merge(&self.tokens[self.pos - 1].span);
        Ok(Stmt::While(cond, block, span))
    }

    // ---- FOR var = start TO end DO block NEXT [step] ----
    fn parse_for(&mut self) -> ParseResult<Stmt> {
        let start = self.peek().span;
        self.consume(TokenKind::For)?;
        let (var, var_span) = self.consume_ident()?;
        self.consume(TokenKind::Assign)?;
        let start_expr = self.parse_expression()?;
        self.consume(TokenKind::To)?;
        let end_expr = self.parse_expression()?;
        self.consume(TokenKind::Do)?;
        let body = self.parse_block(&[TokenKind::Next])?;
        self.consume(TokenKind::Next)?;
        // optional step
        let step = if Self::is_expr_start(&self.peek().kind) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        let full_span = start.merge(&self.tokens[self.pos - 1].span);
        Ok(Stmt::For(
            var,
            Box::new(start_expr),
            Box::new(end_expr),
            step,
            body,
            var_span,
            full_span,
        ))
    }

    // ---- FUNC name(params) block END ----
    fn parse_func(&mut self) -> ParseResult<Stmt> {
        let start = self.peek().span;
        let doc = self.peek().doc.clone();
        self.consume(TokenKind::Func)?;
        let (name, name_span) = self.consume_ident()?;
        self.consume(TokenKind::LParen)?;
        let mut params = Vec::new();
        if !self.peek_if(TokenKind::RParen) {
            loop {
                let (param, _) = self.consume_ident()?;
                params.push(param);
                if self.peek_if(TokenKind::Comma) {
                    self.consume(TokenKind::Comma)?;
                } else {
                    break;
                }
            }
        }
        self.consume(TokenKind::RParen)?;
        let block = self.parse_block(&[TokenKind::End])?;
        self.consume(TokenKind::End)?;
        let full_span = start.merge(&self.tokens[self.pos - 1].span);
        Ok(Stmt::Func(name, params, block, doc, name_span, full_span))
    }

    // ---- RETURN [expr] ----
    fn parse_return(&mut self) -> ParseResult<Stmt> {
        let start = self.peek().span;
        self.consume(TokenKind::Ret)?;
        let expr = if self.peek_if(TokenKind::Eof) || self.peek_if(TokenKind::End) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        let span = if let Some(ref e) = expr {
            start.merge(&e.span())
        } else {
            start
        };
        Ok(Stmt::Return(expr, span))
    }

    // ---- BREAK ----
    fn parse_break(&mut self) -> ParseResult<Stmt> {
        let start = self.peek().span;
        self.consume(TokenKind::Break)?;
        Ok(Stmt::Break(start))
    }

    // ---- PRINT expr, expr, ... ----
    fn parse_print(&mut self) -> ParseResult<Stmt> {
        let start = self.peek().span;
        self.consume(TokenKind::Print)?;
        let mut args = Vec::new();
        if !self.peek_if(TokenKind::Eof) && !self.peek_if(TokenKind::End) {
            loop {
                args.push(self.parse_expression()?);
                if self.peek_if(TokenKind::Comma) {
                    self.consume(TokenKind::Comma)?;
                } else {
                    break;
                }
            }
        }
        let span = start.merge(&self.tokens[self.pos - 1].span);
        Ok(Stmt::Print(args, span))
    }

    fn parse_import(&mut self) -> ParseResult<Stmt> {
        let start = self.peek().span;
        self.consume(TokenKind::Import)?;
        let path = self.consume_string()?;
        let alias = if self.peek_if(TokenKind::As) {
            self.consume(TokenKind::As)?;
            let ident = self.consume_ident()?;
            Some(ident.0)
        } else {
            None
        };
        let span = start.merge(&self.tokens[self.pos - 1].span);
        Ok(Stmt::Import(path, alias, span))
    }

    // ---- Block: sequence of statements until END or Eof ----
    fn parse_block(&mut self, stop_kinds: &[TokenKind]) -> ParseResult<Block> {
        let mut stmts = Vec::new();
        while !self.peek_if(TokenKind::Eof) && !stop_kinds.contains(&self.peek().kind) {
            stmts.push(self.parse_statement()?);
        }
        Ok(stmts)
    }

    // ---- Parsing expressions (Pratt parser) ----
    fn parse_expression(&mut self) -> ParseResult<Expr> {
        self.parse_expr_precedence(0)
    }

    // Precedence table (higher number binds tighter)
    fn precedence(&self, kind: &TokenKind) -> i32 {
        match kind {
            TokenKind::Or => 1,
            TokenKind::And => 2,
            TokenKind::EqEq | TokenKind::Neq => 3,
            TokenKind::Gt | TokenKind::Ge | TokenKind::Lt | TokenKind::Le => 4,
            TokenKind::Plus | TokenKind::Minus => 5,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => 6,
            TokenKind::Caret => 7,
            _ => 0,
        }
    }

    // Main recursive descent with precedence
    fn parse_expr_precedence(&mut self, min_prec: i32) -> ParseResult<Expr> {
        let mut left = self.parse_primary()?;

        while let Some(op_kind) = self.peek_binary_op() {
            let prec = self.precedence(&op_kind);
            if prec < min_prec {
                break;
            }
            self.pos += 1; // consume operator
            let right = self.parse_expr_precedence(prec + 1)?;
            let op_span = self.tokens[self.pos - 1].span;
            let op = self.binop_from_token(&op_kind, op_span)?;
            let span = left.span().merge(&right.span());
            left = Expr::Binary(op, Box::new(left), Box::new(right), span);
        }
        Ok(left)
    }

    fn peek_binary_op(&self) -> Option<TokenKind> {
        let kind = &self.peek().kind;
        if self.precedence(kind) > 0 {
            Some(kind.clone())
        } else {
            None
        }
    }

    fn binop_from_token(&self, kind: &TokenKind, span: Span) -> ParseResult<BinOp> {
        match kind {
            TokenKind::Plus => Ok(BinOp::Add),
            TokenKind::Minus => Ok(BinOp::Sub),
            TokenKind::Star => Ok(BinOp::Mul),
            TokenKind::Slash => Ok(BinOp::Div),
            TokenKind::Percent => Ok(BinOp::Mod),
            TokenKind::Caret => Ok(BinOp::Pow),
            TokenKind::EqEq => Ok(BinOp::Eq),
            TokenKind::Neq => Ok(BinOp::Ne),
            TokenKind::Gt => Ok(BinOp::Gt),
            TokenKind::Ge => Ok(BinOp::Ge),
            TokenKind::Lt => Ok(BinOp::Lt),
            TokenKind::Le => Ok(BinOp::Le),
            TokenKind::And => Ok(BinOp::And),
            TokenKind::Or => Ok(BinOp::Or),
            _ => Err(ParseError {
                message: format!("Unexpected operator: {:?}", kind),
                span,
            }),
        }
    }

    // ---- Primary expressions ----
    fn parse_primary(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_atom()?;

        while self.peek_if(TokenKind::LBracket) {
            self.consume(TokenKind::LBracket)?;
            let index = self.parse_expression()?;
            self.consume(TokenKind::RBracket)?;
            let span = expr.span().merge(&index.span());
            expr = Expr::Index(Box::new(expr), Box::new(index), span);
        }
        Ok(expr)
    }

    fn parse_atom(&mut self) -> ParseResult<Expr> {
        let tok = self.peek().clone();
        match &tok.kind {
            TokenKind::Int(i) => {
                self.pos += 1;
                Ok(Expr::Int(*i, tok.span))
            }
            TokenKind::Float(f) => {
                self.pos += 1;
                Ok(Expr::Float(*f, tok.span))
            }
            TokenKind::String(s) => {
                self.pos += 1;
                Ok(Expr::String(s.clone(), tok.span))
            }
            TokenKind::True | TokenKind::False => {
                let val = matches!(tok.kind, TokenKind::True);
                self.pos += 1;
                Ok(Expr::Bool(val, tok.span))
            }
            TokenKind::Ident(sym) => {
                self.pos += 1;
                if self.peek_if(TokenKind::LParen) {
                    self.parse_call(*sym, tok.span)
                } else if self.peek_if(TokenKind::Colon) {
                    self.consume(TokenKind::Colon)?;
                    let func_ident = self.consume_ident()?;
                    let func_sym = func_ident.0;
                    let span = tok.span.merge(&func_ident.1);
                    if self.peek_if(TokenKind::LParen) {
                        self.parse_call_module(*sym, func_sym, span)
                    } else {
                        Ok(Expr::ModuleAccess(*sym, func_sym, span))
                    }
                } else {
                    Ok(Expr::Variable(*sym, tok.span))
                }
            }
            TokenKind::LParen => {
                self.pos += 1;
                let expr = self.parse_expression()?;
                self.consume(TokenKind::RParen)?;
                Ok(expr)
            }
            TokenKind::LBracket => self.parse_list(),
            TokenKind::LBrace => self.parse_dict(),
            TokenKind::Not => {
                self.pos += 1;
                let expr = self.parse_expr_precedence(0)?;
                let span = tok.span.merge(&expr.span());
                Ok(Expr::Unary(UnOp::Not, Box::new(expr), span))
            }
            TokenKind::Minus => {
                self.pos += 1;
                let expr = self.parse_expr_precedence(0)?;
                let span = tok.span.merge(&expr.span());
                Ok(Expr::Unary(UnOp::Neg, Box::new(expr), span))
            }
            TokenKind::Plus => {
                self.pos += 1;
                let expr = self.parse_expr_precedence(0)?;
                Ok(expr)
            }
            _ => Err(ParseError {
                span: tok.span,
                message: format!("Unexpected token in expression: {:?}", tok.kind),
            }),
        }
    }

    // ---- Function call: name(args) ----
    fn parse_call(&mut self, name: Symbol, name_span: Span) -> ParseResult<Expr> {
        self.consume(TokenKind::LParen)?;
        let mut args = Vec::new();
        if !self.peek_if(TokenKind::RParen) {
            loop {
                args.push(self.parse_expression()?);
                if self.peek_if(TokenKind::Comma) {
                    self.consume(TokenKind::Comma)?;
                } else {
                    break;
                }
            }
        }
        let rparen = self.consume(TokenKind::RParen)?;
        let span = name_span.merge(&rparen.span);
        Ok(Expr::Call(name, args, span))
    }

    // ---- Lists: [expr, expr, ...] ----
    fn parse_list(&mut self) -> ParseResult<Expr> {
        let start = self.peek().span;
        self.consume(TokenKind::LBracket)?;
        let mut elements = Vec::new();
        if !self.peek_if(TokenKind::RBracket) {
            loop {
                elements.push(self.parse_expression()?);
                if self.peek_if(TokenKind::Comma) {
                    self.consume(TokenKind::Comma)?;
                } else {
                    break;
                }
            }
        }
        let end = self.consume(TokenKind::RBracket)?;
        let span = start.merge(&end.span);
        Ok(Expr::List(elements, span))
    }

    // ---- Dicts: {key = value, key = value, ...} ----
    fn parse_dict(&mut self) -> ParseResult<Expr> {
        let start = self.peek().span;
        self.consume(TokenKind::LBrace)?;
        let mut pairs = Vec::new();
        if !self.peek_if(TokenKind::RBrace) {
            loop {
                let key = self.parse_expression()?;
                self.consume(TokenKind::Assign)?;
                let value = self.parse_expression()?;
                pairs.push((key, value));
                if self.peek_if(TokenKind::Comma) {
                    self.consume(TokenKind::Comma)?;
                } else {
                    break;
                }
            }
        }
        let end = self.consume(TokenKind::RBrace)?;
        let span = start.merge(&end.span);
        Ok(Expr::Dict(pairs, span))
    }

    fn parse_call_module(
        &mut self,
        module: Symbol,
        func: Symbol,
        name_span: Span,
    ) -> ParseResult<Expr> {
        self.consume(TokenKind::LParen)?;
        let mut args = Vec::new();
        if !self.peek_if(TokenKind::RParen) {
            loop {
                args.push(self.parse_expression()?);
                if self.peek_if(TokenKind::Comma) {
                    self.consume(TokenKind::Comma)?;
                } else {
                    break;
                }
            }
        }
        let rparen = self.consume(TokenKind::RParen)?;
        let span = name_span.merge(&rparen.span);
        Ok(Expr::CallModule(module, func, args, span))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinOp, Span, UnOp};
    use crate::parser::lexer::Lexer;

    // ---- Helper functions for removing spans ----
    pub fn strip_spans_expr(expr: &Expr) -> Expr {
        match expr {
            Expr::Int(i, _) => Expr::Int(*i, Span::dummy()),
            Expr::Float(f, _) => Expr::Float(*f, Span::dummy()),
            Expr::String(s, _) => Expr::String(s.clone(), Span::dummy()),
            Expr::Bool(b, _) => Expr::Bool(*b, Span::dummy()),
            Expr::Variable(name, _) => Expr::Variable(name.clone(), Span::dummy()),
            Expr::Binary(op, left, right, _) => Expr::Binary(
                *op,
                Box::new(strip_spans_expr(left)),
                Box::new(strip_spans_expr(right)),
                Span::dummy(),
            ),
            Expr::Unary(op, inner, _) => {
                Expr::Unary(*op, Box::new(strip_spans_expr(inner)), Span::dummy())
            }
            Expr::Index(base, index, _) => Expr::Index(
                Box::new(strip_spans_expr(base)),
                Box::new(strip_spans_expr(index)),
                Span::dummy(),
            ),
            Expr::List(elems, _) => {
                Expr::List(elems.iter().map(strip_spans_expr).collect(), Span::dummy())
            }
            Expr::Dict(pairs, _) => Expr::Dict(
                pairs
                    .iter()
                    .map(|(k, v)| (strip_spans_expr(k), strip_spans_expr(v)))
                    .collect(),
                Span::dummy(),
            ),
            Expr::Call(name, args, _) => Expr::Call(
                name.clone(),
                args.iter().map(strip_spans_expr).collect(),
                Span::dummy(),
            ),
            Expr::CallModule(module, func, args, _) => Expr::CallModule(
                module.clone(),
                func.clone(),
                args.iter().map(strip_spans_expr).collect(),
                Span::dummy(),
            ),
            Expr::ModuleAccess(module, name, _) => {
                Expr::ModuleAccess(module.clone(), name.clone(), Span::dummy())
            }
        }
    }

    pub fn strip_spans_stmt(stmt: &Stmt) -> Stmt {
        match stmt {
            Stmt::Let(name, expr, _, _) => Stmt::Let(
                name.clone(),
                strip_spans_expr(expr),
                Span::dummy(),
                Span::dummy(),
            ),
            Stmt::Input(prompt, var, _) => Stmt::Input(prompt.clone(), var.clone(), Span::dummy()),
            Stmt::For(var, start, end, step, body, _, _) => Stmt::For(
                var.clone(),
                Box::new(strip_spans_expr(start)),
                Box::new(strip_spans_expr(end)),
                step.as_ref().map(|s| Box::new(strip_spans_expr(s))),
                body.iter().map(strip_spans_stmt).collect(),
                Span::dummy(),
                Span::dummy(),
            ),
            Stmt::If(cond, then_block, else_block, _) => Stmt::If(
                strip_spans_expr(cond),
                then_block.iter().map(strip_spans_stmt).collect(),
                else_block
                    .as_ref()
                    .map(|b| b.iter().map(strip_spans_stmt).collect()),
                Span::dummy(),
            ),
            Stmt::While(cond, block, _) => Stmt::While(
                strip_spans_expr(cond),
                block.iter().map(strip_spans_stmt).collect(),
                Span::dummy(),
            ),
            Stmt::Break(_) => Stmt::Break(Span::dummy()),
            Stmt::Func(name, params, block, _, _, _) => Stmt::Func(
                name.clone(),
                params.clone(),
                block.iter().map(strip_spans_stmt).collect(),
                None,
                Span::dummy(),
                Span::dummy(),
            ),
            Stmt::Return(expr, _) => {
                Stmt::Return(expr.as_ref().map(strip_spans_expr), Span::dummy())
            }
            Stmt::Print(args, _) => {
                Stmt::Print(args.iter().map(strip_spans_expr).collect(), Span::dummy())
            }
            Stmt::Assign(lhs, rhs, _) => Stmt::Assign(
                Box::new(strip_spans_expr(lhs)),
                Box::new(strip_spans_expr(rhs)),
                Span::dummy(),
            ),
            Stmt::CompoundAssign(left, op, right, _) => Stmt::CompoundAssign(
                Box::new(strip_spans_expr(left)),
                *op,
                Box::new(strip_spans_expr(right)),
                Span::dummy(),
            ),
            Stmt::Expr(expr, _) => Stmt::Expr(strip_spans_expr(expr), Span::dummy()),
            Stmt::Import(path, alias, _) => {
                Stmt::Import(path.clone(), alias.clone(), Span::dummy())
            }
        }
    }

    pub fn strip_spans_program(prog: &Program) -> Program {
        Program {
            stmts: prog.stmts.iter().map(strip_spans_stmt).collect(),
        }
    }

    // ---- Helper functions for parsing and normalization ----
    fn parse_normalized(input: &str) -> Program {
        let tokens = Lexer::tokenize(input).expect("tokenization failed");
        let mut parser = Parser::new(&tokens);
        let prog = parser.parse().expect("parsing failed");
        strip_spans_program(&prog)
    }

    fn parse_expr_normalized(input: &str) -> Expr {
        let prog = parse_normalized(input);
        assert_eq!(prog.stmts.len(), 1, "Expected exactly one statement");
        match &prog.stmts[0] {
            Stmt::Expr(expr, _) => expr.clone(),
            _ => panic!("Expected expression statement, got {:?}", prog.stmts[0]),
        }
    }

    // ---- Tests ----
    #[test]
    fn test_literals() {
        let expr = parse_expr_normalized("42");
        assert_eq!(expr, Expr::Int(42, Span::dummy()));
        let expr = parse_expr_normalized("3.14");
        assert_eq!(expr, Expr::Float(3.14, Span::dummy()));
        let expr = parse_expr_normalized("\"hello\"");
        assert_eq!(expr, Expr::String("hello".to_string(), Span::dummy()));
        let expr = parse_expr_normalized("TRUE");
        assert_eq!(expr, Expr::Bool(true, Span::dummy()));
        let expr = parse_expr_normalized("FALSE");
        assert_eq!(expr, Expr::Bool(false, Span::dummy()));
        let expr = parse_expr_normalized("x");
        assert_eq!(expr, Expr::Variable(hi_common::intern("x"), Span::dummy()));
    }

    #[test]
    fn test_binary_ops() {
        let expr = parse_expr_normalized("1 + 2");
        assert_eq!(
            expr,
            Expr::Binary(
                BinOp::Add,
                Box::new(Expr::Int(1, Span::dummy())),
                Box::new(Expr::Int(2, Span::dummy())),
                Span::dummy()
            )
        );

        let expr = parse_expr_normalized("1 + 2 * 3");
        assert_eq!(
            expr,
            Expr::Binary(
                BinOp::Add,
                Box::new(Expr::Int(1, Span::dummy())),
                Box::new(Expr::Binary(
                    BinOp::Mul,
                    Box::new(Expr::Int(2, Span::dummy())),
                    Box::new(Expr::Int(3, Span::dummy())),
                    Span::dummy()
                )),
                Span::dummy()
            )
        );

        let expr = parse_expr_normalized("(1 + 2) * 3");
        assert_eq!(
            expr,
            Expr::Binary(
                BinOp::Mul,
                Box::new(Expr::Binary(
                    BinOp::Add,
                    Box::new(Expr::Int(1, Span::dummy())),
                    Box::new(Expr::Int(2, Span::dummy())),
                    Span::dummy()
                )),
                Box::new(Expr::Int(3, Span::dummy())),
                Span::dummy()
            )
        );

        let expr = parse_expr_normalized("a == b AND c != d");
        assert_eq!(
            expr,
            Expr::Binary(
                BinOp::And,
                Box::new(Expr::Binary(
                    BinOp::Eq,
                    Box::new(Expr::Variable(hi_common::intern("a"), Span::dummy())),
                    Box::new(Expr::Variable(hi_common::intern("b"), Span::dummy())),
                    Span::dummy()
                )),
                Box::new(Expr::Binary(
                    BinOp::Ne,
                    Box::new(Expr::Variable(hi_common::intern("c"), Span::dummy())),
                    Box::new(Expr::Variable(hi_common::intern("d"), Span::dummy())),
                    Span::dummy()
                )),
                Span::dummy()
            )
        );
    }

    #[test]
    fn test_unary_not() {
        let expr = parse_expr_normalized("NOT x");
        assert_eq!(
            expr,
            Expr::Unary(
                UnOp::Not,
                Box::new(Expr::Variable(hi_common::intern("x"), Span::dummy())),
                Span::dummy()
            )
        );
        let expr = parse_expr_normalized("NOT (1 + 2)");
        assert_eq!(
            expr,
            Expr::Unary(
                UnOp::Not,
                Box::new(Expr::Binary(
                    BinOp::Add,
                    Box::new(Expr::Int(1, Span::dummy())),
                    Box::new(Expr::Int(2, Span::dummy())),
                    Span::dummy()
                )),
                Span::dummy()
            )
        );
    }

    #[test]
    fn test_call() {
        let expr = parse_expr_normalized("foo(1, 2 + 3)");
        assert_eq!(
            expr,
            Expr::Call(
                hi_common::intern("foo"),
                vec![
                    Expr::Int(1, Span::dummy()),
                    Expr::Binary(
                        BinOp::Add,
                        Box::new(Expr::Int(2, Span::dummy())),
                        Box::new(Expr::Int(3, Span::dummy())),
                        Span::dummy()
                    )
                ],
                Span::dummy()
            )
        );
        let expr = parse_expr_normalized("bar()");
        assert_eq!(
            expr,
            Expr::Call(hi_common::intern("bar"), vec![], Span::dummy())
        );
    }

    #[test]
    fn test_index() {
        let expr = parse_expr_normalized("arr[i + 1]");
        assert_eq!(
            expr,
            Expr::Index(
                Box::new(Expr::Variable(hi_common::intern("arr"), Span::dummy())),
                Box::new(Expr::Binary(
                    BinOp::Add,
                    Box::new(Expr::Variable(hi_common::intern("i"), Span::dummy())),
                    Box::new(Expr::Int(1, Span::dummy())),
                    Span::dummy()
                )),
                Span::dummy()
            )
        );
        let expr = parse_expr_normalized("matrix[i][j]");
        assert_eq!(
            expr,
            Expr::Index(
                Box::new(Expr::Index(
                    Box::new(Expr::Variable(hi_common::intern("matrix"), Span::dummy())),
                    Box::new(Expr::Variable(hi_common::intern("i"), Span::dummy())),
                    Span::dummy()
                )),
                Box::new(Expr::Variable(hi_common::intern("j"), Span::dummy())),
                Span::dummy()
            )
        );
    }

    #[test]
    fn test_list() {
        let expr = parse_expr_normalized("[1, 2, 3]");
        assert_eq!(
            expr,
            Expr::List(
                vec![
                    Expr::Int(1, Span::dummy()),
                    Expr::Int(2, Span::dummy()),
                    Expr::Int(3, Span::dummy())
                ],
                Span::dummy()
            )
        );
        let expr = parse_expr_normalized("[]");
        assert_eq!(expr, Expr::List(vec![], Span::dummy()));
    }

    #[test]
    fn test_dict() {
        let expr = parse_expr_normalized("{a = 1, b = 2 + 3}");
        assert_eq!(
            expr,
            Expr::Dict(
                vec![
                    (
                        Expr::Variable(hi_common::intern("a"), Span::dummy()),
                        Expr::Int(1, Span::dummy())
                    ),
                    (
                        Expr::Variable(hi_common::intern("b"), Span::dummy()),
                        Expr::Binary(
                            BinOp::Add,
                            Box::new(Expr::Int(2, Span::dummy())),
                            Box::new(Expr::Int(3, Span::dummy())),
                            Span::dummy()
                        )
                    )
                ],
                Span::dummy()
            )
        );
        let expr = parse_expr_normalized("{}");
        assert_eq!(expr, Expr::Dict(vec![], Span::dummy()));
    }

    #[test]
    fn test_let() {
        let prog = parse_normalized("LET x = 10");
        assert_eq!(prog.stmts.len(), 1);
        match &prog.stmts[0] {
            Stmt::Let(name, expr, _, _) => {
                assert_eq!(*name, hi_common::intern("x"));
                assert_eq!(*expr, Expr::Int(10, Span::dummy()));
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_if() {
        let prog = parse_normalized("IF x > 0 THEN LET y = 1 END");
        assert_eq!(prog.stmts.len(), 1);
        match &prog.stmts[0] {
            Stmt::If(cond, then_block, else_block, _) => {
                assert_eq!(
                    *cond,
                    Expr::Binary(
                        BinOp::Gt,
                        Box::new(Expr::Variable(hi_common::intern("x"), Span::dummy())),
                        Box::new(Expr::Int(0, Span::dummy())),
                        Span::dummy()
                    )
                );
                assert_eq!(then_block.len(), 1);
                match &then_block[0] {
                    Stmt::Let(name, expr, _, _) => {
                        assert_eq!(*name, hi_common::intern("y"));
                        assert_eq!(*expr, Expr::Int(1, Span::dummy()));
                    }
                    _ => panic!("Expected Let in then block"),
                }
                assert!(else_block.is_none());
            }
            _ => panic!("Expected If statement"),
        }

        let prog = parse_normalized("IF cond THEN PRINT 1 ELSE PRINT 2 END");
        match &prog.stmts[0] {
            Stmt::If(_, then_block, Some(else_block), _) => {
                assert_eq!(then_block.len(), 1);
                assert_eq!(else_block.len(), 1);
            }
            _ => panic!("Expected If with else"),
        }
    }

    #[test]
    fn test_while() {
        let prog = parse_normalized("WHILE x < 5 DO x = x + 1 END");
        match &prog.stmts[0] {
            Stmt::While(cond, block, _) => {
                assert_eq!(
                    *cond,
                    Expr::Binary(
                        BinOp::Lt,
                        Box::new(Expr::Variable(hi_common::intern("x"), Span::dummy())),
                        Box::new(Expr::Int(5, Span::dummy())),
                        Span::dummy()
                    )
                );
                assert_eq!(block.len(), 1);
                match &block[0] {
                    Stmt::Assign(lhs, rhs, _) => {
                        assert_eq!(**lhs, Expr::Variable(hi_common::intern("x"), Span::dummy()));
                        assert_eq!(
                            **rhs,
                            Expr::Binary(
                                BinOp::Add,
                                Box::new(Expr::Variable(hi_common::intern("x"), Span::dummy())),
                                Box::new(Expr::Int(1, Span::dummy())),
                                Span::dummy()
                            )
                        );
                    }
                    _ => panic!("Expected assignment in while body"),
                }
            }
            _ => panic!("Expected While statement"),
        }
    }

    #[test]
    fn test_func() {
        let prog = parse_normalized("FUNC add(a, b) RET a + b END");
        match &prog.stmts[0] {
            Stmt::Func(name, params, block, _, _, _) => {
                assert_eq!(*name, hi_common::intern("add"));
                assert_eq!(params, &[hi_common::intern("a"), hi_common::intern("b")]);
                assert_eq!(block.len(), 1);
                match &block[0] {
                    Stmt::Return(Some(expr), _) => {
                        assert_eq!(
                            *expr,
                            Expr::Binary(
                                BinOp::Add,
                                Box::new(Expr::Variable(hi_common::intern("a"), Span::dummy())),
                                Box::new(Expr::Variable(hi_common::intern("b"), Span::dummy())),
                                Span::dummy()
                            )
                        );
                    }
                    _ => panic!("Expected Return statement inside function"),
                }
            }
            _ => panic!("Expected Func statement"),
        }
    }

    #[test]
    fn test_return() {
        let prog = parse_normalized("RET 42");
        match &prog.stmts[0] {
            Stmt::Return(Some(expr), _) => {
                assert_eq!(*expr, Expr::Int(42, Span::dummy()));
            }
            _ => panic!("Expected Return with expr"),
        }
        let prog = parse_normalized("RET");
        match &prog.stmts[0] {
            Stmt::Return(None, _) => {}
            _ => panic!("Expected Return without expr"),
        }
    }

    #[test]
    fn test_break() {
        let prog = parse_normalized("BREAK");
        match &prog.stmts[0] {
            Stmt::Break(_) => {}
            _ => panic!("Expected Break statement"),
        }
    }

    #[test]
    fn test_print() {
        let prog = parse_normalized("PRINT 1, 2 + 3, \"hi\"");
        match &prog.stmts[0] {
            Stmt::Print(args, _) => {
                assert_eq!(args.len(), 3);
                assert_eq!(args[0], Expr::Int(1, Span::dummy()));
                assert_eq!(
                    args[1],
                    Expr::Binary(
                        BinOp::Add,
                        Box::new(Expr::Int(2, Span::dummy())),
                        Box::new(Expr::Int(3, Span::dummy())),
                        Span::dummy()
                    )
                );
                assert_eq!(args[2], Expr::String("hi".to_string(), Span::dummy()));
            }
            _ => panic!("Expected Print statement"),
        }
        let prog = parse_normalized("PRINT");
        match &prog.stmts[0] {
            Stmt::Print(args, _) => assert!(args.is_empty()),
            _ => panic!("Expected Print with no args"),
        }
    }

    #[test]
    fn test_assign() {
        let prog = parse_normalized("x = 10");
        match &prog.stmts[0] {
            Stmt::Assign(lhs, rhs, _) => {
                assert_eq!(**lhs, Expr::Variable(hi_common::intern("x"), Span::dummy()));
                assert_eq!(**rhs, Expr::Int(10, Span::dummy()));
            }
            _ => panic!("Expected Assign statement"),
        }
        let prog = parse_normalized("arr[0] = 42");
        match &prog.stmts[0] {
            Stmt::Assign(lhs, rhs, _) => {
                assert_eq!(
                    **lhs,
                    Expr::Index(
                        Box::new(Expr::Variable(hi_common::intern("arr"), Span::dummy())),
                        Box::new(Expr::Int(0, Span::dummy())),
                        Span::dummy()
                    )
                );
                assert_eq!(**rhs, Expr::Int(42, Span::dummy()));
            }
            _ => panic!("Expected Assign with index lhs"),
        }
    }

    #[test]
    fn test_block() {
        let prog = parse_normalized("LET x = 1\nPRINT x\nx = x + 1");
        assert_eq!(prog.stmts.len(), 3);
    }

    #[test]
    fn test_input() {
        let prog = parse_normalized("INPUT x");
        match &prog.stmts[0] {
            Stmt::Input(None, var, _) => assert_eq!(*var, hi_common::intern("x")),
            _ => panic!("Expected Input without prompt"),
        }

        let prog = parse_normalized("INPUT \"Enter: \" x");
        match &prog.stmts[0] {
            Stmt::Input(Some(prompt), var, _) => {
                assert_eq!(prompt, "Enter: ");
                assert_eq!(*var, hi_common::intern("x"));
            }
            _ => panic!("Expected Input with prompt"),
        }
    }

    #[test]
    fn test_for() {
        let prog = parse_normalized("FOR i = 0 TO 10 DO PRINT i NEXT 2");
        match &prog.stmts[0] {
            Stmt::For(var, start, end, step, body, _, _) => {
                assert_eq!(*var, hi_common::intern("i"));
                assert_eq!(**start, Expr::Int(0, Span::dummy()));
                assert_eq!(**end, Expr::Int(10, Span::dummy()));
                assert_eq!(**step.as_ref().unwrap(), Expr::Int(2, Span::dummy()));
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected For statement"),
        }

        let prog = parse_normalized("FOR i = 0 TO 10 DO PRINT i NEXT");
        match &prog.stmts[0] {
            Stmt::For(_, _, _, step, _, _, _) => assert!(step.is_none()),
            _ => panic!("Expected For without step"),
        }
    }

    // ---- Tests errors  ----
    #[test]
    fn test_parse_error_unexpected_token() {
        let tokens = Lexer::tokenize("+").unwrap();
        let mut parser = Parser::new(&tokens);
        let err = parser.parse().unwrap_err();
        assert!(err.message.contains("Unexpected token"));
    }

    #[test]
    fn test_parse_error_missing_rparen() {
        let tokens = Lexer::tokenize("(1 + 2").unwrap();
        let mut parser = Parser::new(&tokens);
        let err = parser.parse().unwrap_err();
        assert!(err.message.contains("Expected RParen"));
    }

    #[test]
    fn test_parse_error_missing_end() {
        let tokens = Lexer::tokenize("IF x THEN y = 1").unwrap();
        let mut parser = Parser::new(&tokens);
        let err = parser.parse().unwrap_err();
        assert!(err.message.contains("Expected End"));
    }

    #[test]
    fn test_complex_expression() {
        let expr = parse_expr_normalized("(a + b) * (c - d) / e ^ 2");
        match expr {
            Expr::Binary(BinOp::Div, left, right, _) => {
                match (*left, *right) {
                    (Expr::Binary(BinOp::Mul, _, _, _), Expr::Binary(BinOp::Pow, _, _, _)) => {
                        // pass
                    }
                    _ => panic!("Wrong structure"),
                }
            }
            _ => panic!("Expected division at top"),
        }
    }

    #[test]
    fn test_import() {
        let prog = parse_normalized("IMPORT \"lib.hi\"");
        match &prog.stmts[0] {
            Stmt::Import(path, alias, _) => {
                assert_eq!(path, "lib.hi");
                assert!(alias.is_none());
            }
            _ => panic!("Expected Import statement"),
        }
    }

    #[test]
    fn test_import_as() {
        let prog = parse_normalized("IMPORT \"lib.hi\" AS l");
        match &prog.stmts[0] {
            Stmt::Import(path, Some(alias), _) => {
                assert_eq!(path, "lib.hi");
                assert_eq!(*alias, hi_common::intern("l"));
            }
            _ => panic!("Expected Import with alias"),
        }
    }

    #[test]
    fn test_module_access() {
        let expr = parse_expr_normalized("m:sin(10)");
        match expr {
            Expr::CallModule(module, func, args, _) => {
                assert_eq!(module, hi_common::intern("m"));
                assert_eq!(func, hi_common::intern("sin"));
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0], Expr::Int(10, _)));
            }
            _ => panic!("Expected CallModule"),
        }
    }

    #[test]
    fn test_module_variable() {
        let expr = parse_expr_normalized("m:PI");
        match expr {
            Expr::ModuleAccess(module, var, _) => {
                assert_eq!(module, hi_common::intern("m"));
                assert_eq!(var, hi_common::intern("PI"));
            }
            _ => panic!("Expected ModuleAccess"),
        }
    }
}
