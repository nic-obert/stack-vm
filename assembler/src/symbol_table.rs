use std::rc::Rc;
use std::collections::HashMap;
use std::cell::{Ref, RefCell, UnsafeCell};
use std::borrow::Cow;

use crate::lang::AsmValue;
use crate::tokenizer::SourceToken;


pub struct Symbol<'a> {

    pub source: Rc<SourceToken<'a>>,
    pub name: &'a str,
    pub value: Option<AsmValue>,

}


#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SymbolID(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct StaticID(pub usize);


pub enum StaticValue<'a> {
    StringLiteral(Cow<'a, str>)
}


pub struct SymbolTable<'a> {

    symbols: UnsafeCell<Vec<RefCell<Symbol<'a>>>>,
    symbol_ids: UnsafeCell<HashMap<&'a str, SymbolID>>,
    statics: UnsafeCell<Vec<RefCell<StaticValue<'a>>>>,

}

impl<'a> SymbolTable<'a> {

    pub fn new() -> Self {
        Self {
            symbols: Default::default(),
            symbol_ids: Default::default(),
            statics: Default::default(),
        }
    }


    pub fn declare_static(&self, value: StaticValue<'a>) -> StaticID {

        let statics = unsafe { &mut *self.statics.get() };

        let id = statics.len();
        statics.push(RefCell::new(value));
        StaticID(id)
    }

    
    pub fn get_static(&'a self, id: StaticID) -> Ref<'a, StaticValue<'a>> {
        let statics = unsafe { &*self.statics.get() };
        Ref::map(statics[id.0].borrow(), |val| val)
    }


    pub fn define_symbol(&self, id: SymbolID, value: Option<AsmValue>, definition_source: Rc<SourceToken<'a>>) {
        
        let symbols = unsafe { &mut *self.symbols.get() };
        
        let mut symbol = symbols[id.0].borrow_mut();
        symbol.value = value;
        symbol.source = definition_source;
    }


    /// Declares the given symbol and returns the symbol id if the symbol wasn't declared before.
    /// If the symbol was already declared, return the previous symbol.
    pub fn declare_symbol(&self, name: &'a str, symbol: Symbol<'a>) -> Result<SymbolID, &RefCell<Symbol<'a>>> {

        let symbols = unsafe { &mut *self.symbols.get() };
        let symbol_ids = unsafe { &mut *self.symbol_ids.get() };

        let symbol_id = SymbolID(symbols.len());

        if let Some(prev) = symbol_ids.insert(name, symbol_id) {
            return Err(&symbols[prev.0])
        }

        symbols.push(RefCell::new(symbol));
        Ok(symbol_id)
    }


    pub fn get_symbol_id(&self, name: &str) -> Option<SymbolID> {
        let symbol_ids = unsafe { &*self.symbol_ids.get() };
        symbol_ids.get(name).cloned()
    }


    /// Returns the symbol with the given id.
    /// Assumes the symbol exists in the symbol table and is reachable.
    /// Since the symbol id was issued by the symbol table itself, there shouldn't be unmatched symbol ids.
    pub fn get_symbol(&self, id: SymbolID) -> &RefCell<Symbol<'a>> {
        let symbols = unsafe { &*self.symbols.get() };
        &symbols[id.0]
    }

}

