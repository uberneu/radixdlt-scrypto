use crate::model::method_authorization::{HardProofRule, HardProofRuleResource};
use crate::model::{MethodAuthorization, ValidatedData};
use sbor::any::{Fields, Value};
use sbor::*;
use scrypto::engine::types::*;
use scrypto::prelude::{NonFungibleAddress, ProofRuleResource};
use scrypto::resource::ProofRule;
use scrypto::rust::collections::*;
use scrypto::rust::string::String;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;
use scrypto::types::CustomType;

/// A component is an instance of blueprint.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Component {
    package_id: PackageId,
    blueprint_name: String,
    auth_rules: HashMap<String, ProofRule>,
    state: Vec<u8>,
}

impl Component {
    pub fn new(
        package_id: PackageId,
        blueprint_name: String,
        auth_rules: HashMap<String, ProofRule>,
        state: Vec<u8>,
    ) -> Self {
        Self {
            package_id,
            blueprint_name,
            auth_rules,
            state,
        }
    }

    fn get_from_vector<'a>(path: &'a [usize], values: &'a [Value]) -> Option<&'a Value> {
        let (index_slice, extended_path) = path.split_at(1);
        let index = index_slice[0];
        values
            .get(index)
            .and_then(|value| Self::get_from_value(extended_path, value))
    }

    fn get_from_fields<'a>(path: &'a [usize], fields: &'a Fields) -> Option<&'a Value> {
        match fields {
            Fields::Named(values) | Fields::Unnamed(values) => Self::get_from_vector(path, values),
            Fields::Unit => Option::None,
        }
    }

    fn get_from_value<'a>(path: &'a [usize], value: &'a Value) -> Option<&'a Value> {
        if path.is_empty() {
            return Option::Some(value);
        }

        match value {
            Value::Struct(fields) | Value::Enum(_, fields) => Self::get_from_fields(path, fields),
            Value::Array(_, values) | Value::Vec(_, values) => Self::get_from_vector(path, values),
            _ => Option::None,
        }
    }

    fn soft_to_hard_resource(proof_rule_resource: &ProofRuleResource, dom: &Value) -> Option<HardProofRuleResource> {
        match proof_rule_resource {
            ProofRuleResource::FromComponent(path) => {
                match Self::get_from_value(path, dom) {
                    Some(Value::Custom(type_id, bytes)) => {
                        match CustomType::from_id(*type_id).unwrap() {
                            CustomType::ResourceDefId => Option::Some(ResourceDefId::try_from(bytes.as_slice()).unwrap().into()),
                            CustomType::NonFungibleAddress => Option::Some(NonFungibleAddress::try_from(bytes.as_slice()).unwrap().into()),
                            _ => Option::None
                        }
                    }
                    _ => Option::None
                }
            },
            ProofRuleResource::NonFungible(non_fungible_address) => Some(HardProofRuleResource::NonFungible(non_fungible_address.clone())),
            ProofRuleResource::Resource(resource_def_id) => Some(HardProofRuleResource::Resource(resource_def_id.clone())),
        }
    }

    fn soft_to_hard_rule(proof_rule: &ProofRule, dom: &Value) -> HardProofRule {
        match proof_rule {
            ProofRule::This(proof_rule_resource) => {
                match Self::soft_to_hard_resource(proof_rule_resource, dom) {
                    Some(resource) => HardProofRule::This(resource),
                    None => HardProofRule::OneOf(vec![]),
                }
            }
            ProofRule::SomeOfResource(amount, resource_def_id) => {
                HardProofRule::SomeOfResource(*amount, *resource_def_id)
            }
            ProofRule::AllOf(rules) => {
                let hard_rules = rules
                    .into_iter()
                    .map(|proof_rule| Self::soft_to_hard_rule(proof_rule, dom))
                    .collect();
                HardProofRule::AllOf(hard_rules)
            }
            ProofRule::OneOf(rules) => {
                let hard_rules = rules
                    .into_iter()
                    .map(|proof_rule| Self::soft_to_hard_rule(proof_rule, dom))
                    .collect();
                HardProofRule::OneOf(hard_rules)
            }
            ProofRule::CountOf { count, resources } => {
                let mut hard_resources = Vec::new();
                for soft_resource in resources {
                    if let Some(resource) = Self::soft_to_hard_resource(soft_resource, dom) {
                        hard_resources.push(resource);
                    }
                }
                HardProofRule::CountOf {
                    count: *count,
                    resources: hard_resources,
                }
            }
        }
    }

    pub fn initialize_method(&self, method_name: &str) -> (ValidatedData, MethodAuthorization) {
        let data = ValidatedData::from_slice(&self.state).unwrap();
        let authorization = match self.auth_rules.get(method_name) {
            Some(proof_rule) => {
                MethodAuthorization::Protected(Self::soft_to_hard_rule(proof_rule, &data.dom))
            }
            None => MethodAuthorization::Public,
        };

        (data, authorization)
    }

    pub fn auth_rules(&self) -> &HashMap<String, ProofRule> {
        &self.auth_rules
    }

    pub fn package_id(&self) -> PackageId {
        self.package_id
    }

    pub fn blueprint_name(&self) -> &str {
        &self.blueprint_name
    }

    pub fn state(&self) -> &[u8] {
        &self.state
    }

    pub fn set_state(&mut self, new_state: Vec<u8>) {
        self.state = new_state;
    }
}
