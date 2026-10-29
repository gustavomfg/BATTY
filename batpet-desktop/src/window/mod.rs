use bevy::{
    log::info,
    prelude::{IVec2, Query, Res, ResMut, Resource, State, UVec2, With},
    window::{
        CompositeAlphaMode, Monitor, PrimaryWindow, Window, WindowLevel, WindowPlugin,
        WindowPosition, WindowResolution,
    },
};

use crate::{debug::DebugOptions, pet::BatState};

pub const WINDOW_WIDTH: u32 = 320;
pub const WINDOW_HEIGHT: u32 = 320;
pub const WINDOW_RIGHT_MARGIN: u32 = 24;
pub const WINDOW_TOP_MARGIN: u32 = 8;

#[derive(Debug, Default, Resource)]
pub struct WindowPlacement {
    pub positioned: bool,
}

pub fn primary_window() -> Window {
    // Xwayland on the review host exposes only opaque surfaces. This opt-in
    // rehearsal still uses native cursor events; normal desktop stays transparent.
    let cursor_rehearsal = std::env::args().any(|arg| arg == "--review-os-cursor")
        && std::env::var_os("WAYLAND_DISPLAY").is_none();
    Window {
        title: "BATTY Desktop".to_owned(),
        resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT)
            .with_scale_factor_override(1.0),
        transparent: !cursor_rehearsal,
        decorations: false,
        resizable: false,
        // The final top-right position is calculated once monitor information is available.
        // Wayland compositors may ignore absolute window positioning.
        position: WindowPosition::Automatic,
        window_level: WindowLevel::AlwaysOnTop,
        #[cfg(target_os = "linux")]
        composite_alpha_mode: if cursor_rehearsal {
            CompositeAlphaMode::Opaque
        } else {
            CompositeAlphaMode::PreMultiplied
        },
        ..Default::default()
    }
}

pub fn log_startup(
    debug: Res<DebugOptions>,
    windows: Query<&Window, With<PrimaryWindow>>,
    state: Res<State<BatState>>,
) {
    if !debug.enabled {
        return;
    }

    let Some(window) = windows.iter().next() else {
        return;
    };

    info!("renderer iniciado: bevy2d window backend=winit");
    info!(
        "window compositor={} size={:.0}x{:.0} transparent={} decorations={} resizable={} always_on_top=requested",
        compositor_hint(),
        window.resolution.width(),
        window.resolution.height(),
        window.transparent,
        window.decorations,
        window.resizable,
    );
    info!("bat state={}", state.get().label());
}

pub fn place_window_top_right(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    monitors: Query<&Monitor>,
    mut placement: ResMut<WindowPlacement>,
    debug: Res<DebugOptions>,
) {
    if placement.positioned {
        return;
    }

    let Some(monitor) = monitors.iter().next() else {
        return;
    };
    let Some(mut window) = windows.iter_mut().next() else {
        return;
    };

    let position = top_right_position(
        monitor.physical_position,
        monitor.physical_size(),
        window.physical_size(),
        UVec2::new(WINDOW_RIGHT_MARGIN, WINDOW_TOP_MARGIN),
    );
    window.position.set(position);
    placement.positioned = true;

    if debug.enabled {
        info!(
            "window placement=top-right position=({}, {}) monitor={}x{}",
            position.x, position.y, monitor.physical_width, monitor.physical_height
        );
    }
}

pub const fn top_right_position(
    monitor_position: IVec2,
    monitor_size: UVec2,
    window_size: UVec2,
    margin: UVec2,
) -> IVec2 {
    let available_width = monitor_size
        .x
        .saturating_sub(window_size.x.saturating_add(margin.x));
    IVec2::new(
        monitor_position.x + available_width as i32,
        monitor_position.y + margin.y as i32,
    )
}

fn compositor_hint() -> &'static str {
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        "Wayland"
    } else if std::env::var_os("DISPLAY").is_some() {
        "X11"
    } else {
        "unknown"
    }
}

pub fn plugin_with_window(window: Window) -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(window),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_the_top_right_position_inside_a_monitor() {
        assert_eq!(
            top_right_position(
                IVec2::ZERO,
                UVec2::new(1920, 1080),
                UVec2::new(320, 320),
                UVec2::new(24, 8),
            ),
            IVec2::new(1576, 8)
        );
    }

    #[test]
    fn preserves_monitor_origin_for_multi_monitor_layouts() {
        assert_eq!(
            top_right_position(
                IVec2::new(-1920, 0),
                UVec2::new(1920, 1080),
                UVec2::new(320, 320),
                UVec2::new(24, 8),
            ),
            IVec2::new(-344, 8)
        );
    }
}
