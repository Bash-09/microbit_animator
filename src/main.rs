use std::ffi::OsString;
use std::fs::File;
use std::io::{Write, BufReader, BufRead};
use std::ops::RangeInclusive;
use std::path::{PathBuf, Path};
use std::time::Instant;

use egui::{Pos2, Vec2, Rect, Color32, Rounding, DragValue, Separator, Button, Slider};
use egui::epaint::RectShape;
use glium_app::glutin::event::VirtualKeyCode;
use glium_app::{Application, PhysicalSize, WindowBuilder};

extern crate glium_app;

mod frame;
use frame::Frame;
use rfd::FileDialog;



fn main() {
    
    let wb = WindowBuilder::new()
        .with_title("Microbit Animator")
        .with_resizable(true)
        .with_inner_size(PhysicalSize::new(800i32, 600i32));

    let app = Box::new(State::new());
    glium_app::run(app, wb);

}


pub struct State {

    file: Option<PathBuf>,

    frames: Vec<Frame>,
    selected: usize,

    last: Instant,
    running: bool,
    fps: u8,

    last_led: (usize, usize, u8)

}

impl State {
    pub fn new() -> State {
        State {
            
            file: None,

            frames: vec![Frame::new(); 5],
            selected: 0,

            last: Instant::now(),
            running: false,
            fps: 3,

            last_led: (0, 0, 0)

        }
    }


    pub fn load_file(&mut self) {
        let mut rfd = FileDialog::new()
        .add_filter("text", &["txt"]);
        if let Some(pb) = &self.file {
            if let Some(dir) = pb.parent() {
                rfd = rfd.set_directory(dir);
            }
        }


        match rfd.pick_file()  {
            
            Some(pb) => {
                self.file = Some(pb);
                self.read_from_file();
            },
            None => {
                return;
            }
        }
    }

    pub fn save_as(&mut self) {

        let mut rfd = FileDialog::new()
        .add_filter("text", &["txt"])
        .set_file_name("animation.txt");
        if let Some(pb) = &self.file {
            if let Some(dir) = pb.parent() {
                rfd = rfd.set_directory(dir);
            }
        }

        match rfd.save_file()  {
            
            Some(pb) => {

                self.file = Some(pb);
                self.write_to_file();
            },
            None => {
                return;
            }
        }
    }

    pub fn save(&mut self) {

        if self.file.is_none() {
            self.save_as();
            return;
        }

        self.write_to_file();

    }

    fn write_to_file(&self) {
        if self.file.is_none() {
            println!("Can't write to no file!");
        }

        let mut file = File::create(self.file.as_ref().unwrap()).expect("Couldn't create file to write to");

        let mut out = String::new();

        for (i, f) in self.frames.iter().enumerate() {
            out.push_str(&format!("{}", f));
            out.push('\n');
            
            if i == self.frames.len()-1 {
                out.push_str(".byte 0");
            } else {
                out.push_str(".byte 1");
            }

            out.push('\n');
        }

        file.write(out.as_bytes()).expect("Failed to write to output file");
    }

    fn read_from_file(&mut self) {
        if self.file.is_none() {
            println!("Can't read from no file!");
        }

        let file = File::open(self.file.as_ref().unwrap()).expect("Couldn't open file");
        let br = BufReader::new(file);

        let mut frames: Vec<Frame> = Vec::new();

        'lines: for l in br.lines() {
            if let Ok(l) = l {
                let trimmed = l.trim();

                if trimmed == ".byte 0" || trimmed == ".byte 1" {
                    continue;
                }

                if let Some(frame) = l.split(" ").last() {

                    let vals = frame.split(",");
                    let mut leds = [0u8; 25];

                    for (i, val) in vals.enumerate() {
                        if let Ok(val) = val.parse::<u8>() {

                            if i >= 25 {
                                break;
                            }
                            
                            leds[i] = val;

                        } else {
                            continue 'lines;
                        }
                    }

                    frames.push(Frame::with_values(leds));

                } else {
                    continue;
                }
            }
        }

        self.frames = frames;
        
    }
}


impl Application for State {
    fn init(&mut self, ctx: &mut glium_app::context::Context) {
        
    }

    fn update(&mut self, t: &glium_app::Timer, ctx: &mut glium_app::context::Context) {

        // Update animation

        if self.running {
            let dur = 1.0 / self.fps as f32;

            let now = Instant::now();
            let passed = now - self.last;

            if passed.as_secs_f32() >= dur {
                self.selected += 1;

                if self.selected == self.frames.len() {
                    self.selected = 0;
                }

                self.last = now;
            }
        }

        
        // ********* Handle keyboard shortcuts

        // Save
        if ctx.keyboard.is_pressed(&VirtualKeyCode::LControl) && ctx.keyboard.pressed_this_frame(&VirtualKeyCode::S) {
            self.save();
        }

        // Load file
        if ctx.keyboard.is_pressed(&VirtualKeyCode::LControl) && ctx.keyboard.pressed_this_frame(&VirtualKeyCode::L) {
            self.load_file();
        }


        // Draw window

        let mut target = ctx.dis.draw();

        // GUI
        let _repaint = ctx.gui.run(&ctx.dis, |gui_ctx| {

            egui::TopBottomPanel::top("top_panel").show(gui_ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {

                        if ui.button("Open").clicked() {
                            self.load_file();
                        }

                        if ui.button("Save").clicked() {
                            self.save();
                        }

                        if ui.button("Save As").clicked() {
                            self.save_as();
                        }
                    });
                })
            });

            egui::SidePanel::right("settings_panel").resizable(false).show(gui_ctx, |ui| {


                ui.heading("Settings");

                if self.running {
                    if ui.button("Pause").clicked() {
                        self.running = false;
                    }
                } else {
                    if ui.button("Play").clicked() {
                        self.running = true;
                        self.last = Instant::now();
                    }
                }

                ui.horizontal(|ui| {
                    ui.label("FPS");
                    ui.add(Slider::new(&mut self.fps, RangeInclusive::new(1, 15)));
                });

                ui.add(Separator::default().spacing(10.0));

                if ui.button("Invert LEDS").clicked() {
                    self.frames[self.selected].invert();
                }

                if ui.button("Set All LEDS").clicked() {
                    self.frames[self.selected].set_all(self.last_led.2);
                }

                if ui.button("Set Row").clicked() {
                    self.frames[self.selected].set_row(self.last_led.1, self.last_led.2)
                }

                if ui.button("Set Column").clicked() {
                    self.frames[self.selected].set_col(self.last_led.0, self.last_led.2)
                }

                ui.add(Separator::default().spacing(10.0));

                egui::ScrollArea::vertical().show(ui, |ui| {

                    let mut i = 0;
                    while i < self.frames.len() {

                        ui.horizontal(|ui| {

                            let mut selected = i == self.selected;
                            ui.checkbox(&mut selected, &format!("{}", i));
                            if selected {
                                self.selected = i;
                            }


                            if ui.add_enabled(i != 0, Button::new("/\\")).clicked() {
                                self.frames.swap(i, i-1);
                                self.selected = i - 1;
                            }

                            if ui.add_enabled(i != (self.frames.len()-1), Button::new("\\/")).clicked() {
                                self.frames.swap(i, i+1);
                                self.selected = i+1;
                            }

                            if ui.button("+").clicked() {
                                self.frames.insert(i, self.frames[i].clone());
                                self.selected = i + 1;
                            }

                            if ui.add_enabled(self.frames.len() != 1, Button::new("-")).clicked() {
                                self.frames.remove(i);
                                self.selected = i;
                                if i == self.frames.len() {
                                    self.selected -= 1;
                                }
                            }

                        });

                        i += 1;
                    }
                });

            });

            // Get bounds of central panel
            let central_space = gui_ctx.available_rect();
            let Pos2 {x: left, y: top} = central_space.min;
            let Pos2 {x: right, y: bottom} = central_space.max;

            egui::CentralPanel::default().show(gui_ctx, |ui| {

                // Calculate spacing for LEDs
                let width = right - left;
                let height = bottom - top;

                let mut size = width.min(height);
                size /= 5.0;

                let width = size * 2.0 / 3.0;
                let half_width = width / 2.0;

                let frame = &mut self.frames[self.selected];
                for (i, led) in frame.leds.iter_mut().enumerate() {

                    let x = i / 5;
                    let y = i % 5;

                    let p_x = left + size * (x as f32 + 0.5);
                    let p_y = top + size * (y as f32 + 0.5);

                    let col = Color32::from_rgba_unmultiplied(255, 0, 0, *led);
                    ui.painter().add(RectShape::filled(Rect::from_min_size(Pos2::new(p_x - half_width, p_y - half_width), Vec2::new(width, width)), Rounding::same(width/3.0), col));
                    if ui.put(Rect::from_center_size(Pos2::new(p_x, p_y), Vec2::new(0.0, 0.0)), DragValue::new(led)).clicked() {
                        self.last_led = (x, y, *led);
                    }

                }

            });
            
        });
        ctx.gui.paint(&ctx.dis, &mut target);

        target.finish().unwrap();

    }

    fn close(&mut self) {
        
    }

    fn handle_event(&mut self, ctx: &mut glium_app::context::Context, event: &glium_app::Event<()>) {
        
    }
}
