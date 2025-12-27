use eframe::egui;
use std::time::Instant;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 600.0])
            .with_title("Physics Puzzle Game"),
        ..Default::default()
    };

    eframe::run_native(
        "Physics Puzzle Game",
        options,
        Box::new(|_cc| Ok(Box::new(PhysicsApp::default()))),
    )
}

#[derive(Clone)]
struct PhysicsObject {
    pos: Vec2,
    vel: Vec2,
    acc: Vec2,
    radius: f32,
    mass: f32,
    color: egui::Color32,
    bounciness: f32,
    is_goal: bool,
    is_player: bool,
    fixed: bool,
    initial_pos: Vec2,
    initial_vel: Vec2,
}

#[derive(Clone, Copy)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn normalized(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self { x: self.x / len, y: self.y / len }
        } else {
            *self
        }
    }

    fn dot(&self, other: &Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;
    fn add(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x - other.x, self.y - other.y)
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, scalar: f32) -> Vec2 {
        Vec2::new(self.x * scalar, self.y * scalar)
    }
}

#[derive(Clone)]
struct Wall {
    start: Vec2,
    end: Vec2,
    is_user_placed: bool,
}

struct Spring {
    object_index: usize,
    anchor: Option<usize>,
    anchor_pos: Vec2,
    rest_length: f32,
    stiffness: f32,
}

enum GameState {
    Planning,
    Simulating,
    Won,
}

struct PhysicsApp {
    objects: Vec<PhysicsObject>,
    walls: Vec<Wall>,
    springs: Vec<Spring>,
    gravity: Vec2,
    last_time: Instant,
    bounds: (f32, f32),
    level: u32,
    game_state: GameState,
    placing_wall: Option<Vec2>,
    max_walls: usize,
    win_time: Option<Instant>,
}

impl Default for PhysicsApp {
    fn default() -> Self {
        let mut app = Self {
            objects: Vec::new(),
            walls: Vec::new(),
            springs: Vec::new(),
            gravity: Vec2::new(0.0, 400.0),
            last_time: Instant::now(),
            bounds: (800.0, 600.0),
            level: 2,
            game_state: GameState::Planning,
            placing_wall: None,
            max_walls: 3,
            win_time: None,
        };
        app.setup_level(1);
        app
    }
}

impl PhysicsApp {
    fn setup_level(&mut self, level: u32) {
        self.objects.clear();
        self.walls.clear();
        self.springs.clear();
        self.game_state = GameState::Planning;
        self.placing_wall = None;
        self.win_time = None;

        match level {
            1 => self.setup_level_1(),
            2 => self.setup_level_2(),
            3 => self.setup_level_3(),
            _ => {}
        }
    }

    fn setup_level_1(&mut self) {
        // Level 1: Simple ricochet - ball launches and needs to bounce off walls to hit goal
        self.max_walls = 2;
        
        // Player ball (launches from left)
        self.objects.push(PhysicsObject {
            pos: Vec2::new(100.0, 300.0),
            vel: Vec2::new(400.0, -100.0),
            acc: Vec2::new(0.0, 0.0),
            radius: 20.0,
            mass: 1.0,
            color: egui::Color32::from_rgb(100, 150, 255),
            bounciness: 0.9,
            is_goal: false,
            is_player: true,
            fixed: false,
            initial_pos: Vec2::new(100.0, 300.0),
            initial_vel: Vec2::new(400.0, -100.0),
        });

        // Intermediate ball
        self.objects.push(PhysicsObject {
            pos: Vec2::new(500.0, 200.0),
            vel: Vec2::new(0.0, 0.0),
            acc: Vec2::new(0.0, 0.0),
            radius: 25.0,
            mass: 1.2,
            color: egui::Color32::from_rgb(255, 200, 100),
            bounciness: 0.85,
            is_goal: false,
            is_player: false,
            fixed: false,
            initial_pos: Vec2::new(500.0, 200.0),
            initial_vel: Vec2::new(0.0, 0.0),
        });

        // Goal ball
        self.objects.push(PhysicsObject {
            pos: Vec2::new(700.0, 450.0),
            vel: Vec2::new(0.0, 0.0),
            acc: Vec2::new(0.0, 0.0),
            radius: 30.0,
            mass: 1.5,
            color: egui::Color32::from_rgb(100, 255, 100),
            bounciness: 0.8,
            is_goal: true,
            is_player: false,
            fixed: false,
            initial_pos: Vec2::new(700.0, 450.0),
            initial_vel: Vec2::new(0.0, 0.0),
        });

        // Fixed walls (boundaries)
        self.walls.push(Wall {
            start: Vec2::new(400.0, 400.0),
            end: Vec2::new(600.0, 400.0),
            is_user_placed: false,
        });
    }

    fn setup_level_2(&mut self) {
        // Level 2: Pendulum redirect
        self.max_walls = 3;
        
        // Player ball
        self.objects.push(PhysicsObject {
            pos: Vec2::new(100.0, 510.0),
            vel: Vec2::new(350.0, -200.0),
            acc: Vec2::new(0.0, 0.0),
            radius: 20.0,
            mass: 1.0,
            color: egui::Color32::from_rgb(100, 150, 255),
            bounciness: 0.9,
            is_goal: false,
            is_player: true,
            fixed: false,
            initial_pos: Vec2::new(100.0, 500.0),
            initial_vel: Vec2::new(350.0, -200.0),
        });

        // Pendulum balls
        for i in 0..2 {
            let x = 350.0 + i as f32 * 200.0;
            self.objects.push(PhysicsObject {
                pos: Vec2::new(x, 300.0),
                vel: Vec2::new(0.0, 0.0),
                acc: Vec2::new(0.0, 0.0),
                radius: 25.0,
                mass: 1.5,
                color: egui::Color32::from_rgb(255, 150, 100),
                bounciness: 0.9,
                is_goal: false,
                is_player: false,
                fixed: false,
                initial_pos: Vec2::new(x, 300.0),
                initial_vel: Vec2::new(0.0, 0.0),
            });

            self.springs.push(Spring {
                object_index: i + 1,
                anchor: None,
                anchor_pos: Vec2::new(x, 100.0),
                rest_length: 200.0,
                stiffness: 80.0,
            });
        }

        // Goal ball
        self.objects.push(PhysicsObject {
            pos: Vec2::new(700.0, 500.0),
            vel: Vec2::new(0.0, 0.0),
            acc: Vec2::new(0.0, 0.0),
            radius: 30.0,
            mass: 1.5,
            color: egui::Color32::from_rgb(100, 255, 100),
            bounciness: 0.8,
            is_goal: true,
            is_player: false,
            fixed: false,
            initial_pos: Vec2::new(700.0, 500.0),
            initial_vel: Vec2::new(0.0, 0.0),
        });

        // Fixed platform
        self.walls.push(Wall {
            start: Vec2::new(250.0, 450.0),
            end: Vec2::new(400.0, 400.0),
            is_user_placed: false,
        });
    }

    fn setup_level_3(&mut self) {
        // Level 3:  cool chainy one
        self.max_walls = 4;
        
        // Player ball
        self.objects.push(PhysicsObject {
            pos: Vec2::new(100.0, 100.0),
            vel: Vec2::new(300.0, 200.0),
            acc: Vec2::new(0.0, 0.0),
            radius: 20.0,
            mass: 1.0,
            color: egui::Color32::from_rgb(100, 150, 255),
            bounciness: 0.95,
            is_goal: false,
            is_player: true,
            fixed: false,
            initial_pos: Vec2::new(100.0, 100.0),
            initial_vel: Vec2::new(300.0, 200.0),
        });

        // Multiple intermediate balls
        self.objects.push(PhysicsObject {
            pos: Vec2::new(300.0, 350.0),
            vel: Vec2::new(0.0, 0.0),
            acc: Vec2::new(0.0, 0.0),
            radius: 30.0,
            mass: 2.0,
            color: egui::Color32::from_rgb(255, 100, 100),
            bounciness: 0.95,
            is_goal: false,
            is_player: false,
            fixed: false,
            initial_pos: Vec2::new(300.0, 350.0),
            initial_vel: Vec2::new(0.0, 0.0),
        });

        // Pendulum
        self.objects.push(PhysicsObject {
            pos: Vec2::new(500.0, 350.0),
            vel: Vec2::new(0.0, 0.0),
            acc: Vec2::new(0.0, 0.0),
            radius: 25.0,
            mass: 1.5,
            color: egui::Color32::from_rgb(255, 200, 100),
            bounciness: 0.9,
            is_goal: false,
            is_player: false,
            fixed: false,
            initial_pos: Vec2::new(500.0, 350.0),
            initial_vel: Vec2::new(0.0, 0.0),
        });

        self.springs.push(Spring {
            object_index: 2,
            anchor: None,
            anchor_pos: Vec2::new(500.0, 150.0),
            rest_length: 200.0,
            stiffness: 70.0,
        });

        // Another ball
        self.objects.push(PhysicsObject {
            pos: Vec2::new(650.0, 250.0),
            vel: Vec2::new(0.0, 0.0),
            acc: Vec2::new(0.0, 0.0),
            radius: 25.0,
            mass: 1.3,
            color: egui::Color32::from_rgb(200, 100, 255),
            bounciness: 0.9,
            is_goal: false,
            is_player: false,
            fixed: false,
            initial_pos: Vec2::new(650.0, 250.0),
            initial_vel: Vec2::new(0.0, 0.0),
        });

        // Goal ball
        self.objects.push(PhysicsObject {
            pos: Vec2::new(700.0, 500.0),
            vel: Vec2::new(0.0, 0.0),
            acc: Vec2::new(0.0, 0.0),
            radius: 30.0,
            mass: 1.5,
            color: egui::Color32::from_rgb(100, 255, 100),
            bounciness: 0.8,
            is_goal: true,
            is_player: false,
            fixed: false,
            initial_pos: Vec2::new(700.0, 500.0),
            initial_vel: Vec2::new(0.0, 0.0),
        });

        // Fixed obstacles
        self.walls.push(Wall {
            start: Vec2::new(200.0, 250.0),
            end: Vec2::new(350.0, 200.0),
            is_user_placed: false,
        });
        self.walls.push(Wall {
            start: Vec2::new(400.0, 500.0),
            end: Vec2::new(550.0, 480.0),
            is_user_placed: false,
        });
    }

    fn count_user_walls(&self) -> usize {
        self.walls.iter().filter(|w| w.is_user_placed).count()
    }

    fn reset_simulation(&mut self) {
        for obj in &mut self.objects {
            obj.pos = obj.initial_pos;
            obj.vel = obj.initial_vel;
            obj.acc = Vec2::new(0.0, 0.0);
        }
        self.game_state = GameState::Planning;
        self.win_time = None;
    }

    fn update_physics(&mut self, dt: f32) {
        if !matches!(self.game_state, GameState::Simulating) {
            return;
        }

        // Apply spring forces
        let spring_forces: Vec<(usize, Vec2)> = self.springs.iter().filter_map(|spring| {
            let obj = self.objects.get(spring.object_index)?;
            
            let anchor_pos = if let Some(anchor_idx) = spring.anchor {
                self.objects.get(anchor_idx)?.pos
            } else {
                spring.anchor_pos
            };

            let to_anchor = anchor_pos - obj.pos;
            let distance = to_anchor.length();
            if distance == 0.0 { return None; }

            let direction = to_anchor * (1.0 / distance);
            let stretch = distance - spring.rest_length;
            let spring_force = direction * (stretch * spring.stiffness);

            Some((spring.object_index, spring_force))
        }).collect();

        for (idx, force) in spring_forces {
            if let Some(obj) = self.objects.get_mut(idx) {
                if !obj.fixed {
                    obj.acc = obj.acc + force * (1.0 / obj.mass);
                }
            }
        }

        // Update physics for all objects
        for obj in &mut self.objects {
            if !obj.fixed {
                obj.acc = obj.acc + self.gravity;
                obj.vel = obj.vel + obj.acc * dt;
                obj.acc = Vec2::new(0.0, 0.0);
                obj.pos = obj.pos + obj.vel * dt;
            }
        }

        // Boundary collisions
        for obj in &mut self.objects {
            if obj.fixed { continue; }
            
            if obj.pos.x - obj.radius < 0.0 {
                obj.pos.x = obj.radius;
                obj.vel.x = -obj.vel.x * obj.bounciness;
            } else if obj.pos.x + obj.radius > self.bounds.0 {
                obj.pos.x = self.bounds.0 - obj.radius;
                obj.vel.x = -obj.vel.x * obj.bounciness;
            }

            if obj.pos.y - obj.radius < 0.0 {
                obj.pos.y = obj.radius;
                obj.vel.y = -obj.vel.y * obj.bounciness;
            } else if obj.pos.y + obj.radius > self.bounds.1 {
                obj.pos.y = self.bounds.1 - obj.radius;
                obj.vel.y = -obj.vel.y * obj.bounciness;
            }
        }

        // Object-to-object collisions
        let len = self.objects.len();
        for i in 0..len {
            for j in (i + 1)..len {
                let (obj1, obj2) = {
                    let (left, right) = self.objects.split_at_mut(j);
                    (&mut left[i], &mut right[0])
                };

                let delta_pos = obj2.pos - obj1.pos;
                let dist = delta_pos.length();
                let min_dist = obj1.radius + obj2.radius;

                if dist < min_dist {
                    // Check for goal hit
                    if (obj1.is_goal && !obj2.is_player) || (obj2.is_goal && !obj1.is_player) {
                        if !matches!(self.game_state, GameState::Won) {
                            self.game_state = GameState::Won;
                            self.win_time = Some(Instant::now());
                        }
                    }

                    let normal = delta_pos.normalized();
                    let overlap = min_dist - dist;
                    let separation = normal * (overlap / 2.0);
                    let total_mass = obj1.mass + obj2.mass;
                    
                    if !obj1.fixed {
                        obj1.pos = obj1.pos - separation * (obj2.mass / total_mass);
                    }
                    if !obj2.fixed {
                        obj2.pos = obj2.pos + separation * (obj1.mass / total_mass);
                    }

                    let rel_vel = obj2.vel - obj1.vel;
                    let vel_along_normal = rel_vel.dot(&normal);

                    let least_bounciness = obj1.bounciness.min(obj2.bounciness);
                    let mut impulse_mag = -(1.0 + least_bounciness) * vel_along_normal;
                    impulse_mag = impulse_mag / (1.0 / obj1.mass + 1.0 / obj2.mass);

                    if !obj1.fixed {
                        obj1.vel = obj1.vel - (normal * impulse_mag) * (1.0 / obj1.mass);
                    }
                    if !obj2.fixed {
                        obj2.vel = obj2.vel + (normal * impulse_mag) * (1.0 / obj2.mass);
                    }
                }
            }
        }

        // Wall collisions
        for obj in &mut self.objects {
            if obj.fixed { continue; }
            
            for wall in &self.walls {
                let wall_vec = wall.end - wall.start;
                let wall_len = wall_vec.length();
                let wall_dir = wall_vec * (1.0 / wall_len);
                
                let to_ball = obj.pos - wall.start;
                let along_wall = to_ball.dot(&wall_dir);
                
                if along_wall >= 0.0 && along_wall <= wall_len {
                    let normal = Vec2::new(-wall_dir.y, wall_dir.x);
                    let dist = to_ball.dot(&normal);
                    
                    if dist.abs() <= obj.radius {
                        let penetration = obj.radius - dist.abs();
                        obj.pos = obj.pos + normal * (penetration * dist.signum());
                        
                        let vel_normal = obj.vel.dot(&normal);
                        if vel_normal * dist < 0.0 {
                            obj.vel = obj.vel - normal * (vel_normal * (1.0 + obj.bounciness));
                        }
                    }
                }
            }
        }
    }

    fn render(&self, ui: &mut egui::Ui) {
        let painter = ui.painter();
        
        // Draw springs
        for spring in &self.springs {
            if let Some(obj) = self.objects.get(spring.object_index) {
                let anchor_pos = if let Some(anchor_idx) = spring.anchor {
                    if let Some(anchor_obj) = self.objects.get(anchor_idx) {
                        egui::pos2(anchor_obj.pos.x, anchor_obj.pos.y)
                    } else {
                        continue;
                    }
                } else {
                    egui::pos2(spring.anchor_pos.x, spring.anchor_pos.y)
                };

                let obj_pos = egui::pos2(obj.pos.x, obj.pos.y);
                let dist = ((obj_pos.x - anchor_pos.x).powi(2) + 
                           (obj_pos.y - anchor_pos.y).powi(2)).sqrt();
                let segments = (dist / 10.0).max(4.0) as i32;
                let dx = (obj_pos.x - anchor_pos.x) / segments as f32;
                let dy = (obj_pos.y - anchor_pos.y) / segments as f32;
                
                let mut points = Vec::new();
                for i in 0..=segments {
                    let x = anchor_pos.x + dx * i as f32;
                    let y = anchor_pos.y + dy * i as f32;
                    let offset = if i % 2 == 0 { 5.0 } else { -5.0 };
                    let normal_x = -dy / dist * offset;
                    let normal_y = dx / dist * offset;
                    points.push(egui::pos2(x + normal_x, y + normal_y));
                }
                
                for i in 0..points.len()-1 {
                    painter.line_segment(
                        [points[i], points[i+1]],
                        egui::Stroke::new(2.0, egui::Color32::DARK_GRAY),
                    );
                }
            }
        }

        // Draw walls
        for wall in &self.walls {
            let color = if wall.is_user_placed {
                egui::Color32::from_rgb(100, 200, 255)
            } else {
                egui::Color32::WHITE
            };
            
            painter.line_segment(
                [egui::pos2(wall.start.x, wall.start.y), egui::pos2(wall.end.x, wall.end.y)],
                egui::Stroke::new(6.0, color),
            );
        }

        // Draw wall preview
        if let Some(start) = self.placing_wall {
            if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
                painter.line_segment(
                    [egui::pos2(start.x, start.y), pointer_pos],
                    egui::Stroke::new(6.0, egui::Color32::from_rgba_premultiplied(100, 200, 255, 150)),
                );
            }
        }
        
        // Draw objects
        for obj in &self.objects {
            let mut color = obj.color;
            if obj.is_goal && matches!(self.game_state, GameState::Won) {
                color = egui::Color32::from_rgb(255, 255, 100);
            }
            
            painter.circle_filled(
                egui::pos2(obj.pos.x, obj.pos.y),
                obj.radius,
                color,
            );
            
            // Draw outline for player ball
            if obj.is_player {
                painter.circle_stroke(
                    egui::pos2(obj.pos.x, obj.pos.y),
                    obj.radius,
                    egui::Stroke::new(3.0, egui::Color32::WHITE),
                );
            }
            
            // Draw star for goal
            if obj.is_goal {
                let star_size = 15.0;
                for i in 0..5 {
                    let angle1 = std::f32::consts::PI * 2.0 * i as f32 / 5.0 - std::f32::consts::PI / 2.0;
                    let angle2 = std::f32::consts::PI * 2.0 * (i as f32 + 0.5) / 5.0 - std::f32::consts::PI / 2.0;
                    
                    let p1 = egui::pos2(
                        obj.pos.x + angle1.cos() * star_size,
                        obj.pos.y + angle1.sin() * star_size
                    );
                    let p2 = egui::pos2(
                        obj.pos.x + angle2.cos() * star_size * 0.5,
                        obj.pos.y + angle2.sin() * star_size * 0.5
                    );
                    
                    painter.line_segment([p1, p2], egui::Stroke::new(2.0, egui::Color32::WHITE));
                }
            }
        }

        // Draw velocity arrow for player ball in planning mode
        if matches!(self.game_state, GameState::Planning) {
            if let Some(player) = self.objects.iter().find(|o| o.is_player) {
                let arrow_scale = 0.15;
                let end_pos = egui::pos2(
                    player.pos.x + player.vel.x * arrow_scale,
                    player.pos.y + player.vel.y * arrow_scale
                );
                
                painter.arrow(
                    egui::pos2(player.pos.x, player.pos.y),
                    end_pos.to_vec2() - egui::pos2(player.pos.x, player.pos.y).to_vec2(),
                    egui::Stroke::new(3.0, egui::Color32::YELLOW),
                );
            }
        }
    }
}

impl eframe::App for PhysicsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        let dt = (now - self.last_time).as_secs_f32().min(0.016);
        self.last_time = now;

        // Check for level progression
        if let Some(win_time) = self.win_time {
            if now.duration_since(win_time).as_secs_f32() > 2.0 {
                if self.level < 3 {
                    self.level += 1;
                    self.setup_level(self.level);
                }
            }
        }

        // Side panel
        egui::SidePanel::left("control_panel")
            .resizable(false)
            .exact_width(200.0)
            .show(ctx, |ui| {
                ui.heading(format!("Level {}", self.level));
                ui.separator();
                
                ui.label(format!("Walls: {}/{}", self.count_user_walls(), self.max_walls));
                ui.add_space(10.0);
                
                match self.game_state {
                    GameState::Planning => {
                        ui.label("📐 Planning Phase");
                        ui.add_space(5.0);
                        ui.label("Click and drag to place walls");
                        ui.add_space(10.0);
                        
                        if ui.button("🚀 Launch Ball").clicked() {
                            self.game_state = GameState::Simulating;
                        }
                        
                        ui.add_space(10.0);
                        if ui.button("🗑 Clear User Walls").clicked() {
                            self.walls.retain(|w| !w.is_user_placed);
                        }
                    }
                    GameState::Simulating => {
                        ui.label("⚙ Simulating...");
                        ui.add_space(10.0);
                        
                        if ui.button("🔄 Reset & Retry").clicked() {
                            self.reset_simulation();
                        }
                    }
                    GameState::Won => {
                        ui.label("🎉 Level Complete!");
                        ui.add_space(10.0);
                        
                        if self.level < 3 {
                            ui.label("Loading next level...");
                        } else {
                            ui.label("All levels complete!");
                            if ui.button("Play Again").clicked() {
                                self.level = 1;
                                self.setup_level(1);
                            }
                        }
                    }
                }
                
                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);
                
                if ui.button("Restart Level").clicked() {
                    self.setup_level(self.level);
                }
                
                ui.add_space(20.0);
                ui.separator();
                ui.heading("Goal");
                ui.label("Hit the green goal ball with any other ball!");
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let rect = ui.available_rect_before_wrap();
                self.bounds = (rect.width(), rect.height());

                // Handle mouse input for wall placement
                if matches!(self.game_state, GameState::Planning) {
                    if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                        let mouse_pos = Vec2::new(pos.x, pos.y);

                        if ui.input(|i| i.pointer.primary_pressed()) {
                            if self.count_user_walls() < self.max_walls {
                                self.placing_wall = Some(mouse_pos);
                            }
                        }

                        if let Some(start) = self.placing_wall {
                            if ui.input(|i| i.pointer.primary_released()) {
                                // Only add wall if it's long enough
                                let length = (mouse_pos - start).length();
                                if length > 20.0 {
                                    self.walls.push(Wall {
                                        start,
                                        end: mouse_pos,
                                        is_user_placed: true,
                                    });
                                }
                                self.placing_wall = None;
                            }
                        }
                    }
                }
                
                self.update_physics(dt);
                self.render(ui);
            });
        });

        ctx.request_repaint();
    }
}