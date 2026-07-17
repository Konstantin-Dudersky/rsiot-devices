const INIT_LEN: usize = 5;
const YAW_THRESHOLD: f32 = 45.0;
const PITCH_THRESHOLD: f32 = 45.0;
const ROLL_THRESHOLD: f32 = 45.0;
const MAX_OUTLINE_COUNT: usize = 3;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OutlineDetectionBufferItem {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum OutlineDetectionState {
    #[default]
    Init,
    Run,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OutlineDetectionData {
    init_prev: Option<OutlineDetectionBufferItem>,
    prev: OutlineDetectionBufferItem,
    state: OutlineDetectionState,
    outline_count: usize,
}

pub fn check_outline(yaw: f32, pitch: f32, roll: f32, buffer: &mut OutlineDetectionData) -> bool {
    let (new_state, correct) = match buffer.state {
        OutlineDetectionState::Init => match &buffer.init_prev {
            Some(v) => {
                if (yaw - v.yaw).abs() > YAW_THRESHOLD
                    || (pitch - v.pitch).abs() > PITCH_THRESHOLD
                    || (roll - v.roll).abs() > ROLL_THRESHOLD
                {
                    buffer.init_prev = None;
                    (OutlineDetectionState::Init, false)
                } else {
                    buffer.prev = OutlineDetectionBufferItem { yaw, pitch, roll };
                    (OutlineDetectionState::Run, false)
                }
            }
            None => {
                buffer.init_prev = Some(OutlineDetectionBufferItem { yaw, pitch, roll });
                (OutlineDetectionState::Init, false)
            }
        },
        OutlineDetectionState::Run => {
            if (yaw - buffer.prev.yaw).abs() > YAW_THRESHOLD
                || (pitch - buffer.prev.pitch).abs() > PITCH_THRESHOLD
                || (roll - buffer.prev.roll).abs() > ROLL_THRESHOLD
            {
                buffer.outline_count += 1;
                if buffer.outline_count > MAX_OUTLINE_COUNT {
                    buffer.outline_count = 0;
                    (OutlineDetectionState::Init, false)
                } else {
                    (OutlineDetectionState::Run, false)
                }
            } else {
                buffer.prev = OutlineDetectionBufferItem { yaw, pitch, roll };
                (OutlineDetectionState::Run, true)
            }
        }
    };

    buffer.state = new_state;

    correct
}
