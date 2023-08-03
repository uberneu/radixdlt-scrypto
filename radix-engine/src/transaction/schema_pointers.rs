use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::package::TypePointer;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use radix_engine_store_interface::interface::SubstateDatabase;
use sbor::rust::prelude::*;

use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::track::TrackedNode;
use crate::transaction::{SystemPartitionDescription, SystemReader};

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub enum SubstateSchemaPointer {
    KeyValueStore,
    Object(PackageAddress, TypePointer),
    TypeInfo,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub enum EventSchemaPointer {
    Function,
    Method(PackageAddress),
}

#[derive(Default, Debug, Clone, ScryptoSbor)]
pub struct SchemaPointers {
    pub substate_schema_pointers:
        IndexMap<NodeId, IndexMap<PartitionNumber, IndexMap<SubstateKey, SubstateSchemaPointer>>>,
    pub event_schema_pointers: IndexMap<EventTypeIdentifier, EventSchemaPointer>,
}

impl SchemaPointers {
    pub fn new<S: SubstateDatabase>(
        substate_db: &S,
        updates: &IndexMap<NodeId, TrackedNode>,
        application_events: &Vec<(EventTypeIdentifier, Vec<u8>)>,
    ) -> Self {
        let substate_schema_pointers = SubstateSchemaMapper::new(substate_db, &updates).run();
        let event_schema_pointers =
            EventSchemaMapper::new(substate_db, &updates, application_events).run();

        SchemaPointers {
            substate_schema_pointers,
            event_schema_pointers,
        }
    }
}

/// Note that the implementation below assumes that substate owned objects can not be
/// detached. If this changes, we will have to account for objects that are removed
/// from a substate.
pub struct SubstateSchemaMapper<'a, S: SubstateDatabase> {
    system_reader: SystemReader<'a, S>,
    tracked: &'a IndexMap<NodeId, TrackedNode>,
}

impl<'a, S: SubstateDatabase> SubstateSchemaMapper<'a, S> {
    pub fn new(substate_db: &'a S, tracked: &'a IndexMap<NodeId, TrackedNode>) -> Self {
        Self {
            system_reader: SystemReader::new(substate_db, tracked),
            tracked,
        }
    }

    pub fn run(
        &self,
    ) -> IndexMap<NodeId, IndexMap<PartitionNumber, IndexMap<SubstateKey, SubstateSchemaPointer>>>
    {
        let mut substate_schema_pointers = index_map_new();
        for (node_id, tracked_node) in self.tracked {
            for (partition_num, tracked_partition) in &tracked_node.tracked_partitions {
                for (_, tracked_substate) in &tracked_partition.substates {
                    let partition_description =
                        self.system_reader.partition_description(partition_num);
                    let schema_pointer = match partition_description {
                        SystemPartitionDescription::Module(module_id, offset) => (|| {
                            let blueprint_id = if let ObjectModuleId::Main = module_id {
                                let main_type_info =
                                    self.system_reader.get_type_info(node_id).unwrap();
                                match main_type_info {
                                    TypeInfoSubstate::Object(info) => {
                                        info.blueprint_info.blueprint_id
                                    }
                                    TypeInfoSubstate::KeyValueStore(..) => {
                                        return SubstateSchemaPointer::KeyValueStore
                                    }
                                    _ => panic!("Unexpected Type Info {:?}", main_type_info),
                                }
                            } else {
                                module_id.static_blueprint().unwrap()
                            };

                            let bp_def = self
                                .system_reader
                                .get_blueprint_definition(&blueprint_id)
                                .unwrap();
                            let type_pointer = bp_def
                                .interface
                                .state
                                .get_type_pointer(&offset, &tracked_substate.substate_key)
                                .unwrap();

                            SubstateSchemaPointer::Object(
                                blueprint_id.package_address,
                                type_pointer,
                            )
                        })(
                        ),
                        SystemPartitionDescription::TypeInfo => SubstateSchemaPointer::TypeInfo,
                    };

                    substate_schema_pointers
                        .entry(node_id.clone())
                        .or_insert(index_map_new())
                        .entry(partition_num.clone())
                        .or_insert(index_map_new())
                        .insert(tracked_substate.substate_key.clone(), schema_pointer);
                }
            }
        }

        substate_schema_pointers
    }
}

/// Note that the implementation below assumes that substate owned objects can not be
/// detached. If this changes, we will have to account for objects that are removed
/// from a substate.
pub struct EventSchemaMapper<'a, S: SubstateDatabase> {
    system_reader: SystemReader<'a, S>,
    application_events: &'a Vec<(EventTypeIdentifier, Vec<u8>)>,
}

impl<'a, S: SubstateDatabase> EventSchemaMapper<'a, S> {
    pub fn new(
        substate_db: &'a S,
        tracked: &'a IndexMap<NodeId, TrackedNode>,
        application_events: &'a Vec<(EventTypeIdentifier, Vec<u8>)>,
    ) -> Self {
        Self {
            system_reader: SystemReader::new(substate_db, tracked),
            application_events,
        }
    }

    pub fn run(&self) -> IndexMap<EventTypeIdentifier, EventSchemaPointer> {
        let mut event_schema_pointers = index_map_new();
        for (event_type_identifier, _) in self.application_events {
            if !event_schema_pointers.contains_key(event_type_identifier) {
                continue;
            }
            let schema_pointer = match &event_type_identifier.0 {
                Emitter::Function(..) => EventSchemaPointer::Function,
                Emitter::Method(node_id, module_id) => {
                    let blueprint_id = if let ObjectModuleId::Main = module_id {
                        let main_type_info = self.system_reader.get_type_info(node_id).unwrap();
                        match main_type_info {
                            TypeInfoSubstate::Object(info) => info.blueprint_info.blueprint_id,
                            _ => panic!("Unexpected Type Info {:?}", main_type_info),
                        }
                    } else {
                        module_id.static_blueprint().unwrap()
                    };
                    EventSchemaPointer::Method(blueprint_id.package_address)
                }
            };

            event_schema_pointers.insert(event_type_identifier.clone(), schema_pointer);
        }

        event_schema_pointers
    }
}