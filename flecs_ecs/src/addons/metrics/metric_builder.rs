use crate::core::*;
use crate::sys;
use core::ffi::c_char;
use core::mem::ManuallyDrop;

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;
use alloc::format;
use alloc::{string::String, vec::Vec};

/// `MetricBuilder` is a builder pattern for creating metrics.
pub struct MetricBuilder<'a> {
    world: WorldRef<'a>,
    desc: sys::ecs_metric_desc_t,
    created: bool,
    str_ptrs_to_free: Vec<ManuallyDrop<String>>,
}

impl Drop for MetricBuilder<'_> {
    fn drop(&mut self) {
        if !self.created {
            unsafe {
                sys::ecs_metric_init(self.world_ptr_mut(), &self.desc);
            }
        }
        for s in self.str_ptrs_to_free.iter_mut() {
            unsafe { ManuallyDrop::drop(s) };
        }
        self.str_ptrs_to_free.clear();
    }
}

impl<'a> MetricBuilder<'a> {
    /// Create a new `MetricBuilder`.
    ///
    /// # Arguments
    ///
    /// * `world` - Reference to the world.
    /// * `entity` - The entity to associate with the metric.
    pub(crate) fn new(world: &'a World, entity: Entity) -> Self {
        Self {
            world: world.world(),
            desc: sys::ecs_metric_desc_t {
                entity: *entity,
                ..Default::default()
            },
            created: false,
            str_ptrs_to_free: Vec::new(),
        }
    }

    /// Set the member for the metric using an entity.
    ///
    /// # Arguments
    ///
    /// * `e` - The entity representing the member.
    pub fn member_id(&mut self, e: impl Into<Entity>) -> &mut Self {
        self.desc.member = *e.into();
        self
    }

    /// Set the member for the metric using a name.
    /// Set the member for the metric using a name.
    ///
    /// If `desc.id` is set, it will attempt to find the member within the scope
    /// of that component type. Otherwise, it will look up the member in the world.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the member.
    pub fn member_named(&mut self, name: &str) -> &mut Self {
        let mut member_id: Entity = Entity::null();

        if self.desc.id != 0 {
            // Get the type id of desc.id
            let type_id = unsafe { sys::ecs_get_typeid(self.world_ptr_mut(), self.desc.id) };
            if type_id != 0 {
                let ent = EntityView::new_from(self.world(), type_id);
                // Lookup the name in the scope of type_id
                member_id = ent.try_lookup(name).map_or(Entity::null(), |e| *e);
            }
        } else {
            // Lookup the name in the world
            member_id = self.world().try_lookup(name).map_or(Entity::null(), |e| *e);
        }

        if member_id == 0 {
            // TODO: this should be a tracing error log
            ecs_assert!(
                member_id != 0,
                FlecsErrorCode::InvalidParameter,
                "member '{}' not found",
                name
            );
            //eprintln!("member '{}' not found", name);
        }

        self.member_id(member_id)
    }

    /// Set the member for the metric using a component type and member name.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the member within the component.
    pub fn member<T>(&mut self, name: &str) -> &mut Self
    where
        T: ComponentId,
    {
        let id = T::id(self.world());

        let ent = EntityView::new_from(self.world(), id);
        let m = ent.try_lookup(name);

        if m.is_none() {
            // TODO: this should be a tracing error log
            ecs_assert!(
                m.is_some(),
                FlecsErrorCode::InvalidParameter,
                "member '{}' not found in type '{}'",
                name,
                core::any::type_name::<T>()
            );
            // eprintln!(
            //     "member '{}' not found in type '{}'",
            //     name,
            //     core::any::type_name::<T>()
            // );
            return self;
        }

        self.member_id(m.unwrap())
    }

    /// Set the `dotmember` expression for the metric.
    ///
    /// # Arguments
    ///
    /// * `expr` - The dot-separated member expression.
    pub fn dotmember_named(&mut self, expr: &str) -> &mut Self {
        let expr_cstr = ManuallyDrop::new(format!("{}\0", expr));
        self.desc.dotmember = expr_cstr.as_ptr() as *const c_char;
        self.str_ptrs_to_free.push(expr_cstr);
        self
    }

    /// Set the `dotmember` expression and component ID for the metric.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type.
    ///
    /// # Arguments
    ///
    /// * `expr` - The dot-separated member expression.
    pub fn dotmember<T>(&mut self, expr: &str) -> &mut Self
    where
        T: ComponentId,
    {
        let expr_cstr = ManuallyDrop::new(format!("{}\0", expr));
        self.desc.dotmember = expr_cstr.as_ptr() as *const c_char;
        self.str_ptrs_to_free.push(expr_cstr);
        self.desc.id = T::id(self.world());

        self
    }

    /// Set the ID for the metric.
    ///
    /// # Arguments
    ///
    /// * `the_id` - The ID to set.
    pub fn id(&mut self, the_id: impl Into<Id>) -> &mut Self {
        self.desc.id = *the_id.into();
        self
    }

    /// Set the ID for the metric using a pair of entities.
    ///
    /// # Arguments
    ///
    /// * `first` - The first entity in the pair.
    /// * `second` - The second entity in the pair.
    pub fn id_pair(&mut self, first: impl Into<Entity>, second: impl Into<Entity>) -> &mut Self {
        self.desc.id = ecs_pair(*first.into(), *second.into());
        self
    }

    /// Set the ID for the metric using a component type.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type.
    pub fn id_type<T>(&mut self) -> &mut Self
    where
        T: ComponentOrPairId,
    {
        self.id(T::get_id(self.world()))
    }

    /// Set the ID for the metric using a component type as the first element and an entity as the second.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The component type for the first element.
    ///
    /// # Arguments
    ///
    /// * `second` - The second entity in the pair.
    pub fn id_first<First>(&mut self, second: impl Into<Entity>) -> &mut Self
    where
        First: ComponentId,
    {
        self.id_pair(First::id(self.world()), second)
    }

    /// Set the ID for the metric using an entity as the first element and a component type as the second.
    ///
    /// # Type Parameters
    ///
    /// * `Second` - The component type for the second element.
    ///
    /// # Arguments
    ///
    /// * `first` - The first entity in the pair.
    pub fn id_second<Second>(&mut self, first: impl Into<Entity>) -> &mut Self
    where
        Second: ComponentId,
    {
        self.id_pair(first, Second::id(self.world()))
    }

    /// Specify whether the metric should include targets.
    ///
    /// # Arguments
    ///
    /// * `value` - If `true`, includes targets; defaults to `true`.
    pub fn targets(&mut self, value: bool) -> &mut Self {
        self.desc.targets = value;
        self
    }

    /// Set the kind of the metric.
    ///
    /// # Arguments
    ///
    /// * `the_kind` - The entity representing the kind.
    pub fn kind_id(&mut self, the_kind: impl Into<Entity>) -> &mut Self {
        self.desc.kind = *the_kind.into();
        self
    }

    /// Set the kind of the metric using a component type.
    ///
    /// # Type Parameters
    ///
    /// * `Kind` - The component type representing the kind.
    pub fn kind<Kind>(&mut self) -> &mut Self
    where
        Kind: ComponentId,
    {
        self.kind_id(Kind::id(self.world()))
    }

    /// Set a brief description for the metric.
    ///
    /// # Arguments
    ///
    /// * `b` - The brief description.
    pub fn brief(&mut self, brief: &str) -> &mut Self {
        let brief = ManuallyDrop::new(format!("{}\0", brief));
        self.desc.brief = brief.as_ptr() as *const c_char;
        self.str_ptrs_to_free.push(brief);
        self
    }
}

impl<'a> WorldProvider<'a> for MetricBuilder<'a> {
    fn world(&self) -> WorldRef<'a> {
        self.world
    }
}
