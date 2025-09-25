use super::FromRepr;
use rsiot::components_config::master_device::Error;

/// Виды запросов
#[derive(FromRepr)]
pub enum RequestKind {
    Init,
    ReadCalibrationOffsets,
    WriteCalibrationOffsets,
    ReadFullScaleConfig,
    WriteFullScaleConfig,
    ReadValues,
}

impl From<RequestKind> for u8 {
    fn from(value: RequestKind) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for RequestKind {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        RequestKind::from_repr(value as usize).ok_or(Error::RequestKindUnknown(value))
    }
}
