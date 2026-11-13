// SPDX-License-Identifier: Apache-2.0

const INPUT_PREFIX: &str = "I_";
const OUT_VAR: &str = "Sink";

#[derive(Clone, Copy, Debug)]
pub struct Location(pub usize, pub usize);

pub mod lex {
    use crate::Location;

    pub fn lex(raw: &str) -> Result<Vec<LocationToken>, LexError> {
        let mut tokens = vec![];
        let mut term = String::new();

        let mut line = 1;
        let mut column = 0;
        for c in raw.chars() {
            column += 1;

            let is_end_of_term = c == ' '
                || c == '\t'
                || c == '\n'
                || c == ';'
                || c == '{'
                || c == '}'
                || c == '('
                || c == ')'
                || c == '['
                || c == ']';

            if is_end_of_term {
                if !term.is_empty() {
                    let location = Location(line, column - term.len());
                    let token = term_to_token(&term).map_err(|kind| LexError(location, kind))?;
                    tokens.push(LocationToken(location, token));
                    term.clear();
                }

                let token = match c {
                    ';' => Some(Token::Semi),
                    '{' => Some(Token::Bracket(Bracket::CurlyOpen)),
                    '}' => Some(Token::Bracket(Bracket::CurlyClose)),
                    '(' => Some(Token::Bracket(Bracket::RoundOpen)),
                    ')' => Some(Token::Bracket(Bracket::RoundClose)),
                    '[' => Some(Token::Bracket(Bracket::SquareOpen)),
                    ']' => Some(Token::Bracket(Bracket::SquareClose)),
                    ' ' | '\t' | '\n' => None,
                    _ => unreachable!(),
                };

                if let Some(token) = token {
                    let location = Location(line, column);
                    tokens.push(LocationToken(location, token));
                }
            }

            if !is_end_of_term {
                term.push(c);
            }

            if c == '\n' {
                column = 0;
                line += 1;
            }
        }

        Ok(tokens)
    }

    fn term_to_token(raw: &str) -> Result<Token, ErrorKind> {
        let token = match raw {
            "alloc" => Token::Keyword(Keyword::Allocate),
            "alloc_C" => Token::Keyword(Keyword::AllocateC),
            "call" => Token::Keyword(Keyword::Call),
            "else" => Token::Keyword(Keyword::Else),
            "fn" => Token::Keyword(Keyword::Function),
            "if" => Token::Keyword(Keyword::If),
            "nil" => Token::Keyword(Keyword::Nil),
            "skip" => Token::Keyword(Keyword::Skip),
            "while" => Token::Keyword(Keyword::While),

            ":=" => Token::Symbol(Symbol::Assign),
            "<-" => Token::Symbol(Symbol::Gets),

            "+" => Token::Symbol(Symbol::Add),
            "-" => Token::Symbol(Symbol::Subtract),
            "==" => Token::Symbol(Symbol::Equals),
            "&&" => Token::Symbol(Symbol::LogicalAnd),
            "!=" => Token::Symbol(Symbol::NotEquals),

            n if is_numeric(n) => Token::Number(n.parse().unwrap()),
            x if is_ident(x) => Token::Ident(x.to_owned()),

            // UNEXPECTED
            _ => return Err(ErrorKind::UnexpectedTerm(raw.to_owned())),
        };

        Ok(token)
    }

    fn is_numeric(raw: &str) -> bool {
        for c in raw.chars() {
            if c.is_numeric() {
                continue;
            }

            return false;
        }

        true
    }

    fn is_ident(raw: &str) -> bool {
        let c = raw.chars().next().unwrap();
        if !c.is_ascii_alphabetic() && c != '_' {
            return false;
        }

        for c in raw.chars().skip(1) {
            if c.is_ascii_alphanumeric() || c == '_' {
                continue;
            }

            return false;
        }

        true
    }

    #[derive(Debug)]
    pub struct LocationToken(pub Location, pub Token);

    #[derive(Debug, Eq, PartialEq)]
    pub enum Token {
        Bracket(Bracket),
        Ident(String),
        Keyword(Keyword),
        Number(usize),
        Semi,
        Symbol(Symbol),
    }

    #[derive(Debug, Eq, PartialEq)]
    pub enum Bracket {
        CurlyOpen,
        CurlyClose,
        RoundOpen,
        RoundClose,
        SquareOpen,
        SquareClose,
    }

    #[derive(Debug, Eq, PartialEq)]
    pub enum Keyword {
        Allocate,
        AllocateC,
        Call,
        Else,
        Function,
        If,
        Nil,
        Skip,
        While,
    }

    #[derive(Debug, Eq, PartialEq)]
    pub enum Symbol {
        Add,
        Assign,
        Equals,
        Gets,
        LogicalAnd,
        NotEquals,
        Subtract,
    }

    #[derive(Debug)]
    pub struct LexError(Location, ErrorKind);

    impl std::error::Error for LexError {}

    impl std::fmt::Display for LexError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "[LEX] ({}:{}) {}", self.0.0, self.0.1, self.1)
        }
    }

    #[derive(Debug)]
    pub enum ErrorKind {
        UnexpectedTerm(String),
    }

    impl std::fmt::Display for ErrorKind {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ErrorKind::UnexpectedTerm(term) => write!(f, "Unexpected term '{term}'"),
            }
        }
    }
}

pub mod parse {
    #[cfg(feature = "wasm")]
    use wasm_bindgen::prelude::*;

    use crate::INPUT_PREFIX;
    use crate::Location;
    use crate::lex::{Bracket, Keyword, LocationToken, Symbol, Token};

    use std::collections::HashSet;

    pub fn parse(mut tokens: Vec<LocationToken>) -> Result<Program, ParseError> {
        tokens.reverse();
        let command = parse_commands(&mut tokens).map_err(ParseError)?;
        Ok(Program(command))
    }

    fn parse_assign(tokens: &mut Vec<LocationToken>) -> Result<LocationCommand, ErrorKind> {
        let (location, identifier) = parse_identifier(tokens)?;
        let token = tokens.pop().ok_or(ErrorKind::OutOfTokens)?;
        let command = match token.1 {
            // <x> ':=' <e> | <x> ':=' <x_o> '[' <e_i> ']'
            Token::Symbol(Symbol::Assign) => {
                let peek = if tokens.len() > 2 {
                    tokens.get(tokens.len() - 2)
                } else {
                    None
                };

                if let Some(token) = peek
                    && token.1 == Token::Bracket(Bracket::SquareOpen)
                {
                    let (_, object) = parse_identifier(tokens)?;
                    let _ = expect_token(tokens, Token::Bracket(Bracket::SquareOpen))?;
                    let index = parse_expression(tokens)?;
                    let _ = expect_token(tokens, Token::Bracket(Bracket::SquareClose))?;
                    Command::Fetch(identifier, object, index)
                } else {
                    let expression = parse_expression(tokens)?;
                    Command::Assign(identifier, expression)
                }
            }

            // <x> '[' <e_1> ']' ':=' <e>
            Token::Bracket(Bracket::SquareOpen) => {
                let index = parse_expression(tokens)?;
                let _ = expect_token(tokens, Token::Bracket(Bracket::SquareClose))?;
                let _ = expect_token(tokens, Token::Symbol(Symbol::Assign))?;
                let expression = parse_expression(tokens)?;
                Command::Store(identifier, index, expression)
            }

            // <x> '<-' 'alloc' | <x> '<-' 'alloc_C'
            Token::Symbol(Symbol::Gets) => {
                let token = tokens.pop().ok_or(ErrorKind::OutOfTokens)?;
                match token.1 {
                    // <x> '<-' 'alloc'
                    Token::Keyword(Keyword::Allocate) => Command::Allocate(identifier),

                    // <x> '<-' 'alloc_C'
                    Token::Keyword(Keyword::AllocateC) => Command::AllocateC(identifier),

                    // UNEXPECTED
                    _ => return Err(ErrorKind::UnexpectedToken(token)),
                }
            }

            // UNEXPECTED
            _ => return Err(ErrorKind::UnexpectedToken(token)),
        };

        Ok(LocationCommand(location, command))
    }

    fn parse_command(tokens: &mut Vec<LocationToken>) -> Result<LocationCommand, ErrorKind> {
        let token = tokens.last().ok_or(ErrorKind::OutOfTokens)?;
        let command = match token.1 {
            // 'skip'
            Token::Keyword(Keyword::Skip) => parse_skip(tokens)?,

            // <x> ':=' <e> |
            // <x> '[' <e_i> ']' ':=' <e> | <x> ':=' <x_o> '[' <e_i> ']' |
            // <x> '<-' 'alloc' | <x> '<-' 'alloc_C'
            Token::Ident(..) => parse_assign(tokens)?,

            // 'call' <x>
            Token::Keyword(Keyword::Call) => parse_call(tokens)?,

            // 'fn' <x> '{' <c> '}'
            Token::Keyword(Keyword::Function) => parse_function(tokens)?,

            // 'if' <e> '{' <c1> '}' 'else' '{' <c2> '}'
            Token::Keyword(Keyword::If) => parse_if(tokens)?,

            // 'while' <e> '{' <c> '}'
            Token::Keyword(Keyword::While) => parse_while(tokens)?,

            // UNEXPECTED
            _ => return Err(ErrorKind::UnexpectedToken(tokens.pop().unwrap())),
        };

        Ok(command)
    }

    fn parse_call(tokens: &mut Vec<LocationToken>) -> Result<LocationCommand, ErrorKind> {
        // 'call'
        let location = expect_token(tokens, Token::Keyword(Keyword::Call))?;

        // <x>
        let (_, identifier) = parse_identifier(tokens)?;

        let command = Command::Call(identifier);
        Ok(LocationCommand(location, command))
    }

    fn parse_commands(tokens: &mut Vec<LocationToken>) -> Result<LocationCommand, ErrorKind> {
        // <c1> ';' <c2>
        let mut command = parse_command(tokens)?;
        while tokens
            .last()
            .is_some_and(|token| matches!(token.1, Token::Semi))
        {
            let token = tokens.pop().unwrap();
            if tokens.last().is_none()
                || tokens
                    .last()
                    .is_some_and(|token| matches!(token.1, Token::Bracket(Bracket::CurlyClose)))
            {
                break;
            }

            let c1 = command;
            let c2 = parse_command(tokens)?;
            let cmd = Command::Sequence(Box::new(c1), Box::new(c2));
            command = LocationCommand(token.0, cmd);
        }

        Ok(command)
    }

    fn parse_expression(tokens: &mut Vec<LocationToken>) -> Result<Expression, ErrorKind> {
        let token = tokens.pop().ok_or(ErrorKind::OutOfTokens)?;
        let mut expression = match token.1 {
            // <v>  (<n> | 'nil')
            Token::Number(number) => Expression::Value(Value::Number(number)),
            Token::Keyword(Keyword::Nil) => Expression::Value(Value::Nil),

            // <x>
            Token::Ident(identifier) => Expression::Variable(Identifier(identifier)),

            // '(' <e> ')'
            Token::Bracket(Bracket::RoundOpen) => {
                let expression = parse_expression(tokens)?;
                let _ = expect_token(tokens, Token::Bracket(Bracket::RoundClose))?;
                expression
            }

            // UNEXPECTED
            _ => return Err(ErrorKind::UnexpectedToken(token)),
        };

        // <e1> <op> <e2>
        let peek = tokens.last();
        if let Some(token) = peek {
            let op = match token.1 {
                // '+'
                Token::Symbol(Symbol::Add) => Some(Operation::Add),

                // '-'
                Token::Symbol(Symbol::Subtract) => Some(Operation::Subtract),

                // '=='
                Token::Symbol(Symbol::Equals) => Some(Operation::Equals),

                // '!='
                Token::Symbol(Symbol::NotEquals) => Some(Operation::NotEquals),

                // '&&'
                Token::Symbol(Symbol::LogicalAnd) => Some(Operation::LogicalAnd),

                // n/a
                _ => None,
            };

            if let Some(op) = op {
                let lhs = expression;
                let _ = tokens.pop();
                let rhs = parse_expression(tokens)?;
                expression = Expression::Operation(Box::new(lhs), op, Box::new(rhs));
            }
        }

        Ok(expression)
    }

    fn parse_function(tokens: &mut Vec<LocationToken>) -> Result<LocationCommand, ErrorKind> {
        // 'fn'
        let location = expect_token(tokens, Token::Keyword(Keyword::Function))?;

        // <x>
        let (_, identifier) = parse_identifier(tokens)?;

        // '{'
        let _ = expect_token(tokens, Token::Bracket(Bracket::CurlyOpen))?;

        // <c>
        let body = parse_commands(tokens)?;

        // '}'
        let _ = expect_token(tokens, Token::Bracket(Bracket::CurlyClose))?;

        // Push a ';' to continue parsing the next command, if any
        tokens.push(LocationToken(Location(0, 0), Token::Semi));

        let command = Command::Function(identifier, Box::new(body));
        Ok(LocationCommand(location, command))
    }

    fn parse_identifier(
        tokens: &mut Vec<LocationToken>,
    ) -> Result<(Location, Identifier), ErrorKind> {
        // '_' | 'a' | 'b' | 'c' | ... | '_1' | ...
        let token = tokens.pop().ok_or(ErrorKind::OutOfTokens)?;
        if let Token::Ident(ident) = token.1 {
            Ok((token.0, Identifier(ident)))
        } else {
            Err(ErrorKind::UnexpectedToken(token))
        }
    }

    fn parse_if(tokens: &mut Vec<LocationToken>) -> Result<LocationCommand, ErrorKind> {
        // 'if'
        let location = expect_token(tokens, Token::Keyword(Keyword::If))?;

        // <e>
        let guard = parse_expression(tokens)?;

        // '{'
        let _ = expect_token(tokens, Token::Bracket(Bracket::CurlyOpen))?;

        // <c1>
        let if_branch = parse_commands(tokens)?;

        // '}'
        let _ = expect_token(tokens, Token::Bracket(Bracket::CurlyClose))?;

        // 'else'
        let _ = expect_token(tokens, Token::Keyword(Keyword::Else))?;

        // '{'
        let _ = expect_token(tokens, Token::Bracket(Bracket::CurlyOpen))?;

        // <c2>
        let else_branch = parse_commands(tokens)?;

        // '}'
        let _ = expect_token(tokens, Token::Bracket(Bracket::CurlyClose))?;

        // Push a ';' to continue parsing the next command, if any
        tokens.push(LocationToken(Location(0, 0), Token::Semi));

        let command = Command::If(guard, Box::new(if_branch), Box::new(else_branch));
        Ok(LocationCommand(location, command))
    }

    fn parse_skip(tokens: &mut Vec<LocationToken>) -> Result<LocationCommand, ErrorKind> {
        // 'skip'
        let location = expect_token(tokens, Token::Keyword(Keyword::Skip))?;
        Ok(LocationCommand(location, Command::Skip))
    }

    fn parse_while(tokens: &mut Vec<LocationToken>) -> Result<LocationCommand, ErrorKind> {
        // 'while'
        let location = expect_token(tokens, Token::Keyword(Keyword::While))?;

        // <e>
        let guard = parse_expression(tokens)?;

        // '{'
        let _ = expect_token(tokens, Token::Bracket(Bracket::CurlyOpen))?;

        // <c>
        let body = parse_commands(tokens)?;

        // '}'
        let _ = expect_token(tokens, Token::Bracket(Bracket::CurlyClose))?;

        // Push a ';' to continue parsing the next command, if any
        tokens.push(LocationToken(Location(0, 0), Token::Semi));

        let command = Command::While(guard, Box::new(body));
        Ok(LocationCommand(location, command))
    }

    fn expect_token(tokens: &mut Vec<LocationToken>, want: Token) -> Result<Location, ErrorKind> {
        let got = tokens.pop().ok_or(ErrorKind::OutOfTokens)?;
        if got.1 == want {
            Ok(got.0)
        } else {
            Err(ErrorKind::UnexpectedToken(got))
        }
    }

    pub fn detect_inputs(program: &Program) -> Vec<String> {
        let mut inputs = variables_in_command(&program.0)
            .into_iter()
            .filter(|x| x.starts_with(INPUT_PREFIX))
            .collect::<Vec<_>>();
        inputs.sort();
        inputs
    }

    fn variables_in_command(command: &LocationCommand) -> HashSet<String> {
        match &command.1 {
            Command::Skip => HashSet::new(),
            Command::Sequence(c1, c2) => {
                let mut inputs = variables_in_command(c1);
                inputs.extend(variables_in_command(c2));
                inputs
            }
            Command::Assign(_, e) => variables_in_expression(e),
            Command::Fetch(_, _, e_i) => variables_in_expression(e_i),
            Command::Store(_, e_i, e) => {
                let mut inputs = variables_in_expression(e_i);
                inputs.extend(variables_in_expression(e));
                inputs
            }
            Command::Allocate(..) => HashSet::new(),
            Command::AllocateC(..) => HashSet::new(),
            Command::Call(..) => HashSet::new(),
            Command::Function(_, c) => variables_in_command(c),
            Command::If(e, c1, c2) => {
                let mut inputs = variables_in_expression(e);
                inputs.extend(variables_in_command(c1));
                inputs.extend(variables_in_command(c2));
                inputs
            }
            Command::While(e, c) => {
                let mut inputs = variables_in_expression(e);
                inputs.extend(variables_in_command(c));
                inputs
            }
        }
    }

    fn variables_in_expression(expression: &Expression) -> HashSet<String> {
        match expression {
            Expression::Value(..) => HashSet::new(),
            Expression::Variable(x) => {
                let mut variables = HashSet::new();
                variables.insert(x.0.clone());
                variables
            }
            Expression::Operation(e1, _, e2) => {
                let mut variables = variables_in_expression(e1);
                variables.extend(variables_in_expression(e2));
                variables
            }
        }
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[derive(Clone, Debug)]
    pub struct Program(pub(crate) LocationCommand);

    #[derive(Clone, Debug)]
    pub struct LocationCommand(pub(crate) Location, pub(crate) Command);

    impl std::fmt::Display for LocationCommand {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.1)
        }
    }

    #[derive(Clone, Debug)]
    pub enum Command {
        Skip,
        Sequence(Box<LocationCommand>, Box<LocationCommand>),
        Assign(Identifier, Expression),
        Fetch(Identifier, Identifier, Expression),
        Store(Identifier, Expression, Expression),
        Allocate(Identifier),
        AllocateC(Identifier),
        Call(Identifier),
        Function(Identifier, Box<LocationCommand>),
        If(Expression, Box<LocationCommand>, Box<LocationCommand>),
        While(Expression, Box<LocationCommand>),
    }

    impl std::fmt::Display for Command {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Skip => write!(f, "skip"),
                Self::Sequence(c1, c2) => write!(f, "{c1}; {c2}"),
                Self::Assign(x, e) => write!(f, "{x} := {e}"),
                Self::Fetch(x, x_o, e_i) => write!(f, "{x} := {x_o}[{e_i}]"),
                Self::Store(x, e_i, e) => write!(f, "{x}[{e_i}] := {e}"),
                Self::Allocate(x) => write!(f, "{x} <- alloc"),
                Self::AllocateC(x) => write!(f, "{x} <- alloc_C"),
                Self::Call(x) => write!(f, "call {x}"),
                Self::Function(x, c) => write!(f, "fn {x} {{ {c} }}"),
                Self::If(e, c1, c2) => write!(f, "if {e} {{ {c1} }} else {{ {c2} }}"),
                Self::While(e, c) => write!(f, "while {e} {{ {c} }}"),
            }
        }
    }

    #[derive(Clone, Debug)]
    pub enum Expression {
        Value(Value),
        Variable(Identifier),
        Operation(Box<Expression>, Operation, Box<Expression>),
    }

    impl std::fmt::Display for Expression {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self {
                Self::Value(v) => write!(f, "{v}"),
                Self::Variable(x) => write!(f, "{x}"),
                Self::Operation(e1, op, e2) => write!(f, "{e1} {op} {e2}"),
            }
        }
    }

    #[derive(Clone, Debug)]
    pub enum Value {
        Number(usize),
        Nil,
    }

    impl std::fmt::Display for Value {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self {
                Self::Number(n) => write!(f, "{n}"),
                Self::Nil => write!(f, "nil"),
            }
        }
    }

    #[derive(Clone, Debug)]
    pub enum Operation {
        Add,
        Equals,
        LogicalAnd,
        NotEquals,
        Subtract,
    }

    impl std::fmt::Display for Operation {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self {
                Self::Add => write!(f, "+"),
                Self::Equals => write!(f, "=="),
                Self::LogicalAnd => write!(f, "&&"),
                Self::NotEquals => write!(f, "!="),
                Self::Subtract => write!(f, "-"),
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct Identifier(pub String);

    impl std::fmt::Display for Identifier {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    #[derive(Debug)]
    pub struct ParseError(ErrorKind);

    impl std::error::Error for ParseError {}

    impl std::fmt::Display for ParseError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "[PARSE] {}", self.0)
        }
    }

    #[derive(Debug)]
    pub enum ErrorKind {
        OutOfTokens,
        UnexpectedToken(LocationToken),
    }

    impl std::fmt::Display for ErrorKind {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::OutOfTokens => write!(f, "Unexpected end of file"),
                Self::UnexpectedToken(token) => write!(
                    f,
                    "({}:{}) Unexpected token {:?}",
                    token.0.0, token.0.1, token.1
                ),
            }
        }
    }
}

pub mod interpret {
    #[cfg(feature = "wasm")]
    use wasm_bindgen::prelude::*;

    use crate::Location as FileLocation;
    use crate::parse::{Command, Expression, Identifier, LocationCommand, Operation, Program};
    use crate::state::{Location, State, Table, Value};
    use crate::trace::{Observation, Trace};

    pub fn step(
        configuration: Configuration,
        trace: &mut Trace,
    ) -> Result<Configuration, RuntimeError> {
        let configuration = interpret_command(configuration, trace)?;
        Ok(configuration)
    }

    fn interpret_command(
        configuration: Configuration,
        trace: &mut Trace,
    ) -> Result<Configuration, RuntimeError> {
        let LocationCommand(location, command) = configuration.0.unwrap();
        let mut state = configuration.1;

        let next = match command {
            // E-Skip
            Command::Skip => {
                // <ε, 𝜎, H, F, C>
                Configuration(None, state)
            }

            // E-Seq | E-Seq-ε
            Command::Sequence(c1, c2) => {
                // <c1, 𝜎, H, F, C>
                let c1_configuration = Configuration(Some(*c1), state);

                // -> <c1'|ε, 𝜎', H', F', C'>
                let Configuration(new_c1, new_state) = interpret_command(c1_configuration, trace)?;

                match new_c1 {
                    // E-Seq
                    Some(c1) => {
                        let command = LocationCommand(c1.0, Command::Sequence(Box::new(c1), c2));
                        Configuration(Some(command), new_state)
                    }

                    // E-Seq-ε
                    None => Configuration(Some(*c2), new_state),
                }
            }

            // E-Assign
            Command::Assign(x, e) => {
                // v = [[e]]_S
                let v = interpret_expression(&e, &state).map_err(attach_location(location))?;

                // 𝜎[x |-> v]
                state.insert_store(&x, v);

                // <ε, 𝜎', H, F, C>
                Configuration(None, state)
            }

            // E-Store
            Command::Store(x, e_i, e) => {
                // l_o = S(x)
                let Ok(l_o) = state.get_variable(&x) else {
                    return Err(RuntimeError(location, ErrorKind::UnknownVariable(x)));
                };

                // l_o \in Loc_H
                let Value::Location(Location::Heap(l_o)) = l_o else {
                    return Err(RuntimeError(
                        location,
                        ErrorKind::IndexNonLocation(x, e_i, l_o),
                    ));
                };
                let l_o = Location::Heap(l_o);

                // n = [[e_i]]_S
                let n = interpret_expression(&e_i, &state).map_err(attach_location(location))?;

                // n \in Z
                let Value::Number(n) = n else {
                    return Err(RuntimeError(
                        location,
                        ErrorKind::IndexNonNumeric(x, e_i, n),
                    ));
                };

                // v = [[e]]_S
                let v = interpret_expression(&e, &state).map_err(attach_location(location))?;

                // H(l_o)[n |-> v]
                let table = state.get_mut_heap(&l_o);
                let old_v = table.get(&n);
                table.set(n, v);

                // -[\pi^{w}_{l_o,n}]->
                trace.push(Observation::Write {
                    location: l_o,
                    index: n,
                    old: old_v,
                    new: v,
                });

                // <ε, 𝜎, H', F, C>
                Configuration(None, state)
            }

            // E-Fetch
            Command::Fetch(x, x_o, e_i) => {
                // l_o = S(x_o)
                let Ok(l_o) = state.get_variable(&x_o) else {
                    return Err(RuntimeError(location, ErrorKind::UnknownVariable(x_o)));
                };

                // l_o \in Loc_H
                let Value::Location(Location::Heap(l_o)) = l_o else {
                    return Err(RuntimeError(
                        location,
                        ErrorKind::IndexNonLocation(x_o, e_i, l_o),
                    ));
                };
                let l_o = Location::Heap(l_o);

                // n = [[e_i]]_S
                let n = interpret_expression(&e_i, &state).map_err(attach_location(location))?;

                // n \in Z
                let Value::Number(n) = n else {
                    return Err(RuntimeError(
                        location,
                        ErrorKind::IndexNonNumeric(x_o, e_i, n),
                    ));
                };

                // v = H(l_o, n)
                let v = state.get_heap(&l_o).get(&n);

                // 𝜎[x |-> v]
                state.insert_store(&x, v);

                // -[\pi^{r}_{l_o,n}]->
                trace.push(Observation::Read {
                    location: l_o,
                    index: n,
                    value: v,
                });

                // <ε, 𝜎', H, F, C>
                Configuration(None, state)
            }

            // E-Alloc
            Command::Allocate(x) => {
                // l_o \not\in H
                let l_o = state.fresh_heap_location();

                // 𝜎[x |-> l_o]
                state.insert_store(&x, Value::Location(l_o));

                // H[l_o |-> {}]
                state.insert_heap(&l_o, Table::empty());

                // <ε, 𝜎', H', F, C>
                Configuration(None, state)
            }

            // E-Alloc-C
            Command::AllocateC(x) => {
                // l_o \not\in H
                let l_o = state.fresh_heap_location();

                // H[l_o |-> {}]
                state.insert_heap(&l_o, Table::empty());

                // C[x |-> l_o]
                state.insert_context(&x, l_o);

                // <ε, 𝜎, H', F, C'>
                Configuration(None, state)
            }

            // E-Func
            Command::Function(x, c) => {
                // l_f = F
                let l_f = state.fresh_function_location();

                // 𝜎[x |-> l_f]
                state.insert_store(&x, Value::Location(l_f));

                // F[l_f |-> c]
                state.insert_function(&l_f, *c);

                // <ε, 𝜎', H, F', C>
                Configuration(None, state)
            }

            // E-Call
            Command::Call(x) => {
                // l_f = S(x)
                let Ok(v) = state.get_variable(&x) else {
                    return Err(RuntimeError(location, ErrorKind::UnknownVariable(x)));
                };

                // l_f \in Loc_F
                let Value::Location(Location::Function(l_f)) = v else {
                    return Err(RuntimeError(location, ErrorKind::UnknownVariable(x)));
                };
                let l_f = Location::Function(l_f);

                // c = F(l_f)
                let Ok(c) = state.get_function(&l_f) else {
                    return Err(RuntimeError(location, ErrorKind::CallNonFunction(x, v)));
                };

                // <c, 𝜎, H, F, C>
                Configuration(Some(c), state)
            }

            // E-If-Then | E-If-Else
            Command::If(e, c1, c2) => {
                // [[e]]_S
                match interpret_expression(&e, &state).map_err(attach_location(location))? {
                    // E-If-Then (n = [[e]]_S  n \in Z  n != 0)
                    Value::Number(n) if n.0 != 0 => {
                        // <c1, 𝜎, H, F, C>
                        Configuration(Some(*c1), state)
                    }

                    // E-If-Else ((n = [[e]]_S  n \in Z)  n == 0)
                    Value::Number(n) if n.0 == 0 => {
                        // <c2, 𝜎, H, F, C>
                        Configuration(Some(*c2), state)
                    }

                    v => return Err(RuntimeError(location, ErrorKind::GuardNonNumeric(v))),
                }
            }

            // E-While
            Command::While(e, c) => {
                // c; while e { c }
                let c1_0 = *c.clone();
                let c1_1 = LocationCommand(location, Command::While(e.clone(), c));
                let c1 =
                    LocationCommand(location, Command::Sequence(Box::new(c1_0), Box::new(c1_1)));

                // if e { c; while e { c } } else { skip }
                let c2 = LocationCommand(location, Command::Skip);
                let unrolled =
                    LocationCommand(location, Command::If(e, Box::new(c1), Box::new(c2)));

                // <if e { c; while e { c } } else { skip }, 𝜎, H, F, C>
                Configuration(Some(unrolled), state)
            }
        };

        Ok(next)
    }

    fn interpret_expression(expression: &Expression, state: &State) -> Result<Value, ErrorKind> {
        use super::parse;

        let value = match expression {
            // [[v]]_S = v
            Expression::Value(value) => match value {
                parse::Value::Nil => Value::Nil,
                parse::Value::Number(n) => Value::number(*n as isize),
            },

            // [[x]]_S = S(x)
            Expression::Variable(variable) => {
                // S(x)
                state
                    .get_variable(variable)
                    .map_err(|_| ErrorKind::UnknownVariable(variable.to_owned()))?
            }

            // [[e1 <op> e2]]_S = [[e1]]_S <op> [[e2]]_S
            Expression::Operation(e1, op, e2) => {
                // [[e1]]_S
                let lhs = interpret_expression(e1, state)?;

                // [[e2]]_S
                let rhs = interpret_expression(e2, state)?;

                // [[e1]]_S <op> [[e2]]_S
                let value =
                    match (lhs, op, rhs) {
                        // n1 + n2
                        (Value::Number(n1), Operation::Add, Value::Number(n2)) => n1.0 + n2.0,

                        // n1 - n2
                        (Value::Number(n1), Operation::Subtract, Value::Number(n2)) => n1.0 - n2.0,

                        // n1 == n2
                        (Value::Number(n1), Operation::Equals, Value::Number(n2)) => {
                            if n1.0 == n2.0 { 1 } else { 0 }
                        }

                        // n1 && n2
                        (Value::Number(n1), Operation::LogicalAnd, Value::Number(n2)) => {
                            if n1.0 != 0 && n2.0 != 0 { 1 } else { 0 }
                        }

                        // n1 != n2
                        (Value::Number(n1), Operation::NotEquals, Value::Number(n2)) => {
                            if n1.0 == n2.0 { 0 } else { 1 }
                        }

                        // n == nil | nil == n
                        (Value::Number(..), Operation::Equals, Value::Nil)
                        | (Value::Nil, Operation::Equals, Value::Number(..)) => 0,

                        // n != nil | nil != n
                        (Value::Number(..), Operation::NotEquals, Value::Nil)
                        | (Value::Nil, Operation::NotEquals, Value::Number(..)) => 1,

                        // l1 == l2
                        (Value::Location(l1), Operation::Equals, Value::Location(l2)) => {
                            if l1 == l2 { 1 } else { 0 }
                        }

                        // l1 != l2
                        (Value::Location(l1), Operation::NotEquals, Value::Location(l2)) => {
                            if l1 == l2 { 0 } else { 1 }
                        }

                        // l == nil | nil == l
                        (Value::Location(..), Operation::Equals, Value::Nil)
                        | (Value::Nil, Operation::Equals, Value::Location(..)) => 0,

                        // l != nil | nil != l
                        (Value::Location(..), Operation::NotEquals, Value::Nil)
                        | (Value::Nil, Operation::NotEquals, Value::Location(..)) => 1,

                        // n == l | l == n
                        (Value::Number(_), Operation::Equals, Value::Location(_))
                        | (Value::Location(_), Operation::Equals, Value::Number(_)) => 0,

                        // n != l | l != n
                        (Value::Number(_), Operation::NotEquals, Value::Location(_))
                        | (Value::Location(_), Operation::NotEquals, Value::Number(_)) => 1,

                        // nil == nil
                        (Value::Nil, Operation::Equals, Value::Nil) => 1,

                        // nil != nil
                        (Value::Nil, Operation::NotEquals, Value::Nil) => 0,

                        // INVALID
                        _ => {
                            return Err(ErrorKind::InvalidOp(
                                *e1.clone(),
                                *e2.clone(),
                                lhs,
                                rhs,
                                op.clone(),
                            ));
                        }
                    };

                Value::number(value)
            }
        };

        Ok(value)
    }

    fn attach_location(location: FileLocation) -> impl FnOnce(ErrorKind) -> RuntimeError {
        move |kind| RuntimeError(location, kind)
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[derive(Clone)]
    pub struct Configuration(Option<LocationCommand>, State);

    impl Configuration {
        pub fn new(program: Program, state: State) -> Self {
            Self(Some(program.0), state)
        }

        pub fn is_done(&self) -> bool {
            self.0.is_none()
        }

        pub fn get_command(&self) -> Option<Command> {
            self.0.clone().map(|lc| lc.1)
        }

        pub fn get_state(&self) -> State {
            self.1.clone()
        }
    }

    impl std::fmt::Display for Configuration {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            writeln!(f, "<")?;
            match &self.0 {
                Some(c) => writeln!(f, "{c}")?,
                None => writeln!(f, "ε")?,
            }
            writeln!(f, "{}", self.1)?;
            write!(f, ">")
        }
    }

    #[derive(Debug)]
    pub struct RuntimeError(FileLocation, ErrorKind);

    impl std::error::Error for RuntimeError {}

    impl std::fmt::Display for RuntimeError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "[RUNTIME] ({}:{}) ", self.0.0, self.0.1)?;
            match &self.1 {
                ErrorKind::CallNonFunction(x, v) => write!(
                    f,
                    "Variable '{x}' in 'call {x}' did not hold a location, got: '{v}'",
                ),
                ErrorKind::GuardNonNumeric(v) => write!(
                    f,
                    "If-statement guard did not evaluate to a number, got: '{v}'",
                ),
                ErrorKind::IndexNonLocation(x, e_i, v) => write!(
                    f,
                    "Variable '{x}' in '{x}[{e_i}]' did not hold a location, got: '{v}'",
                ),
                ErrorKind::IndexNonNumeric(x, e_i, v) => write!(
                    f,
                    "Index '{e_i}' in '{x}[{e_i}]' did not evaluate to a number, got: '{v}'",
                ),
                ErrorKind::InvalidOp(e1, e2, v1, v2, op) => write!(
                    f,
                    "Invalid operation in '{e1} {op} {e2}', got: '{v1} {op} {v2}'",
                ),
                ErrorKind::UnknownVariable(x) => write!(f, "Unknown variable '{x}'",),
            }
        }
    }

    #[derive(Debug)]
    pub enum ErrorKind {
        CallNonFunction(Identifier, Value),
        GuardNonNumeric(Value),
        IndexNonLocation(Identifier, Expression, Value),
        IndexNonNumeric(Identifier, Expression, Value),
        InvalidOp(Expression, Expression, Value, Value, Operation),
        UnknownVariable(Identifier),
    }
}

pub mod state {
    use std::collections::HashMap;

    use crate::OUT_VAR;
    use crate::parse::{Identifier, LocationCommand};

    #[derive(Clone)]
    pub struct State {
        store: Store,
        heap: Heap,
        functions: Functions,
        context: Context,
    }

    impl State {
        pub fn new() -> Self {
            Self {
                store: Store::new(),
                heap: Heap::new(),
                functions: Functions::new(),
                context: Context::new(),
            }
        }

        pub fn fresh_function_location(&mut self) -> Location {
            self.functions.fresh()
        }

        pub fn fresh_heap_location(&mut self) -> Location {
            self.heap.fresh()
        }

        pub fn get_function(&self, location: &Location) -> Result<LocationCommand, ()> {
            self.functions.get(location).ok_or(())
        }

        pub fn get_output(&self) -> Result<Value, ()> {
            self.get_variable_by_name(OUT_VAR)
        }

        pub fn get_variable(&self, name: &Identifier) -> Result<Value, ()> {
            let name = &name.0;
            if self.store.contains_key(name) {
                Ok(self.store.get(name))
            } else if self.context.contains_key(name) {
                Ok(self.context.get(name))
            } else {
                Err(())
            }
        }

        pub fn get_variable_by_name(&self, name: &str) -> Result<Value, ()> {
            let identifier = Identifier(name.to_owned());
            self.get_variable(&identifier)
        }

        pub fn get_heap(&self, location: &Location) -> &Table {
            self.heap.get(location)
        }

        pub fn get_mut_heap(&mut self, location: &Location) -> &mut Table {
            self.heap.get_mut(location)
        }

        pub fn insert_context(&mut self, name: &Identifier, location: Location) {
            self.context.insert(&name.0, location);
        }

        pub fn insert_function(&mut self, location: &Location, body: LocationCommand) {
            if !matches!(location, Location::Function(..)) {
                panic!("insert_function must be provided with a Location::Function");
            }

            self.functions.insert(location, body)
        }

        pub fn insert_heap(&mut self, location: &Location, table: Table) {
            self.heap.insert(*location, table);
        }

        pub fn insert_input(&mut self, name: &str, value: isize) {
            self.store.insert(name, Value::Number(Number(value)))
        }

        pub fn insert_store(&mut self, name: &Identifier, value: Value) {
            self.store.insert(&name.0, value);
        }

        pub fn is_shared(&self, query: &Location) -> bool {
            let mut visited: Vec<Location> = vec![];
            let mut locations: Vec<Location> =
                self.context.storage.values().copied().collect::<Vec<_>>();
            while let Some(location) = locations.pop() {
                if location == *query {
                    return true;
                }

                let table = self.heap.get(&location);
                for value in table.storage.values() {
                    let Value::Location(Location::Heap(l)) = value else {
                        continue;
                    };

                    let location = Location::Heap(*l);
                    if !visited.contains(&location) {
                        visited.push(location);
                        locations.push(location);
                    }
                }
            }

            false
        }

        pub fn iter_context(&self) -> impl Iterator<Item = (&Variable, &Location)> {
            self.keys_context()
                .map(move |k| (k, &self.context.storage[k]))
        }

        pub fn iter_functions(&self) -> impl Iterator<Item = (&Location, &LocationCommand)> {
            self.keys_functions()
                .map(move |k| (k, &self.functions.storage[k]))
        }

        pub fn iter_heap(&self) -> impl Iterator<Item = (&Location, &Table)> {
            self.keys_heap().map(move |k| (k, &self.heap.storage[k]))
        }

        pub fn iter_store(&self) -> impl Iterator<Item = (&Variable, &Value)> {
            self.keys_store().map(move |k| (k, &self.store.storage[k]))
        }

        pub fn keys_context(&self) -> impl Iterator<Item = &Variable> {
            let mut keys = self.context.storage.keys().collect::<Vec<_>>();
            keys.sort();
            keys.into_iter()
        }

        pub fn keys_functions(&self) -> impl Iterator<Item = &Location> {
            let mut keys = self.functions.storage.keys().collect::<Vec<_>>();
            keys.sort();
            keys.into_iter()
        }

        pub fn keys_heap(&self) -> impl Iterator<Item = &Location> {
            let mut keys = self.heap.storage.keys().collect::<Vec<_>>();
            keys.sort();
            keys.into_iter()
        }

        pub fn keys_store(&self) -> impl Iterator<Item = &Variable> {
            let mut keys = self.store.storage.keys().collect::<Vec<_>>();
            keys.sort();
            keys.into_iter()
        }

        pub fn raw_get_context(&self, variable: &Variable) -> Option<&Location> {
            self.context.storage.get(variable)
        }

        pub fn raw_get_heap(&self, location: &Location) -> Option<&Table> {
            self.heap.storage.get(location)
        }

        pub fn raw_get_store(&self, variable: &Variable) -> Option<&Value> {
            self.store.storage.get(variable)
        }
    }

    impl std::fmt::Display for State {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            // Store
            write!(f, "𝜎[")?;
            for (i, (key, value)) in self.iter_store().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }

                write!(f, "{key} -> ")?;
                if let Value::Location(l) = value
                    && self.is_shared(l)
                {
                    write!(f, "\x1b[1m")?;
                }
                write!(f, "{value}")?;
                if let Value::Location(l) = value
                    && self.is_shared(l)
                {
                    write!(f, "\x1b[0m")?;
                }
            }
            writeln!(f, "]")?;

            // Heap
            write!(f, "H[")?;
            for (i, (key, table)) in self.iter_heap().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }

                let is_shared = self.is_shared(key);
                if is_shared {
                    write!(f, "\x1b[1m")?;
                }
                write!(f, "{key} -> {{")?;

                for (i, (key, value)) in table.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{key} -> ")?;
                    if let Value::Location(l) = value
                        && self.is_shared(l)
                        && !is_shared
                    {
                        write!(f, "\x1b[1m")?;
                    }
                    write!(f, "{value}")?;
                    if let Value::Location(l) = value
                        && self.is_shared(l)
                        && !is_shared
                    {
                        write!(f, "\x1b[0m")?;
                    }
                }

                write!(f, "}}")?;
                if is_shared {
                    write!(f, "\x1b[0m")?;
                }
            }
            writeln!(f, "]")?;

            // Context
            write!(f, "F[")?;
            for (i, (key, value)) in self.iter_functions().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }

                write!(f, "{key} -> {value}")?;
            }
            writeln!(f, "]")?;

            // Context
            write!(f, "C[")?;
            for (i, (key, value)) in self.iter_context().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }

                write!(f, "{key} -> \x1b[1m{value}\x1b[0m")?;
            }
            write!(f, "]")
        }
    }

    #[derive(Clone)]
    pub struct Store {
        storage: HashMap<Variable, Value>,
    }

    impl Store {
        fn new() -> Self {
            Self {
                storage: HashMap::new(),
            }
        }

        fn contains_key(&self, variable: &str) -> bool {
            self.storage.contains_key(variable)
        }

        fn get(&self, variable: &str) -> Value {
            if let Some(value) = self.storage.get(variable) {
                *value
            } else {
                Value::Nil
            }
        }

        fn insert(&mut self, variable: &str, value: Value) {
            self.storage.insert(variable.to_owned(), value);
        }
    }

    #[derive(Clone)]
    pub struct Heap {
        storage: HashMap<Location, Table>,
    }

    impl Heap {
        fn new() -> Self {
            Self {
                storage: HashMap::new(),
            }
        }

        fn fresh(&mut self) -> Location {
            Location::Heap(self.storage.len())
        }

        fn get(&self, location: &Location) -> &Table {
            self.storage.get(location).unwrap()
        }

        fn get_mut(&mut self, location: &Location) -> &mut Table {
            self.storage.get_mut(location).unwrap()
        }

        fn insert(&mut self, location: Location, table: Table) {
            self.storage.insert(location, table);
        }
    }

    #[derive(Clone)]
    pub struct Functions {
        storage: HashMap<Location, LocationCommand>,
    }

    impl Functions {
        fn new() -> Self {
            Self {
                storage: HashMap::new(),
            }
        }

        fn fresh(&mut self) -> Location {
            Location::Function(self.storage.len())
        }

        fn get(&self, location: &Location) -> Option<LocationCommand> {
            self.storage.get(location).cloned()
        }

        fn insert(&mut self, location: &Location, body: LocationCommand) {
            self.storage.insert(location.to_owned(), body);
        }
    }

    #[derive(Clone)]
    pub struct Context {
        storage: HashMap<Variable, Location>,
    }

    impl Context {
        fn new() -> Self {
            Self {
                storage: HashMap::new(),
            }
        }

        fn contains_key(&self, variable: &Variable) -> bool {
            self.storage.contains_key(variable)
        }

        fn get(&self, variable: &Variable) -> Value {
            if let Some(location) = self.storage.get(variable) {
                Value::Location(*location)
            } else {
                Value::Nil
            }
        }

        fn insert(&mut self, name: &Variable, location: Location) {
            self.storage.insert(name.to_owned(), location);
        }
    }

    #[derive(Clone)]
    pub struct Table {
        storage: HashMap<Number, Value>,
    }

    impl Table {
        pub fn empty() -> Self {
            Self::new()
        }

        fn new() -> Self {
            Self {
                storage: HashMap::new(),
            }
        }

        pub fn contains_key(&self, index: &Number) -> bool {
            self.storage.contains_key(index)
        }

        pub fn get(&self, index: &Number) -> Value {
            if let Some(value) = self.storage.get(index) {
                *value
            } else {
                Value::Nil
            }
        }

        pub fn iter(&self) -> impl Iterator<Item = (&Number, &Value)> {
            self.keys().map(move |key| (key, &self.storage[key]))
        }

        pub fn keys(&self) -> impl Iterator<Item = &Number> {
            let mut keys = self.storage.keys().collect::<Vec<_>>();
            keys.sort();
            keys.into_iter()
        }

        pub fn set(&mut self, index: Number, value: Value) {
            self.storage.insert(index, value);
        }

        pub fn raw_get(&self, index: &Number) -> Option<&Value> {
            self.storage.get(index)
        }
    }

    impl std::fmt::Display for Table {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{{")?;
            for (i, (key, value)) in self.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }

                write!(f, "{key} -> {value}")?;
            }
            write!(f, "}}")
        }
    }

    type Variable = String;

    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub enum Location {
        Function(usize),
        Heap(usize),
    }

    impl Location {
        pub fn index(&self) -> usize {
            match self {
                Self::Function(i) => *i,
                Self::Heap(i) => *i,
            }
        }
    }

    impl std::fmt::Display for Location {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Function(l) => write!(f, "`{l}f"),
                Self::Heap(l) => write!(f, "`{l}"),
            }
        }
    }

    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct Number(pub isize);

    impl std::fmt::Display for Number {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum Value {
        Location(Location),
        Number(Number),
        Nil,
    }

    impl Value {
        pub fn number(n: isize) -> Self {
            Self::Number(Number(n))
        }
    }

    impl std::fmt::Display for Value {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self {
                Self::Location(l) => write!(f, "{l}"),
                Self::Number(n) => write!(f, "{n}"),
                Self::Nil => write!(f, "nil"),
            }
        }
    }
}

pub mod trace {
    #[cfg(feature = "wasm")]
    use wasm_bindgen::prelude::*;

    use crate::state::{Location, Number, Value};

    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    pub struct Trace(Vec<Observation>);

    impl Trace {
        pub fn new() -> Self {
            Self(vec![])
        }

        pub fn iter(&self) -> impl Iterator<Item = &Observation> {
            self.0.iter()
        }

        pub fn push(&mut self, observation: Observation) {
            self.0.push(observation);
        }
    }

    impl std::fmt::Display for Trace {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let reads = self
                .0
                .clone()
                .into_iter()
                .filter(Observation::is_read)
                .collect::<Vec<_>>();
            let writes = self
                .0
                .clone()
                .into_iter()
                .filter(Observation::is_write)
                .collect::<Vec<_>>();

            write!(f, "all:")?;
            for (i, observation) in self.0.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                }

                write!(f, " {observation}")?;
            }

            writeln!(f)?;
            write!(f, "reads:")?;
            for (i, observation) in reads.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                }

                write!(f, " {observation}")?;
            }

            writeln!(f)?;
            write!(f, "writes:")?;
            for (i, observation) in writes.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                }

                write!(f, " {observation}")?;
            }

            Ok(())
        }
    }

    #[derive(Clone)]
    pub enum Observation {
        Read {
            location: Location,
            index: Number,
            value: Value,
        },
        Write {
            location: Location,
            index: Number,
            old: Value,
            new: Value,
        },
    }

    impl Observation {
        pub fn index(&self) -> &Number {
            match self {
                Observation::Read { index, .. } => index,
                Observation::Write { index, .. } => index,
            }
        }

        pub fn is_read(&self) -> bool {
            matches!(self, Observation::Read { .. })
        }

        pub fn is_write(&self) -> bool {
            matches!(self, Observation::Write { .. })
        }

        pub fn location(&self) -> &Location {
            match self {
                Observation::Read { location, .. } => location,
                Observation::Write { location, .. } => location,
            }
        }

        pub fn value(&self) -> &Value {
            match self {
                Observation::Read { value, .. } => value,
                Observation::Write { new, .. } => new,
            }
        }
    }

    impl Eq for Observation {}

    impl PartialEq for Observation {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (
                    Self::Write {
                        location: l1,
                        index: i1,
                        //new: v1,
                        ..
                    },
                    Self::Write {
                        location: l2,
                        index: i2,
                        //new: v2,
                        ..
                    },
                ) => l1 == l2 && i1 == i2, //&& v1 == v2,
                (
                    Self::Read {
                        location: l1,
                        index: i1,
                        //value: v1,
                        ..
                    },
                    Self::Read {
                        location: l2,
                        index: i2,
                        //value: v2,
                        ..
                    },
                ) => l1 == l2 && i1 == i2, //&& v1 == v2,
                _ => false,
            }
        }
    }

    impl std::fmt::Display for Observation {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self {
                Self::Read {
                    location,
                    index,
                    value,
                } => {
                    write!(f, "{location}@{index}#{value}")?;
                }
                Self::Write {
                    location,
                    index,
                    old,
                    new,
                } => {
                    write!(f, "{location}@{index}#{old}->{new}")?;
                }
            }

            Ok(())
        }
    }
}

pub mod compare {
    use crate::state::{State, Value};
    use crate::trace::Trace;
    use crate::{INPUT_PREFIX, OUT_VAR};

    pub fn compare(state1: &State, trace1: &Trace, state2: &State, trace2: &Trace) {
        let mut diff = vec![];

        diff.extend(compare_inputs(state1, state2));
        diff.extend(compare_outputs(state1, state2));
        diff.extend(compare_stores(state1, state2));
        diff.extend(compare_heaps(state1, state2));
        diff.extend(compare_contexts(state1, state2));
        diff.extend(compare_traces(state1, trace1, state2, trace2));

        diff.iter().for_each(|diff| println!("{diff}"));
    }

    fn compare_inputs(state1: &State, state2: &State) -> Vec<String> {
        let mut diff = vec![];

        for (variable, value1) in state1.iter_store() {
            if !variable.starts_with(INPUT_PREFIX) {
                continue;
            }

            let value2 = state2.raw_get_store(variable).unwrap();
            if value1 != value2 {
                diff.push(format!("  {variable}: {value1} != {value2}"));
            }
        }

        if !diff.is_empty() {
            diff.insert(0, "Inputs:".to_owned());
        }

        diff
    }

    fn compare_outputs(state1: &State, state2: &State) -> Vec<String> {
        let mut diff = vec![];

        let out1 = state1.get_variable_by_name(OUT_VAR).unwrap_or(Value::Nil);
        let out2 = state2.get_variable_by_name(OUT_VAR).unwrap_or(Value::Nil);
        if out1 != out2 {
            diff.push("Output:".to_owned());
            diff.push(format!("  {out1} != {out2}"));
        }

        diff
    }

    fn compare_stores(state1: &State, state2: &State) -> Vec<String> {
        let mut diff = vec![];

        for (variable, value1) in state1.iter_store() {
            if variable.starts_with(INPUT_PREFIX) || variable == OUT_VAR {
                continue;
            }

            if let Some(value2) = state2.raw_get_store(variable) {
                if value1 != value2 {
                    diff.push(format!(
                        "  s1({variable}) != s2({variable})  [{value1} != {value2}]"
                    ));
                }
            } else {
                diff.push(format!("  {variable} in s1 but not in s2  [{value1}]"))
            }
        }

        for (variable, value1) in state2.iter_store() {
            if variable.starts_with(INPUT_PREFIX) || variable == OUT_VAR {
                continue;
            }

            if state1.raw_get_store(variable).is_none() {
                diff.push(format!("  {variable} in s2 but not in s1  [{value1}]"))
            }
        }

        if !diff.is_empty() {
            diff.insert(0, "Store:".to_owned());
        }

        diff
    }

    fn compare_heaps(state1: &State, state2: &State) -> Vec<String> {
        let mut diff = vec![];

        for (location, table1) in state1.iter_heap() {
            if let Some(table2) = state2.raw_get_heap(location) {
                let location = if state1.is_shared(location) {
                    format!("\x1b[1m{location}\x1b[0m")
                } else {
                    format! {"{location}" }
                };

                for (index, value1) in table1.iter() {
                    if let Some(value2) = table2.raw_get(index) {
                        if value1 != value2 {
                            diff.push(format!("  H1({location}, {index}) != H2({location}, {index})  [{value1} != {value2}]"));
                        }
                    } else {
                        diff.push(format!(
                            "  {index} in H1({location}) but not in H2({location})  [H1({location}, {index})={value1}]"
                        ))
                    }
                }
                for (index, value2) in table2.iter() {
                    if table1.raw_get(index).is_none() {
                        diff.push(format!(
                            "  {index} in H2({location}) but not in H1({location})  [H2({location}, {index})={value2}]"
                        ))
                    }
                }
            } else {
                diff.push(format!("  {location} in H1 but not in H2  [{table1}]"))
            }
        }

        for (location, table2) in state2.iter_heap() {
            if state1.raw_get_heap(location).is_none() {
                let location = if state1.is_shared(location) {
                    format!("\x1b[1m{location}\x1b[0m")
                } else {
                    format! {"{location}" }
                };
                diff.push(format!("  {location} in H2 but not in H1  [{table2}]"))
            }
        }

        if !diff.is_empty() {
            diff.insert(0, "Heap:".to_owned());
        }

        diff
    }

    fn compare_contexts(state1: &State, state2: &State) -> Vec<String> {
        let mut diff = vec![];

        // in both, different
        for (variable, location1) in state1.iter_context() {
            if let Some(location2) = state2.raw_get_context(variable)
                && location1 != location2
            {
                diff.push(
                    format!(
                        "C1({variable}) != C2({variable})  [\x1b[1m{location1}\x1b[0m != \x1b[1m{location2}\x1b[0m]",
                    ),
                );
            }
        }

        // in 1, not 2
        for (variable, location1) in state1.iter_context() {
            if state2.raw_get_context(variable).is_none() {
                diff.push(format!(
                    "{variable} in C1 but not in C2  [\x1b[1m{location1}\x1b[0m]",
                ));
            }
        }

        // in 2, not 1
        for (variable, location2) in state2.iter_context() {
            if state1.raw_get_context(variable).is_none() {
                diff.push(format!(
                    "{variable} in C2 but not in C1  [\x1b[1m{location2}\x1b[0m]",
                ));
            }
        }

        if !diff.is_empty() {
            diff.insert(0, "Context:".to_owned());
        }

        diff
    }

    fn compare_traces(
        state1: &State,
        trace1: &Trace,
        state2: &State,
        trace2: &Trace,
    ) -> Vec<String> {
        let mut trace1 = trace1.iter().cloned().collect::<Vec<_>>();
        let mut trace2 = trace2.iter().cloned().collect::<Vec<_>>();

        let mut used = vec![false; trace2.len()];
        let mut remove1 = Vec::new();
        let mut remove2 = Vec::new();
        for (i1, o1) in trace1.iter().enumerate() {
            if let Some(i2) = trace2
                .iter()
                .enumerate()
                .position(|(i2, o2)| !used[i2] && o1 == o2)
            {
                used[i2] = true;
                remove1.push(i1);
                remove2.push(i2);
            }
        }

        remove1.sort();
        for i in remove1.iter().rev() {
            trace1.remove(*i);
        }

        remove2.sort();
        for i in remove2.iter().rev() {
            trace2.remove(*i);
        }

        // Diff
        let mut diff = vec![];

        let reads_in_1_not_in_2 = trace1.iter().filter(|o| o.is_read()).collect::<Vec<_>>();
        for observation in reads_in_1_not_in_2 {
            let location = observation.location();
            let index = observation.index();
            let value = observation.value();
            diff.push(format!(
                "  read {value} from H1({}, {index})",
                if state1.is_shared(location) {
                    format!("\x1b[1m{location}\x1b[0m")
                } else {
                    format! {"{location}" }
                }
            ));
        }

        let writes_in_1_not_in_2 = trace1.iter().filter(|o| o.is_write()).collect::<Vec<_>>();
        for observation in writes_in_1_not_in_2 {
            let location = observation.location();
            let index = observation.index();
            let value = observation.value();
            diff.push(format!(
                "  write {value} to H1({}, {index})",
                if state1.is_shared(location) {
                    format!("\x1b[1m{location}\x1b[0m")
                } else {
                    format! {"{location}" }
                }
            ));
        }

        let reads_in_2_not_in_1 = trace2.iter().filter(|o| o.is_read()).collect::<Vec<_>>();
        for observation in reads_in_2_not_in_1 {
            let location = observation.location();
            let index = observation.index();
            let value = observation.value();
            diff.push(format!(
                "  read {value} from H2({}, {index})",
                if state2.is_shared(location) {
                    format!("\x1b[1m{location}\x1b[0m")
                } else {
                    format! {"{location}" }
                }
            ));
        }

        let writes_in_2_not_in_1 = trace2.iter().filter(|o| o.is_write()).collect::<Vec<_>>();
        for observation in writes_in_2_not_in_1 {
            let location = observation.location();
            let index = observation.index();
            let value = observation.value();
            diff.push(format!(
                "  write {value} to H2({}, {index})",
                if state2.is_shared(location) {
                    format!("\x1b[1m{location}\x1b[0m")
                } else {
                    format! {"{location}" }
                }
            ));
        }

        if !diff.is_empty() {
            diff.insert(0, "Trace:".to_owned());
        }

        diff
    }
}

pub mod condition {
    use crate::state::State;
    use crate::trace::Trace;

    pub fn weakness(state1: &State, trace1: &Trace, state2: &State, trace2: &Trace) -> Weakness {
        let mut intersection1 = vec![];
        for observation in trace1.iter() {
            let location = observation.location();
            if state1.is_shared(location) {
                intersection1.push(observation);
            }
        }

        let mut intersection2 = vec![];
        for observation in trace2.iter() {
            let location = observation.location();
            if state2.is_shared(location) {
                intersection2.push(observation);
            }
        }

        let mut used = vec![false; intersection2.len()];
        let mut remove1 = Vec::new();
        let mut remove2 = Vec::new();
        for (i1, o1) in intersection1.iter().enumerate() {
            if let Some(i2) = intersection2
                .iter()
                .enumerate()
                .position(|(i2, o2)| !used[i2] && o1 == o2)
            {
                used[i2] = true;
                remove1.push(i1);
                remove2.push(i2);
            }
        }

        remove1.sort();
        for i in remove1.iter().rev() {
            intersection1.remove(*i);
        }

        remove2.sort();
        for i in remove2.iter().rev() {
            intersection2.remove(*i);
        }

        if intersection1.is_empty() && intersection2.is_empty() {
            return Weakness::None;
        }

        let all_read1 = intersection1
            .iter()
            .all(|observation| observation.is_read());
        let all_read2 = intersection2
            .iter()
            .all(|observation| observation.is_read());
        if all_read1 && all_read2 {
            Weakness::Stateless
        } else {
            Weakness::Stateful
        }
    }

    pub enum Weakness {
        None,
        Stateless,
        Stateful,
    }
}

pub mod visualize {
    use crate::state::{Location, State, Value};
    use crate::trace::{Observation, Trace};

    pub fn memory_diagram(state: &State, trace: &Trace) -> Diagram {
        let mut mermaid = String::new();
        mermaid.push_str("flowchart TB\n");

        mermaid.push_str("  subgraph Heap\n");
        // Heap tables
        for (location, table) in state.iter_heap() {
            let l = location.index();
            mermaid.push_str(&format!("    subgraph H{l}[\"{l}\"]\n"));

            // Content
            for (index, value) in table.iter() {
                mermaid.push_str("      ");
                mermaid.push_str(&format!("H{l}{index}[\"{index}: {value}\"]"));
                mermaid.push('\n');
            }

            // Observations
            for observation in trace.iter().filter(|o| o.location() == location) {
                mermaid.push_str("      ");
                match observation {
                    Observation::Read { index, value, .. } => {
                        mermaid.push_str(&format!("H{l}{index}"));
                        if !state.get_heap(location).contains_key(index) {
                            mermaid.push_str(&format!("[\"{index}: -\"]"));
                        }
                        mermaid.push_str("-. \" ");
                        mermaid.push_str(&value.to_string());
                        mermaid.push_str(&format!(" \" .->read{l}(\"read\")"));
                    }
                    Observation::Write {
                        index, old, new, ..
                    } => {
                        mermaid.push_str(&format!("write{l}(\"write\")-. \" "));
                        mermaid.push_str(&old.to_string());
                        mermaid.push_str("->");
                        mermaid.push_str(&new.to_string());
                        mermaid.push_str(" \" .->");
                        mermaid.push_str(&format!("H{l}{index}"));
                    }
                }
                mermaid.push('\n');
            }

            mermaid.push_str("    end");
        }
        // Inter heap connections
        for (location, table) in state.iter_heap() {
            let l = location.index();
            for (index, value) in table.iter() {
                let index = index.0;
                if let Value::Location(Location::Heap(target)) = value {
                    mermaid.push_str("    ");
                    mermaid.push_str(&format!("H{l}{index}-->H{target}"));
                    mermaid.push('\n');
                }
            }
        }
        // Shared memory
        for (location, _) in state.iter_heap() {
            if state.is_shared(location) {
                let location = location.index();
                mermaid.push_str("    style ");
                mermaid.push_str(&format!("H{location}"));
                mermaid.push_str(" fill:red");
                mermaid.push('\n');
            }
        }
        mermaid.push_str("  end\n");

        mermaid.push_str("  subgraph Context\n");
        for (index, (variable, location)) in state.iter_context().enumerate() {
            let l = location.index();
            mermaid.push_str("    ");
            mermaid.push_str(&format!("C{index}[\"{variable}: `{l}\"]-->H{l}"));
            mermaid.push('\n');
        }
        mermaid.push_str("  end\n");

        mermaid.push_str("  subgraph Store\n");
        for (index, (variable, value)) in state.iter_store().enumerate() {
            mermaid.push_str("    ");
            mermaid.push_str(&format!("s{index}[\"{variable}: {value}\"]"));
            if let Value::Location(Location::Heap(location)) = value {
                mermaid.push_str("-->");
                mermaid.push_str(&format!("H{location}"));
            }
            mermaid.push('\n');
        }
        mermaid.push_str("  end\n");

        mermaid
    }

    pub type Diagram = String;
}

#[cfg(feature = "wasm")]
mod wasm_api {
    use std::collections::HashMap;

    use wasm_bindgen::prelude::*;

    use crate::interpret::{self, Configuration};
    use crate::lex;
    use crate::parse::{self, Program};
    use crate::state::{Location, Number, State, Value};
    use crate::trace::Trace;

    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    pub fn parse(program: &str) -> Result<Program, String> {
        let tokens = lex::lex(program).map_err(|e| e.to_string())?;
        parse::parse(tokens).map_err(|e| e.to_string())
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    pub fn start(program: Program, inputs: &js_sys::Map) -> JsConfiguration {
        let mut input_map = HashMap::new();
        inputs.for_each(&mut |value, key| {
            let key = key.as_string().unwrap();
            let value = value.as_f64().unwrap() as isize;
            input_map.insert(key, value);
        });

        let mut state = State::new();
        for (input, value) in input_map {
            state.insert_input(&input, value);
        }

        let trace = Trace::new();
        let configuration = Configuration::new(program, state);
        JsConfiguration(configuration, trace)
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    pub fn step(configuration: JsConfiguration) -> Result<JsConfiguration, String> {
        let mut trace = configuration.1;
        let configuration = configuration.0;
        let configuration2 =
            interpret::step(configuration, &mut trace).map_err(|e| e.to_string())?;
        Ok(JsConfiguration(configuration2, trace))
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    pub struct JsConfiguration(Configuration, Trace);

    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    impl JsConfiguration {
        pub fn is_done(&self) -> bool {
            self.0.is_done()
        }

        pub fn get_command(&self) -> Option<String> {
            self.0.get_command().map(|command| command.to_string())
        }

        pub fn get_output(&self) -> String {
            self.0
                .get_state()
                .get_output()
                .map(|v| v.to_string())
                .unwrap_or("nil".to_owned())
        }

        pub fn get_state(&self) -> JsState {
            let state = self.0.get_state();
            JsState(state)
        }
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    pub struct JsState(State);

    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    impl JsState {
        pub fn get_store(&self) -> JsValue {
            let object = js_sys::Object::new();
            for (key, value) in self.0.iter_store() {
                js_sys::Reflect::set(&object, &key.into(), &value.into()).unwrap();
            }

            object.into()
        }

        pub fn get_heap(&self) -> JsValue {
            todo!()
        }

        pub fn get_functions(&self) -> JsValue {
            let object = js_sys::Object::new();
            for (key, value) in self.0.iter_functions() {
                js_sys::Reflect::set(&object, &key.into(), &value.1.to_string().into()).unwrap();
            }

            object.into()
        }

        pub fn get_context(&self) -> JsValue {
            let object = js_sys::Object::new();
            for (key, value) in self.0.iter_context() {
                js_sys::Reflect::set(&object, &key.into(), &value.into()).unwrap();
            }

            object.into()
        }
    }

    impl Into<JsValue> for &Value {
        fn into(self) -> JsValue {
            match self {
                Value::Location(l) => l.into(),
                Value::Number(n) => n.into(),
                Value::Nil => "nil".into(),
            }
        }
    }

    impl Into<JsValue> for &Number {
        fn into(self) -> JsValue {
            let n = self.0;
            n.into()
        }
    }
    impl Into<JsValue> for &Location {
        fn into(self) -> JsValue {
            self.to_string().into()
        }
    }
}
