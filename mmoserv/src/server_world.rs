use std::sync::Arc;

use clap::Parser;
use futures::Future;
use hashbrown::{HashMap, HashSet};
use mmolib::component;
use redis::{aio::MultiplexedConnection, AsyncCommands, Client, RedisError};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{args, change_tracker::Change, query};

pub fn get_redis_connection_string(host: &str, port: u16) -> String {
    format!("redis://{}:{}/", host, port)
}

#[derive(Serialize, Deserialize, Clone)]
struct Bob {}
impl mmolib::component::ComponentType for Bob {}

#[derive(Debug)]
pub enum ServerWorldError {
    RedisError(RedisError),
    SerdeError(serde_json::Error),
    ComponentChanged,
    ComponentNotFound
}

#[derive(Clone)]
pub struct ServerWorld {
    world_name: Arc<String>,
    conn: MultiplexedConnection,
    cached_queries: Arc<RwLock<HashMap<query::Query, query::QueryResult>>>,
    cached_components: Arc<RwLock<HashMap<mmolib::component::ComponentInstanceId, mmolib::component::Component>>>,
    changes: Arc<
        RwLock<
            HashMap<
                mmolib::component::ComponentInstanceId,
                Change,
            >,
        >,
    >,
    raws: Arc<mmolib::raws::RawTree>,
    tick: u64,
}

impl ServerWorld {
    pub async fn new(
        connection_url: &str,
        world_name: &str,
        raw_path: &str,
    ) -> Result<ServerWorld, ServerWorldError> {
        let x = redis::Client::open(get_redis_connection_string(connection_url, 6379))
            .map_err(|e| ServerWorldError::RedisError(e))?;
        Ok(ServerWorld {
            conn: x
                .get_multiplexed_tokio_connection()
                .await
                .map_err(|e| ServerWorldError::RedisError(e))?,
            world_name: Arc::new(world_name.to_owned()),
            changes: Arc::new(RwLock::new(HashMap::new())),
            raws: Arc::new(mmolib::raws::RawTree::new(raw_path)),
            tick: 0,
            cached_queries: Arc::new(RwLock::new(HashMap::new())),
            cached_components: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    async fn write_component<T: mmolib::component::ComponentType + 'static>(
        &self,
        entity_id: mmolib::entity_id::EntityId,
        component: &T,
    ) -> Result<(), ServerWorldError> {
        let mut conn = self.conn.clone();
        let component_data_key = format!(
            "{}:{}:{}",
            self.world_name,
            entity_id.id(),
            mmolib::component::get_type_id::<T>().get_number()
        );
        let entity_key = format!("{}:{}", self.world_name, entity_id.id());
        let component_entity_key = format!(
            "{}:{}",
            self.world_name,
            mmolib::component::get_type_id::<T>().get_number()
        );
        let serialization =
            serde_json::to_string(component).map_err(|e| ServerWorldError::SerdeError(e))?;
        conn.set(component_data_key, serialization)
            .await
            .map_err(|e| ServerWorldError::RedisError(e))?;
        conn.sadd(
            entity_key,
            mmolib::component::get_type_id::<T>().get_number(),
        )
        .await
        .map_err(|e| ServerWorldError::RedisError(e))?;
        conn.sadd(component_entity_key, entity_id.id())
            .await
            .map_err(|e| ServerWorldError::RedisError(e))?;
        Ok(())
    }
    async fn delete_component<T: mmolib::component::ComponentType + 'static>(
        &self,
        entity_id: mmolib::entity_id::EntityId,
    ) -> Result<(), ServerWorldError> {
        let mut conn = self.conn.clone();
        let component_data_key = format!(
            "{}:{}:{}",
            self.world_name,
            entity_id.id(),
            mmolib::component::get_type_id::<T>().get_number()
        );
        let entity_key = format!("{}:{}", self.world_name, entity_id.id());
        let component_entity_key = format!(
            "{}:{}",
            self.world_name,
            mmolib::component::get_type_id::<T>().get_number()
        );
        conn.del(component_data_key)
            .await
            .map_err(|e| ServerWorldError::RedisError(e))?;
        conn.srem(
            entity_key,
            mmolib::component::get_type_id::<T>().get_number(),
        )
        .await
        .map_err(|e| ServerWorldError::RedisError(e))?;
        conn.srem(component_entity_key, entity_id.id())
            .await
            .map_err(|e| ServerWorldError::RedisError(e))?;
        Ok(())
    }
    async fn get_entities_with_component_type_ids(
        &self,
        component_type_ids: impl IntoIterator<Item = mmolib::component::ComponentTypeId>,
    ) -> Result<HashSet<mmolib::entity_id::EntityId>, ServerWorldError> {
        let mut conn = self.conn.clone();
        Ok(conn
            .sinter::<Vec<String>, Vec<u64>>(
                component_type_ids
                    .into_iter()
                    .map(|x| format!("{}:{}", self.world_name, x.get_number()))
                    .collect::<Vec<String>>(),
            )
            .await
            .map_err(|e| ServerWorldError::RedisError(e))?
            .iter()
            .map(|x| mmolib::entity_id::EntityId::new_with_number(*x))
            .collect())
    }
    async fn get_component<T: mmolib::component::ComponentType + 'static>(
        &self,
        entity_id: mmolib::entity_id::EntityId,
    ) -> Result<T, ServerWorldError> {
        let s = self
            .conn
            .clone()
            .get::<&str, String>(&format!(
                "{}:{}:{}",
                &self.world_name,
                entity_id.id(),
                mmolib::component::get_type_id::<T>()
            ))
            .await
            .map_err(|e| ServerWorldError::RedisError(e))?;
        let r: T = serde_json::from_str(&s).map_err(|e| ServerWorldError::SerdeError(e))?;
        Ok(r)
    }
    pub async fn get_component_ref<T: mmolib::component::ComponentType + 'static>(
        &self,
        entity_id: mmolib::entity_id::EntityId,
    ) -> Result<mmolib::component::ComponentRef<T>, ServerWorldError> {
        //check if we have a cached version
        if let Some(component) = self.cached_components.read().await.get(&mmolib::component::ComponentInstanceId::new::<T>(entity_id)) {
            if let Some(result) = component.get_ref::<T>() {
                Ok(result)
            } else {
                Err(ServerWorldError::ComponentNotFound)
            }
        } 
        //if not try to load it from redis
        else if let Ok(component) = self.get_component::<T>(entity_id).await {
            //get instance id of component
            let id = mmolib::component::ComponentInstanceId::new::<T>(entity_id);
            //insert it into the cache
            let mut cache = self.cached_components.write().await;
            cache.insert(id, mmolib::component::Component::new(component));
            //return it
            Ok(cache.get(&id).unwrap().get_ref::<T>().unwrap())
        }
        //otherwise fail
        else {
            Err(ServerWorldError::ComponentNotFound)
        }
        
    }
    pub async fn query(&self) -> &query::Query {
        query::Query::new(self.clone());
        todo!()
    }
}

#[tokio::test]
async fn test_connection() -> () {
    //let args = args::Args::parse();
    //let s = get_redis_connection_string(&args.database_host, 6379);
    let client = redis::Client::open(get_redis_connection_string("dockercuck.prizrak.me", 6379))
        .expect("could not open connection");
    let mut con = client
        .get_async_connection()
        .await
        .expect("could not get connection");
    // throw away the result, just make sure it does not fail
    let _: () = con.set("my_key", 42).await.expect("could not set key");
    // read back the key and return it.  Because the return value
    // from the function is a result for integer this will automatically
    // convert into one.
    let r = con
        .get::<&str, i64>("my_key")
        .await
        .expect("could not get key");
    println!("result {} ", r);
}
