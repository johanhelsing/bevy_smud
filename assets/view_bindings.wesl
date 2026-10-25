import bevy_render::{globals::Globals, view::View};

@group(0) @binding(0) var<uniform> view: View;

@group(0) @binding(1) var<uniform> globals: Globals;
