use crate::{
    Database, 
    DatabaseError,
    Id,
    parser::TypeBroker,
};

impl Database {
    pub fn populate_pass3_link(&mut self, mut broker: TypeBroker) -> Result<(), DatabaseError> {
        broker.fulfill(self.assets.keys().map(|id| id.clone()).collect::<Vec<_>>(), self);
        Ok(())
    }
}