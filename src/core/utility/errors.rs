pub struct InvalidStrFromId {
    pub id: u64,
}

impl std::fmt::Display for InvalidStrFromId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid string conversion from id: {}", self.id)
    }
}
