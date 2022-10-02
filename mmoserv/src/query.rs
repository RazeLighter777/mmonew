use std::{
    any::Any,
    hash::{Hash, Hasher}, sync::Arc,
};


use futures::future::join_all;
use hashbrown::{HashMap, HashSet};
use mmolib::{component::{ComponentTypeId, ComponentRef}, entity_id::EntityId};
use mmolib;
use crate::server_world::{self, ServerWorld};

enum QueryEntry {
    Union(mmolib::component::ComponentTypeId),
    Option(mmolib::component::ComponentTypeId),
}

pub struct Query {
    entries: Vec<QueryEntry>,
    world: server_world::ServerWorldRef,
}

impl Query {
    pub fn new(world: server_world::ServerWorldRef) -> Self {
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
    pub async fn execute(&self) -> Result<QueryResult, server_world::ServerWorldError> {
        let mut union = Vec::new();
        for entry in &self.entries {
            match entry {
                QueryEntry::Union(x) => {
                    union.push(*x);
                }
                QueryEntry::Option(x) => {
                    todo!()
                }
            }
        }
        let res = self.world.get_entities_with_component_type_ids(union).await?;
        let mut result = QueryResult::new(res, self.world.clone());
        Ok(result)
    }
}

pub struct QueryResult {
    entities: HashSet<EntityId>,
    world: server_world::ServerWorldRef,
}

impl QueryResult {
    fn new(ids : impl IntoIterator<Item = EntityId>, world: server_world::ServerWorldRef) -> Self {
        QueryResult {
            entities : ids.into_iter().collect(),
            world : world,
        }
    }
}

pub struct Entity {
    entity_id : mmolib::entity_id::EntityId,
    server_world : server_world::ServerWorldRef,
}

impl Entity {
    pub async fn get<T: mmolib::component::ComponentType + 'static>(&self) -> Result<mmolib::component::ComponentRef<T>, server_world::ServerWorldError> {
        Ok(self.server_world.get_component_ref::<T>(self.entity_id).await?)
    }
}


impl QueryResult {
    pub fn iter(&self) -> impl Iterator<Item = Entity>  + '_ {
        self.entities.iter().map(|x| Entity {
            entity_id : *x,
            server_world : self.world.clone(),
        })
    }
}


#[tokio::test]
async fn test_query() -> Result<(), server_world::ServerWorldError> {
    let w = ServerWorld::new("dockercuck.prizrak.me","test","C:\\Users\\justin.suess\\Code\\mmonew\\raws").await?;
    //for i in 0..1000 {
        //w.write_component(mmolib::entity_id::EntityId::new(), &mmolib::position::Position { x : 1, y : 2 }).await?;
        // let mut q = Query::new(w.clone());
        // q.add_union::<mmolib::position::Position>();
        // let res = q.execute().await?;
        // for x in res.iter() {
        //     println!("Iterated through entity");
        //     let pos = x.get::<mmolib::position::Position>().await?;
        // }
   //}
   join_all((0..1000).map(|x| { w.write_component(mmolib::entity_id::EntityId::new(), &mmolib::position::Position { x : 1, y : 2 }) })).await;

    Ok(())
}