use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    fmt::Debug,
    rc::Rc,
};

use uid::Id;

use crate::{
    fru_err_res,
    interpreter::{
        control::{returned, returned_nothing},
        error::FruError,
        expression::FruExpression,
        identifier::{Identifier, OperatorIdentifier},
        scope::Scope,
        statement::FruStatement,
        value::{
            fru_function::FruFunction, fru_object::FruObject, fru_value::FruValue,
            function_helpers::EvaluatedArgumentList, native_object::OfObject,
            operator::AnyOperator,
        },
    },
};

#[derive(Clone)]
pub struct FruType {
    internal: Rc<FruTypeInternal>,
}

#[derive(Clone)]
pub struct FruTypeInternal {
    ident: Identifier,
    type_flavor: TypeFlavor,
    fields: Vec<FruField>,
    // TODO: change for FruField?
    static_fields: RefCell<HashMap<Identifier, FruValue>>,
    properties: HashMap<Identifier, Property>,
    static_properties: HashMap<Identifier, Property>,
    methods: HashMap<Identifier, FruFunction>,
    static_methods: HashMap<Identifier, FruFunction>,
    operators: RefCell<HashMap<OperatorIdentifier, AnyOperator>>,
    scope: Rc<Scope>,
    uid: Id<OfObject>,
}

#[derive(Clone)]
pub struct FruField {
    pub is_public: bool,
    pub ident: Identifier,
    pub type_ident: Option<Identifier>, // useless for now
}

#[derive(Debug, Clone)]
pub struct Property {
    pub ident: Identifier,
    pub getter: Option<Rc<FruExpression>>,
    pub setter: Option<(Identifier, Rc<FruStatement>)>, // ident for value variable
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeFlavor {
    Struct,
    Class,
    Data,
}

impl FruType {
    pub fn new_value(
        ident: Identifier,
        type_flavor: TypeFlavor,
        fields: Vec<FruField>,
        static_fields: RefCell<HashMap<Identifier, FruValue>>,
        properties: HashMap<Identifier, Property>,
        static_properties: HashMap<Identifier, Property>,
        methods: HashMap<Identifier, FruFunction>,
        static_methods: HashMap<Identifier, FruFunction>,
        scope: Rc<Scope>,
    ) -> FruValue {
        FruValue::Type(Self {
            internal: Rc::new(FruTypeInternal {
                ident,
                type_flavor,
                fields,
                static_fields,
                properties,
                methods,
                static_methods,
                static_properties,
                operators: Default::default(),
                scope,
                uid: Id::new(),
            }),
        })
    }

    pub fn get_uid(&self) -> Id<OfObject> {
        self.internal.uid
    }

    pub fn get_ident(&self) -> Identifier {
        self.internal.ident
    }

    pub fn get_type_flavor(&self) -> TypeFlavor {
        self.internal.type_flavor
    }

    pub fn get_scope(&self) -> Rc<Scope> {
        self.internal.scope.clone()
    }

    pub fn get_fields(&self) -> &[FruField] {
        self.internal.fields.as_slice()
    }

    pub fn get_field_k(&self, ident: Identifier) -> Option<usize> {
        self.internal.fields.iter().enumerate().find_map(|(i, field_ident)| {
            if field_ident.ident == ident {
                Some(i)
            } else {
                None
            }
        })
    }

    pub fn get_property(&self, ident: Identifier) -> Option<Property> {
        self.internal.properties.get(&ident).cloned()
    }

    pub fn get_method(&self, ident: Identifier) -> Option<FruFunction> {
        self.internal.methods.get(&ident).cloned()
    }

    /// In this case means static field of method
    pub fn get_prop(&self, ident: Identifier) -> Result<FruValue, FruError> {
        if let Some(field) = self.internal.static_fields.borrow().get(&ident) {
            return Ok(field.clone());
        }

        if let Some(property) = self.internal.static_properties.get(&ident) {
            let new_scope = Scope::new_with_type(self.clone());

            return match &property.getter {
                Some(getter) => returned(getter.evaluate(new_scope)),

                None => fru_err_res!("static property `{}` has no getter", ident),
            };
        }

        if let Some(static_method) = self.internal.static_methods.get(&ident) {
            return Ok(FruValue::Function(Rc::new(FruFunction {
                parameters: static_method.parameters.clone(),
                body: static_method.body.clone(),
                scope: Scope::new_with_type(self.clone()),
            })));
        }

        fru_err_res!("static prop `{}` not found", ident)
    }

    pub fn set_prop(&self, ident: Identifier, value: FruValue) -> Result<(), FruError> {
        if let Some(field) = self.internal.static_fields.borrow_mut().get_mut(&ident) {
            *field = value;
            return Ok(());
        }

        if let Some(property) = self.internal.static_properties.get(&ident) {
            return match &property.setter {
                Some((ident, setter)) => {
                    let new_scope = Scope::new_with_type(self.clone());

                    new_scope.let_variable(*ident, value)?;

                    returned_nothing(setter.execute(new_scope))
                }

                None => fru_err_res!("static property `{}` has no setter", ident),
            };
        }

        fru_err_res!("static prop `{}` not found", ident)
    }

    pub fn get_operator(&self, ident: OperatorIdentifier) -> Option<AnyOperator> {
        self.internal.operators.borrow().get(&ident).cloned()
    }

    pub fn set_operator(
        &self,
        ident: OperatorIdentifier,
        value: AnyOperator,
    ) -> Result<(), FruError> {
        match self.internal.operators.borrow_mut().entry(ident) {
            Entry::Occupied(_) => {
                fru_err_res!("operator `{:?}` is already set", ident.op)
            }
            Entry::Vacant(entry) => {
                entry.insert(value);
                Ok(())
            }
        }
    }

    pub fn instantiate(&self, mut args: EvaluatedArgumentList) -> Result<FruValue, FruError> {
        let mut obj_fields = HashMap::new();

        let fields = self.get_fields();

        for (n, (ident, value)) in args.args.drain(..).enumerate() {
            let ident = match ident {
                Some(ident) => ident,
                None => fields[n].ident,
            };
            if obj_fields.contains_key(&ident) {
                return fru_err_res!("field `{}` is set more than once", ident);
            }
            obj_fields.insert(ident, value);
        }

        let mut args = Vec::new();

        for FruField { ident, .. } in fields {
            match obj_fields.remove(ident) {
                Some(value) => args.push(value),
                None => return fru_err_res!("missing field `{}`", ident),
            }
        }

        if let Some(ident) = obj_fields.keys().next() {
            return fru_err_res!("field `{}` does not exist", *ident);
        }

        Ok(FruObject::new_object(self.clone(), args))
    }
}

impl PartialEq for FruType {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.internal, &other.internal)
    }
}

impl Debug for FruField {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.is_public {
            write!(f, "pub ")?;
        }
        write!(f, "{}", self.ident)?;
        if let Some(type_ident) = &self.type_ident {
            write!(f, ": {}", type_ident)?;
        }
        Ok(())
    }
}

impl Debug for FruType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.internal.ident)
    }
}
