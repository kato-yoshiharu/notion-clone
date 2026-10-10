macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(pub uuid::Uuid);

        #[cfg(test)]
        #[allow(clippy::new_without_default)]
        impl $name {
            pub fn new() -> Self {
                Self(uuid::Uuid::new_v4())
            }
        }
    };
}

#[allow(unused)]
macro_rules! define_name {
    ($name:ident) => {
        pub struct $name(pub String);

        impl std::str::FromStr for $name {
            type Err = std::string::ParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(s.to_string()))
            }
        }
    };
}
