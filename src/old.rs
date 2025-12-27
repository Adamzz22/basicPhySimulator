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

struct Spring {
    object_index: usize,
    anchor: Option<usize>, //  for connecting two objects
    anchor_pos: Vec2,      
    rest_length: f32,
    stiffness: f32,
}

struct PhysicsApp {
    objects: Vec<PhysicsObject>,
    walls: Vec<Wall>,
    springs: Vec<Spring>,
    gravity: Vec2,
    last_time: Instant,
    bounds: (f32, f32),
    paused: bool,
    dragged_object: Option<usize>,
    pull_start: Option<Vec2>,
    pull_object: Option<usize>,
    placing_wall: Option<Vec2>,
    level: u32
}

impl Default for PhysicsApp {
    fn default() -> Self {

        let mut springs= Vec::new();
        /*springs.push(Spring {
            object_index: 0,
            anchor: None,
            anchor_pos: Vec2::new(400.0, 50.0),
            rest_length: 100.0,
            stiffness: 50.0,
        });
        */ 
        
        Self {
            objects: Vec::new(),
            walls: Vec::new(),
            springs: springs,
            gravity: Vec2::new(0.0, 550.0),
            last_time: Instant::now(),
            bounds: (800.0, 600.0),
            paused: false,
            dragged_object: None,
            pull_start: None,
            pull_object: None,
            placing_wall: None,
            level: 1
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

        let spring_forces: Vec<(usize, Vec2)> = self.springs.iter().filter_map(|spring| {
            let obj = self.objects.get(spring.object_index)?;
            
            // Get anchor position (either from fixed point or another object)
            let anchor_pos = if let Some(anchor_idx) = spring.anchor {
                self.objects.get(anchor_idx)?.pos
            } else {
                spring.anchor_pos
            };

            // Calculate spring force
            let to_anchor = anchor_pos - obj.pos;
            let distance = to_anchor.length();
            if distance == 0.0 { return None; }

            let direction = to_anchor * (1.0 / distance);
            
            // Spring force (F = -kx)
            let stretch = distance - spring.rest_length;
            let spring_force = direction * (stretch * spring.stiffness);

            // Damping force (F = -bv)
            // let relative_vel = if let Some(anchor_idx) = spring.anchor {
            //     obj.vel - self.objects.get(anchor_idx)?.vel
            // } else {
            //     obj.vel
            // };
            // let damping_force = relative_vel * -spring.damping;

            Some((spring.object_index, spring_force))
        }).collect();

        // Apply spring forces
        for (idx, force) in spring_forces {
            if let Some(obj) = self.objects.get_mut(idx) {
                obj.acc = obj.acc + force * (1.0 / obj.mass);
            }
        }

        // object dragging 
        if let Some(idx) = self.dragged_object {
            if let Some(mouse_pos) = mouse_pos {
                if let Some(obj) = self.objects.get_mut(idx) {
                    let spring_strength = 15.0; 
                    let damping = 0.89;
                    
                    let vec_to_mouse = mouse_pos - obj.pos;
                    obj.acc = obj.acc + vec_to_mouse * spring_strength;
                    
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

        // object-to-object collision detection
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
                    let normal = delta_pos.normalized();
                    
                    
                    let overlap = min_dist - dist;
                    let separation = normal * (overlap / 2.0);
                    let total_mass = obj1.mass + obj2.mass;
                    obj1.pos = obj1.pos - separation * (obj2.mass / total_mass);
                    obj2.pos = obj2.pos + separation * (obj1.mass / total_mass);

                    
                    let rel_vel = obj2.vel - obj1.vel;
                    let vel_along_normal = rel_vel.dot(&normal);

                    // I calculate collisions as if they are elastic but reduce them slightly so they appear inelastic 
                    let least_bounciness= obj1.bounciness.min(obj2.bounciness); 
                    let mut impulse_mag = -(1.0 + least_bounciness) * vel_along_normal;
                    impulse_mag = impulse_mag / (1.0 / obj1.mass + 1.0 / obj2.mass);

                    obj1.vel = obj1.vel - ( normal * impulse_mag) * (1.0 / obj1.mass);
                    obj2.vel = obj2.vel + (normal * impulse_mag) * (1.0 / obj2.mass);
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

                // Draw spring as a zigzag line
                let obj_pos = egui::pos2(obj.pos.x, obj.pos.y);
                let dist = ((obj_pos.x - anchor_pos.x).powi(2) + 
                           (obj_pos.y - anchor_pos.y).powi(2)).sqrt();
                let segments = (dist / 10.0).max(4.0) as i32;
                let dx = (obj_pos.x - anchor_pos.x) / segments as f32;
                let dy = (obj_pos.y - anchor_pos.y) / segments as f32;
                
                let mut points = Vec::new();
                for i in 0..=segments {
                    let t = i as f32 / segments as f32;
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
                        egui::Stroke::new(2.0, egui::Color32::YELLOW),
                    );
                }
            }
        }

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
        if level < 1{
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
                if ui.button("Add inelastic ball").clicked() {
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

                if ui.button("Add elastic ball").clicked() {
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
                        bounciness: 1.0,
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
        }
        // Main panel with the physics simulation
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let rect = ui.available_rect_before_wrap();
                self.bounds = (rect.width(), rect.height());
                
                // Handle mouse 
                if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                    let mouse_pos = Vec2::new(pos.x, pos.y);

                    // wall placement
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

                    // Handle object releasing
                    if ui.input(|i| i.pointer.primary_released()) {
                        if let Some(start) = self.pull_start {
                            if let Some(idx) = self.pull_object {
                                if let Some(obj) = self.objects.get_mut(idx) {
                                    let pull_vector = start - mouse_pos;
                                    obj.vel = pull_vector * 2.0; // pull strength
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
