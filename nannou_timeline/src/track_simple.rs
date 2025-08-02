//! Simple track type for the timeline (stubbed for UI development)

use crate::LayerId;

/// A track in the timeline
#[derive(Clone, Debug)]
pub struct Track {
    pub layer_id: LayerId,
    pub height: f32,
    pub is_expanded: bool,
}

impl Track {
    pub fn new(layer_id: LayerId) -> Self {
        Self {
            layer_id,
            height: 30.0,
            is_expanded: true,
        }
    }
}