#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub kind: DeviceKind,
    pub label: String,
    pub save_size: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DeviceKind {
    GbxCart,
    FileImage,
    InMemory,
}

impl DeviceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GbxCart => "gbxcart",
            Self::FileImage => "file_image",
            Self::InMemory => "in_memory",
        }
    }
}
