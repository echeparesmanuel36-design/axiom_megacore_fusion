use macroquad::prelude::*;
use rand::gen_range;
use rayon::prelude::*; // El turbo multinúcleo para gestionar la horda iluminada

// 1. ESTRUCTURAS DE DATOS (Obstáculos físicos y Agentes IA)
#[derive(Clone, Copy)]
struct Wall {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(Clone, Copy)]
struct Boid {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
}

struct SwarmEngine {
    boids: Vec<Boid>,
}

impl SwarmEngine {
    fn new() -> Self {
        Self { boids: Vec::new() }
    }

    fn spawn_swarm(&mut self, count: usize) {
        for _ in 0..count {
            self.boids.push(Boid {
                x: gen_range(100.0, screen_width() - 100.0),
                y: gen_range(100.0, screen_height() - 100.0),
                vx: gen_range(-2.5, 2.5),
                vy: gen_range(-2.5, 2.5),
            });
        }
    }

    // 🔥 EL CEREBRO DE LA COLMENA + COLISIÓN DE PAREDES EN PARALELO
    fn update_swarm(&mut self, dt: f32, width: f32, height: f32, walls: &[Wall]) {
        let current_boids = self.boids.clone();

        self.boids.par_iter_mut().for_each(|boid| {
            let mut close_dx = 0.0;
            let mut close_dy = 0.0;
            let mut x_vel_avg = 0.0;
            let mut y_vel_avg = 0.0;
            let mut neighboring_boids = 0;

            let visual_range = 35.0;
            let protected_range = 8.0;

            // Lógica de comportamiento de enjambre (Flocking)
            for other in &current_boids {
                let dx = boid.x - other.x;
                let dy = boid.y - other.y;
                let distance = (dx*dx + dy*dy).sqrt();

                if distance < visual_range && distance > 0.0 {
                    if distance < protected_range {
                        close_dx += dx;
                        close_dy += dy;
                    }
                    x_vel_avg += other.vx;
                    y_vel_avg += other.vy;
                    neighboring_boids += 1;
                }
            }

            if neighboring_boids > 0 {
                x_vel_avg /= neighboring_boids as f32;
                y_vel_avg /= neighboring_boids as f32;
                boid.vx += (x_vel_avg - boid.vx) * 0.05;
                boid.vy += (y_vel_avg - boid.vy) * 0.05;
            }

            boid.vx += close_dx * 0.15;
            boid.vy += close_dy * 0.15;

            // Atracción sutil al centro de la pantalla
            boid.vx += (width / 2.0 - boid.x) * 0.0003;
            boid.vy += (height / 2.0 - boid.y) * 0.0003;

            // 🔥 FÍSICA INYECTADA: Colisión elástica contra las paredes físicas
            for wall in walls {
                if boid.x > wall.x && boid.x < wall.x + wall.w && boid.y > wall.y && boid.y < wall.y + wall.h {
                    // Si entra en una pared, rebota con furia hacia fuera
                    let mid_x = wall.x + wall.w / 2.0;
                    let mid_y = wall.y + wall.h / 2.0;
                    boid.vx = (boid.x - mid_x).signum() * 3.0;
                    boid.vy = (boid.y - mid_y).signum() * 3.0;
                }
            }

            // Limitador de velocidad de crucero militar
            let speed = (boid.vx*boid.vx + boid.vy*boid.vy).sqrt();
            if speed > 4.5 {
                boid.vx = (boid.vx / speed) * 4.5;
                boid.vy = (boid.vy / speed) * 4.5;
            }

            boid.x += boid.vx * dt * 60.0;
            boid.y += boid.vy * dt * 60.0;

            // Bordes infinitos estilo Pac-Man
            if boid.x < 0.0 { boid.x = width; }
            if boid.x > width { boid.x = 0.0; }
            if boid.y < 0.0 { boid.y = height; }
            if boid.y > height { boid.y = 0.0; }
        });
    }

    // RENDERIZADO DE AGENTES SENSING (Cambian de color al entrar en el haz de luz)
    fn render_swarm(&self, light_x: f32, light_y: f32) {
        for boid in &self.boids {
            let dx = boid.x - light_x;
            let dy = boid.y - light_y;
            let dist_to_light = (dx*dx + dy*dy).sqrt();

            // Si están cerca de la luz brillan en amarillo neón, si están ocultos en la sombra son verdes oscuros
            let color = if dist_to_light < 250.0 {
                Color::new(1.0, 1.0, 0.0, 0.85) // Revelados por el haz
            } else {
                Color::new(0.0, 0.5, 0.3, 0.4)  // Ocultos en el búnker
            };

            draw_circle(boid.x, boid.y, 1.8, color);
        }
    }
}

// 3. EL BUCLE SUPREMO
#[macroquad::main("AXIOM MEGACORE // THE FUSION")]
async fn main() {
    let mut swarm = SwarmEngine::new();
    
    // Spawneamos 10.000 agentes de IA en paralelo compartiendo espacio con la luz
    swarm.spawn_swarm(10000);

    let walls = vec![
        Wall { x: 200.0, y: 150.0, w: 120.0, h: 120.0 },
        Wall { x: 550.0, y: 120.0, w: 150.0, h: 90.0 },
        Wall { x: 220.0, y: 420.0, w: 100.0, h: 140.0 },
        Wall { x: 600.0, y: 380.0, w: 110.0, h: 110.0 },
    ];

    loop {
        clear_background(Color::new(0.01, 0.01, 0.02, 1.0)); // Modo búnker puro

        let dt = get_frame_time();
        let (light_x, light_y) = mouse_position();

        // 1. Ejecutamos la IA colmena en hilos paralelos cruzando obstáculos
        swarm.update_swarm(dt, screen_width(), screen_height(), &walls);

        // 2. 🔥 EL MOTOR DE SOMBRAS EN TIEMPO REAL
        let num_rays = 280; // Balance perfecto para rendimiento extremo
        for i in 0..num_rays {
            let angle = (i as f32 * (360.0 / num_rays as f32)).to_radians();
            let ray_dir_x = angle.cos();
            let ray_dir_y = angle.sin();
            let mut max_dist = 250.0; // Radio de alcance de tu linterna espacial

            for wall in &walls {
                let t1 = (wall.x - light_x) / ray_dir_x;
                let t2 = (wall.x + wall.w - light_x) / ray_dir_x;
                let t3 = (wall.y - light_y) / ray_dir_y;
                let t4 = (wall.y + wall.h - light_y) / ray_dir_y;

                let tmin = t1.min(t2).max(t3.min(t4));
                let tmax = t1.max(t2).min(t3.max(t4));

                if tmax >= tmin && tmin > 0.0 && tmin < max_dist {
                    max_dist = tmin; // Choca la luz, genera sombra detrás de la caja
                }
            }

            let end_x = light_x + ray_dir_x * max_dist;
            let end_y = light_y + ray_dir_y * max_dist;

            // Dibujamos los vectores de luz atmosféricos
            draw_line(light_x, light_y, end_x, end_y, 1.2, Color::new(1.0, 0.9, 0.5, 0.08));
        }

        // 3. Renderizamos los bichos inteligentes reaccionando al haz lumínico
        swarm.render_swarm(light_x, light_y);

        // 4. Dibujamos las estructuras de piedra física
        for wall in &walls {
            draw_rectangle(wall.x, wall.y, wall.w, wall.h, Color::new(0.08, 0.08, 0.12, 1.0));
            draw_rectangle_lines(wall.x, wall.y, wall.w, wall.h, 2.0, CYAN); // Bordes neón Axiom
        }

        // El núcleo de luz
        draw_circle(light_x, light_y, 5.0, WHITE);

        // HUD de Telemetría Axiom Systems
        draw_rectangle(10.0, 10.0, 320.0, 80.0, Color::new(0.0, 0.0, 0.0, 0.85));
        draw_text(&format!("FPS: {}", get_fps()), 20.0, 30.0, 20.0, GREEN);
        draw_text("FUSION CORE: SWARM IA + RAYCAST LIGHTS", 20.0, 50.0, 13.0, MAGENTA);
        draw_text(&format!("ACTIVE AGENTS IN MOTION: {}", swarm.boids.len()), 20.0, 70.0, 14.0, CYAN);

        next_frame().await
    }
}
