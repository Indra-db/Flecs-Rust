pub trait EventData {}

pub trait EventBuilderImpl {
    type BuiltType;

    fn set_event_data(&mut self, data: Self::BuiltType) -> &mut Self;
}

impl EventData for i32 {}
