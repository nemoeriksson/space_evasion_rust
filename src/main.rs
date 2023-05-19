use macroquad::prelude::*;
use std::time::SystemTime;

#[macroquad::main("Space Evasion : Rust Edition")]
#[derive(PartialEq)]

async fn main() {
    fn total(arr: &Vec<i32>) -> i32{
        let mut total:i32 = 0;
        for value in arr{
            total += value;
        }
        return total;
    }

    fn spawn_asteroid() -> Asteroid{
        let start_x:f32 = rand::gen_range::<f32>(0.,screen_width());
        let start_y:f32 = rand::gen_range::<f32>(0.,screen_height());
        let start_side:u8 = rand::gen_range::<u8>(0,4);
        let spawn_pos:Vec2;
        let spawn_vel:Vec2;
        match start_side{
            0_u8 => {  // Top -> Bottom
                spawn_pos = Vec2::new(start_x, 0.);
                spawn_vel = Vec2::new(0., ASTEROID_SPEED);
            },
            1_u8 => { // Bottom -> Top
                spawn_pos = Vec2::new(start_x, screen_height());
                spawn_vel = Vec2::new(0., -ASTEROID_SPEED);
            },
            2_u8 => { // Left -> Right
                spawn_pos = Vec2::new(0., start_y);
                spawn_vel = Vec2::new(ASTEROID_SPEED, 0.);
            },
            3_u8 => { // Right -> Left
                spawn_pos = Vec2::new(screen_width(), start_y);
                spawn_vel = Vec2::new(-ASTEROID_SPEED, 0.);
            },
            _ => {
                spawn_pos = Vec2::new(0., 0.,);
                spawn_vel = Vec2::new(ASTEROID_SPEED, ASTEROID_SPEED);
            }
        };
        let new_asteroid:Asteroid = Asteroid{
            size: rand::gen_range::<f32>(14., 18.), pos:spawn_pos, vel:spawn_vel
        };
        return new_asteroid;
    }
    
    struct Player{
        rot: f32,
        pos: Vec2,
        vel: Vec2
    }
    struct Bullet{
        rot: f32,
        pos: Vec2,
        vel: Vec2
    }
    struct Asteroid{
        size: f32,
        pos: Vec2,
        vel: Vec2
    }
    impl PartialEq<Asteroid> for Asteroid{
        fn eq(&self, other:&Asteroid) -> bool{
            return self.pos[0] == other.pos[0] && self.pos[1] == other.pos[1] && self.size == other.size;
        }
    }
    impl Asteroid{
        fn is_outside(&self) -> bool{
            return self.pos[0]<0. || self.pos[0]>screen_width() || self.pos[1]<0. || self.pos[1]>screen_height(); 
        }
    }

    const PLAYER_HEIGHT:f32 = 36.;
    const PLAYER_BASE:f32 = 36.;
    const FRICTION:f32 = 27.5;
    const COLLISION_DISABLE_TIME_MS:u128 = 150;
    const SHOOT_TIMEOUT_MS:u128 = 150;
    const MAX_BULLETS:usize = 32;
    const BULLET_SPEED:f32 = 7.;
    const ASTEROID_SPEED:f32 = 4.;
    const ASTEROID_ESCAPE_SCORE:u32 = 25;
    const ASTEROID_DESTROYED_SCORE:u32 = 150;
    const AMMO_CAPACITY:u8 = 8;

    let mut player = Player{
        pos: Vec2::new(screen_width() / 2., screen_height() / 2.),
        rot: 0.,
        vel: Vec2::new(0., 0.),
    };
    let mut player_lives:u8 = 3;
    let mut score:u32 = 0;
    let mut ammo:u8 = AMMO_CAPACITY;
    let mut ammo_text_color = WHITE;
    let mut bullets = Vec::new();
    let mut asteroids = Vec::new();
    let mut last_spawn_time = SystemTime::now();
    let mut asteroid_spawn_interval_ms = 800;
    let min_asteroid_spawn_interval_ms = 250;
    let mut lock_movement:bool = false;
    let mut lock_shoot:bool = false;
    let mut collided_time = SystemTime::now();
    let mut last_shot_time = SystemTime::now();
    let mut fps_history:Vec<i32> = vec![];
    let mut average_fps:i32;
    let mut game_over:bool = false;
    let player_texture = Texture2D::from_image(&Image::from_file_with_format(
        include_bytes!("../imgs/player.png"),
        Some(ImageFormat::Png)
    ));
    let background_texture = Texture2D::from_image(&Image::from_file_with_format(
        include_bytes!("../imgs/background.png"),
        Some(ImageFormat::Png)
    ));
    let bullet_texture = Texture2D::from_image(&Image::from_file_with_format(
        include_bytes!("../imgs/bullet.png"),
        Some(ImageFormat::Png)
    ));
    let asteroid_texture = Texture2D::from_image(&Image::from_file_with_format(
        include_bytes!("../imgs/asteroid.png"),
        Some(ImageFormat::Png)
    ));

    loop {
        // Exit the game
        if is_key_down(KeyCode::Escape){
            break;
        }

        if !game_over{
            // Spawn asteroids
            let mut spawning_asteroid:bool = false;
            match last_spawn_time.elapsed(){
                Ok(last_spawn_time) => {
                    if last_spawn_time.as_millis() > asteroid_spawn_interval_ms{
                        spawning_asteroid = true;
                    }
                },
                Err(e) => println!("Error: {e:?}")
            };
            if spawning_asteroid{
                asteroids.push(spawn_asteroid());
                last_spawn_time = SystemTime::now();
                if asteroid_spawn_interval_ms > min_asteroid_spawn_interval_ms {
                    asteroid_spawn_interval_ms -= 5;
                }
            }

            // Get player movement input
            let mut acc = -player.vel / FRICTION;
            let rotation = player.rot.to_radians();

            if (is_key_down(KeyCode::Up)&&!lock_movement) || (is_key_down(KeyCode::W)&&!lock_movement) {
                acc = Vec2::new(rotation.sin(), -rotation.cos()) / 3.;
            }

            if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
                player.rot += 5.;
            } 
            else if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
                player.rot -= 5.;
            }
            
            // Player movement/physics
            player.vel += acc;
            if player.vel.length() > 5. {
                player.vel = player.vel.normalize() * 5.;
            }
            player.pos += player.vel;

            // Shoot bullet
            if is_key_down(KeyCode::Space){
                if !lock_shoot && ammo > 0{
                    let new_bullet:Bullet = Bullet{
                        pos: player.pos,
                        vel: Vec2::new(
                            BULLET_SPEED*rotation.sin(),
                            BULLET_SPEED*-rotation.cos()
                        ),
                        rot: rotation
                    };
                    bullets.push(new_bullet);
                    if bullets.len() > MAX_BULLETS{
                        bullets.remove(0);
                    }
                    lock_shoot = true;
                    last_shot_time = SystemTime::now();
                    ammo -= 1;
                }
            }

            // Check so player is in the screen
            if player.pos[0] < 0.{
                player.vel[0] *= -1.;
                player.pos[0] = 0.;
                lock_movement = true;
                collided_time = SystemTime::now(); 
            } else if player.pos[0] > screen_width(){
                player.vel[0] *= -1.;
                player.pos[0] = screen_width();
                lock_movement = true;
                collided_time = SystemTime::now();
            } else if player.pos[1] < 0.{
                player.vel[1] *= -1.;
                player.pos[1] = 0.;
                lock_movement = true;
                collided_time = SystemTime::now(); 
            } else if player.pos[1] > screen_height(){
                player.vel[1] *= -1.;
                player.pos[1] = screen_height();
                lock_movement = true;
                collided_time = SystemTime::now(); 
            }

            // Checks if player can move after collision
            if lock_movement {
                match collided_time.elapsed(){
                    Ok(collided_time) => {
                        if lock_movement && collided_time.as_millis() > COLLISION_DISABLE_TIME_MS{
                            lock_movement = false;
                        }
                    },
                    Err(e) => println!("Error: {e:?}")
                }
            } // Checks if player can shoot
            if lock_shoot{
                match last_shot_time.elapsed(){
                    Ok(last_shot_time) => {
                        if lock_shoot && last_shot_time.as_millis() > SHOOT_TIMEOUT_MS{
                            lock_shoot = false;
                        }
                    },
                    Err(e) => println!("Error: {e:?}")
                };
            }

            // Get stats
            fps_history.push(get_fps());
            if fps_history.len() > 16{
                fps_history.remove(0);
            }
            average_fps = total(&fps_history)/(fps_history.len() as i32);

            let mut ammo_text = (0..ammo).map(|_| "|").collect::<String>();
            if ammo == 0{
                ammo_text_color = RED;
                ammo_text = "_".to_string();
            }

            let lives_text = (0..player_lives*2).map(|_| "O").collect::<String>();

            // Drawing textures
            draw_texture_ex(background_texture, 0., 0., WHITE, DrawTextureParams{
                dest_size: Some(Vec2::new(screen_width(), screen_height())),
                source: None, rotation: 0., flip_x: false, flip_y: false, pivot: None
            });

            // Renders & updates bullets
            let mut bullet_collisions = Vec::new();
            for bullet in &mut bullets{
                bullet.pos += bullet.vel;

                let bullet_rect = Rect::new(bullet.pos[0], bullet.pos[1], 2., 2.);
                for asteroid in &mut asteroids{
                    if Rect::new(asteroid.pos[0], asteroid.pos[1], asteroid.size, asteroid.size).overlaps(&bullet_rect){
                        bullet_collisions.push(Asteroid{
                            size: asteroid.size,
                            pos: asteroid.pos,
                            vel: asteroid.vel
                        });
                        bullet.vel *= 0.875;
                        score += ASTEROID_DESTROYED_SCORE;
                    }
                }

                draw_texture_ex(bullet_texture, bullet.pos[0]-8., bullet.pos[1]-8., WHITE, DrawTextureParams{
                    dest_size: Some(Vec2::new(16., 16.)),
                    source: None, rotation: bullet.rot, flip_x: true, flip_y:true, pivot: None
                });
            }
            asteroids.retain(|a| {
                if a.is_outside() { score+=ASTEROID_ESCAPE_SCORE; }
                !bullet_collisions.contains(&a) && !a.is_outside()});

            // Renders & updates asteroids
            let player_rect = Rect::new(player.pos[0]-PLAYER_BASE/2., player.pos[1]-PLAYER_HEIGHT/2., PLAYER_BASE, PLAYER_HEIGHT);
            let mut asteroid_collisions = Vec::new();
            for asteroid in &mut asteroids{
                asteroid.pos += asteroid.vel;
                if Rect::new(asteroid.pos[0], asteroid.pos[1], asteroid.size, asteroid.size).overlaps(&player_rect){
                    player_lives -= 1;
                    asteroid_collisions.push(Asteroid{
                        size: asteroid.size, pos: asteroid.pos, vel: asteroid.vel
                    });
                    if player_lives <= 0 { game_over = true; }
                }
                draw_texture_ex(asteroid_texture, asteroid.pos[0], asteroid.pos[1], WHITE, DrawTextureParams{
                    dest_size: Some(Vec2::new(asteroid.size, asteroid.size)),
                    source: None, rotation: 0., flip_x: false, flip_y: false, pivot: None
                }); 
            }
            // Remove collided asteroids
            asteroids.retain(|a| !asteroid_collisions.contains(&a));

            // Render player
            draw_texture_ex(player_texture, player.pos[0]-PLAYER_BASE/2., player.pos[1]-PLAYER_HEIGHT/2., WHITE, DrawTextureParams{
                dest_size: Some(Vec2::new(PLAYER_BASE, PLAYER_HEIGHT)),
                source: None, rotation: rotation, flip_x: false, flip_y: false, pivot: None
            });
            
            // Render stats
            draw_text(&average_fps.to_string(), 8., 16., 20., WHITE);
            draw_text(&"AMMO: ".to_string(), 8., screen_height()-16., 20., ammo_text_color);
            let lives_text_size:TextDimensions = measure_text(&lives_text, None, 20, 1.);
            draw_text(&lives_text, (screen_width()-lives_text_size.width)/2., screen_height()-16., 20., WHITE);
            draw_text(&ammo_text, 54., screen_height()-16., 20., WHITE);
            let score_text_size:TextDimensions = measure_text(&score.to_string(), None, 20, 1.);
            draw_text(&score.to_string(), screen_width()-16.-score_text_size.width/2., screen_height()-16., 20., WHITE);
        }else { // Game over screen
            let color_value:f32 = rand::gen_range::<f32>(0.08, 0.11);
            let background_color:Color = Color{r: color_value, g: color_value, b: color_value, a:1.};
            clear_background(background_color);
            let game_over_text_size:TextDimensions = measure_text(&"GAME OVER".to_string(), None, 30, 1.);
            draw_text(&"GAME OVER".to_string(), (screen_width()-game_over_text_size.width)/2., (screen_height()-game_over_text_size.height)/2.-16., 30., WHITE);
            let restart_text_size:TextDimensions = measure_text(&"Press [ENTER] to restart".to_string(), None, 20, 1.);
            draw_text(&"Press [ENTER] to restart".to_string(), (screen_width()-restart_text_size.width)/2., (screen_height()-restart_text_size.height)/2.+16., 20., WHITE);
            
            if is_key_down(KeyCode::Enter){
                player = Player{
                    pos: Vec2::new(screen_width() / 2., screen_height() / 2.),
                    rot: 0.,
                    vel: Vec2::new(0., 0.),
                };
                player_lives= 3;
                score = 0;
                ammo = AMMO_CAPACITY;
                ammo_text_color = WHITE;
                bullets = Vec::new();
                asteroids = Vec::new();
                last_spawn_time = SystemTime::now();
                asteroid_spawn_interval_ms = 800;
                lock_movement = false;
                lock_shoot = false;
                collided_time = SystemTime::now();
                last_shot_time = SystemTime::now();
                fps_history = vec![];
                game_over = false;   
            }
        }
        next_frame().await
    }
}