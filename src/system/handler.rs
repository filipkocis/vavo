use super::System;

pub enum SystemStage {
    PreStartup,
    Startup,
    First,
    PreUpdate,
    FixedUpdate,
    Update,
    PostUpdate,
    Last,
    PreRender,
    Render,
    PostRender,
    FrameEnd,
}

impl SystemStage {
    /// True for any stage that has a fixed time schedule
    pub fn has_fixed_time(&self) -> bool {
        match self {
            SystemStage::FixedUpdate => true,
            _ => false,
        }
    }
}

pub(crate) struct SystemHandler {
    pre_startup: Vec<System>, 
    startup: Vec<System>, 
    first: Vec<System>, 
    pre_update: Vec<System>, 
    fixed_update: Vec<System>, 
    update: Vec<System>, 
    post_update: Vec<System>, 
    last: Vec<System>, 
    pre_render: Vec<System>, 
    render: Vec<System>, 
    post_render: Vec<System>, 
    frame_end: Vec<System>, 
}

impl SystemHandler {
    pub fn new() -> Self {
        SystemHandler {
            pre_startup: Vec::new(),
            startup: Vec::new(),
            first: Vec::new(),
            pre_update: Vec::new(),
            fixed_update: Vec::new(),
            update: Vec::new(),
            post_update: Vec::new(),
            last: Vec::new(),
            pre_render: Vec::new(),
            render: Vec::new(),
            post_render: Vec::new(),
            frame_end: Vec::new(),
        }
    }

    /// Register a system to a specific stage 
    pub(crate) fn register_system(&mut self, system: System, stage: SystemStage) {
        match stage {
            SystemStage::PreStartup => self.pre_startup.push(system),
            SystemStage::Startup => self.startup.push(system),
            SystemStage::First => self.first.push(system),
            SystemStage::PreUpdate => self.pre_update.push(system),
            SystemStage::FixedUpdate => self.fixed_update.push(system),
            SystemStage::Update => self.update.push(system),
            SystemStage::PostUpdate => self.post_update.push(system),
            SystemStage::Last => self.last.push(system),
            SystemStage::PreRender => self.pre_render.push(system),
            SystemStage::Render => self.render.push(system),
            SystemStage::PostRender => self.post_render.push(system),
            SystemStage::FrameEnd => self.post_render.push(system),
        }
    }

    /// Get the systems for the given stage
    pub(crate) fn get_systems(&mut self, stage: &SystemStage) -> &mut Vec<System> {
        match stage {
            SystemStage::PreStartup => &mut self.pre_startup,
            SystemStage::Startup => &mut self.startup,
            SystemStage::First => &mut self.first,
            SystemStage::PreUpdate => &mut self.pre_update,
            SystemStage::FixedUpdate => &mut self.fixed_update,
            SystemStage::Update => &mut self.update,
            SystemStage::PostUpdate => &mut self.post_update,
            SystemStage::Last => &mut self.last,
            SystemStage::PreRender => &mut self.pre_render,
            SystemStage::Render => &mut self.render,
            SystemStage::PostRender => &mut self.post_render,
            SystemStage::FrameEnd => &mut self.frame_end,
        }
    }
}
