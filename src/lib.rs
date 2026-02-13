use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

const NUM_WORMS: usize = 12;
const WORM_SEGMENTS: usize = 20;
const SEGMENT_RADIUS: f64 = 6.0;
const SEGMENT_SPACING: f64 = 10.0;

struct Worm {
    /// Head position and angle
    x: f64,
    y: f64,
    angle: f64,
    /// Turn speed (radians per frame)
    turn_speed: f64,
    /// Forward speed
    speed: f64,
    /// Trail of segment positions
    segments: Vec<(f64, f64)>,
    /// Color hue
    hue: f64,
    /// Time accumulator for wobble
    wobble_phase: f64,
    wobble_rate: f64,
}

impl Worm {
    fn new(x: f64, y: f64, hue: f64, seed: f64) -> Self {
        let angle = seed * 6.28;
        let segments = (0..WORM_SEGMENTS)
            .map(|i| {
                let offset = i as f64 * SEGMENT_SPACING;
                (x - angle.cos() * offset, y - angle.sin() * offset)
            })
            .collect();
        Self {
            x,
            y,
            angle,
            turn_speed: 0.03 + seed * 0.04,
            speed: 1.2 + seed * 1.0,
            segments,
            hue,
            wobble_phase: seed * 6.28,
            wobble_rate: 2.0 + seed * 3.0,
        }
    }

    fn update(&mut self, width: f64, height: f64, time: f64) {
        // Wobble the heading
        self.angle += (time * self.wobble_rate + self.wobble_phase).sin() * self.turn_speed;

        // Move the head
        self.x += self.angle.cos() * self.speed;
        self.y += self.angle.sin() * self.speed;

        // Wrap around edges with a margin
        let margin = 20.0;
        if self.x < -margin {
            self.x = width + margin;
        }
        if self.x > width + margin {
            self.x = -margin;
        }
        if self.y < -margin {
            self.y = height + margin;
        }
        if self.y > height + margin {
            self.y = -margin;
        }

        // Update segment trail: head is first
        self.segments.insert(0, (self.x, self.y));
        self.segments.truncate(WORM_SEGMENTS);
    }

    fn draw(&self, ctx: &CanvasRenderingContext2d) {
        let total = self.segments.len() as f64;
        for (i, &(sx, sy)) in self.segments.iter().enumerate() {
            let t = i as f64 / total;
            let radius = SEGMENT_RADIUS * (1.0 - t * 0.6);
            let lightness = 45.0 + t * 20.0;
            let alpha = 1.0 - t * 0.3;

            let color = format!(
                "hsla({}, 70%, {}%, {})",
                self.hue, lightness, alpha
            );
            ctx.set_fill_style_str(&color);
            ctx.begin_path();
            ctx.arc(sx, sy, radius, 0.0, std::f64::consts::TAU)
                .unwrap();
            ctx.fill();
        }

        // Draw eyes on the head
        if let Some(&(hx, hy)) = self.segments.first() {
            let eye_offset = SEGMENT_RADIUS * 0.5;
            let perp = self.angle + std::f64::consts::FRAC_PI_2;
            for &side in &[-1.0, 1.0] {
                let ex = hx + perp.cos() * eye_offset * side + self.angle.cos() * 3.0;
                let ey = hy + perp.sin() * eye_offset * side + self.angle.sin() * 3.0;
                // White of eye
                ctx.set_fill_style_str("white");
                ctx.begin_path();
                ctx.arc(ex, ey, 3.0, 0.0, std::f64::consts::TAU).unwrap();
                ctx.fill();
                // Pupil
                ctx.set_fill_style_str("black");
                ctx.begin_path();
                ctx.arc(
                    ex + self.angle.cos() * 1.2,
                    ey + self.angle.sin() * 1.2,
                    1.5,
                    0.0,
                    std::f64::consts::TAU,
                )
                .unwrap();
                ctx.fill();
            }
        }
    }
}

struct World {
    worms: Vec<Worm>,
    width: f64,
    height: f64,
    time: f64,
}

impl World {
    fn new(width: f64, height: f64) -> Self {
        let worms = (0..NUM_WORMS)
            .map(|i| {
                let seed = i as f64 / NUM_WORMS as f64;
                let x = seed * width;
                let y = (seed * 3.7 % 1.0) * height;
                let hue = seed * 360.0;
                Worm::new(x, y, hue, seed)
            })
            .collect();
        Self {
            worms,
            width,
            height,
            time: 0.0,
        }
    }

    fn tick(&mut self) {
        self.time += 1.0;
        for worm in &mut self.worms {
            worm.update(self.width, self.height, self.time);
        }
    }

    fn draw(&self, ctx: &CanvasRenderingContext2d) {
        // Semi-transparent clear for motion trails
        ctx.set_fill_style_str("rgba(30, 20, 40, 0.25)");
        ctx.fill_rect(0.0, 0.0, self.width, self.height);

        for worm in &self.worms {
            worm.draw(ctx);
        }

        // Draw title text
        ctx.set_font("bold 48px monospace");
        ctx.set_text_align("center");

        // Text shadow / glow
        ctx.set_shadow_color("rgba(180, 100, 255, 0.8)");
        ctx.set_shadow_blur(20.0);
        ctx.set_fill_style_str("#f0e0ff");
        ctx.fill_text("hello worms :P", self.width / 2.0, 60.0)
            .unwrap();

        // Reset shadow
        ctx.set_shadow_blur(0.0);
    }
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap();
}

#[wasm_bindgen(start)]
pub fn main() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document
        .get_element_by_id("worm-canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();

    let width = 800.0;
    let height = 600.0;
    canvas.set_width(width as u32);
    canvas.set_height(height as u32);

    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();

    // Initial background fill
    ctx.set_fill_style_str("#1e1428");
    ctx.fill_rect(0.0, 0.0, width, height);

    let world = Rc::new(RefCell::new(World::new(width, height)));

    // Animation loop
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();
    let w = world.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let mut world = w.borrow_mut();
        world.tick();
        world.draw(&ctx);
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
}
