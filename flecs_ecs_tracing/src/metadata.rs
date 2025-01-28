//! Construct [`tracing_core::Metadata`] objects in the way required by that ecosystem.
//!
//! Dynamic call site stuff based on <https://github.com/slowli/tracing-toolbox/>

use crate::util::*;
use hashbrown::HashMap;
use std::{
    borrow::Cow,
    sync::{LazyLock, OnceLock, RwLock},
};
use tracing_core::{field::FieldSet, metadata::Level, Dispatch, Kind, Metadata};

/// Registry of dynamic tracing metadata generated from Flecs
pub(crate) static REGISTRY: LazyLock<RwLock<Registry>> = LazyLock::new(Default::default);

/// Registry of dynamic tracing metadata generated from Flecs
pub(crate) struct Registry {
    pub(crate) metadata: HashMap<MetadataRequest<'static>, &'static Metadata<'static>>,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            // Capacity was chosen arbitrarily
            metadata: HashMap::with_capacity(256),
        }
    }
}

pub(crate) fn init() {
    LazyLock::force(&REGISTRY);
}

#[derive(Default)]
pub(crate) struct DynamicCallsite {
    /// Due to the opaque [`tracing_core::callsite::Identifier`],
    /// which borrows [`tracing_core::Callsite`] for `'static`, have to use interior mutability.
    ///
    /// See [`MetadataRequest::register`] for details
    pub(crate) metadata: OnceLock<Metadata<'static>>,
}

impl tracing_core::Callsite for DynamicCallsite {
    fn set_interest(&self, _interest: tracing_core::Interest) {
        // No-op
    }

    fn metadata(&self) -> &Metadata<'_> {
        self.metadata.get().unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum MetadataKind {
    // hash/eq skip filename/line since they may differ between start & stop
    // only used when perf_trace is enabled
    #[cfg_attr(not(feature = "perf_trace"), allow(unused))]
    Span,
    Event,
}

#[derive(Clone, Debug)]
pub(crate) struct MetadataRequest<'a> {
    pub(crate) kind: MetadataKind,
    pub(crate) level: Level,
    pub(crate) name: Cow<'a, str>,
    pub(crate) filename: Option<Cow<'a, str>>,
    pub(crate) line: Option<u32>,
}

impl<'a> PartialEq for MetadataRequest<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.name == other.name
            && self.level == other.level
            && match self.kind {
                MetadataKind::Span => true,
                MetadataKind::Event => self.filename == other.filename && self.line == other.line,
            }
    }
}

impl<'a> Eq for MetadataRequest<'a> {}

impl<'a> std::hash::Hash for MetadataRequest<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.name.hash(state);
        self.level.hash(state);
        match self.kind {
            MetadataKind::Span => {}
            MetadataKind::Event => {
                self.filename.hash(state);
                self.line.hash(state);
            }
        }
    }
}

impl<'a> MetadataRequest<'a> {
    /// Construct a new [`Metadata`].
    ///
    /// Does not interact with [`crate::REGISTRY`] by design to avoid having to deal with locking;
    /// the caller should handle memoization if necessary.
    pub(crate) fn register(
        self,
        dispatch: &Dispatch,
    ) -> (&'static Metadata<'static>, MetadataRequest<'static>) {
        // `Dispatch::register_callsite` demands `&'static Metadata`, this is the simplest way to get that
        let callsite = Box::leak(Box::new(DynamicCallsite::default()));

        // This macro is the only 'public' way of constructing [`Identifier`]
        let id = tracing_core::identify_callsite!(callsite);

        let name = leak_cowstr(self.name);
        let filename: Option<&'static str> = self.filename.map(leak_cowstr);

        let metadata = Metadata::new(
            name,
            "flecs",
            self.level,
            filename,
            self.line,
            Some("flecs_ecs_tracing"),
            match self.kind {
                MetadataKind::Span { .. } => FieldSet::new(&[], id),
                MetadataKind::Event { .. } => FieldSet::new(&["message"], id),
            },
            match self.kind {
                MetadataKind::Span { .. } => Kind::SPAN,
                MetadataKind::Event { .. } => Kind::EVENT,
            },
        );

        // Store the new Metadata
        callsite.metadata.set(metadata).unwrap();

        // Since the `DynamicCallsite` is alive forever we can also use it to get
        // a `&'static` to the metadata, without extra allocations
        let metadata: &'static Metadata = callsite.metadata.get().unwrap();

        // Tell `tracing` subscribers about the new callsite
        dispatch.register_callsite(metadata);

        (
            metadata,
            MetadataRequest {
                name: Cow::Borrowed(name),
                kind: self.kind,
                filename: filename.map(Cow::Borrowed),
                line: self.line,
                level: self.level,
            },
        )
    }
}
