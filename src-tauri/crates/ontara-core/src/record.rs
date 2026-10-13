use std::fmt::Debug;

/// A persisted domain record with a stable identity.
pub trait Record: Clone {
    type Id: Clone + Debug + PartialEq;

    fn id(&self) -> Self::Id;
}
