use crate::ast::*;
use crate::lexer::{Token, TokenKind};

pub struct Parser { tokens: Vec<Token>, pos: usize }

impl Parser {
    pub fn new(mut tokens: Vec<Token>) -> Self {
        tokens.retain(|t| t.kind != TokenKind::Newline);
        Self { tokens, pos: 0 }
    }
    fn current(&self) -> &Token { self.tokens.get(self.pos).unwrap_or(&Token { kind: TokenKind::Eof, line: 0, col: 0 }) }
    fn is_at_end(&self) -> bool { self.pos >= self.tokens.len() || self.current().kind == TokenKind::Eof }
    fn check(&self, kind: &TokenKind) -> bool { std::mem::discriminant(&self.current().kind) == std::mem::discriminant(kind) }
    fn peek_is(&self, kind: &TokenKind) -> bool {
        self.tokens.get(self.pos + 1).map_or(false, |t| std::mem::discriminant(&t.kind) == std::mem::discriminant(kind))
    }
    fn advance(&mut self) -> Token { let t = self.current().clone(); if !self.is_at_end() { self.pos += 1; } t }
    fn expect_token(&mut self, kind: &TokenKind) -> Result<Token, String> {
        if self.check(kind) { Ok(self.advance()) }
        else { Err(format!("Expected {:?}, got {:?} at {}:{}", kind, self.current().kind, self.current().line, self.current().col)) }
    }
    fn expect_ident(&mut self) -> Result<String, String> {
        let t = self.current().clone();
        if let TokenKind::Identifier(s) = &t.kind { let s = s.clone(); self.advance(); Ok(s) }
        else { Err(format!("Expected identifier, got {:?} at {}:{}", t.kind, t.line, t.col)) }
    }
    fn expect_string(&mut self) -> Result<String, String> {
        if let TokenKind::String(s) = &self.current().kind { let s = s.clone(); self.advance(); Ok(s) }
        else { let t = self.current().clone(); Err(format!("Expected string, got {:?} at {}:{}", t.kind, t.line, t.col)) }
    }
    fn expect_int(&mut self) -> Result<i64, String> {
        if let TokenKind::Integer(n) = self.current().kind { self.advance(); Ok(n) }
        else { let t = self.current().clone(); Err(format!("Expected int, got {:?} at {}:{}", t.kind, t.line, t.col)) }
    }
    pub fn parse(&mut self) -> Result<Script, String> {
        self.expect_token(&TokenKind::Script)?;
        let name = self.expect_string()?;
        self.expect_token(&TokenKind::LeftBrace)?;
        let mut version = 0i64;
        let mut items = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            match &self.current().kind {
                TokenKind::Version => { self.advance(); self.expect_token(&TokenKind::Colon)?; version = self.expect_int()?; }
                TokenKind::Var => items.push(ScriptItem::Var(self.parse_var_decl()?)),
                TokenKind::Function => items.push(ScriptItem::Function(self.parse_function_def()?)),
                TokenKind::On => items.push(ScriptItem::OnStart(self.parse_on_start()?)),
                _ => { let t = self.current().clone(); return Err(format!("Unexpected {:?} at {}:{}", t.kind, t.line, t.col)); }
            }
        }
        self.expect_token(&TokenKind::RightBrace)?;
        Ok(Script { name, version, items })
    }
    fn parse_var_decl(&mut self) -> Result<VarDecl, String> { self.advance(); let n = self.expect_ident()?; self.expect_token(&TokenKind::Eq)?; Ok(VarDecl { name: n, init: self.parse_expr()? }) }
    fn parse_function_def(&mut self) -> Result<FunctionDef, String> {
        self.advance(); let name = self.expect_ident()?;
        self.expect_token(&TokenKind::LeftParen)?;
        let mut params = Vec::new();
        if !self.check(&TokenKind::RightParen) { params.push(self.expect_ident()?); while self.check(&TokenKind::Comma) { self.advance(); params.push(self.expect_ident()?); } }
        self.expect_token(&TokenKind::RightParen)?;
        Ok(FunctionDef { name, params, body: self.parse_block()? })
    }
    fn parse_on_start(&mut self) -> Result<Vec<Stmt>, String> { self.advance(); self.expect_token(&TokenKind::Start)?; self.parse_block() }
    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        self.expect_token(&TokenKind::LeftBrace)?;
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() { stmts.push(self.parse_stmt()?); }
        self.expect_token(&TokenKind::RightBrace)?;
        Ok(stmts)
    }
    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match &self.current().kind {
            TokenKind::Var => Ok(Stmt::VarDecl(self.parse_var_decl()?)),
            TokenKind::If => Ok(Stmt::If(self.parse_if_stmt()?)),
            TokenKind::Loop => { self.advance(); Ok(Stmt::Loop(self.parse_block()?)) }
            TokenKind::While => Ok(Stmt::While(self.parse_while_stmt()?)),
            TokenKind::Break => { self.advance(); Ok(Stmt::Break) }
            TokenKind::Restart => { self.advance(); Ok(Stmt::Restart) }
            TokenKind::Return => self.parse_return(),
            TokenKind::Wait => self.parse_wait_until(),
            TokenKind::Pause => Ok(Stmt::Action(ActionStmt::Pause(self.parse_pause_value()?))),
            TokenKind::Key => Ok(Stmt::Action(self.parse_key_action()?)),
            TokenKind::Mouse => Ok(Stmt::Action(self.parse_mouse_action()?)),
            TokenKind::Call => { self.advance(); let n = self.expect_ident()?; self.expect_token(&TokenKind::LeftParen)?; let a = self.parse_expr_list()?; self.expect_token(&TokenKind::RightParen)?; Ok(Stmt::Action(ActionStmt::Call { name: n, args: a })) }
            TokenKind::Identifier(_) if self.peek_is(&TokenKind::Eq) => { let n = self.expect_ident()?; self.advance(); Ok(Stmt::Assignment(Assignment { name: n, value: self.parse_expr()? })) }
            _ => { let t = self.current().clone(); Err(format!("Unexpected {:?} at {}:{}", t.kind, t.line, t.col)) }
        }
    }
    fn parse_if_stmt(&mut self) -> Result<IfStmt, String> {
        self.advance(); let cond = self.parse_condition()?; let then = self.parse_block()?;
        let els = if self.check(&TokenKind::Else) { self.advance(); Some(self.parse_block()?) } else { None };
        Ok(IfStmt { condition: cond, then_body: then, else_body: els })
    }
    fn parse_while_stmt(&mut self) -> Result<WhileStmt, String> { self.advance(); Ok(WhileStmt { condition: self.parse_expr()?, body: self.parse_block()? }) }
    fn parse_return(&mut self) -> Result<Stmt, String> {
        self.advance();
        if self.check(&TokenKind::RightBrace) || self.is_at_end() { Ok(Stmt::Return(None)) } else { Ok(Stmt::Return(Some(self.parse_expr()?))) }
    }
    fn parse_wait_until(&mut self) -> Result<Stmt, String> {
        self.advance(); self.expect_token(&TokenKind::Until)?; let _cond = self.parse_condition()?;
        let mut _timeout = None;
        if self.check(&TokenKind::Timeout) { self.advance(); _timeout = Some(Box::new(self.parse_expr()?)); }
        let _body = self.parse_block()?;
        let _els = if self.check(&TokenKind::Else) { self.advance(); Some(self.parse_block()?) } else { None };
        Ok(Stmt::Action(ActionStmt::Call { name: "wait_until".into(), args: vec![] }))
    }
    fn parse_key_action(&mut self) -> Result<ActionStmt, String> {
        self.advance(); self.expect_token(&TokenKind::Dot)?; let m = self.expect_ident()?;
        self.expect_token(&TokenKind::LeftParen)?; let a = self.parse_expr()?; self.expect_token(&TokenKind::RightParen)?;
        match m.as_str() { "tap" => Ok(ActionStmt::KeyTap { key: a }), "hold" => Ok(ActionStmt::KeyHold { key: a }), "release" => Ok(ActionStmt::KeyRelease { key: a }), "type" => Ok(ActionStmt::KeyTypeText { text: a }), _ => Err(format!("Unknown key method: {}", m)) }
    }
    fn parse_mouse_action(&mut self) -> Result<ActionStmt, String> {
        self.advance(); self.expect_token(&TokenKind::Dot)?; let m = self.expect_ident()?;
        self.expect_token(&TokenKind::LeftParen)?;
        match m.as_str() {
            "click" => { let b = self.parse_expr()?; self.expect_token(&TokenKind::RightParen)?; Ok(ActionStmt::MouseClick { button: b }) }
            "press" => { let b = self.parse_expr()?; self.expect_token(&TokenKind::RightParen)?; Ok(ActionStmt::MousePress { button: b }) }
            "release" => { let b = self.parse_expr()?; self.expect_token(&TokenKind::RightParen)?; Ok(ActionStmt::MouseRelease { button: b }) }
            "move" => { let x = self.parse_expr()?; self.expect_token(&TokenKind::Comma)?; let y = self.parse_expr()?; self.expect_token(&TokenKind::RightParen)?; Ok(ActionStmt::MouseMove { x, y }) }
            _ => Err(format!("Unknown mouse method: {}", m)),
        }
    }
    fn parse_pause_value(&mut self) -> Result<PauseValue, String> {
        self.advance();
        if self.check(&TokenKind::Human) { self.advance(); return Ok(PauseValue::Human); }
        let min = self.parse_expr()?;
        if self.check(&TokenKind::DotDot) { self.advance(); Ok(PauseValue::Range { min: Box::new(min), max: Box::new(self.parse_expr()?) }) }
        else { Ok(PauseValue::Fixed(min)) }
    }
}

impl Parser {
    fn parse_condition(&mut self) -> Result<Condition, String> {
        if self.check(&TokenKind::Not) { self.advance(); return Ok(Condition::Negated(Box::new(self.parse_condition()?))); }
        if self.check(&TokenKind::Wait) { return self.parse_wait_condition(); }
        // Handle grouped conditions: ( condition )
        if self.check(&TokenKind::LeftParen) {
            self.advance();
            let cond = self.parse_condition()?;
            self.expect_token(&TokenKind::RightParen)?;
            return Ok(cond);
        }
        if let TokenKind::Identifier(ref name) = self.current().kind {
            if name == "pixel" { return self.parse_pixel_condition(); }
            if name == "region" { return self.parse_region_condition(); }
        }
        Ok(Condition::Expression(Box::new(self.parse_expr()?)))
    }
    fn parse_pixel_condition(&mut self) -> Result<Condition, String> {
        self.advance(); self.expect_token(&TokenKind::LeftParen)?;
        let x = Box::new(self.parse_expr()?); self.expect_token(&TokenKind::Comma)?;
        let y = Box::new(self.parse_expr()?); self.expect_token(&TokenKind::RightParen)?;
        self.expect_token(&TokenKind::Matches)?;
        let color = Box::new(self.parse_expr()?);
        let tol = if self.check(&TokenKind::Within) { self.advance(); Some(Box::new(self.parse_expr()?)) } else { None };
        Ok(Condition::PixelMatches { x, y, color, tolerance: tol })
    }
    fn parse_region_condition(&mut self) -> Result<Condition, String> {
        self.advance(); self.expect_token(&TokenKind::LeftParen)?;
        let x = Box::new(self.parse_expr()?); self.expect_token(&TokenKind::Comma)?;
        let y = Box::new(self.parse_expr()?); self.expect_token(&TokenKind::Comma)?;
        let w = Box::new(self.parse_expr()?); self.expect_token(&TokenKind::Comma)?;
        let h = Box::new(self.parse_expr()?); self.expect_token(&TokenKind::RightParen)?;
        self.expect_token(&TokenKind::Matches)?; self.expect_ident()?;
        self.expect_token(&TokenKind::LeftParen)?;
        let img = Box::new(self.parse_expr()?); self.expect_token(&TokenKind::RightParen)?;
        let conf = if self.check(&TokenKind::Confidence) { self.advance(); Some(Box::new(self.parse_expr()?)) } else { None };
        Ok(Condition::RegionMatches { x, y, width: w, height: h, image_path: img, confidence: conf })
    }
    fn parse_wait_condition(&mut self) -> Result<Condition, String> {
        self.advance(); self.expect_token(&TokenKind::Until)?;
        let inner = Box::new(self.parse_condition()?);
        let mut timeout = None;
        if self.check(&TokenKind::Timeout) { self.advance(); timeout = Some(Box::new(self.parse_expr()?)); }
        let body = self.parse_block()?;
        let els = if self.check(&TokenKind::Else) { self.advance(); Some(self.parse_block()?) } else { None };
        Ok(Condition::WaitUntil { condition: inner, timeout, body, else_body: els })
    }
}

impl Parser {
    fn parse_expr(&mut self) -> Result<Expr, String> { self.parse_comparison() }
    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_addition()?;
        loop {
            let op = match self.current().kind { TokenKind::EqEq => BinOp::Eq, TokenKind::NotEq => BinOp::Neq, TokenKind::Lt => BinOp::Lt, TokenKind::Gt => BinOp::Gt, _ => break };
            self.advance(); left = Expr::BinaryOp { op, left: Box::new(left), right: Box::new(self.parse_addition()?) };
        }
        Ok(left)
    }
    fn parse_addition(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplication()?;
        loop {
            let op = match self.current().kind { TokenKind::Plus => BinOp::Add, TokenKind::Minus => BinOp::Sub, _ => break };
            self.advance(); left = Expr::BinaryOp { op, left: Box::new(left), right: Box::new(self.parse_multiplication()?) };
        }
        Ok(left)
    }
    fn parse_multiplication(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.current().kind { TokenKind::Star => BinOp::Mul, TokenKind::Slash => BinOp::Div, _ => break };
            self.advance(); left = Expr::BinaryOp { op, left: Box::new(left), right: Box::new(self.parse_unary()?) };
        }
        Ok(left)
    }
    fn parse_unary(&mut self) -> Result<Expr, String> {
        if self.check(&TokenKind::Not) { self.advance(); return Ok(Expr::UnaryNot(Box::new(self.parse_primary()?))); }
        self.parse_primary()
    }
    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.current().kind.clone() {
            TokenKind::Integer(n) => { self.advance(); Ok(Expr::IntNumber(n)) }
            TokenKind::Float(f) => { self.advance(); Ok(Expr::Number(f)) }
            TokenKind::String(s) => { self.advance(); Ok(Expr::String(s)) }
            TokenKind::Color(c) => { self.advance(); Ok(Expr::Color(c)) }
            TokenKind::True => { self.advance(); Ok(Expr::Bool(true)) }
            TokenKind::False => { self.advance(); Ok(Expr::Bool(false)) }
            TokenKind::Identifier(name) => {
                self.advance();
                if self.check(&TokenKind::LeftParen) {
                    self.advance(); let args = self.parse_expr_list()?; self.expect_token(&TokenKind::RightParen)?;
                    if name == "pixel" && args.len() == 2 { Ok(Expr::PixelColor { x: Box::new(args[0].clone()), y: Box::new(args[1].clone()) }) }
                    else { Ok(Expr::FuncCall { name, args }) }
                } else { Ok(Expr::Identifier(name)) }
            }
            TokenKind::LeftParen => { self.advance(); let e = self.parse_expr()?; self.expect_token(&TokenKind::RightParen)?; Ok(e) }
            _ => { let t = self.current().clone(); Err(format!("Unexpected {:?} in expr at {}:{}", t.kind, t.line, t.col)) }
        }
    }
    fn parse_expr_list(&mut self) -> Result<Vec<Expr>, String> {
        let mut exprs = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            exprs.push(self.parse_expr()?);
            while self.check(&TokenKind::Comma) { self.advance(); exprs.push(self.parse_expr()?); }
        }
        Ok(exprs)
    }
}

pub fn parse(source: &str) -> Result<Script, String> {
    Parser::new(crate::lexer::tokenize(source)).parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_minimal_script() {
        let s = parse(r#"script "T" { version: 1 on start { } }"#).unwrap();
        assert_eq!(s.name, "T"); assert_eq!(s.version, 1);
    }
    #[test]
    fn test_var_and_function() {
        let s = parse(r#"
script "E" { version: 1 var x = 10 function greet() { key.tap("1") }
on start { call greet() } }"#).unwrap();
        assert_eq!(s.items.len(), 3);
    }
    #[test]
    fn test_pixel_condition() {
        let s = parse(r#"
script "T" { version: 1 on start {
if pixel(100, 200) matches #FF0000 within 10 { key.tap("1") } } }"#).unwrap();
        if let ScriptItem::OnStart(stmts) = &s.items[0] {
            if let Stmt::If(i) = &stmts[0] { assert!(matches!(i.condition, Condition::PixelMatches { .. })); }
            else { panic!("Expected If"); }
        }
    }
    #[test]
    fn test_actions_and_pause() {
        let s = parse(r#"
script "T" { version: 1 on start {
key.tap("a") key.hold("shift") key.release("shift")
mouse.click("left") mouse.move(100, 200)
pause 500 pause 300..600 pause human } }"#).unwrap();
        if let ScriptItem::OnStart(stmts) = &s.items[0] { assert_eq!(stmts.len(), 8); }
    }
    #[test]
    fn test_loop_break() {
        let s = parse(r#"
script "T" { version: 1 on start { loop { key.tap("1") break } } }"#).unwrap();
        if let ScriptItem::OnStart(stmts) = &s.items[0] {
            if let Stmt::Loop(body) = &stmts[0] { assert!(matches!(body[1], Stmt::Break)); }
        }
    }
    #[test]
    fn test_expressions() {
        let s = parse(r#"
script "T" { version: 1 on start { var x = 10 + 5 * 2 var y = x == 20 } }"#).unwrap();
        if let ScriptItem::OnStart(stmts) = &s.items[0] { assert_eq!(stmts.len(), 2); }
    }
}
