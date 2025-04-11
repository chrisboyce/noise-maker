use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;
use std::f64::consts::PI;

const CELL_COUNT: usize = 128;
fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    stream: audio::Stream<Audio>,
    chamber: Chamber<CELL_COUNT>,
}
impl Model {
    fn reset(&mut self) {
        self.chamber.cells.cur = [0.0; CELL_COUNT];
        self.chamber.cells.prev = [0.0; CELL_COUNT];
    }
}

struct Chamber<const N: usize> {
    cells: Cells<N>,
}
struct Cells<const N: usize> {
    prev: [f64; N],
    cur: [f64; N],
}
impl<const N: usize> Cells<N> {
    fn new() -> Self {
        Self {
            prev: [0.0; N],
            cur: [0.0; N],
        }
    }
}

impl<const N: usize> Chamber<N> {
    fn new() -> Self {
        Self {
            cells: Cells::new(),
        }
    }

    fn add_pressure(&mut self, pressure: f64) {
        self.cells.cur[0] += pressure;
        // self.cells[0] += pressure;
    }
    // Given:
    // - N: number of cells
    // - u_prev[N]: pressure values at previous time step (t-1)
    // - u_curr[N]: pressure values at current time step (t)
    // - u_next[N]: pressure values to compute (t+1)
    // - C: Courant number = (wave_speed * dt / dx), must be <= 1 for stability

    // Algorithm:
    // 1. For each cell i (excluding boundaries):
    //     u_next[i] = 2 * u_curr[i]
    //                 - u_prev[i]
    //                 + C^2 * (u_curr[i+1] - 2*u_curr[i] + u_curr[i-1])

    // 2. Apply boundary conditions:
    //     e.g., fixed ends: u_next[0] = 0, u_next[N-1] = 0
    //          or open end, or injected source

    // 3. Rotate buffers for next step:
    //     u_prev = u_curr
    //     u_curr = u_next
    // for i from 1 to N - 2:
    // u_next[i] = 2 * u_curr[i]
    //             - u_prev[i]
    //             + C * C * (u_curr[i+1] - 2 * u_curr[i] + u_curr[i-1])

    fn update_pressures(&mut self) {
        let mut next = [0.0; N];
        for index in 1..(N - 1) {
            let cur = self.cells.cur[index];
            let prev = self.cells.prev[index];
            let left = self.cells.cur[index - 1];
            let right = self.cells.cur[index + 1];
            next[index] = (2.0 * cur) - prev + 0.5 * (right - 2.0 * cur + left);
        }
        // for (index, next_value) in next.iter_mut().enumerate() {
        //     let cur = self.cells.cur[index];
        //     let prev = self.cells.prev[index];
        //     let left = if index > 0 {
        //         self.cells.cur[index - 1]
        //     } else {
        //         0.0
        //     };
        //     let right = if index < (N - 2) {
        //         self.cells.cur[index + 1]
        //     } else {
        //         0.0
        //     };
        //     *next_value = (2.0 * cur) - prev + 0.1 * (right - 2.0 * cur + left);
        // }
        self.cells.prev = self.cells.cur;
        self.cells.cur = next;
    }
}

struct Audio {
    phase: f64,
    hz: f64,
}

fn model(app: &App) -> Model {
    // Create a window to receive key pressed events.
    let window = app
        .new_window()
        .key_pressed(key_pressed)
        .view(view)
        .build()
        .unwrap();

    // Initialise the audio API so we can spawn an audio stream.
    let audio_host = audio::Host::new();
    let chamber = Chamber::new();

    // Initialise the state that we want to live on the audio thread.
    let model = Audio {
        phase: 0.0,
        hz: 440.0,
    };

    let stream = audio_host
        .new_output_stream(model)
        .render(audio)
        .build()
        .unwrap();

    // stream.play().unwrap();

    Model { stream, chamber }
}

// A function that renders the given `Audio` to the given `Buffer`.
// In this case we play a simple sine wave at the audio's current frequency in `hz`.
fn audio(audio: &mut Audio, buffer: &mut Buffer) {
    let sample_rate = buffer.sample_rate() as f64;
    let volume = 0.5;
    for frame in buffer.frames_mut() {
        let sine_amp = (2.0 * PI * audio.phase).sin() as f32;
        audio.phase += audio.hz / sample_rate;
        audio.phase %= sample_rate;
        for channel in frame {
            *channel = sine_amp * volume;
        }
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::R => {
            model.reset();
        }
        Key::A => {
            model.chamber.add_pressure(0.1);
        }
        // Pause or unpause the audio when Space is pressed.
        Key::Space => {
            if model.stream.is_playing() {
                model.stream.pause().unwrap();
            } else {
                model.stream.play().unwrap();
            }
        }
        // Raise the frequency when the up key is pressed.
        Key::Up => {
            model
                .stream
                .send(|audio| {
                    audio.hz += 10.0;
                })
                .unwrap();
        }
        // Lower the frequency when the down key is pressed.
        Key::Down => {
            model
                .stream
                .send(|audio| {
                    audio.hz -= 10.0;
                })
                .unwrap();
        }
        _ => {}
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let r = frame.rect();
    frame.clear(DIMGRAY);
    let draw = app.draw();
    let cell_width = 500.0 / (CELL_COUNT as f32);
    for i in 0..CELL_COUNT {
        let pressure = model.chamber.cells.cur[i];
        draw.quad()
            .w_h(cell_width, 30.0)
            .x_y(((i as f32 * cell_width) as f32) - 250.0, 100.0)
            .color(Gray::new(pressure + 0.5, pressure + 0.5, pressure + 0.5));
    }
    draw.to_frame(app, &frame).unwrap();
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    // model.chamber.add_pressure(0.01);
    model.chamber.update_pressures();
}
