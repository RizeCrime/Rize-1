use std::{collections::HashMap, fmt::Debug, sync::Arc};

use bevy::prelude::*;
use collection::init_interpreters;
use systems::auto_step;

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
        // ------------------------------------ //
        // Insert All Interpreters as Resources //
        // ------------------------------------ //
        app.insert_resource::<InterpreterRes>(InterpreterRes::default());

        app.add_systems(Startup, init_interpreters)
            .add_systems(OnEnter(CpuCycleStage::Fetch), systems::fetch)
            .add_systems(OnEnter(CpuCycleStage::Decode), systems::decode)
            .add_systems(OnEnter(CpuCycleStage::Execute), systems::execute)
            .add_systems(
                Update,
                auto_step.run_if(in_state(CpuCycleStage::AutoStep)),
            );
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
        sn_cpu: ResMut<NextState<CpuCycleStage>>,
    ) -> Option<()>;
    fn file_type(&self) -> String;
}
