use mongodb::bson::{doc, oid::ObjectId, Document};
use serde::{de::DeserializeOwned, Serialize};

/// Models used by [`crate::handlers::handler_factory`] (Natours `handlerFactory.js`).
pub trait FactoryModel: Serialize + DeserializeOwned + Send + Sync + Unpin + 'static {
    fn collection_name() -> &'static str;

    /// Merged into every list/find filter (e.g. hide `secretTour`, inactive users).
    fn list_filter() -> Document {
        doc! {}
    }

    /// Default field projection for list queries (`APIFeatures.limitFields`).
    fn list_projection() -> Option<Document> {
        None
    }

    /// Called before `createOne` insert (e.g. slug generation).
    fn prepare_create(&mut self) {}

    /// Set `_id` on the in-memory doc after insert (for API response).
    fn set_id(&mut self, id: ObjectId) {
        let _ = id;
    }
}
