//! [`AlertBuilder`] is a builder pattern for creating [`Alert`]s.

use crate::core::internals::*;
use crate::core::*;
use crate::sys;

use super::Alert;
use super::SeverityAlert;

/// [`AlertBuilder`] is a builder pattern for creating [`Alert`]s.
pub struct AlertBuilder<'a, T>
where
    T: QueryTuple,
{
    pub(crate) desc: sys::ecs_alert_desc_t,
    term_builder: TermBuilder,
    world: WorldRef<'a>,
    severity_filter_count: i32,
    str_ptrs_to_free: Vec<StringToFree>,
    _phantom: std::marker::PhantomData<&'a T>,
}

impl<'a, T> Drop for AlertBuilder<'a, T>
where
    T: QueryTuple,
{
    fn drop(&mut self) {
        for string_parts in self.str_ptrs_to_free.iter() {
            unsafe {
                String::from_raw_parts(
                    string_parts.ptr as *mut u8,
                    string_parts.len,
                    string_parts.capacity,
                );
            }
        }
        self.str_ptrs_to_free.clear();
    }
}

impl<'a, T> AlertBuilder<'a, T>
where
    T: QueryTuple,
{
    /// Create a new `AlertBuilder`
    pub(crate) fn new(world: &'a World) -> Self {
        let mut obj = Self {
            desc: Default::default(),
            term_builder: TermBuilder::default(),
            world: world.into(),
            severity_filter_count: 0,
            str_ptrs_to_free: Vec::new(),
            _phantom: std::marker::PhantomData,
        };

        T::populate(&mut obj);

        obj
    }

    pub(crate) fn new_from_desc(world: &'a World, desc: sys::ecs_alert_desc_t) -> Self {
        let mut obj = Self {
            desc,
            term_builder: TermBuilder::default(),
            world: world.into(),
            severity_filter_count: 0,
            str_ptrs_to_free: Vec::new(),
            _phantom: std::marker::PhantomData,
        };

        if obj.desc.entity == 0 {
            obj.desc.entity =
                unsafe { sys::ecs_entity_init(obj.world_ptr_mut(), &Default::default()) };
        }

        T::populate(&mut obj);

        #[cfg(feature = "flecs_pipeline")]
        unsafe {
            sys::ecs_add_id(
                world.world_ptr_mut(),
                obj.desc.entity,
                ecs_dependson(ECS_ON_UPDATE),
            );
            sys::ecs_add_id(world.world_ptr_mut(), obj.desc.entity, ECS_ON_UPDATE);
        }

        obj
    }

    /// Create a new `AlertBuilder` with a name
    pub(crate) fn new_named(world: &'a World, name: &str) -> Self {
        let name = compact_str::format_compact!("{}\0", name);

        let mut obj = Self {
            desc: Default::default(),
            term_builder: TermBuilder::default(),
            world: world.into(),
            severity_filter_count: 0,
            str_ptrs_to_free: Vec::new(),
            _phantom: std::marker::PhantomData,
        };

        let entity_desc = sys::ecs_entity_desc_t {
            name: name.as_ptr() as *const _,
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            ..Default::default()
        };

        obj.desc.entity = unsafe { sys::ecs_entity_init(world.world_ptr_mut(), &entity_desc) };

        T::populate(&mut obj);
        obj
    }

    /// Set the alert message.
    ///
    /// # Arguments
    ///
    /// * `message` - The alert message.
    ///
    /// # See also
    ///
    /// * `ecs_alert_desc_t::message`
    #[doc(alias = "ecs_alert_desc_t::message")]
    pub fn message(&mut self, message: &str) -> &mut Self {
        let message = format!("{}\0", message);
        self.str_ptrs_to_free.push(StringToFree {
            ptr: message.as_ptr() as *mut _,
            len: message.len(),
            capacity: message.capacity(),
        });
        self.desc.message = message.as_ptr() as *const _;
        core::mem::forget(message);
        self
    }

    /// Set brief description for alert.
    ///
    /// # Arguments
    ///
    /// * `brief` - Brief description.
    ///
    /// # See also
    ///
    /// * `ecs_alert_desc_t::brief`
    #[doc(alias = "ecs_alert_desc_t::brief")]
    pub fn brief(&mut self, brief: &str) -> &mut Self {
        let brief = format!("{}\0", brief);
        self.str_ptrs_to_free.push(StringToFree {
            ptr: brief.as_ptr() as *mut _,
            len: brief.len(),
            capacity: brief.capacity(),
        });
        self.desc.brief = brief.as_ptr() as *const _;
        core::mem::forget(brief);
        self
    }

    /// Set doc name for alert.
    ///
    /// # Arguments
    ///
    /// * `doc_name` - Documentation name.
    ///
    /// # See also
    ///
    /// * `ecs_alert_desc_t::doc_name`
    #[doc(alias = "ecs_alert_desc_t::doc_name")]
    pub fn doc_name(&mut self, doc_name: &str) -> &mut Self {
        let doc_name = format!("{}\0", doc_name);
        self.str_ptrs_to_free.push(StringToFree {
            ptr: doc_name.as_ptr() as *mut _,
            len: doc_name.len(),
            capacity: doc_name.capacity(),
        });
        self.desc.doc_name = doc_name.as_ptr() as *const _;
        core::mem::forget(doc_name);
        self
    }

    /// Set severity of alert (default is Error).
    ///
    /// # Arguments
    ///
    /// * `severity` - The severity entity.
    ///
    /// # See also
    ///
    /// * `ecs_alert_desc_t::severity`
    #[doc(alias = "ecs_alert_desc_t::severity")]
    pub fn severity_id(&mut self, severity: impl Into<Entity>) -> &mut Self {
        self.desc.severity = *severity.into();
        self
    }

    /// Set severity of alert (default is Error).
    ///
    /// # Type Parameters
    ///
    /// * `Severity` - The severity component.
    ///
    /// # See also
    ///
    /// * `ecs_alert_desc_t::severity`
    #[doc(alias = "ecs_alert_desc_t::severity")]
    pub fn severity<Severity>(&mut self) -> &mut Self
    where
        Severity: ComponentId + SeverityAlert,
    {
        self.severity_id(Severity::id(self.world()))
    }

    /// Set retain period of alert.
    ///
    /// # Arguments
    ///
    /// * `period` - Retain period.
    ///
    /// # See also
    ///
    /// * `ecs_alert_desc_t::retain_period`
    #[doc(alias = "ecs_alert_desc_t::retain_period")]
    pub fn retain_period(&mut self, period: f32) -> &mut Self {
        self.desc.retain_period = period;
        self
    }

    /// Add severity filter.
    ///
    /// # Arguments
    ///
    /// * `severity` - Severity entity.
    /// * `with` - Filter with this id.
    /// * `var` - Variable name (optional).
    ///
    /// # See also
    ///
    /// * `ecs_alert_desc_t::severity_filters`
    #[doc(alias = "ecs_alert_desc_t::severity_filters")]
    pub fn severity_filter_id(
        &mut self,
        severity: impl Into<Entity>,
        with: impl Into<Id>,
        var: Option<&str>,
    ) -> &mut Self {
        ecs_assert!(
            self.severity_filter_count < sys::ECS_ALERT_MAX_SEVERITY_FILTERS as i32,
            "Maximum number of severity filters reached"
        );

        let filter = &mut self.desc.severity_filters[self.severity_filter_count as usize];
        self.severity_filter_count += 1;
        filter.severity = *severity.into();
        filter.with = *with.into();
        if let Some(var) = var {
            let var = format!("{}\0", var);
            filter.var = var.as_ptr() as *const _;
            self.str_ptrs_to_free.push(StringToFree {
                ptr: var.as_ptr() as *mut _,
                len: var.as_bytes().len(),
                capacity: var.as_bytes().len(),
            });
            core::mem::forget(var);
        }
        self
    }

    /// Add severity filter.
    ///
    /// # Type Parameters
    ///
    /// * `Severity` - Severity component.
    ///
    /// # Arguments
    ///
    /// * `with` - Filter with this id.
    /// * `var` - Variable name (optional).
    ///
    /// # See also
    ///
    /// * `ecs_alert_desc_t::severity_filters`
    #[doc(alias = "ecs_alert_desc_t::severity_filters")]
    pub fn severity_filter<Severity>(&mut self, with: impl Into<Id>, var: Option<&str>) -> &mut Self
    where
        Severity: ComponentId,
    {
        let severity_id = Severity::id(self.world());
        self.severity_filter_id(severity_id, with, var)
    }

    /// Add severity filter for non-enum components.
    ///
    /// # Type Parameters
    ///
    /// * `Severity` - Severity component.
    /// * `T` - Component type.
    ///
    /// # Arguments
    ///
    /// * `var` - Variable name (optional).
    ///
    /// # See also
    ///
    /// * `ecs_alert_desc_t::severity_filters`
    #[doc(alias = "ecs_alert_desc_t::severity_filters")]
    pub fn severity_filter_component<Severity, With>(&mut self, var: Option<&str>) -> &mut Self
    where
        Severity: ComponentId,
        With: ComponentId + ComponentType<Struct>,
    {
        let severity_id = Severity::id(self.world());
        let with_id = With::id(self.world());
        self.severity_filter_id(severity_id, with_id, var)
    }

    /// Add severity filter for enum components.
    ///
    /// # Type Parameters
    ///
    /// * `Severity` - Severity component.
    /// * `T` - Enum component type.
    ///
    /// # Arguments
    ///
    /// * `with` - Enum variant.
    /// * `var` - Variable name (optional).
    ///
    /// # See also
    ///
    /// * `ecs_alert_desc_t::severity_filters`
    #[doc(alias = "ecs_alert_desc_t::severity_filters")]
    pub fn severity_filter_enum<Severity, With>(
        &mut self,
        with: With,
        var: Option<&str>,
    ) -> &mut Self
    where
        Severity: ComponentId,
        With: EnumComponentInfo + ComponentType<Enum>,
    {
        let world = self.world();
        let severity_id = Severity::id(world);
        let with_id = With::id(world);
        let constant_id = with.id_variant(world);
        let pair_id = ecs_pair(with_id, *constant_id.id());
        self.severity_filter_id(severity_id, pair_id, var)
    }

    /// Set member to create an alert for out of range values.
    ///
    /// # Arguments
    ///
    /// * `member` - Member entity.
    ///
    /// # See also
    ///
    /// * `ecs_alert_desc_t::member`
    #[doc(alias = "ecs_alert_desc_t::member")]
    pub fn member_id(&mut self, member: impl Into<Entity>) -> &mut Self {
        self.desc.member = *member.into();
        self
    }

    /// Set (component) id for member (optional). If `.member()` is set and id
    /// is not set, the id will default to the member parent.
    ///
    /// # Arguments
    ///
    /// * `id` - Component id.
    ///
    /// # See also
    ///
    /// * `ecs_alert_desc_t::id`
    #[doc(alias = "ecs_alert_desc_t::id")]
    pub fn id(&mut self, id: impl Into<Id>) -> &mut Self {
        self.desc.id = *id.into();
        self
    }

    /// Set member to create an alert for out of range values.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Component type.
    ///
    /// # Arguments
    ///
    /// * `member_name` - Member name.
    /// * `var` - Variable name (optional).
    ///
    /// # See also
    ///
    /// * `ecs_alert_desc_t::member`
    #[doc(alias = "ecs_alert_desc_t::member")]
    pub fn member_type<With>(&mut self, member_name: &str, var: Option<&str>) -> &mut Self
    where
        With: ComponentId,
    {
        let member_name = compact_str::format_compact!("{}\0", member_name);
        let world = self.world();
        let id = With::id(world);
        let member_id = unsafe {
            sys::ecs_lookup_path_w_sep(
                world.world_ptr_mut(),
                id,
                member_name.as_ptr() as *const _,
                SEPARATOR.as_ptr(),
                SEPARATOR.as_ptr(),
                false,
            )
        };

        ecs_assert!(
            member_id != 0,
            FlecsErrorCode::InvalidParameter,
            "Member {} not found in component {}",
            member_name,
            core::any::type_name::<With>()
        );

        if let Some(var) = var {
            let var = format!("{}\0", var);
            self.str_ptrs_to_free.push(StringToFree {
                ptr: var.as_ptr() as *mut _,
                len: var.len(),
                capacity: var.capacity(),
            });
            self.desc.var = var.as_ptr() as *const _;
            core::mem::forget(var);
        }

        self.member_id(member_id)
    }

    /// Set source variable for member (optional, defaults to `$this`).
    ///
    /// # Arguments
    ///
    /// * `var` - Variable name.
    ///
    /// # See also
    ///
    /// * `ecs_alert_desc_t::var`
    #[doc(alias = "ecs_alert_desc_t::var")]
    pub fn var(&mut self, var: &str) -> &mut Self {
        let var = format!("{}\0", var);
        self.str_ptrs_to_free.push(StringToFree {
            ptr: var.as_ptr() as *mut _,
            len: var.len(),
            capacity: var.capacity(),
        });
        self.desc.var = var.as_ptr() as *const _;
        core::mem::forget(var);
        self
    }
}

#[doc(hidden)]
impl<'a, T: QueryTuple> internals::QueryConfig<'a> for AlertBuilder<'a, T> {
    #[inline(always)]
    fn term_builder(&self) -> &TermBuilder {
        &self.term_builder
    }

    #[inline(always)]
    fn term_builder_mut(&mut self) -> &mut TermBuilder {
        &mut self.term_builder
    }

    #[inline(always)]
    fn query_desc(&self) -> &sys::ecs_query_desc_t {
        &self.desc.query
    }

    #[inline(always)]
    fn query_desc_mut(&mut self) -> &mut sys::ecs_query_desc_t {
        &mut self.desc.query
    }
    #[inline(always)]
    fn count_generic_terms(&self) -> i32 {
        T::COUNT
    }
}

impl<'a, T: QueryTuple> TermBuilderImpl<'a> for AlertBuilder<'a, T> {}

impl<'a, T: QueryTuple> QueryBuilderImpl<'a> for AlertBuilder<'a, T> {}

impl<'a, T> Builder<'a> for AlertBuilder<'a, T>
where
    T: QueryTuple,
{
    type BuiltType = Alert<'a>;

    /// Build the `AlertBuilder` into an Alert
    ///
    /// See also
    ///
    /// * C++ API: `node_builder::build`
    #[doc(alias = "node_builder::build")]
    fn build(&mut self) -> Self::BuiltType {
        let alert = Alert::new(self.world(), self.desc);
        for string_parts in self.term_builder.str_ptrs_to_free.iter() {
            unsafe {
                String::from_raw_parts(
                    string_parts.ptr as *mut u8,
                    string_parts.len,
                    string_parts.capacity,
                );
            }
        }
        alert
    }
}

impl<'a, T: QueryTuple> WorldProvider<'a> for AlertBuilder<'a, T> {
    fn world(&self) -> WorldRef<'a> {
        self.world
    }
}
