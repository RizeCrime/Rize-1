use std::{ffi::OsStr, path::PathBuf};

use bevy::{prelude::*, text::cosmic_text::ttf_parser::name};
use rand::Rng;
use Display::*;
use FlexDirection::*;

use super::*;
use crate::{
    interpreter::{ActiveProgram, AzmPrograms},
    *,
};

const UI_ROOT_NAME: &str = "R1_UiRoot";

/// --------------- ///
/// Startup Systems ///
/// --------------- ///

pub fn setup_ui_root(mut commands: Commands) {
    let ui_root = commands
        .spawn(create_ui_node(
            UI_ROOT_NAME.into(),
            NodeBuilder::new()
                .width(Val::Percent(100.0))
                .height(Val::Percent(100.0))
                .build(),
        ))
        .id();
}

pub fn setup_gp_registers(
    mut commands: Commands,
    r_registers: Res<Registers>,
    q_ui_root: Query<(Entity, &Name), With<UiElement>>,
) {
    let ui_root = get_ui_root_from_query(&q_ui_root);

    let ui_registers = commands
        .spawn(create_ui_node(
            "R1_UiRegisters".into(),
            NodeBuilder::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .float("left")
                .float("bottom")
                .build(),
        ))
        .id();

    let ui_registers_text = commands
        .spawn((
            Text::new("General Purpose Registers"),
            Name::new("R1_UiRegistersText"),
            create_random_border_color(),
            UiElement,
        ))
        .id();

    commands.entity(ui_root).add_child(ui_registers);
    commands.entity(ui_registers).add_child(ui_registers_text);

    let r_registers: &Registers = r_registers.as_ref();

    for (name, register) in r_registers.all().iter() {
        if name.starts_with('g') {
            let bits_container = commands
                .spawn(create_ui_node(
                    format!("ui_register_bits_{name}"),
                    NodeBuilder::new()
                        .display(Display::Flex)
                        .flex_direction(FlexDirection::Row)
                        .build(),
                ))
                .id();

            let bits_container_text = commands
                .spawn((
                    Text::new(format!(" {name}a :\t")),
                    Name::new(format!("ui-register-bits-{name}-text")),
                    UiElement,
                ))
                .id();

            let parsed_u16 = commands
                .spawn((
                    Text::new("_"),
                    Name::new(format!("ui-register-parsed-{name}-u16")),
                    UiElement,
                ))
                .id();
            let parsed_ascii = commands
                .spawn((
                    Text::new("_"),
                    Name::new(format!("ui-register-parsed-{name}-ascii")),
                    UiElement,
                ))
                .id();
            let parsed_hex = commands
                .spawn((
                    Text::new("_"),
                    Name::new(format!("ui-register-parsed-{name}-hex")),
                    UiElement,
                ))
                .id();

            commands
                .entity(bits_container)
                .add_child(bits_container_text);

            commands.entity(ui_registers).add_child(bits_container);
            commands.entity(ui_registers).add_children(&[
                parsed_u16,
                parsed_hex,
                parsed_ascii,
            ]);

            let bits = match register.read() {
                Ok(b) => b,
                Err(e) => {
                    error!("Failed to read register {}: {}", name, e);
                    continue;
                }
            };

            for (idx, bit) in bits.iter().enumerate() {
                let bit_container = commands
                    .spawn(create_ui_node(
                        format!("ui-register-bit-{name}-{idx}"),
                        NodeBuilder::new().float("left").build(),
                    ))
                    .id();

                let bit_value = commands
                    .spawn((
                        Text::new(bit.bit_to_string()),
                        Name::new(format!(
                            "ui-register-bit-{name}-{idx}-value"
                        )),
                        UiElement,
                    ))
                    .id();

                commands.entity(bits_container).add_child(bit_container);
                commands.entity(bit_container).add_child(bit_value);
            }
        }
    }
}

pub fn setup_core_registers(
    mut commands: Commands,
    r_registers: Res<Registers>,
    q_ui_root: Query<(Entity, &Name), With<UiElement>>,
) {
    let ui_root = get_ui_root_from_query(&q_ui_root);

    // Create main container for core registers
    let ui_core_registers = commands
        .spawn(create_ui_node(
            "R1_UiCoreRegisters".into(),
            NodeBuilder::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .float("left") // Position bottom-left like GP registers
                .float("bottom")
                .build(),
        ))
        .id();

    // Add title for core registers
    let ui_core_registers_text = commands
        .spawn((
            Text::new("Core Registers"),
            Name::new("R1_UiCoreRegistersText"),
            create_random_border_color(),
            UiElement,
        ))
        .id();

    // Add container and title to the UI hierarchy
    commands.entity(ui_root).add_child(ui_core_registers);
    commands
        .entity(ui_core_registers)
        .add_child(ui_core_registers_text);

    let r_registers_map = r_registers.all(); // Get the register map

    // Define the names of the core registers to display
    let core_register_names = ["mar", "mdr", "pc"];

    // Iterate through the core register names
    for name in core_register_names {
        // Get the register data from the resource map
        if let Some(register) = r_registers_map.get(name) {
            // Container for the bits display (Flex Row)
            let bits_container = commands
                .spawn(create_ui_node(
                    format!("ui_register_bits_{name}"),
                    NodeBuilder::new()
                        .display(Display::Flex)
                        .flex_direction(FlexDirection::Row)
                        .build(),
                ))
                .id();

            // Text displaying the register name
            let bits_container_text = commands
                .spawn((
                    Text::new(format!(" {name} :\t")), // Display register name
                    Name::new(format!("ui-register-bits-{name}-text")),
                    UiElement,
                ))
                .id();

            // Placeholder Text nodes for parsed values
            let parsed_u16 = commands
                .spawn((
                    Text::new("_"), // Initial placeholder
                    Name::new(format!("ui-register-parsed-{name}-u16")),
                    UiElement,
                ))
                .id();
            let parsed_ascii = commands
                .spawn((
                    Text::new("_"), // Initial placeholder
                    Name::new(format!("ui-register-parsed-{name}-ascii")),
                    UiElement,
                ))
                .id();
            let parsed_hex = commands
                .spawn((
                    Text::new("_"), // Initial placeholder
                    Name::new(format!("ui-register-parsed-{name}-hex")),
                    UiElement,
                ))
                .id();

            // Add register name text to the bits container
            commands
                .entity(bits_container)
                .add_child(bits_container_text);

            // Add bits container and parsed value placeholders to the main core registers container
            commands.entity(ui_core_registers).add_child(bits_container);
            commands.entity(ui_core_registers).add_children(&[
                parsed_u16,
                parsed_hex,
                parsed_ascii,
            ]);

            // Read the bits from the register
            let bits = match register.read() {
                Ok(b) => b,
                Err(e) => {
                    // Log error and skip this register if read fails
                    error!("Failed to read register {}: {}", name, e);
                    continue;
                }
            };

            // Create UI elements for each individual bit
            for (idx, bit) in bits.iter().enumerate() {
                let bit_container = commands
                    .spawn(create_ui_node(
                        format!("ui-register-bit-{name}-{idx}"),
                        NodeBuilder::new().float("left").build(), // Position bits left-to-right
                    ))
                    .id();

                let bit_value = commands
                    .spawn((
                        Text::new(bit.bit_to_string()), // Display '0' or '1'
                        Name::new(format!(
                            "ui-register-bit-{name}-{idx}-value"
                        )),
                        UiElement,
                    ))
                    .id();

                // Add the bit value text to its container
                commands.entity(bit_container).add_child(bit_value);
                // Add the bit container to the row of bits
                commands.entity(bits_container).add_child(bit_container);
            }
        } else {
            // Log a warning if a defined core register name is not found
            warn!("Core register '{}' not found in resource map.", name);
        }
    }
}

pub fn setup_instruction_ui(
    mut commands: Commands,
    q_ui_root: Query<(Entity, &Name), With<UiElement>>,
) {
    let ui_root = get_ui_root_from_query(&q_ui_root);

    let ui_instruction_container = commands
        .spawn(create_ui_node(
            "ui-instruction-container".into(),
            NodeBuilder::new()
                .absolute()
                .width(Val::Percent(35.0))
                .height(Val::Percent(20.0))
                .float("top")
                .margin(UiRect {
                    left: Val::Auto,
                    right: Val::Auto,
                    top: Val::Percent(5.0),
                    bottom: Val::Auto,
                })
                .display(Flex)
                .flex_direction(Row)
                .build(),
        ))
        .id();

    commands.entity(ui_root).add_child(ui_instruction_container);

    let names = ["OPCODE", "ARG1", "ARG2", "ARG3"];
    for name in names {
        let container = commands
            .spawn(create_ui_node(
                format!("ui-{name}-container"),
                NodeBuilder::new()
                    .width(Val::Percent(25.0))
                    .flex_direction(Column)
                    .build(),
            ))
            .id();

        commands
            .entity(ui_instruction_container)
            .add_child(container);

        let text = commands
            .spawn((
                Text::new("Opcode:"),
                Name::new(format!("ui-{name}-text")),
                UiElement,
            ))
            .id();

        let value = commands
            .spawn((
                Text::new("None"),
                Name::new(format!("ui-{name}-value")),
                UiElement,
            ))
            .id();

        commands.entity(container).add_children(&[text, value]);
    }
}

pub fn setup_ui_cpu_cycle_stage(
    mut commands: Commands,
    q_ui_root: Query<(Entity, &Name), With<UiElement>>,
) {
    let ui_root = get_ui_root_from_query(&q_ui_root);

    let ui_cpu_cycle_stage = commands
        .spawn(create_ui_node(
            "ui-cpu-cycle-stage".into(),
            NodeBuilder::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .float("top")
                .float("right")
                .build(),
        ))
        .id();

    commands.entity(ui_root).add_child(ui_cpu_cycle_stage);

    let ui_cpu_cycle_stage_text = commands
        .spawn((
            Text::new("CPU Cycle Stage"),
            Name::new("ui-cpu-cycle-stage-text"),
            TextLayout {
                justify: JustifyText::Center,
                ..Default::default()
            },
            create_random_border_color(),
            UiElement,
        ))
        .id();

    let ui_cpu_cycle_stage_value = commands
        .spawn((
            Text::new("Awaiting Tick..."),
            Name::new("ui-cpu-cycle-stage-value"),
            TextLayout {
                justify: JustifyText::Center,
                ..Default::default()
            },
            create_random_border_color(),
            UiElement,
        ))
        .id();

    let ui_cpu_cycle_advance_button = commands
        .spawn((
            Button,
            Text::new("Advance"),
            Name::new("ui-cpu-cycle-advance-button"),
            TextLayout {
                justify: JustifyText::Center,
                ..Default::default()
            },
            create_random_border_color(),
            UiElement,
        ))
        .id();

    commands.entity(ui_cpu_cycle_stage).add_children(&[
        ui_cpu_cycle_stage_text,
        ui_cpu_cycle_stage_value,
        ui_cpu_cycle_advance_button,
    ]);
}

pub fn setup_available_programs(
    mut commands: Commands,
    q_ui_root: Query<(Entity, &Name), With<UiElement>>,
) {
    let ui_root = get_ui_root_from_query(&q_ui_root);

    let ui_programs = commands
        .spawn(create_ui_node(
            "ui-programs".into(),
            NodeBuilder::new()
                .float("left")
                .float("top")
                .display(Flex)
                .flex_direction(Column)
                .absolute()
                .build(),
        ))
        .id();

    commands.entity(ui_root).add_child(ui_programs);

    let ui_programs_text = commands
        .spawn((
            Text::new("Available Programs:"),
            Name::new("ui-programs-text"),
            TextLayout {
                justify: JustifyText::Center,
                ..Default::default()
            },
            create_random_border_color(),
            UiElement,
        ))
        .id();

    commands.entity(ui_programs).add_child(ui_programs_text);
}

/// -------------- ///
/// Update Systems ///
/// -------------- ///

pub fn update_cpu_cycle_stage(
    s_current_stage: Res<State<CpuCycleStage>>,
    mut s_next_stage: ResMut<NextState<CpuCycleStage>>,
    q_advance_button: Query<
        (&Interaction, &Name),
        (Changed<Interaction>, With<Button>),
    >,
    mut q_stage_text: Query<(&mut Text, &Name), With<UiElement>>,
) {
    q_advance_button
        .iter()
        .for_each(|(interaction, button_name)| {
            if !(*interaction == Interaction::Pressed) {
                return;
            }
            if !button_name.as_str().eq("ui-cpu-cycle-advance-button") {
                return;
            }

            let mut stage_text = q_stage_text
                .iter_mut()
                .find(|(_, name)| name.as_str() == "ui-cpu-cycle-stage-value")
                .map(|(text, _)| text)
                .expect("Failed to find ui-cpu-cycle-stage-value");

            let mut current_stage: &CpuCycleStage = s_current_stage.get();
            match current_stage {
                CpuCycleStage::Startup => {
                    s_next_stage.set(CpuCycleStage::Fetch);
                    stage_text.0 = "Fetch".into();
                }
                CpuCycleStage::Fetch => {
                    s_next_stage.set(CpuCycleStage::Decode);
                    stage_text.0 = "Decode".into();
                }
                CpuCycleStage::Decode => {
                    s_next_stage.set(CpuCycleStage::Execute);
                    stage_text.0 = "Execute".into();
                }
                CpuCycleStage::Execute => {
                    s_next_stage.set(CpuCycleStage::Fetch);
                    stage_text.0 = "Fetch".into();
                }
                _ => {
                    panic!(
                        "You shouldn't be here, and I should be doing better Error handling #2: Electric Boogaloo!"
                    );
                }
            }
        });
}

pub fn available_programs(
    r_programs: Res<AzmPrograms>,
    mut r_program: ResMut<ActiveProgram>,
    qe: Query<(Entity, &Name), With<UiElement>>,
    qi: Query<(&Interaction, &Name), (Changed<Interaction>, With<Button>)>,
    mut commands: Commands,
) {
    let ui_programs: Entity = qe
        .iter()
        .find(|(_, name)| name.as_str() == "ui-programs")
        .map(|(entity, _)| entity)
        .expect("Failed to find Ui Root Node!");

    let available_programs: &Vec<(PathBuf, String)> = &r_programs.0;

    for program in available_programs {
        let file_stem: String =
            program.0.file_stem().unwrap().to_str().unwrap().to_string();

        if qe
            .iter()
            .find(|(_, name)| name.as_str() == file_stem)
            .is_some()
        {
            continue;
        }

        let ui_program = commands
            .spawn((
                Button,
                Text::new(file_stem.clone()),
                TextLayout {
                    justify: JustifyText::Center,
                    ..Default::default()
                },
                Name::new(file_stem.clone()),
                NodeBuilder::new().build(),
                create_random_border_color(),
                UiElement,
            ))
            .id();

        commands.entity(ui_programs).add_child(ui_program);
    }

    qi.iter().for_each(|(interaction, button_name)| {
        if let Some((path_buf, _)) = r_programs
            .0
            .iter()
            .find(|(_, file_stem)| button_name.as_str() == file_stem)
        {
            if *interaction == Interaction::Pressed {
                info!("Full Path: {:?}", path_buf);
                let program_contents: std::fs::File =
                    std::fs::File::open(path_buf).unwrap();

                r_program.as_mut().path = path_buf.clone();
                r_program.as_mut().file_stem = button_name.clone().into();
                r_program.as_mut().contents =
                    std::io::read_to_string(&program_contents).unwrap();
            }
        }
    });
}

/// ### Dev Metadata
/// 1) for each register, try to find the corresponding ui elements
///     - each bit can be found by their name
///         - "ui-register-bit-{name}-{idx}"
/// 2) update bit state with data from reg.read().iter().enumerate()
pub fn update_registers(
    mut commands: Commands,
    r_registers: Res<Registers>,
    mut q_ui: Query<(&mut Text, &Name), With<UiElement>>,
) {
    for (name, register) in r_registers.all().iter() {
        let bits = match register.read() {
            Ok(b) => b,
            Err(e) => {
                error!("Failed to read register {}: {}", name, e);
                continue; // Skip this register if reading fails
            }
        };

        for (idx, bit) in bits.iter().enumerate() {
            let target_name = format!("ui-register-bit-{name}-{idx}-value");

            // Find the specific UI text element for this bit
            for (mut text, ui_name) in q_ui.iter_mut() {
                if ui_name.as_str() == target_name {
                    // Update the text content
                    text.0 = bit.bit_to_string();
                    break; // Found the element, move to the next bit
                }
            }
        }
    }
}

/// ### Dev Metadata
/// - for each register
///     - get name and bits
///     - get matching ui Text by Name
///     - for each value type
///         - parse bits into value type
pub fn update_register_parsed(
    r_registers: Res<Registers>,
    mut q_ui: Query<(&mut Text, &Name), With<UiElement>>,
) {
    for (name, register) in r_registers.all().iter() {
        let bits = match register.read() {
            Ok(b) => b,
            Err(e) => {
                error!("Failed to read register {}: {}", name, e);
                continue; // Skip this register if reading fails
            }
        };

        // --- Parse and Update U16 ---
        let u16_value = bits_to_u16(&bits).to_string();
        let target_u16_name = format!("ui-register-parsed-{name}-u16");
        for (mut text, ui_name) in q_ui.iter_mut() {
            if ui_name.as_str() == target_u16_name {
                text.0 = u16_value.clone();
                break;
            }
        }

        // --- Parse and Update ASCII ---
        let ascii_value: String = bits_to_u16(&bits)
            .to_le_bytes()
            .iter()
            .map(|&b| if b.is_ascii_graphic() { b as char } else { ' ' }) // Replace non-printable with '.'
            .collect();
        let target_ascii_name = format!("ui-register-parsed-{name}-ascii");
        for (mut text, ui_name) in q_ui.iter_mut() {
            if ui_name.as_str() == target_ascii_name {
                text.0 = ascii_value.clone();
                break;
            }
        }

        // --- Parse and Update Hex ---
        let hex_value = format!("0x{:04X}", bits_to_u16(&bits)); // Format as 4-digit hex
        let target_hex_name = format!("ui-register-parsed-{name}-hex");
        for (mut text, ui_name) in q_ui.iter_mut() {
            if ui_name.as_str() == target_hex_name {
                text.0 = hex_value.clone();
                break;
            }
        }
    }
}

pub fn update_instruction_ui(
    r_active_program: Res<ActiveProgram>,
    mut q_ui: Query<(&mut Text, &Name), With<UiElement>>,
) {
    let opcode = &r_active_program.raw_opcode;
    let arg1 = &r_active_program.arg1.raw;
    let arg2 = &r_active_program.arg2.raw;
    let arg3 = &r_active_program.arg3.raw;

    let instruction_parts = [
        ("OPCODE", opcode),
        ("ARG1", arg1),
        ("ARG2", arg2),
        ("ARG3", arg3),
    ];

    for (ui_name_base, part_value) in instruction_parts.iter() {
        let target_ui_name = format!("ui-{ui_name_base}-value");

        for (mut text, ui_name) in q_ui.iter_mut() {
            if ui_name.as_str() == target_ui_name {
                text.0 = part_value.to_string();
                break;
            }
        }
    }
}

/// ---------------- ///
/// Helper Functions ///
/// ---------------- ///

/// Converts a slice of i8 (representing bits, 0 or 1) into a u16.
/// Assumes MSB first ordering within the slice.
fn bits_to_u16(bits: &[i8]) -> u16 {
    let mut value: u16 = 0;
    for &bit in bits {
        value <<= 1; // Shift existing bits left
        if bit == 1 {
            value |= 1; // Set the least significant bit if the current bit is 1
        }
        // If bit is 0, no change needed after the shift
    }
    value
}

// Helper struct for building Node components using the builder pattern
struct NodeBuilder {
    node: Node,
}

impl NodeBuilder {
    fn new() -> Self {
        Self {
            node: Node {
                border: if cfg!(debug_assertions) {
                    UiRect::all(Val::Px(2.0))
                } else {
                    UiRect::ZERO
                },
                margin: UiRect::all(Val::Auto),
                padding: UiRect::all(Val::Px(4.0)),
                width: Val::Auto,
                height: Val::Auto,
                ..Default::default()
            },
        }
    }

    fn margin(mut self, margin: UiRect) -> Self {
        self.node.margin = margin;
        self
    }

    fn display(mut self, display: Display) -> Self {
        self.node.display = display;
        self
    }

    fn flex_direction(mut self, flex_direction: FlexDirection) -> Self {
        self.node.flex_direction = flex_direction;
        self
    }

    fn width(mut self, width: Val) -> Self {
        self.node.width = width;
        self
    }

    fn height(mut self, height: Val) -> Self {
        self.node.height = height;
        self
    }

    fn padding(mut self, padding: UiRect) -> Self {
        self.node.padding = padding;
        self
    }

    fn float(mut self, direction: &str) -> Self {
        match direction {
            "left" => {
                self.node.margin.left = Val::Px(0.0);
                self
            }
            "right" => {
                self.node.margin.right = Val::Px(0.0);
                self
            }
            "top" => {
                self.node.margin.top = Val::Px(0.0);
                self
            }
            "bottom" => {
                self.node.margin.bottom = Val::Px(0.0);
                self
            }
            _ => panic!("You shouldn't be here, and I shouldn't do lazy Error Handling, so let's not judge each other, ok? :)")
        }
    }

    fn absolute(mut self) -> Self {
        self.node.position_type = PositionType::Absolute;
        self
    }

    fn build(self) -> Node {
        self.node
    }
}

fn get_ui_root_from_query(
    q_ui_root: &Query<(Entity, &Name), With<UiElement>>,
) -> Entity {
    q_ui_root
        .iter()
        .find(|(_, name)| name.as_str() == UI_ROOT_NAME)
        .map(|(entity, _)| entity)
        .expect("Failed to find Ui Root Node!")
}

fn create_ui_node(
    name: String,
    node: Node,
) -> (Node, BorderColor, Name, UiElement) {
    let mut rng = rand::rng(); // Use rng instead of thread_rng
    let random_color = Color::linear_rgb(
        rng.random_range(0.0..=1.0), // Use random_range for f32
        rng.random_range(0.0..=1.0), // Use random_range for f32
        rng.random_range(0.0..=1.0), // Use random_range for f32
    );

    let border_color = BorderColor(if cfg!(debug_assertions) {
        random_color
    } else {
        Color::NONE
    });
    let name_component = Name::new(name);

    (node, border_color, name_component, UiElement)
}

fn create_random_border_color() -> BorderColor {
    let mut rng = rand::rng(); // Use rng instead of thread_rng
    let random_color = Color::linear_rgb(
        rng.random_range(0.0..=1.0), // Use random_range for f32
        rng.random_range(0.0..=1.0), // Use random_range for f32
        rng.random_range(0.0..=1.0), // Use random_range for f32
    );
    BorderColor(random_color)
}

// Keep the old create_ui_node signature for now if needed, but mark as deprecated or remove
/*
fn create_ui_node(
    name: &str,
    margin: Option<UiRect>,
    display: Option<Display>,
    flex_direction: Option<FlexDirection>,
) -> (Node, BorderColor, Name, UiElement) {
    let builder = create_node_builder(margin, display, flex_direction);
    create_ui_node_bundle(name, builder)
}
*/
// Remove or comment out the old create_node_component as it's replaced by create_node_builder
/*
fn create_node_component(
    margin: Option<UiRect>,
    display: Option<Display>,
    flex_direction: Option<FlexDirection>,
) -> Node {
    create_node_builder(margin, display, flex_direction).build()
}
*/
