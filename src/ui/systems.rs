use std::{ffi::OsStr, path::PathBuf};

use bevy::{prelude::*, text::cosmic_text::ttf_parser::name};
use rand::Rng;
use Display::*;
use FlexDirection::*;

use super::*;
use crate::{interpreter::AzmPrograms, *};

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

pub fn setup_ui_registers(
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

            commands
                .entity(bits_container)
                .add_child(bits_container_text);

            commands.entity(ui_registers).add_child(bits_container);

            for bit in register.read() {
                let bit_container = commands
                    .spawn(create_ui_node(
                        format!("ui-register-bit-{name}-{bit}"),
                        NodeBuilder::new().float("left").build(),
                    ))
                    .id();

                let bit_value = commands
                    .spawn((
                        Text::new(bit.bit_to_string()),
                        Name::new(format!(
                            "ui-register-bit-{name}-{bit}-value"
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
            Text::new("Initializing..."),
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

pub fn available_programs(
    r_programs: Res<AzmPrograms>,
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
            }
        }
    });
}

/// ---------------- ///
/// Helper Functions ///
/// ---------------- ///

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
