use std::{any::Any, fmt::Display, sync::Arc, marker::PhantomData};

use serde::{Deserialize, Serialize};

use crate::{hashing, entity_id};

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub struct ComponentTypeId(u64);

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub struct ComponentInstanceId(entity_id::EntityId, ComponentTypeId);

impl ComponentInstanceId {
    pub fn new_explicit(entity_id: entity_id::EntityId, component_type_id: ComponentTypeId) -> Self {
        ComponentInstanceId(entity_id, component_type_id)
    }
    pub fn new<T: ComponentType + 'static>(entity_id: entity_id::EntityId) -> Self {
        ComponentInstanceId(entity_id, get_type_id::<T>())
    }
    pub fn get_component_type_id(&self) -> ComponentTypeId {
        self.1
    }
    pub fn get_entity_id(&self) -> entity_id::EntityId {
        self.0
    }
}

pub const fn get_type_id<DataType: 'static>() -> ComponentTypeId {
    ComponentTypeId(hashing::string_hash(std::any::type_name::<DataType>()))
}

pub const fn get_type_id_from_str(s: &str) -> ComponentTypeId {
    ComponentTypeId(hashing::string_hash(s))
}

impl ComponentTypeId {
    pub fn new_with_number(id: u64) -> Self {
        ComponentTypeId(id)
    }
    pub fn new<T: 'static>() -> Self {
        let type_id = get_type_id::<T>();
        type_id
    }
    pub fn get_number(&self) -> u64 {
        self.0
    }
}


/**
 * A generic (untyped) owned component
 */
pub struct Component {
    type_id: ComponentTypeId,
    data: Arc<dyn Any>,
    serialization_fn: fn(Component) -> String,
}

impl Component {
    /**
     * Create a new component from a struct that implements ComponentType
     * 
     */
    pub fn new<T: ComponentType + 'static>(data: T) -> Self {
        Component {
            type_id: get_type_id::<T>(),
            data: Arc::new(data),
            serialization_fn: |x| serde_json::to_string(&*(x.get_ref::<T>().unwrap())).unwrap(),
        }
    }
    pub fn get_type_id(&self) -> ComponentTypeId {
        self.type_id
    }
    /**
     * Downcast this component to a specific component reference
     */
    pub fn get_ref<T: ComponentType + 'static>(&self) -> Option<ComponentRef<T>> {
        if let Some(x) = self.data.downcast_ref::<T>() {
            Some(ComponentRef { data: self.data.clone(), changed_data : None })
        } else {
            None
        }
    }
    /**
     * Serialize this component
     */
    pub fn serialize(self) -> String {
        (self.serialization_fn)(self)
    }
}

pub trait ComponentType: serde::de::DeserializeOwned + Serialize  + Any + Send + Sync + Clone {}

pub struct ComponentRef<T: ComponentType> {
    data : Arc<dyn Any>,
    changed_data: Option<T>,
}

impl<'a,T: ComponentType> ComponentRef<T> {
    pub fn clear_changed_data(&mut self) {
        self.changed_data = None;
    }
}
impl<T: ComponentType> std::ops::Deref  for ComponentRef< T> {
    //read fields from our component reference
    fn deref(&self) -> &T {
        //we can unwrap here because we know that the data is of type T
        &self.data.downcast_ref::<T>().unwrap()
    }

    type Target = T;
}
impl<T: ComponentType> std::ops::DerefMut for ComponentRef<T> {
    

    //Copy on write
    fn deref_mut(&mut self) -> &mut T {
        if self.changed_data.is_none() {
            self.changed_data = Some(self.data.downcast_ref::<T>().unwrap().clone());
        }
        self.changed_data.as_mut().unwrap()
    }
}




impl Display for ComponentTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "type<{:X}>", self.0)
    }
}
