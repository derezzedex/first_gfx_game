#[macro_use]
extern crate gfx;

extern crate time;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate cgmath;

use gfx::traits::FactoryExt;
use gfx::{Device, texture};
use cgmath::{Deg, Matrix4, Point3, Vector3};
use cgmath::EuclideanSpace;
use gfx_window_glutin as gfx_glutin;
use gfx::Factory;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
	vertex Vertex{
		pos: [f32; 4] = "a_Pos",
		tex_coord: [f32; 2] = "a_TexCoord",
	}

	constant Locals{
		transform: [[f32; 4]; 4] = "u_Transform",
	}

	pipeline pipe{
		vbuf: gfx::VertexBuffer<Vertex> = (),
		transform: gfx::Global<[[f32; 4]; 4]> = "u_Transform",
		locals: gfx::ConstantBuffer<Locals> = "Locals",
		color: gfx::TextureSampler<[f32; 4]> = "t_Color",
		out_color: gfx::RenderTarget<ColorFormat> = "Target0",
		out_depth: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
	}

}

impl Vertex {
    fn new(p: [i8; 3], t: [i8; 2]) -> Vertex {
        Vertex {
            pos: [p[0] as f32, p[1] as f32, p[2] as f32, 1.0],
            tex_coord: [t[0] as f32, t[1] as f32],
        }
    }
}

fn view(eyes: Vector3<f32>, point: Vector3<f32>) -> Matrix4<f32> {
    Matrix4::look_at(
        Point3::from_vec(eyes),
        Point3::from_vec(point),
        Vector3::new(0.0f32, 1.0, 0.0),
    )
}

fn normalize(a: Vector3<f32>) -> Vector3<f32>{
	let ax = a.x;
	let ay = a.y;
	let az = a.z;
	let len = ((ax*ax)+(ay*ay)+(az*az)).sqrt();

	let x = ax/len;
	let y = ay/len;
	let z = az/len;
	Vector3::new(x, y, z)
}

fn deal_with_mouse(py: (f32, f32), last: (f32, f32), x: i32, y: i32, fmouse: bool) -> (Vector3<f32>, bool, f32, f32){
	let (lX, lY) = last;
	let mut lastX = lX as f32;
	let mut lastY = lY as f32; 
	let (p, w) = py;
	let mut pitch = p;
	let mut yaw = w;
	let mut fm = fmouse;

	if fm{
		lastX = x as f32;
		lastY = y as f32;
		fm = false;
	}

	let mut xoffset: f32 = x as f32 - lastX;
	let mut yoffset: f32 = y as f32 - lastY;
	lastX = x as f32;
	lastY = y as f32;

	let sensitivity = 0.005f32;
	xoffset *= sensitivity;
	yoffset *= sensitivity;

	yaw += xoffset;
	pitch += yoffset;

	if pitch > 89.0f32{
		pitch = 89.0f32;
	}
	if pitch < -89.0f32{
		pitch = -89.0f32;
	}

	let nx = pitch.cos() * yaw.cos();
	let ny = pitch.sin();
	let nz = pitch.cos() * yaw.cos();

	(normalize(Vector3::new(nx, ny, nz)), fm, lastX, lastY)

}

const INDEX: &[u16] = &[
	0,  1,  2,  2,  3,  0, // top
    4,  5,  6,  6,  7,  4, // bottom
    8,  9, 10, 10, 11,  8, // right
    12, 13, 14, 14, 15, 12, // left
    16, 17, 18, 18, 19, 16, // front
	20, 21, 22, 22, 23, 20, // back
];

fn main(){

	let cube: &[Vertex] = &[
		Vertex::new([-1, -1,  1], [0, 0]),
	    Vertex::new([ 1, -1,  1], [1, 0]),
	    Vertex::new([ 1,  1,  1], [1, 1]),
	    Vertex::new([-1,  1,  1], [0, 1]),
	    // bottom (0, 0, -1)
	    Vertex::new([-1,  1, -1], [1, 0]),
	    Vertex::new([ 1,  1, -1], [0, 0]),
	    Vertex::new([ 1, -1, -1], [0, 1]),
	    Vertex::new([-1, -1, -1], [1, 1]),
	    // right (1, 0, 0)
	    Vertex::new([ 1, -1, -1], [0, 0]),
	    Vertex::new([ 1,  1, -1], [1, 0]),
	    Vertex::new([ 1,  1,  1], [1, 1]),
	    Vertex::new([ 1, -1,  1], [0, 1]),
	    // left (-1, 0, 0)
	    Vertex::new([-1, -1,  1], [1, 0]),
	    Vertex::new([-1,  1,  1], [0, 0]),
	    Vertex::new([-1,  1, -1], [0, 1]),
	    Vertex::new([-1, -1, -1], [1, 1]),
	    // front (0, 1, 0)
	    Vertex::new([ 1,  1, -1], [1, 0]),
	    Vertex::new([-1,  1, -1], [0, 0]),
	    Vertex::new([-1,  1,  1], [0, 1]),
	    Vertex::new([ 1,  1,  1], [1, 1]),
	    // back (0, -1, 0)
	    Vertex::new([ 1, -1,  1], [0, 0]),
	    Vertex::new([-1, -1,  1], [1, 0]),
	    Vertex::new([-1, -1, -1], [1, 1]),
		Vertex::new([ 1, -1, -1], [0, 1]),
	];

	let mut game_title = "First GFX game!".to_string();

	let events_loop = glutin::EventsLoop::new();
	let builder = glutin::WindowBuilder::new()
		.with_title(game_title)
		.with_dimensions(800, 600)
		.with_vsync();

	let (window, mut device, mut factory, main_color, mut main_depth) = gfx_glutin::init::<ColorFormat, DepthFormat>(builder, &events_loop);
	let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
	let (width, height) = window.get_inner_size().unwrap();
	let aspect_ratio: f32 = width as f32/height as f32;

	let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(cube, INDEX);
	let texels = [[0x20, 0xA0, 0xC0, 0xFF]];
	let (_, texture_view) = factory.create_texture_immutable::<gfx::format::Rgba8>(
            texture::Kind::D2(1, 1, texture::AaMode::Single), &[&texels]
	).unwrap();

	let sinfo = texture::SamplerInfo::new(
		texture::FilterMethod::Bilinear,
		texture::WrapMode::Clamp);


	let pso = factory.create_pipeline_simple(
		include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/cube_150.glslv")),
		include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/cube_150.glslf")),
		pipe::new()
	).unwrap();

	let mut cameraPos = Vector3::new(0.0f32, 0.0, 3.0);
	let mut cameraFront = Vector3::new(0.0f32, 0.0, -1.0);

	let pressed = glutin::ElementState::Pressed;
	let released = glutin::ElementState::Released;

	let proj = cgmath::perspective(Deg(45.0f32), aspect_ratio, 0.1, 100.0);

	let mut data = pipe::Data{
		vbuf: vertex_buffer,
		transform: (proj * view(cameraPos, cameraPos + cameraFront)).into(),
        locals: factory.create_constant_buffer(1),
        color: (texture_view, factory.create_sampler(sinfo)),
        out_color: main_color,
		out_depth: main_depth,
	};

	let mut dT = 0.0f32;
	let mut lastFrame = time::Duration::milliseconds(0);
	let mut currentFrame = time::PreciseTime::now();

	let mut firstMouse = true;

	let mut pitch = 0.0f32;
	let mut yaw = -90.0f32;

	let mut lastX = 400.0f32;
	let mut lastY = 300.0f32;
	let sensitivity = 0.001f32;

	window.set_cursor_state(glutin::CursorState::Grab);
	window.set_cursor(glutin::MouseCursor::NoneCursor);

	let mut rec = false;
	let mut running = true;
	while running{
		let mut currentFrame = currentFrame.to(time::PreciseTime::now());
		let duration_dT = currentFrame - lastFrame;
		dT = duration_dT.num_milliseconds() as f32;
		lastFrame = currentFrame;

		let camera_speed = 10f32 * (dT/1000.0); // dt in seconds
		events_loop.poll_events(|glutin::Event::WindowEvent{window_id: _, event}| {
			use glutin::WindowEvent::*;
			match event{
				KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape), _)
				| Closed => running = false,
				KeyboardInput(_, pressed, Some(glutin::VirtualKeyCode::W), _) => {rec = true;cameraPos+=camera_speed*cameraFront;},
				KeyboardInput(_, pressed, Some(glutin::VirtualKeyCode::S), _) => {rec = true;cameraPos-=camera_speed*cameraFront;},
				KeyboardInput(_, pressed, Some(glutin::VirtualKeyCode::A), _) => {rec = true;cameraPos-=normalize(cameraFront.cross(Vector3::new(0.0f32, 1.0, 0.0)))*camera_speed},
				KeyboardInput(_, pressed, Some(glutin::VirtualKeyCode::D), _) => {rec = true;cameraPos+=normalize(cameraFront.cross(Vector3::new(0.0f32, 1.0, 0.0)))*camera_speed},
				//KeyboardInput(_, pressed, Some(glutin::VirtualKeyCode::E), _) => {window.set_cursor_state(glutin::CursorState::Normal);},
				//KeyboardInput(_, pressed, Some(glutin::VirtualKeyCode::LShift), _) => {},
				MouseMoved(a, b) =>{
					/*
					if firstMouse{
						lastX = a as f32;
						lastY = b as f32;
						firstMouse = false;
					}
					*/

					let mut xoffset: f32 = a as f32 - lastX;
					let mut yoffset: f32 = lastY - b as f32;

					let hw = (width/2) as i32;
					let hh = (height/2) as i32;
					window.set_cursor_position(hw, hh);
					lastX = hw as f32;
					lastY = hh as f32;

					xoffset *= sensitivity;
					yoffset *= sensitivity;

					yaw += xoffset;
					pitch += yoffset;

					if pitch >= 90.0f32{
						pitch = 90.0f32;
					}
					if pitch <= -90.0f32{
						pitch = -90.0f32;
					}

					let nx = pitch.cos() * yaw.cos();
					let ny = pitch.sin();
					let nz = yaw.sin() * pitch.cos();

					cameraFront = normalize(Vector3::new(nx, ny, nz));
				},
				Resized(_, _) => {
					let (width, height) = window.get_inner_size().unwrap();
					let aspect_ratio: f32 = width as f32/height as f32;
					let proj = cgmath::perspective(Deg(45.0f32), aspect_ratio, 0.1, 100.0);
					gfx_glutin::update_views(&window, &mut data.out_color, &mut data.out_depth);
					data.transform = (proj * view(cameraPos, cameraPos + cameraFront)).into();
				},
				_=> (),
			}

		});

		
		let (width, height) = window.get_inner_size().unwrap();
		let aspect_ratio: f32 = width as f32/height as f32;
		let proj = cgmath::perspective(Deg(45.0f32), aspect_ratio, 0.1, 100.0);
		gfx_glutin::update_views(&window, &mut data.out_color, &mut data.out_depth);
		data.transform = (proj * view(cameraPos, cameraPos + cameraFront)).into();

		let locals = Locals { transform: data.transform};
		encoder.update_constant_buffer(&data.locals, &locals);
		encoder.clear(&data.out_color, [0.156863, 0.156863, 0.156863, 1.0]);
		encoder.clear_depth(&data.out_depth, 1.0);

		//render & update
		encoder.draw(&slice, &pso, &data);
		encoder.flush(&mut device);
		window.swap_buffers().unwrap();
		device.cleanup();
	}

}