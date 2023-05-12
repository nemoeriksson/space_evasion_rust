use macroquad::prelude::*;
use std::time::SystemTime;

#[macroquad::main("Space Evasion : Rust Edition")]

async fn main() {
    fn total(arr: &Vec<i32>) -> i32{
        let mut total:i32 = 0;
        for value in arr{
            total += value;
        }
        return total;
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

    const PLAYER_HEIGHT:f32 = 32.;
    const PLAYER_BASE:f32 = 24.;
    const FRICTION:f32 = 27.5;
    const COLLISION_DISABLE_TIME_MS:u128 = 150;
    const SHOOT_TIMEOUT_MS:u128 = 150;
    const MAX_BULLETS:usize = 32;
    const BULLET_SPEED:f32 = 7.;

    let mut player = Player{
        pos: Vec2::new(screen_width() / 2., screen_height() / 2.),
        rot: 0.,
        vel: Vec2::new(0., 0.),
    };
    let mut bullets = Vec::new();
    let mut lock_movement:bool = false;
    let mut lock_shoot:bool = false;
    let mut collided_time = SystemTime::now();
    let mut last_shot_time = SystemTime::now();
    let mut fps_history:Vec<i32> = vec![];
    let mut average_fps:i32;

    let background_texture = Texture2D::from_image(&Image::from_file_with_format(
        include_bytes!("../imgs/background.png"),
        Some(ImageFormat::Png)
    ));
    let bullet_texture = Texture2D::from_image(&Image::from_file_with_format(
        include_bytes!("../imgs/bullet.png"),
        Some(ImageFormat::Png)
    ));

    loop {
        if is_key_down(KeyCode::Escape){
            break;
        }

        fps_history.push(get_fps());
        if fps_history.len() > 16{
            fps_history.remove(0);
        }
        average_fps = total(&fps_history)/(fps_history.len() as i32);

        let mut acc = -player.vel / FRICTION;
        let rotation = player.rot.to_radians();

        if (is_key_down(KeyCode::Up)&&!lock_movement) || (is_key_down(KeyCode::W)&&!lock_movement){
            acc = Vec2::new(rotation.sin(), -rotation.cos()) / 3.;
        }

        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            player.rot += 5.;
        } 
        else if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            player.rot -= 5.;
        }

        player.vel += acc;
        if player.vel.length() > 5. {
            player.vel = player.vel.normalize() * 5.;
        }
        player.pos += player.vel;

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

        if lock_movement {
            match collided_time.elapsed(){
                Ok(collided_time) => {
                    if lock_movement && collided_time.as_millis() > COLLISION_DISABLE_TIME_MS{
                        lock_movement = false;
                    }
                },
                Err(e) => println!("Error: {e:?}")
            }
        }
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

        let v1 = Vec2::new(
            player.pos.x + rotation.sin() * PLAYER_HEIGHT / 2.,
            player.pos.y - rotation.cos() * PLAYER_HEIGHT / 2.,
        );
        let v2 = Vec2::new(
            player.pos.x - rotation.cos() * PLAYER_BASE / 2. - rotation.sin() * PLAYER_HEIGHT / 2.,
            player.pos.y - rotation.sin() * PLAYER_BASE / 2. + rotation.cos() * PLAYER_HEIGHT / 2.,
        );
        let v3 = Vec2::new(
            player.pos.x + rotation.cos() * PLAYER_BASE / 2. - rotation.sin() * PLAYER_HEIGHT / 2.,
            player.pos.y + rotation.sin() * PLAYER_BASE / 2. + rotation.cos() * PLAYER_HEIGHT / 2.,
        );

        if is_key_down(KeyCode::Space){
            if !lock_shoot {
                let new_bullet:Bullet = Bullet{
                    pos: v1,
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
            }
        }

        draw_texture_ex(background_texture, 0., 0., WHITE, DrawTextureParams{
            dest_size: Some(Vec2::new(screen_width(), screen_height())),
            source: None, rotation: 0., flip_x: false, flip_y: false, pivot: None
        });

        for bullet in &mut bullets{
            bullet.pos += bullet.vel;
            draw_texture_ex(bullet_texture, bullet.pos[0]-8., bullet.pos[1]-8., WHITE, DrawTextureParams{
                dest_size: Some(Vec2::new(16., 16.)),
                source: None, rotation: bullet.rot, flip_x: true, flip_y:true, pivot: None
            })
        }

        draw_triangle_lines(v1, v2, v3, 1.5, LIGHTGRAY); 
        draw_text(&average_fps.to_string(), 8., 16., 20., WHITE);
        next_frame().await
    }
}