/// Controls when a particular structure is updated.
#[derive(Debug)]
pub struct Control {
    do_once: bool,
    do_auto: bool,
    is_outdated: bool,
}

impl Control {
    /// Allow an update this frame
    pub fn once(&mut self) {
        self.do_once = true;
    }

    /// If true, allow updates the following frames (until it is set to false)
    pub fn auto(&mut self) -> &mut bool {
        &mut self.do_auto
    }

    /// Set when the configuration of the structure has changed so that the current version is invalid.
    pub fn set_outdated(&mut self) {
        self.is_outdated = true;
    }

    /// Return true if it makes sense to update on this frame.
    pub fn update(&mut self) -> bool {
        if (self.is_outdated) && (self.do_once || self.do_auto) {
            self.is_outdated = false;
            self.do_once = false;
            true
        } else {
            self.do_once = false;
            false
        }
    }
}

/// Methods for defaults
impl Control {
    pub const FIRST_FRAME_UPDATE: Control = Control {
        do_once: true,
        do_auto: false,
        is_outdated: true,
    };

    pub const AUTO_UPDATE: Control = Control {
        do_once: false,
        do_auto: true,
        is_outdated: true,
    };
}
