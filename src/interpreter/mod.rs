use bevy::prelude::*;
use collection::AzmInterpreter;

// mod display;
// pub use display::*;

mod collection;

pub struct RizeOneInterpreterPlugin;

impl Plugin for RizeOneInterpreterPlugin {
    fn build(&self, app: &mut App) {
        //     app.insert_resource(AzmPrograms::default());
        //     app.insert_resource(ActiveProgram {
        //         ..Default::default()
        //     });
        //     app.insert_resource(ProgramSettings::default());
        //     app.insert_resource(FileCheckTimer(Timer::from_seconds(
        //         0.25,
        //         TimerMode::Repeating,
        //     )));

        //     app.register_type::<AzmPrograms>();
        //     app.register_type::<ActiveProgram>();

        //     #[cfg(debug_assertions)]
        //     app.add_plugins(ResourceInspectorPlugin::<ActiveProgram>::default());

        //     app.add_plugins(RizeOneDisplay);

        //     app.add_systems(Update, check_azm_programs);

        //     // add systems OnEnter, for manual step-through
        //     app.add_systems(OnEnter(CpuCycleStage::Fetch), fetch);
        //     app.add_systems(OnEnter(CpuCycleStage::Decode), decode);
        //     app.add_systems(OnEnter(CpuCycleStage::Execute), execute);

        //     // add systems as Update, for auto-stepping
        //     app.add_systems(
        //         Update,
        //         // (fetch, decode, execute)
        //         // .chain()
        //         (auto_step).run_if(in_state(CpuCycleStage::AutoStep)),
        //     );

        // ------------------------------------ //
        // Insert All Interpreters as Resources //
        // ------------------------------------ //
        app.insert_resource::<InterpreterRes<AzmInterpreter, _>>(
            InterpreterRes {
                name: "RizeOne Interpreter",
                ext: "azm",
                interpreter: AzmInterpreter::default(),
            },
        );
    }
}

#[derive(Debug, Default, Resource)]
pub struct InterpreterRes<T, S>
where
    T: Interpreter + Sync,
    S: Into<String>,
{
    pub name: S,
    pub ext: &'static str,
    pub interpreter: T,
}

pub trait Interpreter {
    fn fetch();
    fn decode();
    fn execute();
}
