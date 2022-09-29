use std::{
    any::Any,
    hash::{Hash, Hasher},
};

use hashbrown::HashMap;
use mmolib::component::{ComponentTypeId, ComponentRef};

use crate::server_world::{self, ServerWorld};

enum QueryEntry {
    Union(mmolib::component::ComponentTypeId),
    Option(mmolib::component::ComponentTypeId),
}

pub struct Query {
    entries: Vec<QueryEntry>,
    world: ServerWorld,
}

impl Query {
    pub fn new(world: server_world::ServerWorld) -> Self {
        Query {
            entries: Vec::new(),
            world,
        }
    }
    pub fn hash(&self) -> u64 {
        let mut y = 1337;
        for entry in &self.entries {
            match entry {
                QueryEntry::Union(x) => y = y ^ x.get_number() + 1,
                QueryEntry::Option(x) => y = y << x.get_number(),
            }
        }
        y
    }
    pub fn add_union<T: mmolib::component::ComponentType + 'static>(&mut self) {
        self.entries
            .push(QueryEntry::Union(mmolib::component::get_type_id::<T>()));
    }
    pub fn add_option<T: mmolib::component::ComponentType + 'static>(&mut self) {
        self.entries
            .push(QueryEntry::Option(mmolib::component::get_type_id::<T>()));
    }
    pub fn execute(&self) -> QueryResult {
        let mut result = QueryResult {
            entities: Vec::new(),
        };
        //self.world.cache_query_result(&self, result);
        result
    }
}

pub struct QueryResult {
    entities: Vec<Entity>,
}

pub struct Entity {
    entity_id : mmolib::entity_id::EntityId,
    world : ServerWorld,
}

impl Entity {
    pub async fn get_component<T: mmolib::component::ComponentType + 'static>(&self) -> ComponentRef<T> {
        self.world.get_component_ref::<T>(self.entity_id).await.unwrap()
    }
}

impl IntoIterator for QueryResult {
    type Item = Entity;

    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.entities.into_iter()
    }
}
