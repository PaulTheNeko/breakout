use fixedstep;
use macroquad::*;
use palette::{rgb::LinSrgb, FromColor, Hsv, RgbHue};

const AREA_HEIGHT: f32 = 30.;
const AREA_WIDTH: f32 = 50.;
const BRICK_HEIGHT: f32 = 5.0;
const HZ: f64 = 60.0;
#[macroquad::main("test")]
async fn main() {
    let mut fixedstep = Some(fixedstep::FixedStep::start(HZ));
    let mut ball = Moving {
        shape: Shape {
            pos: vec2(AREA_WIDTH * 0.5, AREA_HEIGHT * 0.5),
            size: vec2(1.0, 1.0),
            color: WHITE,
        },
        vel: vec2(0.0, -20.0),
    };
    let mut pallet = Moving {
        shape: Shape {
            pos: vec2(AREA_WIDTH * 0.5, AREA_HEIGHT * 0.1),
            size: vec2(5.0, 1.0),
            color: WHITE,
        },
        vel: vec2(0.0, 0.0),
    };

    let mut bricks = create_bricks(
        vec2(0.0, AREA_HEIGHT - BRICK_HEIGHT - 5.0),
        vec2(1.0, 1.0),
        vec2(AREA_WIDTH, BRICK_HEIGHT),
        10,
        3,
    );

    loop {
        set_camera(create_camera());
        for i in 0..10 {
            let i: f32 = i as f32;
            clear_background(BLACK);
            draw_rectangle(i * 20., i * 20., i * 10., i * 10., WHITE);
        }

        if let Some(ref mut step) = fixedstep {
            while step.update() {
                ball.go();
                pallet.go();

                if ball.shape.pos.y() < ball.shape.size.y() {
                    fixedstep = None;
                    break;
                };

                ball.rev_vel_out_range(0.0..AREA_WIDTH, 0.0..AREA_HEIGHT);
                ball.shape.fit_in(0.0..AREA_WIDTH, 0.0..AREA_HEIGHT);
                pallet.shape.fit_in(0.0..AREA_WIDTH, 0.0..AREA_HEIGHT);

                let mut x_reflect = false;
                let mut y_reflect = false;
                if let Some(collision) = ball.shape.collision(&pallet.shape) {
                    y_reflect = true;
                    *ball.vel.x_mut() += collision.x() * 4.0;
                    *ball.vel.x_mut() = ball.vel.x().signum()*ball.vel.x().abs().min(23.0);
                    *ball.shape.pos.y_mut() = pallet.shape.pos.y()
                        + ball.shape.size.y() / 2.0
                        + pallet.shape.size.y() / 2.0;
                }
                bricks.retain(|i| {
                    if let Some(collision) = ball.shape.collision(i) {
                        let ret = i.size.x() / i.size.y();
                        let cret = collision.x().abs() / collision.y().abs();
                        if ret < cret {
                            x_reflect = true;
                        } else {
                            y_reflect = true;
                        }
                        false
                    } else {
                        true
                    }
                });
                if x_reflect {
                    *ball.vel.x_mut() = -ball.vel.x()
                };
                if y_reflect {
                    *ball.vel.y_mut() = -ball.vel.y()
                };

                }
            }
        }

        *pallet.vel.x_mut() = input_to_pallet_vel() * 25.0;

        ball.shape.draw();
        pallet.shape.draw();
        bricks.iter().for_each(|x| x.draw());
        next_frame().await;
    }
}

struct Shape {
    pos: Vec2,
    size: Vec2,
    color: Color,
}

struct Moving {
    shape: Shape,
    vel: Vec2,
}

impl Shape {
    fn fit_in(&mut self, range_x: std::ops::Range<f32>, range_y: std::ops::Range<f32>) {
        fit(
            self.pos.x_mut(),
            tighten_range(range_x, self.size.x() / 2.0),
        );
        fit(
            self.pos.y_mut(),
            tighten_range(range_y, self.size.y() / 2.0),
        );

        fn fit(num: &mut f32, range: std::ops::Range<f32>) {
            if *num > range.end {
                *num = range.end
            }
            if *num < range.start {
                *num = range.start
            }
        }
    }

    fn collision(&self, second: &Self) -> Option<Vec2> {
        let sizes = second.size + self.size;
        let distance = self.pos - second.pos;

        if (distance.y().abs() < sizes.y() / 2.0) && (distance.x().abs() < sizes.x() / 2.0) {
            Some(distance)
        } else {
            None
        }
    }

    fn draw(&self) {
        let Shape { pos, size, color } = self;
        let (x, y) = (pos.x() - size.x() / 2.0, pos.y() - size.y() / 2.0);
        draw_rectangle(x, y, size.x(), size.y(), *color);
    }
}

impl Moving {
    fn go(&mut self) {
        self.shape.pos += self.vel / HZ as f32;
    }

    // TODO: minimize
    fn rev_vel_out_range(&mut self, range_x: std::ops::Range<f32>, range_y: std::ops::Range<f32>) {
        if !tighten_range(range_x, self.shape.size.x() / 2.0).contains(&self.shape.pos.x()) {
            *self.vel.x_mut() = -self.vel.x();
        }
        if !tighten_range(range_y, self.shape.size.y() / 2.0).contains(&self.shape.pos.y()) {
            *self.vel.y_mut() = -self.vel.y();
        }
    }
}

fn create_camera() -> Camera2D {
    let (w, h) = (screen_height(), screen_width());
    let rret = AREA_HEIGHT / AREA_WIDTH;
    let ret = w / h;
    let xret = ret / rret;
    let mut cam_w = AREA_WIDTH;
    let mut cam_h = AREA_HEIGHT;

    if xret < 1.0 {
        cam_w = cam_w / xret;
    } else {
        cam_h = cam_h * xret;
    }

    let zoom = vec2(2.0 / cam_w, 2.0 / cam_h);
    Camera2D {
        zoom,
        target: vec2(AREA_WIDTH / 2.0, AREA_HEIGHT / 2.0),
        ..Default::default()
    }
}

fn input_to_pallet_vel() -> f32 {
    use KeyCode::*;
    let mut num = 0.0;
    if short_inp(&[Left]) {
        num -= 1.0
    }
    if short_inp(&[Right]) {
        num += 1.0
    }
    return num;

    fn short_inp(keys: &[KeyCode]) -> bool {
        for i in keys {
            if is_key_down(*i) {
                return true;
            }
        }
        return false;
    }
}

fn tighten_range(range: std::ops::Range<f32>, num: f32) -> std::ops::Range<f32> {
    range.start + num..range.end - num
}

fn create_bricks(start: Vec2, spacing: Vec2, size: Vec2, amount: u8, rows: u8) -> Vec<Shape> {
    // let mut col = [WHITE, RED, BLUE, GREEN, GOLD, ORANGE, YELLOW].iter().cycle();
    let mut vec = Vec::new();
    let start = start - spacing;
    let size = size + spacing * 2.0;

    let amvec = vec2(amount.into(), rows.into());
    let brick_size = size / amvec;
    let x = size / (amvec + vec2(1.0, 1.0));
    for row in 0..rows {
        for brick in 0..amount {
            let shift = vec2(brick.into(), row.into());
            let shift = shift + vec2(1.0, 1.0);
            let pos = shift * size / (amvec + vec2(1.0, 1.0));
            let pos = pos + start;

            let amount: f32 = amount.into();
            let rows: f32 = rows.into();
            let row: f32 = 2.0 - row as f32;
            let brick: f32 = brick.into();
            let deg = (brick + row) * 360.0 / (rows + amount);

            vec.push(Shape {
                pos: pos,
                size: brick_size - spacing,
                color: color_from_deg(deg),
            });
        }
    }
    vec
}

fn color_from_deg(deg: f32) -> Color {
    let hue = RgbHue::from_degrees(deg);
    let hsv = Hsv::new(hue, 0.5, 1.0);
    let rgb = LinSrgb::from_hsv(hsv);
    let (r, g, b) = rgb.into_components();
    Color::new(r, g, b, 1.0)
}
