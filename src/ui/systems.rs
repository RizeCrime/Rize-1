use std::collections::HashMap;
use std::{ffi::OsStr, path::PathBuf};

use bevy::image::{ImageSampler, ImageSamplerDescriptor};
use rand::Rng;
use Display::*;
use FlexDirection::*;

use super::*;
use crate::interpreter::DisplayMemory;
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
                .absolute()
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

    let ui_core_registers = commands
        .spawn(create_ui_node(
            "R1_UiCoreRegisters".into(),
            NodeBuilder::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .float("right")
                .float("bottom")
                .absolute()
                .build(),
        ))
        .id();

    let ui_core_registers_text = commands
        .spawn((
            Text::new("Core Registers"),
            Name::new("R1_UiCoreRegistersText"),
            create_random_border_color(),
            UiElement,
        ))
        .id();

    commands.entity(ui_root).add_child(ui_core_registers);
    commands
        .entity(ui_core_registers)
        .add_child(ui_core_registers_text);

    let r_registers_map = r_registers.all();

    let core_register_names = ["mar", "mdr", "pc"];

    for name in core_register_names {
        if let Some(register) = r_registers_map.get(name) {
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
                    Text::new(format!(" {name} :\t")),
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

            commands.entity(ui_core_registers).add_child(bits_container);
            commands.entity(ui_core_registers).add_children(&[
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

                commands.entity(bit_container).add_child(bit_value);
                commands.entity(bits_container).add_child(bit_container);
            }
        } else {
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
                .height(Val::Percent(10.0))
                .float("top")
                .margin(UiRect {
                    left: Val::Auto,
                    right: Val::Auto,
                    top: Val::Percent(2.5),
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
                .absolute()
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

pub fn setup_display(
    mut commands: Commands,
    q_ui_root: Query<(Entity, &Name), With<UiElement>>,
    mut r_pixel_display: Option<ResMut<PixelDisplay>>,
    mut r_images: ResMut<Assets<Image>>,
) {
    let image_size = bevy::render::render_resource::Extent3d {
        width: DISPLAY_WIDTH as u32,
        height: DISPLAY_HEIGHT as u32,
        depth_or_array_layers: 1,
    };
    let data_size = (DISPLAY_WIDTH * DISPLAY_HEIGHT * 4) as usize;
    let mut image_data = vec![0u8; data_size];

    for i in (3..data_size).step_by(4) {
        image_data[i] = 255;
    }

    let mut image = Image::new(
        image_size,
        bevy::render::render_resource::TextureDimension::D2,
        image_data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD
            | bevy::render::render_asset::RenderAssetUsages::MAIN_WORLD,
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::nearest());

    let h_image = r_images.add(image);

    commands.insert_resource(PixelDisplay {
        h_image: h_image.clone(),
    });

    let _display_sprite = commands
        .spawn((
            Sprite {
                image: h_image.clone(),
                ..Default::default()
            },
            Transform::from_xyz(0.0, 0.0, 0.0)
                .with_scale(Vec3::new(16.0, 16.0, 1.0)),
            Name::new("ui-display"),
        ))
        .id();
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
    if let Some((mut stage_text, _)) = q_stage_text
        .iter_mut()
        .find(|(_, name)| name.as_str() == "ui-cpu-cycle-stage-value")
    {
        let current_stage_value: &CpuCycleStage = s_current_stage.get();
        let stage_name = match current_stage_value {
            CpuCycleStage::Startup => "Startup",
            CpuCycleStage::Fetch => "Fetch",
            CpuCycleStage::Decode => "Decode",
            CpuCycleStage::Execute => "Execute",
            CpuCycleStage::Halt => "Halted",
        };
        if stage_text.0 != stage_name {
            stage_text.0 = stage_name.into();
        }
    } else {
        warn!("Failed to find ui-cpu-cycle-stage-value text element.");
    }

    q_advance_button
        .iter()
        .for_each(|(interaction, button_name)| {
            if !(*interaction == Interaction::Pressed) {
                return;
            }
            if !button_name.as_str().eq("ui-cpu-cycle-advance-button") {
                return;
            }

            let current_stage: &CpuCycleStage = s_current_stage.get();
            match current_stage {
                CpuCycleStage::Startup => {
                    s_next_stage.set(CpuCycleStage::Fetch);
                }
                CpuCycleStage::Fetch => {
                    s_next_stage.set(CpuCycleStage::Decode);
                }
                CpuCycleStage::Decode => {
                    s_next_stage.set(CpuCycleStage::Execute);
                }
                CpuCycleStage::Execute => {
                    s_next_stage.set(CpuCycleStage::Fetch);
                }
                CpuCycleStage::Halt => {
                    s_next_stage.set(CpuCycleStage::Startup);
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

pub fn update_display(
    r_display: Res<DisplayMemory>,
    mut r_pixel_display: ResMut<PixelDisplay>,
    mut r_images: ResMut<Assets<Image>>,
) {
    for x in 0..DISPLAY_WIDTH {
        for y in 0..DISPLAY_HEIGHT {
            let target_color = r_display.get_pixel(x as u16, y as u16).unwrap();

            r_pixel_display
                .set_pixel(x as usize, y as usize, target_color, &mut r_images)
                .unwrap();
        }
    }
}

/// ---------------- ///
/// Helper Functions ///
/// ---------------- ///

fn bits_to_u16(bits: &[i8]) -> u16 {
    let mut value: u16 = 0;
    for &bit in bits {
        value <<= 1;
        if bit == 1 {
            value |= 1;
        }
    }
    value
}

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

fn create_ui_pixel(
    x: usize,
    y: usize,
    mut r_meshes: &mut ResMut<Assets<Mesh>>,
    mut r_materials: &mut ResMut<Assets<ColorMaterial>>,
) -> (Mesh2d, MeshMaterial2d<ColorMaterial>, Transform, Name) {
    let name = Name::new(format!("ui-pixel-{x}-{y}"));

    let mut h_meshes = HashMap::new();
    let mut h_materials = HashMap::new();

    let h_mesh: &Handle<Mesh> = h_meshes
        .entry("mesh-ui-pixel")
        .or_insert_with(|| r_meshes.add(Rectangle::default()));

    let h_material: &Handle<ColorMaterial> = h_materials
        .entry(format!("material-ui-pixel-{x}-{y}"))
        .or_insert_with(|| {
            r_materials.add(ColorMaterial::from(Color::linear_rgba(
                x as f32, 0.0, y as f32, 1.0,
            )))
        });

    (
        Mesh2d(h_mesh.clone()),
        MeshMaterial2d(h_material.clone()),
        Transform::default().with_scale(Vec3::splat(256.0)),
        name,
    )
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
