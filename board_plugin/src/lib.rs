pub mod components;
pub mod resources;

use bevy::prelude::*;
use components::*;
use resources::{tile::Tile, tile_map::TileMap, BoardOptions, BoardPosition, TileSize};

#[cfg(feature = "debug")]
use bevy_inspector_egui::RegisterInspectable;

pub struct BoardPlugin;

impl BoardPlugin {
    pub fn create_board() {
        let mut tile_map = TileMap::empty(20, 20);
        tile_map.set_bombs(40);

        #[cfg(feature = "debug")]
        info!("{}", tile_map.console_output());
    }
}

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(create_board);
        info!("Loaded Board Plugin");

        #[cfg(feature = "debug")]
        {
            // registering custom component to be able to edit it in inspector
            app.register_inspectable::<Coordinates>();
            app.register_inspectable::<Bomb>();
            app.register_inspectable::<BombNeighbor>();
            app.register_inspectable::<Uncover>();
        }
    }
}

// Generates the bomb counter text 2d bundle for a given value
fn bomb_count_text_bundle(count: u8, font: Handle<Font>, size: f32) -> Text2dBundle {
    // retrieve te text and the correct color
    let (text, color) = (
        count.to_string(),
        match count {
            1 => Color::WHITE,
            2 => Color::GREEN,
            3 => Color::YELLOW,
            4 => Color::ORANGE,
            _ => Color::PURPLE,
        },
    );

    // generate text bundle
    Text2dBundle {
        text: Text {
            sections: vec![TextSection {
                value: text,
                style: TextStyle {
                    color,
                    font,
                    font_size: size,
                },
            }],
            alignment: TextAlignment {
                vertical: VerticalAlign::Center,
                horizontal: HorizontalAlign::Center,
            },
        },
        transform: Transform::from_xyz(0., 0., 1.),
        ..Default::default()
    }
}

fn adaptative_tile_size(
    window: Res<WindowDescriptor>,
    (min, max): (f32, f32),
    (width, height): (u16, u16),
) -> f32 {
    let max_width = window.width / width as f32;
    let max_height = window.height / height as f32;

    max_width.min(max_height).clamp(min, max)
}

fn spawn_tiles(
    parent: &mut ChildBuilder,
    tile_map: &TileMap,
    size: f32,
    padding: f32,
    color: Color,
    bomb_image: Handle<Image>,
    font: Handle<Font>,
) {
    // Tiles
    for (y, line) in tile_map.iter().enumerate() {
        for (x, tile) in line.iter().enumerate() {
            let coordinates = Coordinates {
                x: x as u16,
                y: y as u16,
            };
            let mut cmd = parent.spawn();
            cmd.insert_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::GRAY,
                    custom_size: Some(Vec2::splat(size - padding)),
                    ..Default::default()
                },
                transform: Transform::from_xyz(
                    (x as f32 * size) + (size / 2.),
                    (y as f32 * size) + (size / 2.),
                    1.,
                ),
                ..Default::default()
            })
            .insert(Name::new(format!("Tile ({x}, {y})")))
            .insert(coordinates);

            match tile {
                // If the tile is a bomb, add the matching component and a sprite child
                Tile::Bomb => {
                    cmd.insert(Bomb).with_children(|parent| {
                        parent.spawn_bundle(SpriteBundle {
                            sprite: Sprite {
                                custom_size: Some(Vec2::splat(size - padding)),
                                ..Default::default()
                            },
                            transform: Transform::from_xyz(0., 0., 1.),
                            texture: bomb_image.clone(),
                            ..Default::default()
                        });
                    });
                }
                // If the tile is a bomb neighbour, add the matching component and a text child
                Tile::BombNeighbor(v) => {
                    cmd.insert(BombNeighbor { count: *v })
                        .with_children(|parent| {
                            parent.spawn_bundle(bomb_count_text_bundle(
                                *v,
                                font.clone(),
                                size - padding,
                            ));
                        });
                }
                Tile::Empty => (),
            }
        }
    }
}

pub fn create_board(
    mut cmds: Commands,
    board_options: Option<Res<BoardOptions>>,
    window: Option<Res<WindowDescriptor>>,
    asset_server: Res<AssetServer>,
) {
    let font: Handle<Font> = asset_server.load("fonts/pixeled.ttf");
    let bomb_image: Handle<Image> = asset_server.load("sprites/bomb.png");
    let options = match board_options {
        Some(o) => o.clone(),
        None => BoardOptions::default(),
    };

    let mut tile_map = TileMap::empty(options.map_size.0, options.map_size.1);
    tile_map.set_bombs(options.bomb_count);

    #[cfg(feature = "debug")]
    // Tile map debugging
    info!("{}", tile_map.console_output());

    if let Some(win) = window {
        // define the size of the tiles in world space
        let tile_size = match options.tile_size {
            TileSize::Fixed(v) => v as f32,
            TileSize::Adaptive { min, max } => {
                adaptative_tile_size(win, (min, max), (tile_map.width(), tile_map.height()))
            }
        };

        // deduce the size of the complete board
        let board_size = Vec2::new(
            tile_map.width() as f32 * tile_size,
            tile_map.height() as f32 * tile_size,
        );
        info!("Board size: {board_size}");

        // define the board anchor position (bottom left)
        let board_position = match options.position {
            BoardPosition::Centered { offset } => {
                Vec3::new(-(board_size.x / 2.), -(board_size.y / 2.), 0.) + offset
            }
            BoardPosition::Custom(p) => p,
        };

        // spawn the board
        cmds.spawn()
            .insert(Name::new("Board"))
            .insert(Transform::from_translation(board_position))
            .insert(GlobalTransform::default())
            .with_children(|parent| {
                parent
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            color: Color::WHITE,
                            custom_size: Some(board_size),
                            ..Default::default()
                        },
                        transform: Transform::from_xyz(board_size.x / 2., board_size.y / 2., 0.),
                        ..Default::default()
                    })
                    .insert(Name::new("Background"));

                // spawn the tiles
                spawn_tiles(
                    parent,
                    &tile_map,
                    tile_size,
                    options.tile_padding,
                    Color::GRAY,
                    bomb_image,
                    font,
                );
            });
    }
}
