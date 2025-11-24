use std::any::Any;
use std::hash::BuildHasherDefault;

use crate::engine::node_ref::NodeMask;
use rustc_hash::FxHasher;

pub mod custom_element;
pub mod dioxus;
pub mod layout_attributes;
pub mod node;
pub mod node_ref;
pub mod node_watcher;
pub mod passes;
pub mod real_dom;
pub mod tree;
pub mod utils;

pub use shipyard::EntityId as NodeId;

pub mod exports {
    //! Important dependencies that are used by the rest of the engine
    //! Re-exported here so downstream code can use the same versions.
    #[doc(hidden)]
    pub use rustc_hash::FxHashSet;
    pub use shipyard;
}

/// A prelude of commonly used engine items
pub mod prelude {
    pub use crate::engine::dioxus::*;
    pub use crate::engine::node::{ElementNode, FromAnyValue, NodeType, OwnedAttributeView, TextNode};
    pub use crate::engine::node_ref::{AttributeMaskBuilder, NodeMaskBuilder, NodeView};
    pub use crate::engine::passes::{run_pass, PassDirection, RunPassView, TypeErasedState};
    pub use crate::engine::passes::{Dependancy, DependancyView, Dependants, State};
    pub use crate::engine::real_dom::{NodeImmutable, NodeMut, NodeRef, RealDom};
    pub use crate::engine::NodeId;
    pub use crate::engine::SendAnyMap;
}

/// A map that can be sent between threads
pub type FxDashMap<K, V> = dashmap::DashMap<K, V, BuildHasherDefault<FxHasher>>;
/// A set that can be sent between threads
pub type FxDashSet<K> = dashmap::DashSet<K, BuildHasherDefault<FxHasher>>;
/// A map of types that can be sent between threads
pub type SendAnyMap = anymap3::Map<dyn Any + Send + Sync + 'static>;
