pub mod game_of_life;
mod adder;
mod log;
mod ring_buffer;

use std::cell::RefCell;
use std::num::NonZeroUsize;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement, Window};
use crate::game_of_life::{CellValue, Field};
use crate::ring_buffer::RingBuffer;

/*
#setup:
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
wasm-bindgen --version # need to use the same version for dependency in cargo.toml

build needs multiple actions, so see build.sh
for testing see adder.rs
 */

const CELL_SIZE_PX: usize = 13;
const CELL_SIZE_PX_F64: f64 = CELL_SIZE_PX as f64;
const DEFAULT_FIELD_SIZE: NonZeroUsize = unsafe {NonZeroUsize::new_unchecked(64)};
const BIG_FIELD_SIZE: NonZeroUsize = unsafe {NonZeroUsize::new_unchecked(400)};

#[derive(Debug)]
struct AnimationState {
    next_frame: Option<i32>,
    next_timeout: Option<i32>,
    reduce_fps: bool,
    last_render_ts_ms: f64,
    time_history_ms: RingBuffer<f64>,
}
impl AnimationState {
    fn new() -> Self {
        Self {
            next_frame: None,
            next_timeout: None,
            reduce_fps: false,
            last_render_ts_ms: 0.0,
            time_history_ms: RingBuffer::new(100),
        }
    }
    fn is_running(&self) -> bool {
        self.next_frame.is_some() || self.next_timeout.is_some()
    }
}


#[wasm_bindgen(start)]
fn run() -> Result<(), JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    let canvas = document.create_element("canvas")?;
    let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let context = canvas.get_context("2d")?
        .expect("failed to get context")
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    let field = Field::generate_by_fn(DEFAULT_FIELD_SIZE, DEFAULT_FIELD_SIZE, |_| js_sys::Math::random() > 0.5);
    draw_initial_state(&field, &canvas, &context);

    let state = AnimationState::new();

    let window = Rc::new(window);
    let context = Rc::new(context);
    let state = Rc::new(RefCell::new(state));
    let field = Rc::new(RefCell::new(field));
    let canvas = Rc::new(canvas);
    
    let fps_element = document.create_element("span")?;
    body.append_child(&fps_element)?;

    let draw_function = init_draw_loop(
        Rc::clone(&window),
        Rc::clone(&field),
        Rc::clone(&state),
        Rc::clone(&context),
        fps_element,
    )?;

    let play_button = create_play_button(&document, Rc::clone(&window), Rc::clone(&state), Rc::clone(&draw_function))?;
    body.append_child(&play_button)?;

    let fps_button = create_fps_button(&document, Rc::clone(&state))?;
    body.append_child(&fps_button)?;

    add_edit_listener(Rc::clone(&canvas), Rc::clone(&context), Rc::clone(&field), Rc::clone(&state))?;

    let init_button = create_init_button(
        "Clear",
        move || Field::new(DEFAULT_FIELD_SIZE, DEFAULT_FIELD_SIZE),
        &document,
        Rc::clone(&field),
        Rc::clone(&canvas),
        Rc::clone(&context),
        Rc::clone(&state),
    )?;
    body.append_child(&init_button)?;

    let init_button = create_init_button(
        "Random",
        move || Field::generate_by_fn(DEFAULT_FIELD_SIZE, DEFAULT_FIELD_SIZE, |_| js_sys::Math::random() > 0.5),
        &document,
        Rc::clone(&field),
        Rc::clone(&canvas),
        Rc::clone(&context),
        Rc::clone(&state),
    )?;
    body.append_child(&init_button)?;

    let init_button = create_init_button(
        "Glider",
        make_glider_field,
        &document,
        Rc::clone(&field),
        Rc::clone(&canvas),
        Rc::clone(&context),
        Rc::clone(&state),
    )?;
    body.append_child(&init_button)?;

    let init_button = create_init_button(
        "Glider Gun",
        make_glider_gun_field,
        &document,
        Rc::clone(&field),
        Rc::clone(&canvas),
        Rc::clone(&context),
        Rc::clone(&state),
    )?;
    body.append_child(&init_button)?;

    let init_button = create_init_button(
        "Fixed",
        move || Field::generate_by_fn(DEFAULT_FIELD_SIZE, DEFAULT_FIELD_SIZE, |i| i % 2 == 0 || i % 7 == 0),
        &document,
        Rc::clone(&field),
        Rc::clone(&canvas),
        Rc::clone(&context),
        Rc::clone(&state),
    )?;
    body.append_child(&init_button)?;

    let init_button = create_init_button(
        "Random Big",
        move || Field::generate_by_fn(BIG_FIELD_SIZE, BIG_FIELD_SIZE, |_| js_sys::Math::random() > 0.5),
        &document,
        Rc::clone(&field),
        Rc::clone(&canvas),
        Rc::clone(&context),
        Rc::clone(&state),
    )?;
    body.append_child(&init_button)?;

    let init_button = create_init_button(
        "Fixed Big",
        move || Field::generate_by_fn(BIG_FIELD_SIZE, BIG_FIELD_SIZE, |i| i % 2 == 0 || i % 7 == 0),
        &document,
        Rc::clone(&field),
        Rc::clone(&canvas),
        Rc::clone(&context),
        Rc::clone(&state),
    )?;
    body.append_child(&init_button)?;

    let br = document.create_element("br")?;
    body.append_child(&br)?;

    body.append_child(&canvas)?;

    Ok(())
}

fn make_glider_field() -> Field {
    let init_state = "
__#____________________
___#___________________
_###___________________
_______________________
_______________________
_______________________
_______________________
_______________________
_______________________
_______________________
";
    Field::from_str(init_state).unwrap()
}

fn make_glider_gun_field() -> Field {
    let init_state = "
______________________________________________________________
______________________________________________________________
______________________________________________________________
____________________________#_________________________________
__________________________#_#_________________________________
________________##______##____________##______________________
_______________#___#____##____________##______________________
____##________#_____#___##____________________________________
____##________#___#_##____#_#_________________________________
______________#_____#_______#_________________________________
_______________#___#__________________________________________
________________##____________________________________________
______________________________________________________________
______________________________________________________________
______________________________________________________________
______________________________________________________________
______________________________________________________________
______________________________________________________________
______________________________________________________________
______________________________________________________________
______________________________________________________________
______________________________________________________________
______________________________________________________________
______________________________________________________________
______________________________________________________________
______________________________________________________________
______________________________________________________________
";
    Field::from_str(init_state).unwrap()
}

fn draw_initial_state(field: &Field, canvas: &HtmlCanvasElement, context: &CanvasRenderingContext2d) {
    canvas.set_height(((field.get_height() * (CELL_SIZE_PX + 1)) + 1) as u32);
    canvas.set_width(((field.get_width() * (CELL_SIZE_PX + 1)) + 1) as u32);
    draw_grid(&context, &field);
    draw_cells(&context, &field, &get_dead_style(), &get_alive_style(), true);
}

type RecursiveJsFunction = Rc<RefCell<Option<js_sys::Function>>>;
fn init_draw_loop(
    window: Rc<web_sys::Window>,
    field: Rc<RefCell<Field>>,
    state: Rc<RefCell<AnimationState>>,
    context: Rc<web_sys::CanvasRenderingContext2d>,
    fps_element: web_sys::Element,
) -> Result<RecursiveJsFunction, JsValue> {
    let draw_frame_closure_wrap = Rc::new(RefCell::new(None));
    let request_draw_closure = {
        let draw_frame_closure_wrap = Rc::clone(&draw_frame_closure_wrap);
        let state = Rc::clone(&state);
        let window = Rc::clone(&window);
        let closure = Closure::<dyn Fn()>::new(move || {
            let frame_id = window.request_animation_frame(
                draw_frame_closure_wrap.borrow().as_ref().unwrap()
            ).unwrap();
            state.borrow_mut().next_frame.replace(frame_id);
        });
        closure.into_js_value().dyn_into::<js_sys::Function>()?
    };
    let draw_frame_closure = {
        let dead_style = get_dead_style();
        let alive_style = get_alive_style();
        let closure = Closure::<dyn Fn()>::new(move || {
            let mut state_inner = state.borrow_mut();

            render_fps(calc_spf(&window, &mut state_inner), &fps_element);

            let mut field = field.borrow_mut();
            let has_alive = field.update();
            draw_cells(&context, &field, &dead_style, &alive_style, false);
            if !has_alive {
                pause(&window, &mut state_inner);
                return;
            }
            if state_inner.reduce_fps {
                // target 30 fps
                let timeout_id = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    &request_draw_closure,
                    30
                ).unwrap();
                state_inner.next_timeout.replace(timeout_id);
            } else {
                // target default fps
                drop(state_inner);
                request_draw_closure.call0(&JsValue::NULL).unwrap();
            }
        });
        closure.into_js_value().dyn_into::<js_sys::Function>()?
    };
    *draw_frame_closure_wrap.borrow_mut() = Some(draw_frame_closure);
    Ok(draw_frame_closure_wrap)
}

fn get_dead_style() -> JsValue {
    JsValue::from_str("#FFFFFF")
}

fn get_alive_style() -> JsValue {
    JsValue::from_str("#000000")
}

fn draw_grid(ctx: &CanvasRenderingContext2d, field: &Field) {
    ctx.begin_path();
    ctx.set_stroke_style(&"#CCCCCC".into());

    // Vertical lines.
    let width = field.get_width();
    let height = field.get_height();
    for i in 0..=width {
        let x = (i * (CELL_SIZE_PX + 1) + 1) as f64;
        let y = ((CELL_SIZE_PX + 1) * height + 1) as f64;
        ctx.move_to(x, 0.0);
        ctx.line_to(x, y);
    }

    // Horizontal lines.
    for j in 0..=height {
        let x = ((CELL_SIZE_PX + 1) * width + 1) as f64;
        let y = (j * (CELL_SIZE_PX + 1) + 1) as f64;
        ctx.move_to(0.0, y);
        ctx.line_to(x, y);
    }

    ctx.stroke();
}

fn draw_cells(ctx: &CanvasRenderingContext2d, field: &Field, dead_style: &JsValue, alive_style: &JsValue, force: bool) {
    ctx.begin_path();

    ctx.set_fill_style(alive_style);
    draw_cells_with_value(ctx, field, CellValue::Alive, force);

    ctx.set_fill_style(dead_style);
    draw_cells_with_value(ctx, field, CellValue::Dead, force);

    ctx.stroke();
}
fn draw_cells_with_value(ctx: &CanvasRenderingContext2d, field: &Field, filter_value: CellValue, force: bool) {
    let increment = CELL_SIZE_PX_F64 + 1.0;
    let start = 1.0;
    let mut grid_row = start;
    for (row, old_row) in field.rows_with_old() {
        let mut grid_col = start;
        for (col_no, &value) in row.iter().enumerate() {
            let old_value = old_row[col_no];
            if (value == filter_value) && ((value != old_value) || force) {
                ctx.fill_rect(grid_col, grid_row, CELL_SIZE_PX_F64, CELL_SIZE_PX_F64);
            }
            grid_col += increment;
        }
        grid_row += increment;
    }
}

fn create_play_button(
    document: &Document,
    window: Rc<Window>,
    state: Rc<RefCell<AnimationState>>,
    draw_function: RecursiveJsFunction
) -> Result<web_sys::Element, JsValue> {
    let button = document.create_element("button")?;
    button.set_text_content(Some("Play/Pause"));
    let control_closure = {
        Closure::<dyn Fn()>::new(move || {
            let mut state_inner = state.borrow_mut();
            if state_inner.is_running() {
                // stop if running
                pause(&window, &mut state_inner);
            } else {
                // start if not running
                drop(state_inner);
                draw_function.borrow().as_ref().unwrap().call0(&JsValue::NULL).unwrap();
            }
        })
    };
    button.add_event_listener_with_callback("click", control_closure.as_ref().unchecked_ref())?;
    control_closure.forget(); // prevent closure from dropping when going out of scope. For more complex applications it's better to store it somewhere instead
    Ok(button)
}

fn pause(window: &web_sys::Window, state: &mut AnimationState) {
    if let Some(frame_id) = state.next_frame.take() {
        window.cancel_animation_frame(frame_id).unwrap();
    }
    if let Some(timeout_id) = state.next_timeout.take() {
        window.clear_timeout_with_handle(timeout_id);
    }
}

fn add_edit_listener(
    canvas: Rc<HtmlCanvasElement>,
    context: Rc<CanvasRenderingContext2d>,
    field: Rc<RefCell<Field>>,
    state: Rc<RefCell<AnimationState>>,
) -> Result<(), JsValue> {
    let edit_closure = {
        let canvas = Rc::clone(&canvas);
        let dead_style = get_dead_style();
        let alive_style = get_alive_style();
        Closure::<dyn Fn(_)>::new(move |event: web_sys::MouseEvent| {
            if state.borrow().is_running() {
                return;
            }

            let mut field = field.borrow_mut();

            let bounding_rect = canvas.get_bounding_client_rect();

            let scale_x = (canvas.width() as f64) / bounding_rect.width();
            let scale_y = (canvas.height() as f64) / bounding_rect.height();

            let canvas_left = (event.client_x() as f64 - bounding_rect.left()) * scale_x;
            let canvas_top = (event.client_y() as f64 - bounding_rect.top()) * scale_y;

            let row = (canvas_top / ((CELL_SIZE_PX + 1) as f64)).floor() as usize;
            let row = std::cmp::min(row, field.get_height() - 1);
            let col = (canvas_left / ((CELL_SIZE_PX + 1) as f64)).floor() as usize;
            let col = std::cmp::min(col, field.get_width() - 1);
            match field.toggle_by_coords(row, col) {
                Some(_) => draw_cells(&context, &field, &dead_style, &alive_style, true),
                None => console_log!("Failed to update, calced coords: row {row}, col {col}"),
            }
        })
    };
    canvas.add_event_listener_with_callback("click", edit_closure.as_ref().unchecked_ref())?;
    edit_closure.forget(); // prevent closure from dropping when going out of scope. For more complex applications it's better to store it somewhere instead
    Ok(())
}

fn create_init_button(
    name: &'static str,
    factory: impl Fn() -> Field + 'static,
    document: &Document,
    field_container: Rc<RefCell<Field>>,
    canvas: Rc<HtmlCanvasElement>,
    context: Rc<CanvasRenderingContext2d>,
    state: Rc<RefCell<AnimationState>>,
) -> Result<web_sys::Element, JsValue> {
    let button = document.create_element("button")?;
    button.set_text_content(Some(name));
    let closure = {
        Closure::<dyn Fn()>::new(move || {
            state.borrow_mut().time_history_ms.truncate();
            let new_field = factory();
            draw_initial_state(&new_field, &canvas, &context);
            field_container.replace(new_field);
        })
    };
    button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
    closure.forget(); // prevent closure from dropping when going out of scope. For more complex applications it's better to store it somewhere instead
    Ok(button)
}

fn create_fps_button(
    document: &Document,
    state: Rc<RefCell<AnimationState>>,
) -> Result<web_sys::Element, JsValue> {
    let button = document.create_element("button")?;
    button.set_text_content(Some("Toggle FPS"));
    let control_closure = {
        Closure::<dyn Fn()>::new(move || {
            let mut state_inner = state.borrow_mut();
            state_inner.reduce_fps = !state_inner.reduce_fps;
            state_inner.time_history_ms.truncate();
        })
    };
    button.add_event_listener_with_callback("click", control_closure.as_ref().unchecked_ref())?;
    control_closure.forget(); // prevent closure from dropping when going out of scope. For more complex applications it's better to store it somewhere instead
    Ok(button)
}

fn calc_spf(window: &web_sys::Window, state: &mut AnimationState) -> f64 {
    let last_ts_ms = state.last_render_ts_ms;
    state.last_render_ts_ms = window.performance().unwrap().now();
    let time_passed_ms = state.last_render_ts_ms - last_ts_ms;
    if time_passed_ms <= 3000.0 {
        // a hack to skip large intervals where we pause/unpause
        // todo: think how to do it better
        state.time_history_ms.push(time_passed_ms);
    }
    let parts = state.time_history_ms.as_slices();
    let sum_delta_ms = parts.0.iter().sum::<f64>() + parts.1.iter().sum::<f64>();
    // todo: make sure that there is no division by zero
    let avg_delta_sec = (sum_delta_ms / state.time_history_ms.len() as f64) / 1000.0;
    avg_delta_sec
}

fn render_fps(spf: f64, element: &web_sys::Element) {
    let fps = 1.0 / spf;
    element.set_text_content(Some(format!("fps: {fps:.2}, spf {spf:.3}").as_str()))
}
