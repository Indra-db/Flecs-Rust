//! The doc module allows for documenting entities (and thus components, systems)
//! by adding brief and/or detailed descriptions as components. Documentation
//! added with the doc module can be retrieved at runtime, and can be used by
//! tooling such as UIs or documentation frameworks.

use crate::core::*;

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;
use alloc::string::String;

//MARK: trait::Doc
///
///
/// ```
/// use flecs_ecs::{addons::doc::Doc, core::World, macros::Component};
///
/// #[derive(Component)]
/// struct Tag;
///
/// let world = World::default();
/// world.component::<Tag>().set_doc_name("A tag");
///
/// world
///     .entity()
///     .set_doc_brief("A vast expanse of nothingness.");
/// ```
pub trait Doc<'a>: WorldProvider<'a> + Into<Entity> + Clone {
    //MARK: _getters

    /// Get human readable name for an entity.
    ///
    /// # Returns
    ///
    /// The human readable name of the entity.
    ///
    /// # See also
    ///
    /// * [`World::doc_name()`]
    fn doc_name(&self) -> Option<String> {
        self.world().doc_name(self.clone())
    }

    /// Get brief description for an entity.
    ///
    /// # Returns
    ///
    /// The brief description of the entity.
    ///
    /// # See also
    ///
    /// * [`World::doc_brief()`]
    fn doc_brief(&self) -> Option<String> {
        self.world().doc_brief(self.clone())
    }

    /// Get detailed description for an entity.
    ///
    /// # Returns
    ///
    /// The detailed description of the entity.
    ///
    /// # See also
    ///
    /// * [`World::doc_detail()`]
    fn doc_detail(&self) -> Option<String> {
        self.world().doc_detail(self.clone())
    }

    /// Get link to external documentation for an entity.
    ///
    /// # Returns
    ///
    /// The link to external documentation of the entity.
    ///
    /// # See also
    ///
    /// * [`World::doc_link()`]
    fn doc_link(&self) -> Option<String> {
        self.world().doc_link(self.clone())
    }

    /// Get color for an entity.
    ///
    /// # Returns
    ///
    /// The color of the entity.
    ///
    /// # See also
    ///
    /// * [`World::doc_color()`]
    fn doc_color(&self) -> Option<String> {
        self.world().doc_color(self.clone())
    }

    /// Get UUID for entity
    ///
    /// # Returns
    ///
    /// The UUID of the entity.
    ///
    /// # See also
    ///
    /// * [`World::doc_uuid()`]
    /// * [`Doc::set_doc_uuid()`]
    /// * [`World::set_doc_uuid()`]
    fn doc_uuid(&self) -> Option<String> {
        self.world().doc_uuid(self.clone())
    }

    //MARK: _setters

    /// Add human-readable name to entity.
    ///
    /// Contrary to entity names, human readable names do not have to be unique and
    /// can contain special characters used in the query language like '*'.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to add.
    ///
    /// # See also
    ///
    /// * [`World::set_doc_name()`]
    fn set_doc_name(&self, name: &str) -> &Self {
        self.world().set_doc_name(self.clone(), name);
        self
    }

    /// Add brief description to entity.
    ///
    /// # Arguments
    ///
    /// * `brief` - The brief description to add.
    ///
    /// # See also
    ///
    /// * [`World::set_doc_brief()`]
    fn set_doc_brief(&self, brief: &str) -> &Self {
        self.world().set_doc_brief(self.clone(), brief);
        self
    }

    /// Add detailed description to entity.
    ///
    /// # Arguments
    ///
    /// * `detail` - The detailed description to add.
    ///
    /// # See also
    ///
    /// * [`World::set_doc_detail()`]
    fn set_doc_detail(&self, detail: &str) -> &Self {
        self.world().set_doc_detail(self.clone(), detail);
        self
    }

    /// Add link to external documentation to entity.
    ///
    /// # Arguments
    ///
    /// * `link` - The link to add.
    ///
    /// # See also
    ///
    /// * [`World::set_doc_link()`]
    fn set_doc_link(&self, link: &str) -> &Self {
        self.world().set_doc_link(self.clone(), link);
        self
    }

    /// Add color to entity.
    ///
    /// UIs can use color as hint to improve visualizing entities.
    ///
    /// # Arguments
    ///
    /// * `world` - The world.
    /// * `color` - The color to add.
    ///
    /// # See also
    ///
    /// * [`World::set_doc_color()`]
    fn set_doc_color(&self, color: &str) -> &Self {
        self.world().set_doc_color(self.clone(), color);
        self
    }

    /// Set doc UUID.
    /// This adds `(flecs.doc.Description, flecs.doc.Uuid)` to the entity.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The UUID to add.
    ///
    /// # See also
    /// * [`World::set_doc_uuid()`]
    /// * [`World::doc_uuid()`]
    /// * [`Doc::doc_uuid()`]
    fn set_doc_uuid(&self, uuid: &str) -> &Self {
        self.world().set_doc_uuid(self.clone(), uuid);
        self
    }
}

impl<'a, T> Doc<'a> for T where T: Into<Entity> + WorldProvider<'a> + Clone {}
