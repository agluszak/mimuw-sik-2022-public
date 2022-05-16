use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use crossbeam_channel::{bounded, Receiver};
use std::net::{Ipv6Addr, SocketAddr, ToSocketAddrs, UdpSocket};

use clap::Parser;
use lazy_static::lazy_static;
use robots::{DisplayMessage, InputMessage, Position, MAX_UDP_LENGTH};
use robots::serialize::{deserializer, serializer};

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, parse(try_from_str = parse_addr))]
    client_address: SocketAddr,

    #[clap(short, long)]
    port: u16,
}

fn parse_addr(s: &str) -> Result<SocketAddr, String> {
    s.to_socket_addrs()
        .map_err(|e| e.to_string())
        .and_then(|mut iter| iter.next().ok_or_else(|| "No address found".to_string()))
}

lazy_static! {
    static ref ARGS: Args = Args::parse();
}

fn main() {
    info!(args = ?ARGS.clone());

    App::new()
        .insert_resource(WindowDescriptor {
            title: "Robots!".to_string(),
            width: 640.,
            height: 480.,
            ..default()
        })
        .insert_resource(bevy::log::LogSettings {
            level: bevy::log::Level::INFO,
            filter: "wgpu=warn,wgpu_core=warn,bevy_ecs=info".to_string(),
        })
        .add_event::<DisplayMessage>()
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_startup_system(setup)
        .add_system(read_stream)
        .add_system(draw)
        .add_system(send_input)
        .run();
}

#[derive(Deref)]
struct DisplayMessageReceiver(Receiver<DisplayMessage>);

#[derive(Deref)]
struct InputMessageSender(UdpSocket);

#[derive(Deref)]
struct LoadedFont(Handle<Font>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let socket = UdpSocket::bind(SocketAddr::from((Ipv6Addr::UNSPECIFIED, ARGS.port))).unwrap();
    let socket_clone = socket.try_clone().unwrap();

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    let (display_tx, display_rx) = bounded::<DisplayMessage>(10);
    std::thread::spawn(move || loop {
        let mut buf = Box::new([0; MAX_UDP_LENGTH]);
        match socket.recv_from(&mut *buf) {
            Ok((amt, _src)) => {
                let message = deserializer::from_bytes::<DisplayMessage>(&buf[0..amt]);
                match message {
                    Ok(message) => {
                        info!(?message, "Received message");
                        display_tx.send(message).unwrap()
                    }
                    Err(e) => error!("{:?}", e),
                }
            }
            Err(e) => error!("{}", e),
        }
    });

    commands.insert_resource(InputMessageSender(socket_clone));
    commands.insert_resource(DisplayMessageReceiver(display_rx));

    let loaded_font = asset_server.load("fonts/FiraSans-Medium.ttf");
    commands.insert_resource(LoadedFont(loaded_font.clone()));

    let text_style = TextStyle {
        font: loaded_font,
        font_size: 25.0,
        color: Color::RED,
    };
    let text_alignment = TextAlignment {
        vertical: VerticalAlign::Top,
        horizontal: HorizontalAlign::Center,
    };
    let text = format!("Waiting for input from client ({}) on port {}", ARGS.client_address, ARGS.port);
    commands
        .spawn_bundle(Text2dBundle {
            text: Text::with_section(text, text_style, text_alignment),
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            ..default()
        })
        .insert(Renderable);
}

// This system reads from the receiver and sends events to Bevy
fn read_stream(receiver: ResMut<DisplayMessageReceiver>, mut events: EventWriter<DisplayMessage>) {
    for message in receiver.try_iter() {
        events.send(message);
    }
}

fn send_input(
    socket: ResMut<InputMessageSender>,
    mut keyboard_input_events: EventReader<KeyboardInput>,
) {
    for event in keyboard_input_events.iter() {
        if event.state.is_pressed() {
            let input_message: InputMessage = match event.key_code {
                Some(KeyCode::W | KeyCode::Up) => InputMessage::Move {
                    direction: robots::Direction::Up,
                },
                Some(KeyCode::A | KeyCode::Left) => InputMessage::Move {
                    direction: robots::Direction::Left,
                },
                Some(KeyCode::S | KeyCode::Down) => InputMessage::Move {
                    direction: robots::Direction::Down,
                },
                Some(KeyCode::D | KeyCode::Right) => InputMessage::Move {
                    direction: robots::Direction::Right,
                },
                Some(KeyCode::Space | KeyCode::J | KeyCode::Z) => InputMessage::PlaceBomb,
                Some(KeyCode::K | KeyCode::X) => InputMessage::PlaceBlock,
                _ => continue,
            };
            info!("Sending {:?}", input_message);
            match socket.0.send_to(&serializer::to_bytes(input_message), ARGS.client_address) {
                Ok(amt) => info!("Sent {} bytes", amt),
                Err(e) => error!("{}", e),
            }
        }
    }
}

const COLORS: &[Color] = &[
    Color::RED,
    Color::GREEN,
    Color::BLUE,
    Color::YELLOW,
    Color::CYAN,
    Color::PURPLE,
    Color::SALMON,
    Color::ORANGE,
    Color::PINK,
    Color::OLIVE,
];

#[derive(Component)]
struct Renderable;

enum SpawnType {
    Player(Color),
    Bomb,
    Block,
    Explosion,
    Grid,
}

struct Spawner {
    length_x: f32,
    length_y: f32,
    grid_shape: shapes::Rectangle,
    bomb_shape: shapes::Ellipse,
    player_shape: shapes::Polygon,
    explosion_shape: shapes::Polygon,
}

const GRID_DISPLAY_SIZE: f32 = 400.0;

impl Spawner {
    fn new(size_x: u16, size_y: u16) -> Self {
        let length_x = GRID_DISPLAY_SIZE / size_x as f32;
        let length_y = GRID_DISPLAY_SIZE / size_y as f32;
        let grid_shape = shapes::Rectangle {
            extents: Vec2::new(length_x, length_y),
            origin: RectangleOrigin::TopLeft,
        };
        let bomb_shape = shapes::Ellipse {
            radii: Vec2::new(length_x / 2.0, length_y / 2.0),
            center: Vec2::new(length_x / 2.0, -length_y / 2.0),
        };
        let explosion_shape = shapes::Polygon {
            points: vec![
                Vec2::new(0.0, -length_y),
                Vec2::new(length_x, 0.0),
                Vec2::new(length_x, -length_y),
                Vec2::new(0.0, 0.0),
            ],
            closed: false,
        };

        let player_shape = shapes::Polygon {
            points: vec![
                Vec2::new(0.0, -length_y),
                Vec2::new(length_x, -length_y),
                Vec2::new(length_x / 2.0, 0.0),
            ],
            closed: true,
        };
        Self {
            grid_shape,
            bomb_shape,
            player_shape,
            explosion_shape,
            length_x,
            length_y,
        }
    }

    fn spawn(&self, commands: &mut Commands, position: Position, what: SpawnType) {
        let color = match what {
            SpawnType::Player(c) => c,
            SpawnType::Bomb => Color::RED,
            SpawnType::Block => Color::DARK_GRAY,
            SpawnType::Explosion => Color::BLACK,
            SpawnType::Grid => Color::WHITE,
        };
        let draw_mode = DrawMode::Outlined {
            fill_mode: FillMode::color(color),
            outline_mode: StrokeMode::new(Color::BLACK, 1.0),
        };
        let z = match what {
            SpawnType::Player(_) => 3.0,
            SpawnType::Bomb => 2.0,
            SpawnType::Block => 1.0,
            SpawnType::Explosion => 4.0,
            SpawnType::Grid => 0.0,
        };
        let transform = Transform::from_xyz(
            position.0 as f32 * self.length_x - GRID_DISPLAY_SIZE / 2.0,
            position.1 as f32 * self.length_y - GRID_DISPLAY_SIZE / 2.0,
            z,
        );
        let geometry = GeometryBuilder::new();

        let geometry = match what {
            SpawnType::Player(_) => geometry.add(&self.player_shape),
            SpawnType::Bomb => geometry.add(&self.bomb_shape),
            SpawnType::Block => geometry.add(&self.grid_shape),
            SpawnType::Explosion => geometry.add(&self.explosion_shape),
            SpawnType::Grid => geometry.add(&self.grid_shape),
        };
        let bundle = geometry.build(draw_mode, transform);

        commands.spawn_bundle(bundle).insert(Renderable);
    }
}

fn draw(
    mut commands: Commands,
    mut reader: EventReader<DisplayMessage>,
    old_renderables: Query<Entity, With<Renderable>>,
    loaded_font: Res<LoadedFont>,
) {
    let text_style = TextStyle {
        font: loaded_font.clone(),
        font_size: 20.0,
        color: Color::WHITE,
    };
    let text_alignment = TextAlignment {
        vertical: VerticalAlign::Top,
        horizontal: HorizontalAlign::Left,
    };

    if let Some(event) = reader.iter().last() {
        for ent in old_renderables.iter() {
            commands.entity(ent).despawn()
        }

        info!("{}", serde_json::to_string(&event).unwrap());

        match event {
            DisplayMessage::Lobby {
                server_name,
                players_count,
                size_x,
                size_y,
                game_length,
                explosion_radius,
                bomb_timer,
                players,
            } => {
                let players = players
                    .iter()
                    .map(|(id, player)| {
                        format!("({}) {} - {}", id.0, player.name, player.socket_addr)
                    })
                    .collect::<String>();
                let text = format!(
                    "Server: {server_name}\nRequired players: {players_count}\n\
                    Size: {size_x}x{size_y}\nGame length: {game_length}\n\
                    Explosion radius: {explosion_radius}\nBomb timer: {bomb_timer}\n\
                    Players:\n\
                    {players}",
                );
                commands
                    .spawn_bundle(Text2dBundle {
                        text: Text::with_section(text, text_style, text_alignment),
                        transform: Transform::from_xyz(-300.0, 200.0, 1.0),
                        ..default()
                    })
                    .insert(Renderable);
            }
            DisplayMessage::Game {
                server_name,
                size_x,
                size_y,
                game_length,
                turn,
                players,
                player_positions,
                blocks,
                bombs,
                explosions,
                scores,
            } => {
                let server_info = format!("{server_name} - Turn {turn}/{game_length}");

                commands
                    .spawn_bundle(Text2dBundle {
                        text: Text::with_section(server_info, text_style, text_alignment),
                        transform: Transform::from_xyz(-100.0, 220.0, 1.0),
                        ..default()
                    })
                    .insert(Renderable);

                let colors = COLORS.iter().cycle();
                let players = players
                    .iter()
                    .zip(colors)
                    .flat_map(|((id, player), &color)| {
                        // TODO: handle missing data
                        let score = scores.get(id).cloned()?;
                        let position = player_positions.get(id).cloned()?;
                        Some((id, player.name.clone(), score, position, color))
                    })
                    .collect::<Vec<_>>();

                let scores_sections = players
                    .iter()
                    .map(|(id, name, score, _, color)| {
                        let score = format!("{}", score.deaths);
                        let id = id.0;
                        let text = format!("({id}) {name}: {score}\n");
                        let color = *color;
                        TextSection {
                            value: text,
                            style: TextStyle {
                                font: loaded_font.clone(),
                                font_size: 15.0,
                                color,
                            },
                        }
                    })
                    .collect::<Vec<_>>();

                let scores_text = Text {
                    sections: scores_sections,
                    alignment: text_alignment,
                };

                commands
                    .spawn_bundle(Text2dBundle {
                        text: scores_text,
                        transform: Transform::from_xyz(-300.0, 200.0, 0.0),
                        ..default()
                    })
                    .insert(Renderable);

                let spawner = Spawner::new(*size_x, *size_y);

                // grid
                for x in 0..*size_x {
                    for y in 0..*size_y {
                        spawner.spawn(&mut commands, Position(x, y), SpawnType::Grid);
                    }
                }

                for block in blocks {
                    spawner.spawn(&mut commands, *block, SpawnType::Block);
                }

                for explosion in explosions {
                    spawner.spawn(&mut commands, *explosion, SpawnType::Explosion);
                }

                for bomb in bombs {
                    spawner.spawn(&mut commands, bomb.position, SpawnType::Bomb);
                    // TODO spawn time left
                }

                players.iter().for_each(|(_, _, _, position, color)| {
                    spawner.spawn(&mut commands, *position, SpawnType::Player(*color));
                });
            }
        }
    }
}
