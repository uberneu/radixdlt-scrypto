use sbor::basic_well_known_types::*;
use sbor::*;
use scrypto::prelude::*;
use scrypto::schema::*;

static LARGE: u32 = u32::MAX / 2;
static MAX: u32 = u32::MAX;
static ZERO: u32 = 0;

#[no_mangle]
pub extern "C" fn LargeReturnSize_f(_args: u64) -> Slice {
    Slice(LARGE as u64)
}

#[no_mangle]
pub extern "C" fn MaxReturnSize_f(_args: u64) -> Slice {
    Slice(MAX as u64)
}

#[no_mangle]
pub extern "C" fn ZeroReturnSize_f(_args: u64) -> Slice {
    Slice(ZERO as u64)
}

#[no_mangle]
pub extern "C" fn LargeReturnSize_schema() -> Slice {
    let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

    let mut fields = Vec::new();
    fields.push(FieldSchema::normal(
        aggregator.add_child_type_and_descendents::<()>(),
    ));

    let mut functions = BTreeMap::new();
    functions.insert(
        "f".to_string(),
        FunctionSchema {
            receiver: None,
            input: LocalTypeIndex::WellKnown(ANY_ID),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export: ExportSchema::normal("LargeReturnSize_f"),
        },
    );

    let schema = BlueprintSchema {
        outer_blueprint: None,
        schema: generate_full_schema(aggregator),
        fields,
        collections: vec![],
        functions,
        virtual_lazy_load_functions: BTreeMap::new(),
        event_schema: [].into(),
        dependencies: btreeset!(),
        features: btreeset!(),
    };

    let function_auth: BTreeMap<String, AccessRule> = btreemap!(
        "f".to_string() => AccessRule::AllowAll,
    );

    let return_data = scrypto::blueprints::package::BlueprintSetup {
        schema,
        function_auth,
        royalty_config: RoyaltyConfig::default(),
        template: scrypto::blueprints::package::BlueprintTemplate {
            method_auth_template: btreemap!(),
            outer_method_auth_template: btreemap!(),
        },
    };

    ::scrypto::engine::wasm_api::forget_vec(
        ::scrypto::data::scrypto::scrypto_encode(&return_data).unwrap(),
    )
}

#[no_mangle]
pub extern "C" fn MaxReturnSize_schema() -> Slice {
    let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
    let mut fields = Vec::new();
    fields.push(FieldSchema::normal(
        aggregator.add_child_type_and_descendents::<()>(),
    ));

    let mut functions = BTreeMap::new();
    functions.insert(
        "f".to_string(),
        FunctionSchema {
            receiver: None,
            input: LocalTypeIndex::WellKnown(ANY_ID),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export: ExportSchema::normal("MaxReturnSize_f"),
        },
    );

    let schema = BlueprintSchema {
        outer_blueprint: None,
        schema: generate_full_schema(aggregator),
        fields,
        collections: vec![],
        functions,
        virtual_lazy_load_functions: BTreeMap::new(),
        event_schema: [].into(),
        dependencies: btreeset!(),
        features: btreeset!(),
    };

    let function_auth: BTreeMap<String, AccessRule> = btreemap!(
        "f".to_string() => AccessRule::AllowAll,
    );

    let return_data = scrypto::blueprints::package::BlueprintSetup {
        schema,
        function_auth,
        royalty_config: RoyaltyConfig::default(),
        template: scrypto::blueprints::package::BlueprintTemplate {
            method_auth_template: btreemap!(),
            outer_method_auth_template: btreemap!(),
        },
    };

    ::scrypto::engine::wasm_api::forget_vec(
        ::scrypto::data::scrypto::scrypto_encode(&return_data).unwrap(),
    )
}

#[no_mangle]
pub extern "C" fn ZeroReturnSize_schema() -> Slice {
    let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

    let mut fields = Vec::new();
    fields.push(FieldSchema::normal(
        aggregator.add_child_type_and_descendents::<()>(),
    ));

    let mut functions = BTreeMap::new();
    functions.insert(
        "f".to_string(),
        FunctionSchema {
            receiver: None,
            input: LocalTypeIndex::WellKnown(ANY_ID),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export: ExportSchema::normal("ZeroReturnSize_f"),
        },
    );

    let schema = BlueprintSchema {
        outer_blueprint: None,
        schema: generate_full_schema(aggregator),
        fields,
        collections: vec![],
        functions,
        virtual_lazy_load_functions: BTreeMap::new(),
        event_schema: [].into(),
        dependencies: btreeset!(),
        features: btreeset!(),
    };

    let function_auth: BTreeMap<String, AccessRule> = btreemap!(
        "f".to_string() => AccessRule::AllowAll,
    );

    let return_data = scrypto::blueprints::package::BlueprintSetup {
        schema,
        function_auth,
        royalty_config: RoyaltyConfig::default(),
        template: scrypto::blueprints::package::BlueprintTemplate {
            method_auth_template: btreemap!(),
            outer_method_auth_template: btreemap!(),
        },
    };

    ::scrypto::engine::wasm_api::forget_vec(
        ::scrypto::data::scrypto::scrypto_encode(&return_data).unwrap(),
    )
}
