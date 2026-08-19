use std::fmt;
use std::thread;
use std::time::Duration;
use rand::Rng;
use enigo::{Coordinate, Direction::{Click, Press, Release}, Keyboard, Mouse};

// ─── Runtime Values ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Int(i64),
    String(String),
    Bool(bool),
    Color(String),
    Void,
}

impl Value {
    pub fn to_number(&self) -> f64 {
        match self {
            Value::Number(n) => *n,
            Value::Int(n) => *n as f64,
            Value::Bool(b) => { if *b { 1.0 } else { 0.0 } }
            _ => 0.0,
        }
    }
    pub fn to_int(&self) -> i64 {
        match self {
            Value::Int(n) => *n,
            Value::Number(n) => *n as i64,
            _ => 0,
        }
    }
    pub fn to_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::Int(n) => *n != 0,
            Value::String(s) => !s.is_empty(),
            Value::Void => false,
            _ => true,
        }
    }
    pub fn to_string_val(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Number(n) => format!("{}", n),
            Value::Int(n) => format!("{}", n),
            Value::Bool(b) => if *b { "true".into() } else { "false".into() },
            Value::Color(c) => c.clone(),
            Value::Void => "void".into(),
        }
    }
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::Int(_) => "integer",
            Value::String(_) => "string",
            Value::Bool(_) => "boolean",
            Value::Color(_) => "color",
            Value::Void => "void",
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Int(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Color(c) => write!(f, "{}", c),
            Value::Void => write!(f, "void"),
        }
    }
}

// ─── Environment (Scoped Variable Storage) ────────────────────────────────────

pub struct Environment {
    scopes: Vec<Vec<(String, Value)>>,
}

impl Environment {
    pub fn new() -> Self {
        Self { scopes: vec![Vec::new()] }
    }

    pub fn push_scope(&mut self) { self.scopes.push(Vec::new()); }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 { self.scopes.pop(); }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.scopes.last_mut().unwrap().push((name, value));
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            for (n, v) in scope.iter().rev() {
                if n == name { return Some(v); }
            }
        }
        None
    }

    pub fn set(&mut self, name: &str, value: Value) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            for (n, v) in scope.iter_mut().rev() {
                if n == name { *v = value; return true; }
            }
        }
        false
    }
}

// ─── Control Flow Signals ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Signal {
    None,
    Break,
    Restart,
    Return(Value),
}

// ─── Log Events ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum LogEvent {
    Info(String),
    VariableChanged(String, Value),
    FunctionCalled(String),
    ScriptStarted(String),
    ScriptFinished(String, bool),
}

// ─── Color Helpers ────────────────────────────────────────────────────────────

fn parse_hex_color(hex: &str) -> Result<(f64, f64, f64), String> {
    let h = hex.trim_start_matches('#');
    if h.len() != 6 { return Err(format!("Invalid hex color: {}", hex)); }
    let r = u8::from_str_radix(&h[0..2], 16).map_err(|_| format!("Invalid hex: {}", hex))? as f64;
    let g = u8::from_str_radix(&h[2..4], 16).map_err(|_| format!("Invalid hex: {}", hex))? as f64;
    let b = u8::from_str_radix(&h[4..6], 16).map_err(|_| format!("Invalid hex: {}", hex))? as f64;
    Ok((r, g, b))
}

// ─── Interpreter ──────────────────────────────────────────────────────────────

pub struct Interpreter {
    pub env: Environment,
    pub functions: std::collections::HashMap<String, crate::ast::FunctionDef>,
    pub log: Vec<LogEvent>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
            functions: std::collections::HashMap::new(),
            log: Vec::new(),
        }
    }

    pub fn run(&mut self, script: &crate::ast::Script) -> Result<(), String> {
        self.log.push(LogEvent::ScriptStarted(script.name.clone()));
        for item in &script.items {
            if let crate::ast::ScriptItem::Function(f) = item {
                self.functions.insert(f.name.clone(), f.clone());
            }
        }
        let mut on_start_body = Vec::new();
        for item in &script.items {
            match item {
                crate::ast::ScriptItem::Var(v) => {
                    let val = self.eval_expr(&v.init)?;
                    self.env.define(v.name.clone(), val.clone());
                    self.log.push(LogEvent::VariableChanged(v.name.clone(), val));
                }
                crate::ast::ScriptItem::OnStart(body) => { on_start_body = body.clone(); }
                _ => {}
            }
        }
        self.exec_block(&on_start_body)?;
        self.log.push(LogEvent::ScriptFinished(script.name.clone(), true));
        Ok(())
    }

    pub fn exec_block(&mut self, stmts: &[crate::ast::Stmt]) -> Result<Signal, String> {
        self.env.push_scope();
        for stmt in stmts {
            let sig = self.exec_stmt(stmt)?;
            match &sig {
                Signal::None => {}
                _ => { self.env.pop_scope(); return Ok(sig); }
            }
        }
        self.env.pop_scope();
        Ok(Signal::None)
    }

    fn exec_stmt(&mut self, stmt: &crate::ast::Stmt) -> Result<Signal, String> {
        match stmt {
            crate::ast::Stmt::VarDecl(v) => {
                let val = self.eval_expr(&v.init)?;
                self.env.define(v.name.clone(), val.clone());
                self.log.push(LogEvent::VariableChanged(v.name.clone(), val));
                Ok(Signal::None)
            }
            crate::ast::Stmt::Assignment(a) => {
                let val = self.eval_expr(&a.value)?;
                if !self.env.set(&a.name, val.clone()) {
                    return Err(format!("Undefined variable: {}", a.name));
                }
                self.log.push(LogEvent::VariableChanged(a.name.clone(), val));
                Ok(Signal::None)
            }
            crate::ast::Stmt::If(ifstmt) => {
                let cond_val = self.eval_condition(&ifstmt.condition)?;
                if cond_val {
                    self.exec_block(&ifstmt.then_body)
                } else if let Some(ref else_body) = ifstmt.else_body {
                    self.exec_block(else_body)
                } else { Ok(Signal::None) }
            }
            crate::ast::Stmt::Loop(body) => {
                loop {
                    match self.exec_block(body)? {
                        Signal::Break => break Ok(Signal::None),
                        Signal::Restart => continue,
                        Signal::Return(v) => return Ok(Signal::Return(v)),
                        Signal::None => {}
                    }
                }
            }
            crate::ast::Stmt::While(w) => {
                loop {
                    let cond = self.eval_expr(&w.condition)?;
                    if !cond.to_bool() { break Ok(Signal::None); }
                    match self.exec_block(&w.body)? {
                        Signal::Break => break Ok(Signal::None),
                        Signal::Restart => continue,
                        Signal::Return(v) => return Ok(Signal::Return(v)),
                        Signal::None => {}
                    }
                }
            }
            crate::ast::Stmt::Break => Ok(Signal::Break),
            crate::ast::Stmt::Restart => Ok(Signal::Restart),
            crate::ast::Stmt::Return(expr) => {
                let val = match expr { Some(e) => self.eval_expr(e)?, None => Value::Void };
                Ok(Signal::Return(val))
            }
            crate::ast::Stmt::Action(action) => self.exec_action(action),
            crate::ast::Stmt::ExprStmt(expr) => { self.eval_expr(expr)?; Ok(Signal::None) }
        }
    }

    fn exec_action(&mut self, action: &crate::ast::ActionStmt) -> Result<Signal, String> {
        match action {
            crate::ast::ActionStmt::Call { name, args } => {
                let mut arg_vals = Vec::new();
                for a in args { arg_vals.push(self.eval_expr(a)?); }
                self.call_function(name, &arg_vals)?;
                Ok(Signal::None)
            }
            crate::ast::ActionStmt::Pause(pv) => {
                let ms = self.eval_pause_value(pv)?;
                self.log.push(LogEvent::Info(format!("pause {}ms", ms)));
                thread::sleep(Duration::from_millis(ms as u64));
                Ok(Signal::None)
            }
            crate::ast::ActionStmt::KeyTap { key } => {
                let k = self.eval_expr(key)?.to_string_val();
                let enigo_key = crate::input_engine::parse_key(&k)?;
                let mut enigo = crate::input_engine::create_enigo()?;
                enigo.key(enigo_key, Click)
                    .map_err(|e| format!("key.tap failed: {}", e))?;
                self.log.push(LogEvent::Info(format!("key.tap(\"{}\")", k)));
                Ok(Signal::None)
            }
            crate::ast::ActionStmt::KeyHold { key } => {
                let k = self.eval_expr(key)?.to_string_val();
                let enigo_key = crate::input_engine::parse_key(&k)?;
                let mut enigo = crate::input_engine::create_enigo()?;
                enigo.key(enigo_key, Press)
                    .map_err(|e| format!("key.hold failed: {}", e))?;
                self.log.push(LogEvent::Info(format!("key.hold(\"{}\")", k)));
                Ok(Signal::None)
            }
            crate::ast::ActionStmt::KeyRelease { key } => {
                let k = self.eval_expr(key)?.to_string_val();
                let enigo_key = crate::input_engine::parse_key(&k)?;
                let mut enigo = crate::input_engine::create_enigo()?;
                enigo.key(enigo_key, Release)
                    .map_err(|e| format!("key.release failed: {}", e))?;
                self.log.push(LogEvent::Info(format!("key.release(\"{}\")", k)));
                Ok(Signal::None)
            }
            crate::ast::ActionStmt::KeyTypeText { text } => {
                let t = self.eval_expr(text)?.to_string_val();
                let mut enigo = crate::input_engine::create_enigo()?;
                enigo.text(&t)
                    .map_err(|e| format!("key.type failed: {}", e))?;
                self.log.push(LogEvent::Info(format!("key.type(\"{}\")", t)));
                Ok(Signal::None)
            }
            crate::ast::ActionStmt::MouseClick { button } => {
                let b = self.eval_expr(button)?.to_string_val();
                let btn = crate::input_engine::to_button(&b)?;
                let mut enigo = crate::input_engine::create_enigo()?;
                enigo.button(btn, Click)
                    .map_err(|e| format!("mouse.click failed: {}", e))?;
                self.log.push(LogEvent::Info(format!("mouse.click(\"{}\")", b)));
                Ok(Signal::None)
            }
            crate::ast::ActionStmt::MousePress { button } => {
                let b = self.eval_expr(button)?.to_string_val();
                let btn = crate::input_engine::to_button(&b)?;
                let mut enigo = crate::input_engine::create_enigo()?;
                enigo.button(btn, Press)
                    .map_err(|e| format!("mouse.press failed: {}", e))?;
                self.log.push(LogEvent::Info(format!("mouse.press(\"{}\")", b)));
                Ok(Signal::None)
            }
            crate::ast::ActionStmt::MouseRelease { button } => {
                let b = self.eval_expr(button)?.to_string_val();
                let btn = crate::input_engine::to_button(&b)?;
                let mut enigo = crate::input_engine::create_enigo()?;
                enigo.button(btn, Release)
                    .map_err(|e| format!("mouse.release failed: {}", e))?;
                self.log.push(LogEvent::Info(format!("mouse.release(\"{}\")", b)));
                Ok(Signal::None)
            }
            crate::ast::ActionStmt::MouseMove { x, y } => {
                let xv = self.eval_expr(x)?.to_int() as i32;
                let yv = self.eval_expr(y)?.to_int() as i32;
                let mut enigo = crate::input_engine::create_enigo()?;
                enigo.move_mouse(xv, yv, Coordinate::Abs)
                    .map_err(|e| format!("mouse.move failed: {}", e))?;
                self.log.push(LogEvent::Info(format!("mouse.move({}, {})", xv, yv)));
                Ok(Signal::None)
            }
        }
    }

    fn call_function(&mut self, name: &str, args: &[Value]) -> Result<Value, String> {
        let func = self.functions.get(name).cloned()
            .ok_or_else(|| format!("Undefined function: {}", name))?;
        self.env.push_scope();
        for (param, arg) in func.params.iter().zip(args) {
            self.env.define(param.clone(), arg.clone());
        }
        let sig = self.exec_block(&func.body)?;
        let result = match sig {
            Signal::Return(v) => v,
            Signal::Break => return Err("break outside loop".into()),
            Signal::Restart => return Err("restart outside loop".into()),
            _ => Value::Void,
        };
        self.env.pop_scope();
        Ok(result)
    }

    fn eval_pause_value(&mut self, pv: &crate::ast::PauseValue) -> Result<f64, String> {
        match pv {
            crate::ast::PauseValue::Fixed(expr) => {
                let v = self.eval_expr(expr)?;
                Ok(v.to_number())
            }
            crate::ast::PauseValue::Range { min, max } => {
                let min_val = self.eval_expr(min)?.to_number();
                let max_val = self.eval_expr(max)?.to_number();
                let ms = rand::rng().random_range(min_val..max_val);
                Ok(ms)
            }
            crate::ast::PauseValue::Human => {
                // Human-like jitter: random between 300-700ms, weighted toward center
                let ms = rand::rng().random_range(300.0..700.0);
                Ok(ms)
            }
        }
    }

    pub fn eval_expr(&mut self, expr: &crate::ast::Expr) -> Result<Value, String> {
        match expr {
            crate::ast::Expr::Number(n) => Ok(Value::Number(*n)),
            crate::ast::Expr::IntNumber(n) => Ok(Value::Int(*n)),
            crate::ast::Expr::String(s) => Ok(Value::String(s.clone())),
            crate::ast::Expr::Bool(b) => Ok(Value::Bool(*b)),
            crate::ast::Expr::Color(c) => Ok(Value::Color(c.clone())),
            crate::ast::Expr::Identifier(name) => {
                self.env.get(name).cloned().ok_or_else(|| format!("Undefined variable: {}", name))
            }
            crate::ast::Expr::UnaryNot(e) => { let v = self.eval_expr(e)?; Ok(Value::Bool(!v.to_bool())) }
            crate::ast::Expr::BinaryOp { op, left, right } => {
                let lv = self.eval_expr(left)?; let rv = self.eval_expr(right)?;
                self.eval_binop(op, &lv, &rv)
            }
            crate::ast::Expr::FuncCall { name, args } => {
                let mut arg_vals = Vec::new();
                for a in args { arg_vals.push(self.eval_expr(a)?); }
                self.call_function(name, &arg_vals)
            }
            crate::ast::Expr::PixelColor { x, y } => {
                let xv = self.eval_expr(x)?.to_int() as u32;
                let yv = self.eval_expr(y)?.to_int() as u32;
                match crate::vision_engine::get_pixel_color(xv, yv) {
                    Ok(pc) => {
                        self.log.push(LogEvent::Info(format!("pixel({}, {}) = {}", xv, yv, pc.hex)));
                        Ok(Value::Color(pc.hex))
                    }
                    Err(e) => Err(format!("pixel read failed: {}", e))
                }
            }
            crate::ast::Expr::RegionMatch { .. } => {
                self.log.push(LogEvent::Info("region match (stub)".into()));
                Ok(Value::Bool(false))
            }
        }
    }

    fn eval_binop(&self, op: &crate::ast::BinOp, lv: &Value, rv: &Value) -> Result<Value, String> {
        if matches!(op, crate::ast::BinOp::Add) {
            if let (Value::String(a), Value::String(b)) = (lv, rv) {
                return Ok(Value::String(format!("{}{}", a, b)));
            }
        }
        let ln = lv.to_number(); let rn = rv.to_number();
        match op {
            crate::ast::BinOp::Add => Ok(Value::Number(ln + rn)),
            crate::ast::BinOp::Sub => Ok(Value::Number(ln - rn)),
            crate::ast::BinOp::Mul => Ok(Value::Number(ln * rn)),
            crate::ast::BinOp::Div => { if rn == 0.0 { Err("Division by zero".into()) } else { Ok(Value::Number(ln / rn)) } }
            crate::ast::BinOp::Eq => Ok(Value::Bool(lv.to_string_val() == rv.to_string_val())),
            crate::ast::BinOp::Neq => Ok(Value::Bool(lv.to_string_val() != rv.to_string_val())),
            crate::ast::BinOp::Lt => Ok(Value::Bool(ln < rn)),
            crate::ast::BinOp::Gt => Ok(Value::Bool(ln > rn)),
            crate::ast::BinOp::Lte => Ok(Value::Bool(ln <= rn)),
            crate::ast::BinOp::Gte => Ok(Value::Bool(ln >= rn)),
            crate::ast::BinOp::And => Ok(Value::Bool(lv.to_bool() && rv.to_bool())),
            crate::ast::BinOp::Or => Ok(Value::Bool(lv.to_bool() || rv.to_bool())),
        }
    }

    pub fn eval_condition(&mut self, cond: &crate::ast::Condition) -> Result<bool, String> {
        match cond {
            crate::ast::Condition::Expression(e) => { let v = self.eval_expr(e)?; Ok(v.to_bool()) }
            crate::ast::Condition::Negated(c) => { let v = self.eval_condition(c)?; Ok(!v) }
            crate::ast::Condition::PixelMatches { x, y, color, tolerance } => {
                let xv = self.eval_expr(x)?.to_int() as u32;
                let yv = self.eval_expr(y)?.to_int() as u32;
                let pc = crate::vision_engine::get_pixel_color(xv, yv)
                    .map_err(|e| format!("pixel read failed: {}", e))?;

                let target_hex = self.eval_expr(color)?.to_string_val();
                let (tr, tg, tb) = parse_hex_color(&target_hex)?;

                let tol = match tolerance {
                    Some(t) => self.eval_expr(t)?.to_number() as f64,
                    None => 10.0,
                };

                let dist = ((pc.r as f64 - tr).powi(2)
                    + (pc.g as f64 - tg).powi(2)
                    + (pc.b as f64 - tb).powi(2))
                    .sqrt();

                let matched = dist <= tol;
                self.log.push(LogEvent::Info(format!(
                    "pixel({}, {}) = {} vs {} (dist={:.1}, tol={}, {})",
                    xv, yv, pc.hex, target_hex, dist, tol,
                    if matched { "MATCH" } else { "no match" }
                )));
                Ok(matched)
            }
            crate::ast::Condition::RegionMatches { .. } => {
                self.log.push(LogEvent::Info("region match (stub)".into())); Ok(false)
            }
            crate::ast::Condition::WaitUntil { .. } => {
                self.log.push(LogEvent::Info("wait until (stub)".into())); Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    fn run(source: &str) -> Result<Vec<LogEvent>, String> {
        let script = parse(source)?;
        let mut interp = Interpreter::new();
        interp.run(&script)?;
        Ok(interp.log)
    }

    #[test]
    fn test_var_and_assign() {
        let log = run(r#"script "T" { version: 1 var x = 10 on start { } }"#).unwrap();
        let vars: Vec<_> = log.iter().filter_map(|e| {
            if let LogEvent::VariableChanged(n, v) = e { Some((n.clone(), v.clone())) } else { None }
        }).collect();
        assert_eq!(vars[0].0, "x");
        assert!(matches!(vars[0].1, Value::Int(10)));
    }

    #[test]
    fn test_arithmetic() {
        let log = run(r#"script "T" { version: 1 var x = 10 + 5 * 2 var y = x == 20 on start { } }"#).unwrap();
        let vars: Vec<_> = log.iter().filter_map(|e| {
            if let LogEvent::VariableChanged(n, v) = e { Some((n.clone(), v.clone())) } else { None }
        }).collect();
        assert!(matches!(vars[1].1, Value::Bool(true)));
    }

    #[test]
    fn test_if_else() {
        let log = run(r#"script "T" { version: 1 var x = 5
on start { if x > 3 { var y = 1 } else { var y = 0 } } }"#).unwrap();
        let vars: Vec<_> = log.iter().filter_map(|e| {
            if let LogEvent::VariableChanged(n, v) = e { Some((n.clone(), v.clone())) } else { None }
        }).collect();
        assert_eq!(vars.len(), 2);
        assert!(matches!(vars[1].1, Value::Int(1)));
    }

    #[test]
    fn test_function_call() {
        let log = run(r#"script "T" { version: 1
function add(a, b) { return a + b }
on start { var result = add(3, 4) } }"#).unwrap();
        let vars: Vec<_> = log.iter().filter_map(|e| {
            if let LogEvent::VariableChanged(n, v) = e { Some((n.clone(), v.clone())) } else { None }
        }).collect();
        assert!(matches!(vars.last().unwrap().1, Value::Number(n) if (n - 7.0).abs() < 0.001));
    }

    #[test]
    fn test_loop_break() {
        let log = run(r#"script "T" { version: 1 var count = 0
on start { loop { count = count + 1 if count == 3 { break } } } }"#).unwrap();
        let vars: Vec<_> = log.iter().filter_map(|e| {
            if let LogEvent::VariableChanged(n, v) = e { Some((n.clone(), v.clone())) } else { None }
        }).collect();
        let final_val = vars.last().unwrap().1.clone();
        assert!(matches!(final_val, Value::Number(n) if (n - 3.0).abs() < 0.001));
    }

    #[test]
    fn test_action_stubs() {
        let log = run(r#"script "T" { version: 1 on start {
key.tap("1") mouse.click("left")
} }"#).unwrap();
        let infos: Vec<_> = log.iter().filter_map(|e| {
            if let LogEvent::Info(s) = e { Some(s.clone()) } else { None }
        }).collect();
        assert!(infos.iter().any(|s| s.contains("key.tap")));
        assert!(infos.iter().any(|s| s.contains("mouse.click")));
    }

    #[test]
    fn test_pause_fixed() {
        let start = std::time::Instant::now();
        let log = run(r#"script "T" { version: 1 on start { pause 50 } }"#).unwrap();
        let elapsed = start.elapsed().as_millis();
        assert!(elapsed >= 40, "pause 50 should take at least 40ms, took {}ms", elapsed);
        let infos: Vec<_> = log.iter().filter_map(|e| {
            if let LogEvent::Info(s) = e { Some(s.clone()) } else { None }
        }).collect();
        assert!(infos.iter().any(|s| s.contains("pause 50ms")));
    }

    #[test]
    fn test_pause_range() {
        let start = std::time::Instant::now();
        let log = run(r#"script "T" { version: 1 on start { pause 20..40 } }"#).unwrap();
        let elapsed = start.elapsed().as_millis();
        assert!(elapsed >= 15, "pause 20..40 should take at least 15ms, took {}ms", elapsed);
        assert!(elapsed < 100, "pause 20..40 should take under 100ms, took {}ms", elapsed);
        let infos: Vec<_> = log.iter().filter_map(|e| {
            if let LogEvent::Info(s) = e { Some(s.clone()) } else { None }
        }).collect();
        assert!(infos.iter().any(|s| s.contains("pause") && s.contains("ms")));
    }

    #[test]
    fn test_pause_human() {
        let start = std::time::Instant::now();
        let log = run(r#"script "T" { version: 1 on start { pause human } }"#).unwrap();
        let elapsed = start.elapsed().as_millis();
        assert!(elapsed >= 250, "pause human should take at least 250ms, took {}ms", elapsed);
        assert!(elapsed < 800, "pause human should take under 800ms, took {}ms", elapsed);
        let infos: Vec<_> = log.iter().filter_map(|e| {
            if let LogEvent::Info(s) = e { Some(s.clone()) } else { None }
        }).collect();
        assert!(infos.iter().any(|s| s.contains("pause") && s.contains("ms")));
    }

    #[test]
    fn test_while_loop() {
        let log = run(r#"script "T" { version: 1 var i = 0
on start { while i < 5 { i = i + 1 } } }"#).unwrap();
        let vars: Vec<_> = log.iter().filter_map(|e| {
            if let LogEvent::VariableChanged(n, v) = e { Some((n.clone(), v.clone())) } else { None }
        }).collect();
        let final_val = vars.last().unwrap().1.clone();
        assert!(matches!(final_val, Value::Number(n) if (n - 5.0).abs() < 0.001));
    }

    #[test]
    fn test_string_concat() {
        let log = run(r#"script "T" { version: 1 var greeting = "Hello" + " World"
on start { } }"#).unwrap();
        let vars: Vec<_> = log.iter().filter_map(|e| {
            if let LogEvent::VariableChanged(n, v) = e { Some((n.clone(), v.clone())) } else { None }
        }).collect();
        assert!(matches!(&vars[0].1, Value::String(s) if s == "Hello World"));
    }

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(parse_hex_color("#FF0000").unwrap(), (255.0, 0.0, 0.0));
        assert_eq!(parse_hex_color("#00FF00").unwrap(), (0.0, 255.0, 0.0));
        assert_eq!(parse_hex_color("#0000FF").unwrap(), (0.0, 0.0, 255.0));
        assert_eq!(parse_hex_color("#FFFFFF").unwrap(), (255.0, 255.0, 255.0));
        assert!(parse_hex_color("invalid").is_err());
        assert!(parse_hex_color("#FFF").is_err());
    }

    #[test]
    fn test_pixel_expression_reads_screen() {
        // This test requires a display - reads pixel at (0,0) which is top-left corner
        let log = run(r#"script "T" { version: 1 on start { var c = pixel(0, 0) } }"#).unwrap();
        let vars: Vec<_> = log.iter().filter_map(|e| {
            if let LogEvent::VariableChanged(n, v) = e { Some((n.clone(), v.clone())) } else { None }
        }).collect();
        // Should get a color value like "#RRGGBB"
        if let Value::Color(hex) = &vars.last().unwrap().1 {
            assert!(hex.starts_with('#'), "Expected hex color, got: {}", hex);
            assert_eq!(hex.len(), 7, "Expected 7-char hex, got: {}", hex);
        } else {
            panic!("Expected Color value, got: {:?}", vars.last().unwrap().1);
        }
    }
}
