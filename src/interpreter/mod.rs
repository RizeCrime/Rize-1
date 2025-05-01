use std::{collections::HashMap, fmt::Debug, sync::Arc};

use bevy::prelude::*;
use collection::init_interpreters;

use crate::{
    display::DisplayMemory,
    types::{ActiveProgram, Registers, RizeError, SystemMemory},
    CpuCycleStage,
};

mod collection;
mod systems;

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
        app.insert_resource::<InterpreterRes>(InterpreterRes::default());

        app.add_systems(Startup, init_interpreters);
    }
}

#[derive(Debug, Default, Resource)]
#[allow(dead_code)]
pub struct InterpreterRes {
    pub active: Option<Arc<dyn Interpreter>>,
    pub all: HashMap<String, Arc<dyn Interpreter>>,
}

#[allow(dead_code)]
pub trait Interpreter: Debug + Send + Sync + 'static {
    fn setup_registers(&self, registers: &mut Registers);
    fn load_program(&self, program: &mut ActiveProgram);
    fn fetch(
        &self,
        registers: &mut Registers,
        program: &mut ActiveProgram,
    ) -> Option<()>;
    fn decode(
        &self,
        program: &mut ActiveProgram,
        registers: &mut Registers,
        memory: &mut SystemMemory,
    ) -> Result<(), RizeError>;
    fn execute(
        &self,
        program: &mut ActiveProgram,
        registers: &mut Registers,
        memory: &mut SystemMemory,
        display_memory: &mut DisplayMemory,
        images: &Assets<Image>,
        next_cpu_stage: ResMut<NextState<CpuCycleStage>>,
    ) -> Option<()>;
}
