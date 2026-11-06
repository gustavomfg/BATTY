mod companion;
mod debug;
mod pet;
mod rendering;
mod review;
mod window;

use bevy::{
    asset::AssetPlugin,
    log::{Level, LogPlugin},
    prelude::*,
};

use companion::CompanionRun;
use debug::DebugOptions;
use pet::{BatState, CursorState, EyeState};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // Internal entry point: Noctalia pipes a screenshot here. No window opens.
    if args.get(1).map(String::as_str) == Some("analyze") {
        std::process::exit(companion::vision::run_analyze_command(&args[2..]));
    }
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print!("{}", companion::USAGE);
        return;
    }
    let options = match companion::Options::parse(&args) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("batty: {error}\n{}", companion::USAGE);
            std::process::exit(2);
        }
    };
    // Speaks in the terminal before the window exists, so `--look` never sees the bat.
    let companion_run = CompanionRun::prepare(&options);

    let debug = DebugOptions::from_args();
    let log_filter = if debug.enabled {
        "error,batpet_desktop=info"
    } else {
        "error"
    };

    App::new()
        .insert_resource(debug)
        .insert_resource(pet::HabitatDebug::from_args())
        .insert_resource(review::Review::from_args())
        .insert_resource(companion_run)
        .insert_resource(ClearColor(
            if std::env::args().any(|arg| arg == "--review-light") {
                Color::srgb_u8(235, 231, 240)
            } else {
                Color::NONE
            },
        ))
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: format!("{}/../assets", env!("CARGO_MANIFEST_DIR")),
                    ..Default::default()
                })
                .set(LogPlugin {
                    filter: log_filter.to_owned(),
                    level: Level::INFO,
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest())
                .set(window::plugin_with_window(window::primary_window())),
        )
        .init_state::<BatState>()
        .init_resource::<CursorState>()
        .init_resource::<EyeState>()
        .init_resource::<pet::IdleScheduler>()
        .init_resource::<pet::Habitat>()
        .init_resource::<pet::FlightPlan>()
        .init_resource::<window::WindowPlacement>()
        .add_systems(
            Startup,
            (
                rendering::setup,
                window::log_startup,
                companion::start_briefing_reaction,
            ),
        )
        .add_systems(Update, companion::exit_after_stay)
        .add_systems(
            Update,
            (
                window::place_window_top_right,
                pet::capture_cursor,
                review::drive,
                pet::trigger_habitat_flight,
                pet::trigger_reaction,
                pet::animate_reaction.run_if(in_state(BatState::Reacting)),
                pet::advance_idle_scheduler.run_if(in_state(BatState::HangingIdle)),
                pet::update_breathing.run_if(in_state(BatState::HangingIdle)),
                pet::update_attention.run_if(in_state(BatState::HangingIdle)),
                pet::update_idle_gaze.run_if(in_state(BatState::HangingIdle)),
                pet::update_ear_twitch.run_if(in_state(BatState::HangingIdle)),
                pet::update_idle_adjustment.run_if(in_state(BatState::HangingIdle)),
                pet::update_blink.run_if(in_state(BatState::HangingIdle)),
                pet::resolve_visual_pose.run_if(in_state(BatState::HangingIdle)),
                pet::apply_idle_motion.run_if(in_state(BatState::HangingIdle)),
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                pet::update_takeoff.run_if(in_state(BatState::Takeoff)),
                pet::update_flying.run_if(in_state(BatState::Flying)),
                pet::update_returning.run_if(in_state(BatState::Returning)),
                pet::update_landing.run_if(in_state(BatState::Landing)),
                pet::update_flight_visual,
                rendering::rig::animate,
                pet::update_eyes,
                review::capture,
            )
                .chain(),
        )
        .add_systems(
            OnEnter(BatState::Reacting),
            (pet::start_reaction, pet::log_reaction_started),
        )
        .add_systems(
            OnEnter(BatState::Takeoff),
            (pet::start_takeoff, pet::log_flight_entered),
        )
        .add_systems(
            OnEnter(BatState::Flying),
            (pet::start_flying, pet::log_flight_entered),
        )
        .add_systems(
            OnEnter(BatState::Returning),
            (pet::start_returning, pet::log_flight_entered),
        )
        .add_systems(
            OnEnter(BatState::Landing),
            (pet::start_landing, pet::log_flight_entered),
        )
        .add_systems(
            OnEnter(BatState::HangingIdle),
            (
                pet::complete_habitat_landing,
                pet::finish_landing,
                pet::log_flight_entered,
            ),
        )
        .run();
}
