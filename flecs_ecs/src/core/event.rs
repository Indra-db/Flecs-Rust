/// Trait to mark component structs as `EventData` to be used in `EventBuilderTyped`.
/// This is used to set the event data for the event to be emitted
/// this is to ensure that the event data is of the correct type and the component is meant to be used with `EventBuilderTyped`
pub trait EventData {}

/// Event builder trait to implement '`set_event_data`' for untyped and typed `EventBuilder`
pub trait EventBuilderImpl {
    type BuiltType;

    fn set_event_data(&mut self, data: Self::BuiltType) -> &mut Self;
}
