use im::hashmap::HashMap;
use std::{
    cell::RefCell,
    collections::hash_map::DefaultHasher,
    fmt::Display,
    hash::{Hash, Hasher},
    rc::Rc,
};

use crate::ast::{
    Binary, BinaryOp, Call, Element, File, First, Function, If, Let, Location, Print, Second, Term,
    Var,
};

#[derive(Clone, Debug)]
pub struct Closure {
    parameters: Vec<Var>,
    body: Box<Term>,
    context: Context,
}

#[derive(Clone, Debug)]
pub struct Tuple {
    first: Box<Value>,
    second: Box<Value>,
}

impl Display for Tuple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let first = self.first.clone();
        let second = self.second.clone();

        write!(f, "({first}, {second})")
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Closure(Closure),
    Int(i64),
    Str(String),
    Bool(bool),
    Tuple(Tuple),
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Closure(_closure) => panic!("this should never be executed"),
            Self::Int(int) => format!("Int({int})").hash(state),
            Self::Str(string) => format!("Str({string})").hash(state),
            Self::Bool(bool) => format!("Bool({bool})").hash(state),
            Self::Tuple(tuple) => format!("Tuple({tuple})").hash(state),
        }
    }
}

type Cache = std::collections::HashMap<String, Value>;

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Closure(_closure) => String::from("[closure]"),
            Self::Int(int) => int.to_string(),
            Self::Str(str) => str.to_string(),
            Self::Bool(bool) => bool.to_string(),
            Self::Tuple(tuple) => {
                format!(
                    "({}, {})",
                    tuple.first.to_string(),
                    tuple.second.to_string()
                )
            }
        };

        f.write_str(&value)
    }
}

type Context = HashMap<String, Value>;

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub message: String,
    pub full_text: String,
    pub location: Location,
}

fn invalid_comparison(l_value: &Value, r_value: &Value, location: &Location) -> RuntimeError {
    RuntimeError {
        message: String::from("invalid comparison"),
        full_text: format!("{} and {} cannot be compared", l_value, r_value),
        location: location.clone(),
    }
}

impl Value {
    pub fn eq(&self, value: &Value, location: &Location) -> Result<Value, RuntimeError> {
        match (self, value) {
            (Value::Bool(l_bool), Value::Bool(r_bool)) => Ok(Value::Bool(l_bool == r_bool)),
            (Value::Str(l_str), Value::Str(r_str)) => Ok(Value::Bool(l_str == r_str)),
            (Value::Int(l_int), Value::Int(r_int)) => Ok(Value::Bool(l_int == r_int)),
            (l_value, r_value) => Err(invalid_comparison(l_value, r_value, location)),
        }
    }

    pub fn neq(&self, value: &Value, location: &Location) -> Result<Value, RuntimeError> {
        match (self, value) {
            (Value::Bool(l_bool), Value::Bool(r_bool)) => Ok(Value::Bool(l_bool != r_bool)),
            (Value::Str(l_str), Value::Str(r_str)) => Ok(Value::Bool(l_str != r_str)),
            (Value::Int(l_int), Value::Int(r_int)) => Ok(Value::Bool(l_int != r_int)),
            (l_value, r_value) => Err(invalid_comparison(l_value, r_value, location)),
        }
    }

    pub fn lt(&self, value: &Value, location: &Location) -> Result<Value, RuntimeError> {
        match (self, value) {
            (Value::Bool(l_bool), Value::Bool(r_bool)) => Ok(Value::Bool(l_bool < r_bool)),
            (Value::Str(l_str), Value::Str(r_str)) => Ok(Value::Bool(l_str < r_str)),
            (Value::Int(l_int), Value::Int(r_int)) => Ok(Value::Bool(l_int < r_int)),
            (l_value, r_value) => Err(invalid_comparison(l_value, r_value, location)),
        }
    }

    pub fn lte(&self, value: &Value, location: &Location) -> Result<Value, RuntimeError> {
        match (self, value) {
            (Value::Bool(l_bool), Value::Bool(r_bool)) => Ok(Value::Bool(l_bool <= r_bool)),
            (Value::Str(l_str), Value::Str(r_str)) => Ok(Value::Bool(l_str <= r_str)),
            (Value::Int(l_int), Value::Int(r_int)) => Ok(Value::Bool(l_int <= r_int)),
            (l_value, r_value) => Err(invalid_comparison(l_value, r_value, location)),
        }
    }

    pub fn gt(&self, value: &Value, location: &Location) -> Result<Value, RuntimeError> {
        match (self, value) {
            (Value::Bool(l_bool), Value::Bool(r_bool)) => Ok(Value::Bool(l_bool > r_bool)),
            (Value::Str(l_str), Value::Str(r_str)) => Ok(Value::Bool(l_str > r_str)),
            (Value::Int(l_int), Value::Int(r_int)) => Ok(Value::Bool(l_int > r_int)),
            (l_value, r_value) => Err(invalid_comparison(l_value, r_value, location)),
        }
    }

    pub fn gte(&self, value: &Value, location: &Location) -> Result<Value, RuntimeError> {
        match (self, value) {
            (Value::Bool(l_bool), Value::Bool(r_bool)) => Ok(Value::Bool(l_bool >= r_bool)),
            (Value::Str(l_str), Value::Str(r_str)) => Ok(Value::Bool(l_str >= r_str)),
            (Value::Int(l_int), Value::Int(r_int)) => Ok(Value::Bool(l_int >= r_int)),
            (l_value, r_value) => Err(invalid_comparison(l_value, r_value, location)),
        }
    }

    pub fn and(&self, value: &Value, location: &Location) -> Result<Value, RuntimeError> {
        match (self, value) {
            (Value::Bool(l_bool), Value::Bool(r_bool)) => Ok(Value::Bool(*l_bool && *r_bool)),
            (_l_val, _r_val) => Err(RuntimeError {
                message: String::from("invalid binary operation"),
                full_text: format!("only booleans can be used on short-circuit operations"),
                location: location.clone(),
            }),
        }
    }

    pub fn or(&self, value: &Value, location: &Location) -> Result<Value, RuntimeError> {
        match (self, value) {
            (Value::Bool(l_bool), Value::Bool(r_bool)) => Ok(Value::Bool(*l_bool || *r_bool)),
            (_l_val, _r_val) => Err(RuntimeError {
                message: String::from("invalid binary operation"),
                full_text: format!("only booleans can be used on short-circuit operations"),
                location: location.clone(),
            }),
        }
    }

    pub fn add(&self, value: &Value, location: &Location) -> Result<Value, RuntimeError> {
        match (self, value) {
            (Value::Int(l_int), Value::Int(r_int)) => Ok(Value::Int(l_int + r_int)),
            (Value::Str(_l_bool), Value::Str(_r_bool)) => Err(RuntimeError {
                message: String::from("invalid numeric operation"),
                full_text: String::from("strings cannot be added"),
                location: location.clone(),
            }),
            (Value::Bool(_l_bool), Value::Bool(_r_bool)) => Err(RuntimeError {
                message: String::from("invalid numeric operation"),
                full_text: String::from("booleans cannot be added"),
                location: location.clone(),
            }),
            (Value::Closure(_l_closure), Value::Closure(_r_closure)) => Err(RuntimeError {
                message: String::from("invalid numeric operation"),
                full_text: String::from("closures cannot be added"),
                location: location.clone(),
            }),
            (_l_val, _r_val) => Err(RuntimeError {
                message: String::from("invalid numeric operation"),
                full_text: String::from("different types cannot be used on the same operation"),
                location: location.clone(),
            }),
        }
    }

    pub fn sub(&self, value: &Value, location: &Location) -> Result<Value, RuntimeError> {
        match (self, value) {
            (Value::Int(l_int), Value::Int(r_int)) => Ok(Value::Int(l_int - r_int)),
            (Value::Str(_l_bool), Value::Str(_r_bool)) => Err(RuntimeError {
                message: String::from("invalid numeric operation"),
                full_text: String::from("strings cannot be subtracted"),
                location: location.clone(),
            }),
            (Value::Bool(_l_bool), Value::Bool(_r_bool)) => Err(RuntimeError {
                message: String::from("invalid numeric operation"),
                full_text: String::from("booleans cannot be subtracted"),
                location: location.clone(),
            }),
            (Value::Closure(_l_closure), Value::Closure(_r_closure)) => Err(RuntimeError {
                message: String::from("invalid numeric operation"),
                full_text: String::from("closures cannot be subtracted"),
                location: location.clone(),
            }),
            (_l_val, _r_val) => Err(RuntimeError {
                message: String::from("invalid numeric operation"),
                full_text: String::from("different types cannot be used on the same operation"),
                location: location.clone(),
            }),
        }
    }

    pub fn mul(&self, value: &Value, location: &Location) -> Result<Value, RuntimeError> {
        match (self, value) {
            (Value::Int(l_int), Value::Int(r_int)) => Ok(Value::Int(l_int - r_int)),
            (Value::Str(_l_bool), Value::Str(_r_bool)) => Err(RuntimeError {
                message: String::from("invalid numeric operation"),
                full_text: String::from("strings cannot be multiplied"),
                location: location.clone(),
            }),
            (Value::Bool(_l_bool), Value::Bool(_r_bool)) => Err(RuntimeError {
                message: String::from("invalid numeric operation"),
                full_text: String::from("booleans cannot be multiplied"),
                location: location.clone(),
            }),
            (Value::Closure(_l_closure), Value::Closure(_r_closure)) => Err(RuntimeError {
                message: String::from("invalid numeric operation"),
                full_text: String::from("closures cannot be multiplied"),
                location: location.clone(),
            }),
            (_l_val, _r_val) => Err(RuntimeError {
                message: String::from("invalid numeric operation"),
                full_text: String::from("different types cannot be used on the same operation"),
                location: location.clone(),
            }),
        }
    }

    pub fn div(&self, value: &Value, location: &Location) -> Result<Value, RuntimeError> {
        match (self, value) {
            (Value::Int(l_int), Value::Int(r_int)) => Ok(Value::Int(l_int / r_int)),
            (Value::Str(_l_bool), Value::Str(_r_bool)) => Err(RuntimeError {
                message: String::from("invalid numeric operation"),
                full_text: String::from("strings cannot be divided"),
                location: location.clone(),
            }),
            (Value::Bool(_l_bool), Value::Bool(_r_bool)) => Err(RuntimeError {
                message: String::from("invalid numeric operation"),
                full_text: String::from("booleans cannot be divided"),
                location: location.clone(),
            }),
            (Value::Closure(_l_closure), Value::Closure(_r_closure)) => Err(RuntimeError {
                message: String::from("invalid numeric operation"),
                full_text: String::from("closures cannot be divided"),
                location: location.clone(),
            }),
            (_l_val, _r_val) => Err(RuntimeError {
                message: String::from("invalid numeric operation"),
                full_text: String::from("different types cannot be used on the same operation"),
                location: location.clone(),
            }),
        }
    }

    pub fn rem(&self, value: &Value, location: &Location) -> Result<Value, RuntimeError> {
        match (self, value) {
            (Value::Int(l_int), Value::Int(r_int)) => Ok(Value::Int(l_int / r_int)),
            (Value::Str(_l_bool), Value::Str(_r_bool)) => Err(RuntimeError {
                message: String::from("invalid numeric operation"),
                full_text: String::from("strings cannot be used with rem"),
                location: location.clone(),
            }),
            (Value::Bool(_l_bool), Value::Bool(_r_bool)) => Err(RuntimeError {
                message: String::from("invalid numeric operation"),
                full_text: String::from("booleans cannot be used with rem"),
                location: location.clone(),
            }),
            (Value::Closure(_l_closure), Value::Closure(_r_closure)) => Err(RuntimeError {
                message: String::from("invalid numeric operation"),
                full_text: String::from("closures cannot be used with rem"),
                location: location.clone(),
            }),
            (_l_val, _r_val) => Err(RuntimeError {
                message: String::from("invalid numeric operation"),
                full_text: String::from("different types cannot be used on the same operation"),
                location: location.clone(),
            }),
        }
    }
}

fn eval_let(let_: Let, context: &Context, cache: &mut Cache) -> Result<Value, RuntimeError> {
    let value = eval(let_.value, context, cache)?;
    let context = context.update(let_.name.text, value);

    eval(let_.next, &context, cache)
}

fn update_context(
    parameters: &[Var],
    arguments: &[Value],
    acc: Context,
    location: Location,
) -> Result<Context, RuntimeError> {
    match (parameters, arguments) {
        ([], [_]) | ([_], []) | ([], [_, ..]) | ([_, ..], []) => Err(RuntimeError {
            message: String::from("invalid arguments"),
            full_text: format!(
                "expecting {} arguments but got {}",
                parameters.len(),
                arguments.len()
            ),
            location,
        }),
        ([], []) => Ok(acc),
        ([parameter], [argument]) => Ok(acc.update(parameter.text.clone(), argument.clone())),
        ([parameter, parameters @ ..], [argument, arguments @ ..]) => {
            let acc = acc.update(parameter.text.clone(), argument.clone());

            update_context(parameters, arguments, acc, location)
        }
    }
}

fn eval_arguments<'a>(
    arguments: &'a [Term],
    acc: Vec<Value>,
    context: &Context,
    cache: &mut Cache,
) -> Result<Vec<Value>, RuntimeError> {
    match arguments {
        [] => Ok(acc),
        [argument, arguments @ ..] => {
            let argument = eval(Box::new(argument.clone()), context, cache)?;
            let acc = [acc, vec![argument]].concat();
            eval_arguments(arguments, acc, context, cache)
        }
    }
}

fn cache_key(body: &Box<Term>, arguments: Vec<Value>) -> Option<String> {
    let arguments = arguments
        .into_iter()
        .map(|argument| match argument {
            Value::Closure(_) => None,
            value => {
                let mut s = DefaultHasher::new();
                // TODO: is ok to define the hasher on each iteration?
                value.hash(&mut s);
                Some(s.finish().to_string())
            }
        })
        .collect::<Option<Vec<String>>>()?;

    let mut s = DefaultHasher::new();
    (*body.clone(), arguments).hash(&mut s);

    Some(s.finish().to_string())
}

fn memo_eval(
    body: Box<Term>,
    arguments: Vec<Value>,
    context: &Context,
    cache: &mut Cache,
) -> Result<Value, RuntimeError> {
    match cache_key(&body, arguments.clone()) {
        Some(cache_key) => match cache.get(&cache_key) {
            Some(cached_value) => Ok(cached_value.clone()),
            None => {
                let value = eval(body, &context, cache)?;
                cache.insert(cache_key, value.clone());

                Ok(value)
            }
        },
        None => eval(body, &context, cache),
    }
}

fn eval_call(call: Call, context: Context, cache: &mut Cache) -> Result<Value, RuntimeError> {
    match eval(call.callee, &context, cache)? {
        Value::Closure(closure) => {
            // TODO: using this approach, closure would have access to values defined before and
            // after the current scope, i.e:
            //
            // let x = 3;
            // let function = () => {y};
            // let y = 4;
            // print(function()): 4

            let context = closure.context.union(context);
            let arguments = eval_arguments(call.arguments.as_slice(), vec![], &context, cache)?;

            let context = update_context(
                closure.parameters.as_slice(),
                arguments.as_slice(),
                context,
                call.location,
            )?;

            match is_pure(&closure.body) {
                true => memo_eval(closure.body, arguments, &context, cache),
                false => eval(closure.body, &context, cache),
            }
        }
        value => Err(RuntimeError {
            message: String::from("invalid function call"),
            full_text: format!("{} cannot be called as a function", value),
            location: call.location,
        }),
    }
}

fn eval_if(if_: If, context: &Context, cache: &mut Cache) -> Result<Value, RuntimeError> {
    let condition_result = eval(if_.condition.clone(), context, cache)?;
    let condition = match condition_result {
        Value::Bool(bool) => Ok(bool),
        _ => Err(RuntimeError {
            message: String::from("invalid if condition"),
            full_text: format!(
                "{} can't be used as an if condition. use a boolean instead",
                condition_result
            ),
            location: if_.condition.location().clone(),
        }),
    }?;

    match condition {
        true => eval(if_.then, context, cache),
        false => eval(if_.otherwise, context, cache),
    }
}

fn eval_binary(
    binary: Binary,
    context: &Context,
    cache: &mut Cache,
) -> Result<Value, RuntimeError> {
    let l_value = eval(binary.lhs.clone(), context, cache)?;
    let r_value = eval(binary.rhs, context, cache)?;

    match binary.op {
        BinaryOp::Eq => l_value.eq(&r_value, binary.lhs.location()),
        BinaryOp::Neq => l_value.neq(&r_value, binary.lhs.location()),
        BinaryOp::Lt => l_value.lt(&r_value, binary.lhs.location()),
        BinaryOp::Lte => l_value.lte(&r_value, binary.lhs.location()),
        BinaryOp::Gt => l_value.gt(&r_value, binary.lhs.location()),
        BinaryOp::Gte => l_value.gte(&r_value, binary.lhs.location()),
        BinaryOp::And => l_value.and(&r_value, binary.lhs.location()),
        BinaryOp::Or => l_value.or(&r_value, binary.lhs.location()),
        BinaryOp::Add => l_value.add(&r_value, binary.lhs.location()),
        BinaryOp::Sub => l_value.sub(&r_value, binary.lhs.location()),
        BinaryOp::Mul => l_value.mul(&r_value, binary.lhs.location()),
        BinaryOp::Div => l_value.div(&r_value, binary.lhs.location()),
        BinaryOp::Rem => l_value.rem(&r_value, binary.lhs.location()),
    }
}

fn eval_var(var: Var, context: &Context) -> Result<Value, RuntimeError> {
    context
        .get(&var.text)
        .ok_or(RuntimeError {
            message: format!("unbound variable \"{}\"", var.text),
            full_text: format!(
                "variable \"{}\" was not defined in the current scope",
                var.text
            ),
            location: var.location,
        })
        .map(|value| value.clone())
}

fn eval_tuple(
    tuple: crate::ast::Tuple,
    context: &Context,
    cache: &mut Cache,
) -> Result<Value, RuntimeError> {
    let first = eval(tuple.first, context, cache)?;
    let second = eval(tuple.second, context, cache)?;

    Ok(Value::Tuple(Tuple {
        first: Box::new(first),
        second: Box::new(second),
    }))
}

fn eval_first(first: First, context: &Context, cache: &mut Cache) -> Result<Value, RuntimeError> {
    match eval(first.value, context, cache)? {
        Value::Tuple(Tuple { first, second: _ }) => Ok(*first),
        _value => Err(RuntimeError {
            message: String::from("invalid expression"),
            full_text: String::from("cannot use first operation from anything but a tuple"),
            location: first.location,
        }),
    }
}

fn eval_second(
    second: Second,
    context: &Context,
    cache: &mut Cache,
) -> Result<Value, RuntimeError> {
    match eval(second.value, context, cache)? {
        Value::Tuple(Tuple { first: _, second }) => Ok(*second),
        _value => Err(RuntimeError {
            message: String::from("invalid expression"),
            full_text: String::from("cannot use second operation from anything but a tuple"),
            location: second.location,
        }),
    }
}

fn eval_print(print: Print, context: &Context, cache: &mut Cache) -> Result<Value, RuntimeError> {
    let print_value = eval(print.value, context, cache)?;
    println!("{}", print_value.clone());

    Ok(print_value)
}

fn is_pure(term: &Term) -> bool {
    match term {
        Term::Function(function) => is_pure(&function.value),
        Term::Print(_) => false,
        _ => true,
    }
}

fn eval_function(function: Function, context: &Context) -> Result<Value, RuntimeError> {
    Ok(Value::Closure(Closure {
        parameters: function.parameters,
        body: function.value.clone(),
        context: context.clone(),
    }))
}

fn eval(term: Box<Term>, context: &Context, cache: &mut Cache) -> Result<Value, RuntimeError> {
    match *term {
        Term::Let(let_) => eval_let(let_, context, cache),
        Term::Int(int) => Ok(Value::Int(int.value)),
        Term::Str(str) => Ok(Value::Str(str.value)),
        Term::Bool(bool) => Ok(Value::Bool(bool.value)),
        Term::Function(function) => eval_function(function, context),
        Term::Call(call) => eval_call(call, context.clone(), cache),
        Term::If(if_) => eval_if(if_, context, cache),
        Term::Binary(binary) => eval_binary(binary, context, cache),
        Term::Var(var) => eval_var(var, context),
        Term::Tuple(tuple) => eval_tuple(tuple, context, cache),
        Term::First(first) => eval_first(first, context, cache),
        Term::Second(second) => eval_second(second, context, cache),
        Term::Print(print) => eval_print(print, context, cache),
    }
}

pub fn eval_file(file: File) -> Result<Value, RuntimeError> {
    let context = Context::new();
    let mut cache = Cache::new();

    eval(Box::new(file.expression), &context, &mut cache)
}

enum Bounce<'a> {
    Let {
        interpreter: &'a mut Interpreter,
        let_: Let,
    },
    Function {
        interpreter: &'a mut Interpreter,
        function: Function,
    },
    Call {
        interpreter: &'a mut Interpreter,
        call: Call,
    },
    If {
        interpreter: &'a mut Interpreter,
        if_: If,
    },
    Binary {
        interpreter: &'a mut Interpreter,
        binary: Binary,
    },
    Var {
        interpreter: &'a mut Interpreter,
        var: Var,
    },
    Tuple {
        interpreter: &'a mut Interpreter,
        tuple: crate::ast::Tuple,
    },
    First {
        interpreter: &'a mut Interpreter,
        first: First,
    },
    Second {
        interpreter: &'a mut Interpreter,
        second: Second,
    },
    Print {
        interpreter: &'a mut Interpreter,
        print: Print,
    },
}

fn bounce_let<'a>(
    interpreter: &'a mut Interpreter,
    let_: Let,
) -> Result<Trampoline<'a>, RuntimeError> {
    let value = interpreter.eval(let_.value.clone()).run()?;
    interpreter.context = interpreter.context.update(let_.name.text.clone(), value);

    Ok(interpreter.eval(let_.next))
}

fn bounce_function<'a>(
    interpreter: &'a mut Interpreter,
    function: Function,
) -> Result<Trampoline<'a>, RuntimeError> {
    Ok(Trampoline::Land(Ok(Value::Closure(Closure {
        parameters: function.parameters,
        body: function.value.clone(),
        context: interpreter.context.clone(),
    }))))
}

fn bounce_call<'a>(
    interpreter: &'a mut Interpreter,
    call: Call,
) -> Result<Trampoline<'a>, RuntimeError> {
    match interpreter.eval(call.callee).run()? {
        Value::Closure(closure) => {
            // TODO: using this approach, closure would have access to values defined before and
            // after the current scope, i.e:
            //
            // let x = 3;
            // let function = () => {y};
            // let y = 4;
            // print(function()): 4

            let context = closure.context.union(interpreter.context.clone());
            let arguments = eval_arguments(
                call.arguments.as_slice(),
                vec![],
                &context,
                &mut interpreter.cache,
            )?;

            let updated_context = update_context(
                closure.parameters.as_slice(),
                arguments.as_slice(),
                context,
                call.location,
            )?;

            interpreter.context = updated_context;

            match is_pure(&closure.body) {
                true => interpreter.memo_eval(closure.body, arguments),
                false => Ok(interpreter.eval(closure.body)),
            }
        }
        value => Err(RuntimeError {
            message: String::from("invalid function call"),
            full_text: format!("{} cannot be called as a function", value),
            location: call.location,
        }),
    }
}

fn bounce_if<'a>(
    interpreter: &'a mut Interpreter,
    if_: If,
) -> Result<Trampoline<'a>, RuntimeError> {
    let condition = interpreter.eval(if_.condition.clone()).run()?;
    let condition = match condition {
        Value::Bool(bool) => Ok(bool),
        _ => Err(RuntimeError {
            message: String::from("invalid if condition"),
            full_text: format!(
                "{} can't be used as an if condition. use a boolean instead",
                condition
            ),
            location: if_.condition.location().clone(),
        }),
    }?;

    Ok(match condition {
        true => interpreter.eval(if_.then),
        false => interpreter.eval(if_.otherwise),
    })
}

fn bounce_binary<'a>(
    interpreter: &'a mut Interpreter,
    binary: Binary,
) -> Result<Trampoline<'a>, RuntimeError> {
    let l_value = interpreter.eval(binary.lhs.clone()).run()?;
    let r_value = interpreter.eval(binary.rhs).run()?;

    let result = match binary.op {
        BinaryOp::Eq => l_value.eq(&r_value, binary.lhs.location()),
        BinaryOp::Neq => l_value.neq(&r_value, binary.lhs.location()),
        BinaryOp::Lt => l_value.lt(&r_value, binary.lhs.location()),
        BinaryOp::Lte => l_value.lte(&r_value, binary.lhs.location()),
        BinaryOp::Gt => l_value.gt(&r_value, binary.lhs.location()),
        BinaryOp::Gte => l_value.gte(&r_value, binary.lhs.location()),
        BinaryOp::And => l_value.and(&r_value, binary.lhs.location()),
        BinaryOp::Or => l_value.or(&r_value, binary.lhs.location()),
        BinaryOp::Add => l_value.add(&r_value, binary.lhs.location()),
        BinaryOp::Sub => l_value.sub(&r_value, binary.lhs.location()),
        BinaryOp::Mul => l_value.mul(&r_value, binary.lhs.location()),
        BinaryOp::Div => l_value.div(&r_value, binary.lhs.location()),
        BinaryOp::Rem => l_value.rem(&r_value, binary.lhs.location()),
    };

    Ok(Trampoline::Land(result))
}

fn bounce_var<'a>(
    interpreter: &'a mut Interpreter,
    var: Var,
) -> Result<Trampoline<'a>, RuntimeError> {
    Ok(Trampoline::Land(
        interpreter
            .context
            .get(&var.text)
            .ok_or(RuntimeError {
                message: format!("unbound variable \"{}\"", var.text),
                full_text: format!(
                    "variable \"{}\" was not defined in the current scope",
                    var.text
                ),
                location: var.location,
            })
            .map(|value| value.clone()),
    ))
}

fn bounce_tuple<'a>(
    interpreter: &'a mut Interpreter,
    tuple: crate::ast::Tuple,
) -> Result<Trampoline<'a>, RuntimeError> {
    let first = interpreter.eval(tuple.first).run()?;
    let second = interpreter.eval(tuple.second).run()?;

    Ok(Trampoline::Land(Ok(Value::Tuple(Tuple {
        first: Box::new(first),
        second: Box::new(second),
    }))))
}

fn bounce_first<'a>(
    interpreter: &'a mut Interpreter,
    first: First,
) -> Result<Trampoline<'a>, RuntimeError> {
    let value = match interpreter.eval(first.value).run()? {
        Value::Tuple(Tuple { first, second: _ }) => Ok(*first),
        _value => Err(RuntimeError {
            message: String::from("invalid expression"),
            full_text: String::from("cannot use first operation from anything but a tuple"),
            location: first.location,
        }),
    };

    Ok(Trampoline::Land(value))
}

fn bounce_second<'a>(
    interpreter: &'a mut Interpreter,
    second: Second,
) -> Result<Trampoline<'a>, RuntimeError> {
    let value = match interpreter.eval(second.value).run()? {
        Value::Tuple(Tuple { first: _, second }) => Ok(*second),
        _value => Err(RuntimeError {
            message: String::from("invalid expression"),
            full_text: String::from("cannot use second operation from anything but a tuple"),
            location: second.location,
        }),
    };

    Ok(Trampoline::Land(value))
}

fn bounce_print<'a>(
    interpreter: &'a mut Interpreter,
    print: Print,
) -> Result<Trampoline<'a>, RuntimeError> {
    let print_value = interpreter.eval(print.value).run()?;
    println!("{}", print_value.clone());

    Ok(Trampoline::Land(Ok(print_value)))
}

impl<'a> Bounce<'a> {
    pub fn run(self) -> Result<Trampoline<'a>, RuntimeError> {
        match self {
            Self::Let { interpreter, let_ } => bounce_let(interpreter, let_),
            Self::Function {
                interpreter,
                function,
            } => bounce_function(interpreter, function),
            Self::Call { interpreter, call } => bounce_call(interpreter, call),
            Self::If { interpreter, if_ } => bounce_if(interpreter, if_),
            Self::Binary {
                interpreter,
                binary,
            } => bounce_binary(interpreter, binary),
            Self::Var { interpreter, var } => bounce_var(interpreter, var),
            Self::Tuple { interpreter, tuple } => bounce_tuple(interpreter, tuple),
            Self::First { interpreter, first } => bounce_first(interpreter, first),
            Self::Second {
                interpreter,
                second,
            } => bounce_second(interpreter, second),
            Self::Print { interpreter, print } => bounce_print(interpreter, print),
            _ => todo!(),
        }
    }
}

enum Trampoline<'a> {
    Bounce(Bounce<'a>),
    Land(Result<Value, RuntimeError>),
}

impl<'a> Trampoline<'a> {
    pub fn run(self) -> Result<Value, RuntimeError> {
        let mut current_trampoline = self;

        loop {
            match current_trampoline {
                Self::Bounce(bounce) => {
                    current_trampoline = bounce.run()?;
                }
                Self::Land(result) => return result.clone(),
            }
        }
    }

    pub fn let_(interpreter: &mut Interpreter, let_: Let) -> Trampoline {
        Trampoline::Bounce(Bounce::Let { interpreter, let_ })
    }

    pub fn function(interpreter: &mut Interpreter, function: Function) -> Trampoline {
        Trampoline::Bounce(Bounce::Function {
            interpreter,
            function,
        })
    }

    pub fn call(interpreter: &mut Interpreter, call: Call) -> Trampoline {
        Trampoline::Bounce(Bounce::Call { interpreter, call })
    }

    pub fn if_(interpreter: &mut Interpreter, if_: If) -> Trampoline {
        Trampoline::Bounce(Bounce::If { interpreter, if_ })
    }

    pub fn binary(interpreter: &mut Interpreter, binary: Binary) -> Trampoline {
        Trampoline::Bounce(Bounce::Binary {
            interpreter,
            binary,
        })
    }

    pub fn var(interpreter: &mut Interpreter, var: Var) -> Trampoline {
        Trampoline::Bounce(Bounce::Var { interpreter, var })
    }

    pub fn tuple(interpreter: &mut Interpreter, tuple: crate::ast::Tuple) -> Trampoline {
        Trampoline::Bounce(Bounce::Tuple { interpreter, tuple })
    }

    pub fn first(interpreter: &mut Interpreter, first: First) -> Trampoline {
        Trampoline::Bounce(Bounce::First { interpreter, first })
    }

    pub fn second(interpreter: &mut Interpreter, second: Second) -> Trampoline {
        Trampoline::Bounce(Bounce::Second {
            interpreter,
            second,
        })
    }

    pub fn print(interpreter: &mut Interpreter, print: Print) -> Trampoline {
        Trampoline::Bounce(Bounce::Print { interpreter, print })
    }
}

#[derive(Clone, Default)]
pub struct Interpreter {
    context: Context,
    cache: Cache,
}

impl Interpreter {
    fn memo_eval<'a>(
        &'a mut self,
        body: Box<Term>,
        arguments: Vec<Value>,
    ) -> Result<Trampoline<'a>, RuntimeError> {
        match cache_key(&body, arguments.clone()) {
            Some(cache_key) => match self.cache.get(&cache_key) {
                Some(cached_value) => Ok(Trampoline::Land(Ok(cached_value.clone()))),
                None => {
                    let value = self.eval(body).run()?;
                    self.cache.insert(cache_key, value.clone());

                    Ok(Trampoline::Land(Ok(value)))
                }
            },
            None => Ok(self.eval(body)),
        }
    }

    fn eval(&mut self, term: Box<Term>) -> Trampoline {
        match *term {
            Term::Int(int) => Trampoline::Land(Ok(Value::Int(int.value))),
            Term::Str(str) => Trampoline::Land(Ok(Value::Str(str.value))),
            Term::Bool(bool) => Trampoline::Land(Ok(Value::Bool(bool.value))),
            Term::Let(let_) => Trampoline::let_(self, let_),
            Term::Function(function) => Trampoline::function(self, function),
            Term::Call(call) => Trampoline::call(self, call),
            Term::If(if_) => Trampoline::if_(self, if_),
            Term::Binary(binary) => Trampoline::binary(self, binary),
            Term::Var(var) => Trampoline::var(self, var),
            Term::Tuple(tuple) => Trampoline::tuple(self, tuple),
            Term::First(first) => Trampoline::first(self, first),
            Term::Second(second) => Trampoline::second(self, second),
            Term::Print(print) => Trampoline::print(self, print),
        }
    }

    pub fn eval_file(file: File) -> Result<Value, RuntimeError> {
        let mut interpreter = Self::default();

        interpreter.eval(Box::new(file.expression)).run()
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{Function, Int, Location, Print, Term};

    use super::is_pure;

    fn location() -> Location {
        Location {
            start: 0,
            end: 0,
            filename: "tests".to_string(),
        }
    }

    fn int() -> Box<Term> {
        Box::new(Term::Int(Int {
            value: 5,
            location: location(),
        }))
    }

    fn function(term: Box<Term>) -> Box<Term> {
        Box::new(Term::Function(Function {
            parameters: vec![],
            value: term,
            location: location(),
        }))
    }

    fn print() -> Box<Term> {
        Box::new(Term::Print(Print {
            value: Box::new(Term::Int(Int {
                value: 1,
                location: location(),
            })),
            location: location(),
        }))
    }

    #[test]
    fn can_infer_function_is_pure() {
        let pure_function = function(int());

        let is_pure = is_pure(&pure_function);
        assert!(is_pure);
    }

    #[test]
    fn can_infer_function_is_inpure() {
        // () => print(5)
        let inpure_function = *function(print());

        let is_pure = is_pure(&inpure_function);
        assert!(!is_pure);
    }

    #[test]
    fn can_infer_two_levels_function_is_inpure() {
        // () => print(5)
        let inpure_function = function(function(print()));
        let is_pure = is_pure(&inpure_function);

        assert!(!is_pure);
    }
}
