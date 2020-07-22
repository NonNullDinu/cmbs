use crate::interpreter::EnvFrameReturns;

pub struct Handle {
    env_frame_returns: EnvFrameReturns,
}

impl Handle {
    pub fn new() -> Self {
        Self {
            env_frame_returns: EnvFrameReturns::empty(),
        }
    }

    pub(crate) fn push_env_frame_returns(&mut self, env_frame_returns: EnvFrameReturns) {
        self.env_frame_returns = env_frame_returns
    }
}

impl Default for Handle {
    fn default() -> Self {
        Self::new()
    }
}
