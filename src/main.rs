use eframe::egui;
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

struct Wall {
    start: Vec2,
    end: Vec2,
}

struct PhysicsApp {
    objects: Vec<PhysicsObject>,
    walls: Vec<Wall>,
    gravity: Vec2,
    last_time: Instant,
    bounds: (f32, f32),
    paused: bool,
    dragged_object: Option<usize>,
    pull_start: Option<Vec2>,
    pull_object: Option<usize>,
    placing_wall: Option<Vec2>,
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
            walls: Vec::new(),
            gravity: Vec2::new(0.0, 550.0),
            last_time: Instant::now(),
            bounds: (800.0, 600.0),
            paused: false,
            dragged_object: None,
            pull_start: None,
            pull_object: None,
            placing_wall: None,
        }
    }
}

impl PhysicsApp {
    fn get_object_at_pos(&self, pos: Vec2) -> Option<usize> {
        self.objects.iter().enumerate().find(|(_, obj)| {
            let delta = obj.pos - pos;
            delta.length() <= obj.radius
        }).map(|(i, _)| i)
    }


    fn update_physics(&mut self, dt: f32, mouse_pos: Option<Vec2>) {
        if self.paused {
            return;
        }

        // Handle dragging with physics
        if let Some(idx) = self.dragged_object {
            if let Some(mouse_pos) = mouse_pos {
                if let Some(obj) = self.objects.get_mut(idx) {
                    let spring_strength = 15.0; // Adjust this to change how strongly the object follows the mouse
                    let damping = 0.8; // Adjust this to change how quickly the object slows down
                    
                    // Apply spring force towards mouse
                    let to_mouse = mouse_pos - obj.pos;
                    obj.acc = obj.acc + to_mouse * spring_strength;
                    
                    // Apply damping to prevent excessive oscillation
                    obj.vel = obj.vel * damping;
                }
            }
        }

        // Update physics for all objects
        for obj in &mut self.objects {
            obj.acc = obj.acc + self.gravity;
            obj.vel = obj.vel + obj.acc * dt;
            obj.acc = Vec2::new(0.0, 0.0);
            obj.pos = obj.pos + obj.vel * dt;
        }
        // colliding with walls 
        for obj in &mut self.objects {
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

                if dist < min_dist && dist > 0.0 {
                    let normal = delta_pos.normalized();
                    
                    // Separate objects
                    let overlap = min_dist - dist;
                    let separation = normal * (overlap / 2.0);
                    obj1.pos = obj1.pos - separation;
                    obj2.pos = obj2.pos + separation;

                    // Calculate relative velocity
                    let rel_vel = obj2.vel - obj1.vel;
                    let vel_along_normal = rel_vel.dot(&normal);

                    if vel_along_normal < 0.0 {
                        continue;
                    }

                    let e = obj1.bounciness.min(obj2.bounciness);

                    let j = -(1.0 + e) * vel_along_normal;
                    let j = j / (1.0 / obj1.mass + 1.0 / obj2.mass);

                    let impulse = normal * j;
                    obj1.vel = obj1.vel - impulse * (1.0 / obj1.mass);
                    obj2.vel = obj2.vel + impulse * (1.0 / obj2.mass);
                }
            }
        }
    }

    fn render(&self, ui: &mut egui::Ui) {
        let painter = ui.painter();
        
        // Draw walls
        for wall in &self.walls {
            painter.line_segment(
                [egui::pos2(wall.start.x, wall.start.y), egui::pos2(wall.end.x, wall.end.y)],
                egui::Stroke::new(4.0, egui::Color32::WHITE),
            );
        }

        // Draw pull line
        if let Some(start) = self.pull_start {
            if let Some(idx) = self.pull_object {
                if let Some(obj) = self.objects.get(idx) {
                    painter.line_segment(
                        [egui::pos2(start.x, start.y), egui::pos2(obj.pos.x, obj.pos.y)],
                        egui::Stroke::new(2.0, egui::Color32::YELLOW),
                    );
                }
            }
        }

        // Draw placing wall preview
        if let Some(start) = self.placing_wall {
            if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
                painter.line_segment(
                    [egui::pos2(start.x, start.y), pointer_pos],
                    egui::Stroke::new(4.0, egui::Color32::from_rgba_premultiplied(255, 255, 255, 100)),
                );
            }
        }
        
        // Draw objects
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
        let now = Instant::now();
        let dt = (now - self.last_time).as_secs_f32().min(0.016); 
        self.last_time = now;

        // Side panel with controls
        egui::SidePanel::right("controls_panel")
            .resizable(true)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Physics Controls");
                ui.add_space(10.0);

                if ui.button(if self.paused { "▶ Resume" } else { "⏸ Pause" }).clicked() {
                    self.paused = !self.paused;
                }

                if ui.button("New Experiment").clicked() {
                    *self = Self::default();
                }

                ui.add_space(20.0);
                ui.label("Gravity Settings:");
                ui.horizontal(|ui| {
                    ui.label("X:");
                    if ui.add(egui::DragValue::new(&mut self.gravity.x).speed(10.0)).changed() {
                        // Gravity X changed
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Y:");
                    if ui.add(egui::DragValue::new(&mut self.gravity.y).speed(10.0)).changed() {
                        // Gravity Y changed
                    }
                });

                ui.add_space(20.0);
                if ui.button("Add Random Object").clicked() {
                    use rand::Rng;
                    let mut rng = rand::rng();
                    let bounds_margin = 50.0;
                    self.objects.push(PhysicsObject {
                        pos: Vec2::new(
                            rng.random_range(bounds_margin..self.bounds.0 - bounds_margin),
                            rng.random_range(bounds_margin..self.bounds.1 - bounds_margin),
                        ),
                        vel: Vec2::new(
                            rng.random_range(-200.0..200.0),
                            rng.random_range(-100.0..100.0),
                        ),
                        acc: Vec2 { x: 0.0, y: 0.0 },
                        radius: rng.random_range(20.0..40.0),
                        mass: rng.random_range(0.5..1.5),
                        color: egui::Color32::from_rgb(
                            rng.random_range(100..255),
                            rng.random_range(100..255),
                            rng.random_range(100..255),
                        ),
                        bounciness: rng.random_range(0.5..0.9),
                    });
                }

                ui.add_space(10.0);
                ui.separator();
                ui.label(format!("Objects: {}", self.objects.len()));

                // Object List
                ui.add_space(20.0);
                ui.separator();
                ui.heading("Wall Controls");
                ui.label("Press 'W' and click-drag to create walls");
                if ui.button("Clear All Walls").clicked() {
                    self.walls.clear();
                }
                
                ui.add_space(20.0);
                ui.separator();
                ui.heading("Objects");
                for (i, obj) in self.objects.iter_mut().enumerate() {
                    ui.collapsing(format!("Object {}", i + 1), |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Mass:");
                            ui.add(egui::DragValue::new(&mut obj.mass).speed(0.1));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Radius:");
                            ui.add(egui::DragValue::new(&mut obj.radius).speed(1.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Bounciness:");
                            ui.add(egui::Slider::new(&mut obj.bounciness, 0.0..=1.0));
                        });
                    });
                }
            });

        // Main panel with the physics simulation
        egui::CentralPanel::default().show(ctx, |ui| {
            // Create a frame for the physics simulation
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let rect = ui.available_rect_before_wrap();
                self.bounds = (rect.width(), rect.height());
                
                // Handle mouse interactions
                if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                    let mouse_pos = Vec2::new(pos.x, pos.y);

                    // Handle wall placement
                    if ui.input(|i| i.key_pressed(egui::Key::W)) {
                        self.placing_wall = Some(mouse_pos);
                    }

                    if let Some(start) = self.placing_wall {
                        if ui.input(|i| i.pointer.primary_released()) {
                            self.walls.push(Wall {
                                start,
                                end: mouse_pos,
                            });
                            self.placing_wall = None;
                        }
                    }

                    // Handle object dragging and pulling
                    if ui.input(|i| i.pointer.primary_pressed()) {
                        if self.placing_wall.is_none() {
                            if let Some(idx) = self.get_object_at_pos(mouse_pos) {
                                if ui.input(|i| i.modifiers.shift) {
                                    // Pull mode
                                    self.pull_start = Some(mouse_pos);
                                    self.pull_object = Some(idx);
                                } else {
                                    // Drag mode
                                    self.dragged_object = Some(idx);
                                }
                            }
                        }
                    }

                    // Update dragged object position
                    if let Some(idx) = self.dragged_object {
                        if let Some(obj) = self.objects.get_mut(idx) {
                            obj.pos = mouse_pos;
                        }
                    }

                    // Handle object launching
                    if ui.input(|i| i.pointer.primary_released()) {
                        if let Some(start) = self.pull_start {
                            if let Some(idx) = self.pull_object {
                                if let Some(obj) = self.objects.get_mut(idx) {
                                    let pull_vector = start - mouse_pos;
                                    obj.vel = pull_vector * 2.0; // Adjust this multiplier to change launch strength
                                }
                            }
                            self.pull_start = None;
                            self.pull_object = None;
                        }
                        self.dragged_object = None;
                    }
                }
                
                let mouse_pos = ui.input(|i| i.pointer.hover_pos())
                    .map(|pos| Vec2::new(pos.x, pos.y));
                
                self.update_physics(dt, mouse_pos);

                // Update wall collisions
                let walls = &self.walls;
                for obj in &mut self.objects {
                    for wall in walls {
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

                self.render(ui);
            });
        });

        ctx.request_repaint();
    }
        
    }