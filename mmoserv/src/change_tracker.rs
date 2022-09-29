use hashbrown::HashMap;

pub struct Change(ChangeType, mmolib::component::ComponentTypeId);
pub enum ChangeType {
    Add(mmolib::component::Component),
    Remove,
    Change(mmolib::component::Component),
}
pub struct ChangeTracker {
    change_type: ChangeType,
    key: (
        mmolib::entity_id::EntityId,
        mmolib::component::ComponentTypeId,
    ),
}

impl ChangeTracker {
    pub fn new_add<T: mmolib::component::ComponentType + 'static>(
        entity_id: mmolib::entity_id::EntityId,
        data: T,
    ) -> Self {
        ChangeTracker {
            change_type: ChangeType::Add(mmolib::component::Component::new(data)),
            key: (entity_id, mmolib::component::get_type_id::<T>()),
        }
    }
    pub fn new_remove(
        entity_id: mmolib::entity_id::EntityId,
        component_type_id: mmolib::component::ComponentTypeId,
    ) -> Self {
        ChangeTracker {
            change_type: ChangeType::Remove,
            key: (entity_id, component_type_id),
        }
    }
    pub fn new_change<T: mmolib::component::ComponentType + 'static>(
        entity_id: mmolib::entity_id::EntityId,
        data: T,
    ) -> Self {
        ChangeTracker {
            change_type: ChangeType::Change(mmolib::component::Component::new(data)),
            key: (entity_id, mmolib::component::get_type_id::<T>()),
        }
    }
    pub fn get_entity_id(&self) -> mmolib::entity_id::EntityId {
        self.key.0
    }
    pub fn get_component_type(&self) -> mmolib::component::ComponentTypeId {
        self.key.1
    }
    pub fn get_change_type(&self) -> &ChangeType {
        &self.change_type
    }
    pub fn get_key(
        &self,
    ) -> (
        mmolib::entity_id::EntityId,
        mmolib::component::ComponentTypeId,
    ) {
        self.key
    }
}
