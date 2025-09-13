use strum::FromRepr;

/// Виды запросов
#[derive(FromRepr)]
pub enum RequestKind {
    SetCommandAutoRst,
    ReadAutoSeq,
}
impl From<RequestKind> for u8 {
    fn from(value: RequestKind) -> Self {
        value as u8
    }
}
impl From<u8> for RequestKind {
    fn from(value: u8) -> Self {
        RequestKind::from_repr(value as usize).unwrap()
    }
}
