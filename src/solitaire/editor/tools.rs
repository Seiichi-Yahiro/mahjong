pub mod place_tool;
pub mod select_tool;

use bevy::prelude::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Tool {
    Select,
    Place,
}

impl Default for Tool {
    fn default() -> Self {
        Self::Place
    }
}

pub fn create_tool_state_system(commands: &mut Commands) {
    commands.insert_resource(State::new(Tool::default()));
}
