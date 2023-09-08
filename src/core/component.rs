use rand::random;
use std::sync::OnceLock;

// Dummy function to simulate ID generation
fn generate_id() -> u64 {
    random()
}

//dummy function to simulate data generation
fn generate_data() -> ComponentData {
    ComponentData {
        id: generate_id(),
        size: 0,
        alignment: 0,
        reset_counter: 0,
        allow_tags: false,
    }
}

pub struct ComponentData {
    pub id: u64,
    pub size: usize,
    pub alignment: usize,
    pub reset_counter: u32,
    pub allow_tags: bool,
}

pub trait CachedComponentData {
    fn get_data() -> &'static ComponentData;

    fn get_id() -> u64 {
        Self::get_data().id
    }

    fn get_size() -> usize {
        Self::get_data().size
    }

    fn get_alignment() -> usize {
        Self::get_data().alignment
    }

    fn get_reset_counter() -> u32 {
        Self::get_data().reset_counter
    }

    fn get_allow_tags() -> bool {
        Self::get_data().allow_tags
    }
}

macro_rules! impl_cached_component_data  {
    ($($t:ty),*) => {
        $(
            impl CachedComponentData for $t {
                fn get_data() -> &'static ComponentData {
                    static ONCE_LOCK : OnceLock<ComponentData> = OnceLock::new();
                    ONCE_LOCK.get_or_init(|| generate_data())
                }
            }
        )*
    };
}
