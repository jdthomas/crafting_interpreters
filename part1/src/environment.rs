use std::collections::HashMap;
use std::default::Default;

use crate::interpreter::Object;
use anyhow::Result;

pub struct Enviornment {
    values: Vec<HashMap<String, Object>>,
}

impl Default for Enviornment {
    fn default() -> Self {
        Enviornment {
            values: vec![HashMap::new()],
        }
    }
}

impl Enviornment {
    pub fn new() -> Self {
        Enviornment {
            ..Default::default()
        }
    }

    pub fn push_scope(&mut self) {
        self.values.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        assert!(self.values.len() > 1);
        self.values.pop();
    }

    pub fn define(&mut self, name: String, value: Object) {
        if self.values.last().unwrap().contains_key(&name) {
            // FIXME: Lox parse error: redefinition
        }
        self.values.last_mut().unwrap().insert(name, value);
    }

    pub fn assign(&mut self, name: String, value: Object) -> Result<()> {
        if let Some(v) = self.values.iter_mut().rev().find(|v| v.contains_key(&name)) {
            v.insert(name, value);
            Ok(())
        } else {
            Err(anyhow::anyhow!(format!("Undefined variable '{}'.", name)))
        }
    }

    pub fn get(&self, name: &str) -> Result<Object> {
        if let Some(v) = self.values.iter().rev().find_map(|v| v.get(name)) {
            Ok(v.clone())
        } else {
            Err(anyhow::anyhow!(format!("Undefined variable '{}'.", name)))
        }
    }
}
