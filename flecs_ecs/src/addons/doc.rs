//! The doc module allows for documenting entities (and thus components, systems)
//! by adding brief and/or detailed descriptions as components. Documentation
//! added with the doc module can be retrieved at runtime, and can be used by
//! tooling such as UIs or documentation frameworks.

use std::ffi::CStr;
use std::ptr::NonNull;

use crate::core::*;
use crate::sys;

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
    /// * [`World::get_doc_name()`]
    /// * C++ API: `doc::get_name()`
    fn get_doc_name(&self) -> Option<String> {
        self.world().get_doc_name_id(self.clone())
    }

    /// Get brief description for an entity.
    ///
    /// # Returns
    ///
    /// The brief description of the entity.
    ///
    /// # See also
    ///
    /// * [`World::get_doc_brief()`]
    /// * C++ API: `doc::get_brief()`
    fn get_doc_brief(&self) -> Option<String> {
        self.world().get_doc_brief_id(self.clone())
    }

    /// Get detailed description for an entity.
    ///
    /// # Returns
    ///
    /// The detailed description of the entity.
    ///
    /// # See also
    ///
    /// * [`World::get_doc_detail()`]
    /// * C++ API: `doc::get_detail()`
    fn get_doc_detail(&self) -> Option<String> {
        self.world().get_doc_detail_id(self.clone())
    }

    /// Get link to external documentation for an entity.
    ///
    /// # Returns
    ///
    /// The link to external documentation of the entity.
    ///
    /// # See also
    ///
    /// * [`World::get_doc_link()`]
    /// * C++ API: `doc::get_link()`
    fn get_doc_link(&self) -> Option<String> {
        self.world().get_doc_link_id(self.clone())
    }

    /// Get color for an entity.
    ///
    /// # Returns
    ///
    /// The color of the entity.
    ///
    /// # See also
    ///
    /// * [`World::get_doc_color()`]
    /// * C++ API: `doc::get_color()`
    fn get_doc_color(&self) -> Option<String> {
        self.world().get_doc_color_id(self.clone())
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
    /// * [`World::set_doc_name_id()`]
    /// * C++ API: `doc::set_name()`
    fn set_doc_name(&self, name: &str) -> &Self {
        self.world().set_doc_name_id(self.clone(), name);
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
    /// * [`World::set_doc_brief_id()`]
    /// * C++ API: `doc::set_brief()`
    fn set_doc_brief(&self, brief: &str) -> &Self {
        self.world().set_doc_brief_id(self.clone(), brief);
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
    /// * [`World::set_doc_detail_id()`]
    /// * C++ API: `doc::set_detail()`
    fn set_doc_detail(&self, detail: &str) -> &Self {
        self.world().set_doc_detail_id(self.clone(), detail);
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
    /// * [`World::set_doc_link_id()`]
    /// * C++ API: `doc::set_link()`
    fn set_doc_link(&self, link: &str) -> &Self {
        self.world().set_doc_link_id(self.clone(), link);
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
    /// * [`World::set_doc_color_id()`]
    /// * C++ API: `doc::set_color()`
    fn set_doc_color(&self, color: &str) -> &Self {
        self.world().set_doc_color_id(self.clone(), color);
        self
    }
}

impl<'a, T> Doc<'a> for T where T: Into<Entity> + WorldProvider<'a> + Clone {}

//MARK: impl World
/// ```
/// use flecs_ecs::{addons::doc::Doc, core::World, macros::Component};
///
/// #[derive(Component)]
/// struct Tag;
///
/// let world = World::default();
/// world.component::<Tag>();
/// world.set_doc_name::<Tag>("A tag");
/// ```
impl World {
    //MARK: _World::getters

    /// Get human readable name for an entity.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type that implements `ComponentId`.
    ///
    /// # Returns
    ///
    /// The human readable name of the entity.
    ///
    /// # See also
    ///
    /// * [`Doc::get_doc_name()`]
    /// * [`World::get_doc_name_id()`]
    /// * C++ API: `doc::get_name()`
    #[doc(alias = "doc::get_name")]
    #[inline(always)]
    pub fn get_doc_name<T: ComponentId>(&self) -> Option<String> {
        self.get_doc_name_id(T::get_id(self))
    }

    /// Get human readable name for an entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity for which to get the human readable name.
    ///
    /// # Returns
    ///
    /// The human readable name of the entity.
    ///
    /// # See also
    ///
    /// * [`Doc::get_doc_name()`]
    /// * [`World::get_doc_name()`]
    /// * C++ API: `world::get_doc_name()`
    #[doc(alias = "doc::get_name")]
    #[inline(always)]
    pub fn get_doc_name_id(&self, entity: impl Into<Entity>) -> Option<String> {
        let cstr = NonNull::new(
            unsafe { sys::ecs_doc_get_name(self.world_ptr(), *entity.into()) } as *mut _,
        )
        .map(|s| unsafe { CStr::from_ptr(s.as_ptr()) });
        cstr.and_then(|s| s.to_str().ok().map(|s| s.to_string()))
    }

    /// Get brief description for an entity.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type that implements `ComponentId`.
    ///
    /// # Returns
    ///
    /// The brief description of the entity.
    ///
    /// # See also
    ///
    /// * [`Doc::get_doc_brief()`]
    /// * [`World::get_doc_brief_id()`]
    /// * C++ API: `doc::get_brief()`
    #[doc(alias = "doc::get_brief")]
    #[inline(always)]
    pub fn get_doc_brief<T: ComponentId>(&self) -> Option<String> {
        self.get_doc_brief_id(T::get_id(self))
    }

    /// Get brief description for an entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity for which to get the brief description
    ///
    /// # Returns
    ///
    /// The brief description of the entity.
    ///
    /// # See also
    ///
    /// * [`Doc::get_doc_brief()`]
    /// * [`World::get_doc_brief()`]
    /// * C++ API: `doc::get_brief`
    #[doc(alias = "doc::get_brief")]
    #[inline(always)]
    pub fn get_doc_brief_id(&self, entity: impl Into<Entity>) -> Option<String> {
        let cstr = NonNull::new(
            unsafe { sys::ecs_doc_get_brief(self.world_ptr(), *entity.into()) } as *mut _,
        )
        .map(|s| unsafe { CStr::from_ptr(s.as_ptr()) });
        cstr.and_then(|s| s.to_str().ok().map(|s| s.to_string()))
    }

    /// Get detailed description for an entity.
    /// # Type Parameters
    ///
    /// * `T` - The type that implements `ComponentId`.
    ///
    /// # Returns
    ///
    /// The detailed description of the entity.
    ///
    /// # See also
    ///
    /// * [`Doc::get_doc_detail()`]
    /// * [`World::get_doc_detail_id()`]
    /// * C++ API: `doc::get_detail()`
    #[doc(alias = "doc::get_detail")]
    #[inline(always)]
    pub fn get_doc_detail<T: ComponentId>(&self) -> Option<String> {
        self.get_doc_detail_id(T::get_id(self))
    }

    /// Get detailed description for an entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity for which to get the detailed description.
    ///
    /// # Returns
    ///
    /// The detailed description of the entity.
    ///
    /// # See also
    ///
    /// * [`Doc::get_doc_detail()`]
    /// * [`World::get_doc_detail()`]
    /// * C++ API: `doc::get_detail`
    #[doc(alias = "doc::get_detail")]
    #[inline(always)]
    pub fn get_doc_detail_id(&self, entity: impl Into<Entity>) -> Option<String> {
        let cstr = NonNull::new(
            unsafe { sys::ecs_doc_get_detail(self.world_ptr(), *entity.into()) } as *mut _,
        )
        .map(|s| unsafe { CStr::from_ptr(s.as_ptr()) });
        cstr.and_then(|s| s.to_str().ok().map(|s| s.to_string()))
    }

    /// Get link to external documentation for an entity.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type that implements `ComponentId`.
    ///
    /// # Returns
    ///
    /// The link to external documentation of the entity.
    ///
    /// # See also
    ///
    /// * [`Doc::get_doc_link()`]
    /// * [`World::get_doc_link_id()`]
    /// * C++ API: `doc::get_link()`
    #[doc(alias = "doc::get_link")]
    #[inline(always)]
    pub fn get_doc_link<T: ComponentId>(&self) -> Option<String> {
        self.get_doc_link_id(T::get_id(self))
    }

    /// Get link to external documentation for an entity.
    /// # Arguments
    ///
    /// * `entity` - The entity for which to get the link to external documentation.
    ///
    /// # Returns
    ///
    /// The link to external documentation of the entity.
    ///
    /// # See also
    ///
    /// * [`Doc::get_doc_link()`]
    /// * [`World::get_doc_link()`]
    /// * C++ API: `doc::get_link`
    #[doc(alias = "doc::get_link")]
    #[inline(always)]
    pub fn get_doc_link_id(&self, entity: impl Into<Entity>) -> Option<String> {
        let cstr = NonNull::new(
            unsafe { sys::ecs_doc_get_link(self.world_ptr(), *entity.into()) } as *mut _,
        )
        .map(|s| unsafe { CStr::from_ptr(s.as_ptr()) });
        cstr.and_then(|s| s.to_str().ok().map(|s| s.to_string()))
    }

    /// Get color for an entity.
    /// # Type Parameters
    ///
    /// * `T` - The type that implements `ComponentId`.
    ///
    /// # Returns
    ///
    /// The color of the entity.
    ///
    /// # See also
    ///
    /// * [`Doc::get_doc_color()`]
    /// * [`World::get_doc_color_id()`]
    /// * C++ API: `doc::get_color()`
    #[doc(alias = "doc::get_color")]
    #[inline(always)]
    pub fn get_doc_color<T: ComponentId>(&self) -> Option<String> {
        self.get_doc_color_id(T::get_id(self))
    }

    /// Get color for an entity.
    /// # Arguments
    ///
    /// * `entity` - The entity for which to get the color.
    ///
    /// # Returns
    ///
    /// The color of the entity.
    ///
    /// # See also
    ///
    /// * [`Doc::get_doc_color()`]
    /// * [`World::get_doc_color()`]
    /// * C++ API: `doc::get_color`
    #[doc(alias = "doc::get_color")]
    #[inline(always)]
    pub fn get_doc_color_id(&self, entity: impl Into<Entity>) -> Option<String> {
        let cstr = NonNull::new(
            unsafe { sys::ecs_doc_get_color(self.world_ptr(), *entity.into()) } as *mut _,
        )
        .map(|s| unsafe { CStr::from_ptr(s.as_ptr()) });
        cstr.and_then(|s| s.to_str().ok().map(|s| s.to_string()))
    }

    //MARK: _World::setters

    /// Add human-readable name to entity.
    ///
    /// Contrary to entity names, human readable names do not have to be unique and
    /// can contain special characters used in the query language like '*'.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type that implements `ComponentId`.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to add.
    ///
    /// # See also
    ///
    /// * [`Doc::set_doc_name()`]
    /// * [`World::set_doc_name_id()`]
    /// * C++ API: `doc::get_name()`
    #[doc(alias = "world::set_doc_name")]
    #[inline(always)]
    pub fn set_doc_name<T: ComponentId>(&self, name: &str) {
        self.set_doc_name_id(T::get_id(self), name);
    }

    /// Add human-readable name to entity.
    ///
    /// Contrary to entity names, human readable names do not have to be unique and
    /// can contain special characters used in the query language like '*'.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to which to add the name.
    /// * `name` - The name to add.
    ///
    /// # See also
    ///
    /// * [`Doc::set_doc_name()`]
    /// * [`World::set_doc_name()`]
    /// * C++ API: `world::set_doc_name()`
    #[doc(alias = "world::set_doc_name")]
    #[inline(always)]
    pub fn set_doc_name_id(&self, entity: impl Into<Entity>, name: &str) {
        let name = compact_str::format_compact!("{}\0", name);
        unsafe { sys::ecs_doc_set_name(self.ptr_mut(), *entity.into(), name.as_ptr() as *const _) };
    }

    /// Add brief description to entity.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type that implements `ComponentId`.
    ///
    /// # Arguments
    ///
    /// * `brief` - The brief description to add.
    ///
    /// # See also
    ///
    /// * [`Doc::set_doc_brief()`]
    /// * [`World::set_doc_brief_id()`]
    /// * C++ API: `world::set_doc_brief()`
    #[doc(alias = "world::set_doc_brief")]
    #[inline(always)]
    pub fn set_doc_brief<T: ComponentId>(&self, brief: &str) {
        self.set_doc_brief_id(T::get_id(self), brief);
    }

    /// Add brief description to entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to which to add the brief description.
    /// * `brief` - The brief description to add.
    ///
    /// # See also
    ///
    /// * [`Doc::set_doc_brief()`]
    /// * [`World::set_doc_brief()`]
    /// * C++ API: `world::set_doc_brief()`
    #[doc(alias = "world::set_doc_brief")]
    #[inline(always)]
    pub fn set_doc_brief_id(&self, entity: impl Into<Entity>, brief: &str) {
        let brief = compact_str::format_compact!("{}\0", brief);
        unsafe {
            sys::ecs_doc_set_brief(self.ptr_mut(), *entity.into(), brief.as_ptr() as *const _);
        };
    }

    /// Add detailed description to entity.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type that implements `ComponentId`.
    ///
    /// # Arguments
    ///
    /// * `detail` - The detailed description to add.
    ///
    /// # See also
    ///
    /// * [`Doc::set_doc_detail()`]
    /// * [`World::set_doc_detail_id()`]
    /// * C++ API: `world::set_doc_detail()`
    #[doc(alias = "world::set_doc_detail")]
    #[inline(always)]
    pub fn set_doc_detail<T: ComponentId>(&self, detail: &str) {
        self.set_doc_detail_id(T::get_id(self), detail);
    }

    /// Add detailed description to entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to which to add the detailed description.
    /// * `detail` - The detailed description to add.
    ///
    /// # See also
    ///
    /// * [`Doc::set_doc_detail()`]
    /// * [`World::set_doc_detail()`]
    /// * C++ API: `world::set_doc_detail()`
    #[doc(alias = "world::set_doc_detail")]
    #[inline(always)]
    pub fn set_doc_detail_id(&self, entity: impl Into<Entity>, detail: &str) {
        let detail = compact_str::format_compact!("{}\0", detail);
        unsafe {
            sys::ecs_doc_set_detail(self.ptr_mut(), *entity.into(), detail.as_ptr() as *const _);
        };
    }

    /// Add link to external documentation to entity.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type that implements `ComponentId`.
    ///
    /// # Arguments
    ///
    /// * `link` - The link to add.
    ///
    /// # See also
    ///
    /// * [`Doc::set_doc_link()`]
    /// * [`World::set_doc_link_id()`]
    /// * C++ API: `world::set_doc_link()`
    #[doc(alias = "world::set_doc_link")]
    #[inline(always)]
    pub fn set_doc_link<T: ComponentId>(&self, link: &str) {
        self.set_doc_link_id(T::get_id(self), link);
    }

    /// Add link to external documentation to entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to which to add the link.
    /// * `link` - The link to add.
    ///
    /// # See also
    ///
    /// * [`Doc::set_doc_link()`]
    /// * [`World::set_doc_link()`]
    /// * C++ API: `world::set_doc_link()`
    #[doc(alias = "world::set_doc_link")]
    #[inline(always)]
    pub fn set_doc_link_id(&self, entity: impl Into<Entity>, link: &str) {
        let link = compact_str::format_compact!("{}\0", link);
        unsafe { sys::ecs_doc_set_link(self.ptr_mut(), *entity.into(), link.as_ptr() as *const _) };
    }

    /// Add color to entity.
    ///
    /// UIs can use color as hint to improve visualizing entities.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type that implements `ComponentId`.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to add.
    ///
    /// # See also
    ///
    /// * [`Doc::set_doc_color()`]
    /// * [`World::set_doc_color_id()`]
    /// * C++ API: `world::set_doc_color()`
    #[doc(alias = "world::set_doc_color")]
    #[inline(always)]
    pub fn set_doc_color<T: ComponentId>(&self, color: &str) {
        self.set_doc_color_id(T::get_id(self), color);
    }

    /// Add color to entity.
    ///
    /// UIs can use color as hint to improve visualizing entities.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to which to add the color.
    /// * `color` - The color to add.
    ///
    /// # See also
    ///
    /// * [`Doc::set_doc_color()`]
    /// * [`World::set_doc_color()`]
    /// * C++ API: `world::set_doc_color()`
    #[doc(alias = "world::set_doc_color")]
    #[inline(always)]
    pub fn set_doc_color_id(&self, entity: impl Into<Entity>, color: &str) {
        let color = compact_str::format_compact!("{}\0", color);
        unsafe {
            sys::ecs_doc_set_color(self.ptr_mut(), *entity.into(), color.as_ptr() as *const _);
        };
    }
}

#[test]
fn test_compile_doc() {
    #[derive(flecs_ecs_derive::Component)]
    struct Tag;

    let world = World::default();
    let entity = world.entity();
    entity.set_doc_name("name");

    let query = world.query::<()>().set_cached().build();
    query.set_doc_name("name");

    let system = world.system::<()>().build();
    system.set_doc_name("name");

    let observer = world
        .observer::<flecs::OnAdd, ()>()
        .with::<Tag>()
        .run(|_| {});
    observer.set_doc_name("name");

    let comp = world.component::<()>();
    comp.set_doc_name("name").set_doc_brief("Unit");
}
