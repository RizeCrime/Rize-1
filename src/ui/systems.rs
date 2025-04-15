use bevy::prelude::*;

use crate::*;

pub fn setup_ui(mut commands: Commands, res_registers: Res<Registers>) {

    let ui_root = commands.spawn((
        Node {
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            border: UiRect::all(Val::Px(2.0)),
            padding: UiRect::all(Val::Px(4.0)),
            ..Default::default()
        },
        BorderColor(Color::linear_rgb(255.0, 127.0, 0.0)),
        Name::new("R1_UiRoot")
    )).id();

    let ui_registers = commands.spawn((
        Node {
            margin: UiRect {
                top: Val::Auto,
                right: Val::Auto,
                bottom: Val::ZERO,
                left: Val::ZERO,
            },
            width: Val::Auto,
            height: Val::Auto,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(4.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..Default::default()
        },
        BorderColor(Color::linear_rgb(255.0, 0.0, 0.0)),
        Name::new("R1_UiRegisters")
    )).id();
    
    let ui_registers_text = commands.spawn((
        Text::new("General Purpose Registers"),
        Name::new("R1_UiRegistersText")
    )).id();

    commands.entity(ui_root).add_child(ui_registers);
    commands.entity(ui_registers).add_child(ui_registers_text);

    let res_registers: &Registers = res_registers.as_ref();
    for (name, register) in res_registers.all().iter() {

        if name.starts_with('g') { 

            let bits_container = commands.spawn((
                Node {
                    height: Val::Auto,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    border: UiRect::all(Val::Px(2.0)),
                    padding: UiRect::all(Val::Px(4.0)),
                    ..Default::default()
                },
                BorderColor(Color::linear_rgb(0.0, 255.0, 0.0)),
                Name::new(format!("ui-register-bits-{name}"))
            )).id();

            let bits_container_text = commands.spawn((
                Text::new(format!(" {name}a :\t")),
                Name::new(format!("ui-register-bits-{name}-text"))
            )).id();

            commands.entity(bits_container).add_child(bits_container_text);
            commands.entity(ui_registers).add_child(bits_container);

            for bit in register.read() {

                let bit_container = commands.spawn((
                    Node {
                        left: Val::Px(0.0),
                        border: UiRect::all(Val::Px(2.0)),
                        padding: UiRect::all(Val::Px(4.0)),
                        ..Default::default()
                    },
                    BorderColor(Color::linear_rgb(0.0, 0.0, 255.0)),
                    Name::new(format!("ui-register-bit-{name}-{bit}"))
                )).id();

                let bit_value = commands.spawn((
                    Text::new(bit.bit_to_string()),
                    Name::new(format!("ui-register-bit-{name}-{bit}-value"))
                )).id();
            
                commands.entity(bits_container).add_child(bit_container);
                commands.entity(bit_container).add_child(bit_value);

            }

        }
    }
}