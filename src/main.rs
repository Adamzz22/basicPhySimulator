use eframe::{App, egui};
use std::time::Instant;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Physics Engine"),
        ..Default::default()
    };

    eframe::run_native(
        "Physics Engine",
        options,
        Box::new(|_cc| Ok(Box::new(PhysicsApp::default()))),
    )
}

enum AppState{
    Simulation,
    Menu
}


#[derive(Clone)]
struct PhysicsObject {
    pos: Vec2,
    vel: Vec2,
    acc: Vec2,
    radius: f32,
    mass: f32,
    color: egui::Color32,
    bounciness: f32, // Bounciness (0-1)
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

struct PhysicsApp {
    objects: Vec<PhysicsObject>,
    gravity: Vec2,
    last_time: Instant,
    bounds: (f32, f32), // width, height
    paused: bool,
    app_state: AppState,
}

impl Default for PhysicsApp {
    fn default() -> Self {
        let mut objects = Vec::new();
        
        // Add some initial objects
        objects.push(PhysicsObject {
            pos: Vec2::new(200.0, 100.0),
            vel: Vec2::new(150.0, 0.0),
            acc: Vec2 { x: 0.0, y: (0.0) },
            radius: 30.0,
            mass: 1.0,
            color: egui::Color32::from_rgb(255, 100, 100),
            bounciness: 0.6,
        });
        
        objects.push(PhysicsObject {
            pos: Vec2::new(400.0, 200.0),
            vel: Vec2::new(-100.0, 50.0),
            acc: Vec2 { x: 0.0, y: (0.0) },
            radius: 25.0,
            mass: 0.8,
            color: egui::Color32::from_rgb(100, 255, 100),
            bounciness: 0.9,
        });
        
        objects.push(PhysicsObject {
            pos: Vec2::new(600.0, 150.0),
            vel: Vec2::new(-80.0, -30.0),
            acc: Vec2 { x: 0.0, y: (0.0) },
            radius: 35.0,
            mass: 1.2,
            color: egui::Color32::from_rgb(100, 100, 255),
            bounciness: 0.7,
        });

        Self {
            objects,
            gravity: Vec2::new(0.0, 550.0),
            last_time: Instant::now(),
            bounds: (800.0, 600.0),
            paused: false,
            app_state: AppState::Menu,
        }
    }
}

impl PhysicsApp {
    fn update_physics(&mut self, dt: f32) {
        if self.paused {
            return;
        }

        for obj in &mut self.objects {
            obj.acc = obj.acc + self.gravity;
            obj.vel = obj.vel + obj.acc * dt;
            obj.acc = Vec2::new(0.0, 0.0);
            obj.pos = obj.pos + obj.vel * dt;
        }

        // Handle boundary collisions
        for obj in &mut self.objects {
            // Left/Right walls
            if obj.pos.x - obj.radius < 0.0 {
                obj.pos.x = obj.radius;
                obj.vel.x = -obj.vel.x * obj.bounciness;
            } else if obj.pos.x + obj.radius > self.bounds.0 {
                obj.pos.x = self.bounds.0 - obj.radius;
                obj.vel.x = -obj.vel.x * obj.bounciness;
            }

            // Top/Bottom walls
            if obj.pos.y - obj.radius < 0.0 {
                obj.pos.y = obj.radius;
                obj.vel.y = -obj.vel.y * obj.bounciness;
            } else if obj.pos.y + obj.radius > self.bounds.1 {
                obj.pos.y = self.bounds.1 - obj.radius;
                obj.vel.y = -obj.vel.y * obj.bounciness;
            }
        }

        // Handle object-to-object collisions
        let len = self.objects.len();
        for i in 0..len {
            for j in (i + 1)..len {
                let (obj1, obj2) = {
                    let (left, right) = self.objects.split_at_mut(j);
                    (&mut left[i], &mut right[0])
                };

                let delta = obj2.pos - obj1.pos;
                let dist = delta.length();
                let min_dist = obj1.radius + obj2.radius;

                if dist < min_dist && dist > 0.0 {
                    // Collision detected
                    let normal = delta.normalized();
                    
                    // Separate objects
                    let overlap = min_dist - dist;
                    let separation = normal * (overlap / 2.0);
                    obj1.pos = obj1.pos - separation;
                    obj2.pos = obj2.pos + separation;

                    // Calculate relative velocity
                    let rel_vel = obj2.vel - obj1.vel;
                    let vel_along_normal = rel_vel.dot(&normal);

                    // Don't resolve if objects are moving apart
                    if vel_along_normal < 0.0 {
                        continue;
                    }

                    // Calculate bounciness (use minimum of both objects)
                    let e = obj1.bounciness.min(obj2.bounciness);

                    // Calculate impulse scalar
                    let j = -(1.0 + e) * vel_along_normal;
                    let j = j / (1.0 / obj1.mass + 1.0 / obj2.mass);

                    // Apply impulse
                    let impulse = normal * j;
                    obj1.vel = obj1.vel - impulse * (1.0 / obj1.mass);
                    obj2.vel = obj2.vel + impulse * (1.0 / obj2.mass);
                }
            }
        }
    }

    fn render(&self, ui: &mut egui::Ui) {
        let painter = ui.painter();
        
        for obj in &self.objects {
            painter.circle_filled(
                egui::pos2(obj.pos.x, obj.pos.y),
                obj.radius,
                obj.color,
            );
            
        }
    }
}

impl eframe::App for PhysicsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        match self.app_state{
            AppState::Menu => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Adamzz physics Engine");
                        ui.add_space(20.0);
                        if ui.button("Start Simulation").clicked() {
                            self.app_state = AppState::Simulation;
                        }
                    });
                });
            },
            AppState::Simulation => {
                let now = Instant::now();
        let dt = (now - self.last_time).as_secs_f32().min(0.016); // Cap at 60 FPS
        self.last_time = now;

        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Physics Engine");
                ui.separator();
                
                if ui.button(if self.paused { "▶ Resume" } else { "⏸ Pause" }).clicked() {
                    self.paused = !self.paused;
                }
                
                if ui.button("New Experiment").clicked() {
                    *self = Self::default();
                }
                
                if ui.button("Add Object").clicked() {
                    use rand::Rng;
                    let mut rng = rand::rng();
                    self.objects.push(PhysicsObject {
                        pos: Vec2::new(
                            rng.random_range(50.0..750.0),
                            rng.random_range(50.0..200.0),
                        ),
                        vel: Vec2::new(
                            rng.random_range(-200.0..200.0),
                            rng.random_range(-100.0..100.0),
                        ),
                        acc: Vec2 { x: 0.0, y: (0.0) },
                        radius: rng.random_range(20.0..40.0),
                        mass: rng.random_range(0.5..1.5),
                        color: egui::Color32::from_rgb(
                            rng.random_range(100..255),
                            rng.random_range(100..255),
                            rng.random_range(100..255),
                        ),
                        bounciness: 0.6
                    });
                }
                
                ui.label(format!("Objects: {}", self.objects.len()));
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.available_rect_before_wrap();
            self.bounds = (rect.width(), rect.height());
            
            self.update_physics(dt);
            
            self.render(ui);
        });

        ctx.request_repaint();
        }
        
    }
}
}