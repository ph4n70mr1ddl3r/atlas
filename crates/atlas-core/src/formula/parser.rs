//! Formula Parser
//! 
//! Tokenizer and parser for formula expressions.

/// Token types for formula lexing
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    String(String),
    Boolean(bool),
    Identifier(String),
    Field(String),
    Function(String),
    Operator(String),
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Dot,
    And,
    Or,
    Not,
    If,
    Else,
    True,
    False,
    Null,
    Eof,
}

/// Lexer for tokenizing formula expressions
pub struct Lexer {
    input: String,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
            pos: 0,
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = vec![];
        
        while self.pos < self.input.len() {
            let ch = self.peek();
            
            if ch.is_whitespace() {
                self.advance();
                continue;
            }
            
            if ch.is_ascii_digit() {
                tokens.push(self.read_number());
                continue;
            }
            
            if ch == '"' {
                tokens.push(self.read_string());
                continue;
            }
            
            if ch.is_alphabetic() || ch == '_' {
                tokens.push(self.read_identifier());
                continue;
            }
            
            let ch = self.peek();
            let token = match ch {
                '(' => Some(Token::LParen),
                ')' => Some(Token::RParen),
                '[' => Some(Token::LBracket),
                ']' => Some(Token::RBracket),
                ',' => Some(Token::Comma),
                '.' => Some(Token::Dot),
                '+' => Some(Token::Operator("+".to_string())),
                '-' => Some(Token::Operator("-".to_string())),
                '*' => Some(Token::Operator("*".to_string())),
                '/' => Some(Token::Operator("/".to_string())),
                '%' => Some(Token::Operator("%".to_string())),
                '=' if self.peek_next() == '=' => {
                    self.advance();
                    Some(Token::Operator("==".to_string()))
                },
                '!' if self.peek_next() == '=' => {
                    self.advance();
                    Some(Token::Operator("!=".to_string()))
                },
                '>' if self.peek_next() == '=' => {
                    self.advance();
                    Some(Token::Operator(">=".to_string()))
                },
                '<' if self.peek_next() == '=' => {
                    self.advance();
                    Some(Token::Operator("<=".to_string()))
                },
                '>' => Some(Token::Operator(">".to_string())),
                '<' => Some(Token::Operator("<".to_string())),
                _ => return Err(format!("Unexpected character: {}", ch)),
            };
            
            if let Some(tok) = token {
                tokens.push(tok);
            }
            self.advance();
        }
        
        tokens.push(Token::Eof);
        Ok(tokens)
    }
    
    fn peek(&self) -> char {
        self.input.chars().nth(self.pos).unwrap_or('\0')
    }
    
    fn peek_next(&self) -> char {
        self.input.chars().nth(self.pos + 1).unwrap_or('\0')
    }
    
    fn advance(&mut self) {
        self.pos += 1;
    }
    
    fn read_number(&mut self) -> Token {
        let start = self.pos;
        let mut has_dot = false;
        
        while self.pos < self.input.len() {
            let ch = self.peek();
            if ch.is_ascii_digit() {
                self.advance();
            } else if ch == '.' && !has_dot {
                has_dot = true;
                self.advance();
            } else {
                break;
            }
        }
        
        let num_str = &self.input[start..self.pos];
        Token::Number(num_str.parse().unwrap_or(0.0))
    }
    
    fn read_string(&mut self) -> Token {
        self.advance(); // Skip opening quote
        let start = self.pos;
        
        while self.pos < self.input.len() && self.peek() != '"' {
            if self.peek() == '\\' {
                self.advance(); // Skip escape char
            }
            self.advance();
        }
        
        let s = self.input[start..self.pos].to_string();
        self.advance(); // Skip closing quote
        Token::String(s)
    }
    
    fn read_identifier(&mut self) -> Token {
        let start = self.pos;
        
        while self.pos < self.input.len() {
            let ch = self.peek();
            if ch.is_alphanumeric() || ch == '_' || ch == '.' {
                self.advance();
            } else {
                break;
            }
        }
        
        let id = &self.input[start..self.pos];
        
        // Check for keywords
        match id.to_uppercase().as_str() {
            "AND" => Token::And,
            "OR" => Token::Or,
            "NOT" => Token::Not,
            "IF" => Token::If,
            "TRUE" => Token::Boolean(true),
            "FALSE" => Token::Boolean(false),
            "NULL" => Token::Null,
            _ if id.contains('.') => Token::Field(id.to_string()),
            _ if self.peek() == '(' => Token::Function(id.to_string()),
            _ => Token::Identifier(id.to_string()),
        }
    }
}

/// Formula parser
pub struct FormulaParser {
    tokens: Vec<Token>,
    pos: usize,
}

impl FormulaParser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }
    
    pub fn parse(&mut self) -> Result<AstNode, String> {
        self.parse_expression()
    }
    
    fn parse_expression(&mut self) -> Result<AstNode, String> {
        self.parse_or()
    }
    
    fn parse_or(&mut self) -> Result<AstNode, String> {
        let mut left = self.parse_and()?;
        
        while self.current() == &Token::Or {
            self.advance();
            let right = self.parse_and()?;
            left = AstNode::BinaryOp { op: "OR".to_string(), left: Box::new(left), right: Box::new(right) };
        }
        
        Ok(left)
    }
    
    fn parse_and(&mut self) -> Result<AstNode, String> {
        let mut left = self.parse_comparison()?;
        
        while self.current() == &Token::And {
            self.advance();
            let right = self.parse_comparison()?;
            left = AstNode::BinaryOp { op: "AND".to_string(), left: Box::new(left), right: Box::new(right) };
        }
        
        Ok(left)
    }
    
    fn parse_comparison(&mut self) -> Result<AstNode, String> {
        let mut left = self.parse_addition()?;
        
        loop {
            let op = match self.current() {
                Token::Operator(op) if op == "==" || op == "!=" || op == ">" || op == ">=" || op == "<" || op == "<=" => {
                    let result = op.clone();
                    self.advance();
                    result
                }
                _ => break,
            };
            
            let right = self.parse_addition()?;
            left = AstNode::BinaryOp { op, left: Box::new(left), right: Box::new(right) };
        }
        
        Ok(left)
    }
    
    fn parse_addition(&mut self) -> Result<AstNode, String> {
        let mut left = self.parse_multiplication()?;
        
        loop {
            let op = match self.current() {
                Token::Operator(op) if op == "+" || op == "-" => {
                    let result = op.clone();
                    self.advance();
                    result
                }
                _ => break,
            };
            
            let right = self.parse_multiplication()?;
            left = AstNode::BinaryOp { op, left: Box::new(left), right: Box::new(right) };
        }
        
        Ok(left)
    }
    
    fn parse_multiplication(&mut self) -> Result<AstNode, String> {
        let mut left = self.parse_unary()?;
        
        loop {
            let op = match self.current() {
                Token::Operator(op) if op == "*" || op == "/" || op == "%" => {
                    let result = op.clone();
                    self.advance();
                    result
                }
                _ => break,
            };
            
            let right = self.parse_unary()?;
            left = AstNode::BinaryOp { op, left: Box::new(left), right: Box::new(right) };
        }
        
        Ok(left)
    }
    
    fn parse_unary(&mut self) -> Result<AstNode, String> {
        if self.current() == &Token::Not || self.current() == &Token::Operator("-".to_string()) {
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(AstNode::UnaryOp { op: "NOT".to_string(), operand: Box::new(operand) });
        }
        
        self.parse_primary()
    }
    
    fn parse_primary(&mut self) -> Result<AstNode, String> {
        let token = self.current().clone();
        
        match token {
            Token::Number(n) => {
                self.advance();
                Ok(AstNode::Number(n))
            }
            Token::String(s) => {
                self.advance();
                Ok(AstNode::String(s))
            }
            Token::Boolean(b) => {
                self.advance();
                Ok(AstNode::Boolean(b))
            }
            Token::Null => {
                self.advance();
                Ok(AstNode::Null)
            }
            Token::Identifier(id) => {
                self.advance();
                Ok(AstNode::Identifier(id))
            }
            Token::Field(field) => {
                self.advance();
                Ok(AstNode::Field(field))
            }
            Token::Function(name) => {
                self.advance();
                self.expect(&Token::LParen)?;
                let args = self.parse_arguments()?;
                self.expect(&Token::RParen)?;
                Ok(AstNode::Function { name, args })
            }
            Token::If => {
                self.advance();
                self.expect(&Token::LParen)?;
                let condition = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let then_branch = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let else_branch = self.parse_expression()?;
                self.expect(&Token::RParen)?;
                Ok(AstNode::If { condition: Box::new(condition), then: Box::new(then_branch), else_: Box::new(else_branch) })
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            _ => Err(format!("Unexpected token: {:?}", token)),
        }
    }
    
    fn parse_arguments(&mut self) -> Result<Vec<AstNode>, String> {
        let mut args = vec![];
        
        if self.current() != &Token::RParen {
            args.push(self.parse_expression()?);
            
            while self.current() == &Token::Comma {
                self.advance();
                args.push(self.parse_expression()?);
            }
        }
        
        Ok(args)
    }
    
    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }
    
    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }
    
    fn expect(&mut self, token: &Token) -> Result<(), String> {
        if self.current() == token {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected {:?}, got {:?}", token, self.current()))
        }
    }
}

/// AST node types
#[derive(Debug, Clone)]
pub enum AstNode {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Identifier(String),
    Field(String),
    BinaryOp { op: String, left: Box<AstNode>, right: Box<AstNode> },
    UnaryOp { op: String, operand: Box<AstNode> },
    Function { name: String, args: Vec<AstNode> },
    If { condition: Box<AstNode>, then: Box<AstNode>, else_: Box<AstNode> },
}

/// Parse a formula string into an AST
#[allow(dead_code)]
pub fn parse_formula(input: &str) -> Result<AstNode, String> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = FormulaParser::new(tokens);
    parser.parse()
}

/// Extract field dependencies from a formula
#[allow(dead_code)]
pub fn extract_dependencies(node: &AstNode) -> Vec<String> {
    let mut deps = vec![];
    
    fn collect(node: &AstNode, deps: &mut Vec<String>) {
        match node {
            AstNode::Field(name)
                if !deps.contains(name) => {
                    deps.push(name.clone());
                }
            AstNode::BinaryOp { left, right, .. } => {
                collect(left, deps);
                collect(right, deps);
            }
            AstNode::UnaryOp { operand, .. } => {
                collect(operand, deps);
            }
            AstNode::Function { args, .. } => {
                for arg in args {
                    collect(arg, deps);
                }
            }
            AstNode::If { condition, then, else_ } => {
                collect(condition, deps);
                collect(then, deps);
                collect(else_, deps);
            }
            _ => {}
        }
    }
    
    collect(node, &mut deps);
    deps
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lexer_numbers() {
        let mut lexer = Lexer::new("123 45.67 0.5");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 4); // Two numbers + Eof
        assert_eq!(tokens[0], Token::Number(123.0));
        assert_eq!(tokens[1], Token::Number(45.67));
        assert_eq!(tokens[2], Token::Number(0.5));
    }
    
    #[test]
    fn test_lexer_strings() {
        let mut lexer = Lexer::new(r#""hello" "world""#);
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0], Token::String("hello".to_string()));
        assert_eq!(tokens[1], Token::String("world".to_string()));
    }
    
    #[test]
    fn test_lexer_operators() {
        let mut lexer = Lexer::new("== != >= <=");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0], Token::Operator("==".to_string()));
        assert_eq!(tokens[1], Token::Operator("!=".to_string()));
        assert_eq!(tokens[2], Token::Operator(">=".to_string()));
        assert_eq!(tokens[3], Token::Operator("<=".to_string()));
    }
    
    #[test]
    fn test_parser_field() {
        let ast = parse_formula("order.amount").unwrap();
        assert!(matches!(ast, AstNode::Field(f) if f == "order.amount"));
    }
    
    #[test]
    fn test_parser_arithmetic() {
        let ast = parse_formula("a + b * c").unwrap();
        
        assert!(matches!(ast, AstNode::BinaryOp { op, .. } if op == "+"));
    }
    
    #[test]
    fn test_parser_function() {
        let ast = parse_formula("SUM(amount)").unwrap();
        
        assert!(matches!(ast, AstNode::Function { name, args } if name == "SUM" && args.len() == 1));
    }
    
    #[test]
    fn test_dependencies() {
        let ast = parse_formula("line.quantity * line.price + line.tax").unwrap();
        let deps = extract_dependencies(&ast);
        
        assert!(deps.contains(&"line.quantity".to_string()));
        assert!(deps.contains(&"line.price".to_string()));
        assert!(deps.contains(&"line.tax".to_string()));
    }
}
