use std::rc::Rc;
use std::collections::HashMap;
use std::cell::RefCell;
use std::borrow::Cow;

use crate::lang::AsmValue;
use crate::tokenizer::SourceToken;


pub struct Symbol<'a> {

    pub source: Rc<SourceToken<'a>>,
    pub name: &'a str,
    pub value: Option<AsmValue>,

}


struct Scope<'a> {
    symbols: HashMap<&'a str, SymbolID>,
}

impl Scope<'_> {

    pub fn new() -> Self {
        Self {
            symbols: HashMap::new()
        }
    }

}


#[derive(Debug, Clone, Copy)]
pub struct SymbolID(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct StaticID(pub usize);


pub enum StaticValue<'a> {
    StringLiteral(Cow<'a, str>)
}


pub struct SymbolTable<'a> {

    scopes: Vec<Scope<'a>>,
    symbols: Vec<RefCell<Symbol<'a>>>,
    statics: Vec<StaticValue<'a>>

}

impl<'a> SymbolTable<'a> {

    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()], // Start with the global scope already pushed
            symbols: Vec::new(),
            statics: Vec::new()
        }
    }


    pub fn declare_static(&mut self, value: StaticValue<'a>) -> StaticID {
        let id = self.statics.len();
        self.statics.push(value);
        StaticID(id)
    }

    
    // pub fn push_scope(&mut self) {
    //     self.scopes.push(Scope::new());
    // }


    // pub fn pop_scope(&mut self) {
    //     self.scopes.pop();
    // }


    pub fn define_symbol(&self, id: SymbolID, value: Option<AsmValue>, definition_source: Rc<SourceToken<'a>>) {
        let mut symbol = self.symbols[id.0].borrow_mut();
        symbol.value = value;
        symbol.source = definition_source;
    }


    /// Returns None if the symbol is already declared in the current scope.
    pub fn declare_symbol(&mut self, name: &'a str, symbol: Symbol<'a>) -> Result<SymbolID, &RefCell<Symbol<'a>>> {

        let scope = self.scopes.last_mut().unwrap();

        let symbol_id = SymbolID(self.symbols.len());
        if let Some(old_symbol) = scope.symbols.insert(name, symbol_id) {
            return Err(&self.symbols[old_symbol.0]);
        }

        self.symbols.push(RefCell::new(symbol));
        Ok(symbol_id)
    }


    pub fn get_symbol_id(&self, name: &str) -> Option<SymbolID> {
        self.scopes.iter().rev().find_map(|scope| scope.symbols.get(name).cloned())
    }


    /// Returns the symbol with the given id.
    /// Assumes the symbol exists in the symbol table and is reachable.
    /// Since the symbol id was issued by the symbol table itself, there shouldn0t be unmatched symbol ids.
    pub fn get_symbol(&self, id: SymbolID) -> &RefCell<Symbol<'a>> {
        &self.symbols[id.0]
    }

}

