use notan::draw::*;
use notan::egui::{self, *};
use notan::math::{Mat4, Vec3, Vec2};
use notan::prelude::*;
use std::default::Default;

const MOVE_SPEED: f32 = 100.0;

//language=glsl
const VERT: ShaderSource = vertex_shader! {
    r#"
    #version 450
    layout(location = 0) in vec3 a_pos;
    layout(location = 1) in vec3 a_color;
    layout(location = 2) in vec2 a_uv;

    layout(location = 0) out vec3 v_color;
    layout(location = 1) out vec2 v_uv;
    
    layout(set = 0, binding = 0) uniform Locals {
        mat4 u_matrix;
    };

    void main() {
        v_color = a_color;
        v_uv = a_uv;
        gl_Position = u_matrix * vec4(a_pos, 1.0);
    }
    "#
};

//language=glsl
const FRAG: ShaderSource = fragment_shader! {
    r#"
    #version 450
    precision mediump float;

    layout(location = 0) in vec3 v_color;
    layout(location = 1) in vec2 v_uv;
    
    layout(location = 0) out vec4 color;
    
	layout(binding = 0) uniform sampler2D u_texture;
    
    void main() {
        //color = vec4(v_color, 1.0);
        color = texture(u_texture, v_uv);
    }
    "#
};

#[derive(AppState)]
struct State {
	font: Font,
    clear_options: ClearOptions,
    pipeline: Pipeline,
    vbo: Buffer,
	ibo: Buffer,
	ubo: Buffer,
	missing_texture: Texture,
	render_texture: RenderTexture,
	texture: Asset<Texture>,
	object_offset: (f32, f32),
}

#[notan_main]
fn main() -> Result<(), String> {
	let win = WindowConfig::new()
        .vsync(true)
        //.lazy_loop(true)
        .high_dpi(true);
	
	notan::init_with(setup)
		.add_config(win)
		.add_config(DrawConfig)
		.add_config(EguiConfig)
		.update(update)
		.draw(draw)
		.build()
}

fn setup(assets: &mut Assets, gfx: &mut Graphics) -> State {
	let font = gfx
		.create_font(include_bytes!("../assets/Ubuntu-Regular.ttf"))
		.unwrap();
	
	let texture = gfx
		.create_texture()
        //.from_bytes(&bytes, width, height)
		.from_empty_buffer(320, 240)
        .build()
        .unwrap();
	let pending_texture = assets.load_asset(&"assets/cat.png").unwrap();
	let render_texture = gfx
		.create_render_texture(320, 240)
		.build()
		.unwrap();
	
	//ClearOptions::color(Color::new(0.1, 0.2, 0.3, 1.0));
	let clear_options = ClearOptions {
		color: Some(Color::BLUE),
		depth: Some(1.0),
		..Default::default()
	};
	let vertex_info = VertexInfo::new()
        .attr(0, VertexFormat::Float32x3)
        .attr(1, VertexFormat::Float32x3)
		.attr(2, VertexFormat::Float32x2);
	let pipeline = gfx
		.create_pipeline()
		.from(&VERT, &FRAG)
		.with_vertex_info(&vertex_info)
		.with_depth_stencil(DepthStencil {
			write: true,
			compare: CompareMode::Less,
		})
		.with_texture_location(0, "u_texture")
		.build()
		.unwrap();
	
	let projection = Mat4::perspective_rh_gl(45.0, 4.0 / 3.0, 0.1, 100.0);
	let view = Mat4::look_at_rh(
		Vec3::new(4.0, 3.0, 3.0),
		Vec3::new(0.0, 0.0, 0.0),
		Vec3::new(0.0, 1.0, 0.0),
	);
	let mvp = Mat4::IDENTITY * projection * view;

	// Remember: opengl is counter-clockwise.
	#[rustfmt::skip]
	let vertices = [
		0.0, 0.0, 0.0,   1.0, 0.2, 0.3,   0.0, 0.0,
		1.0, 0.0, 0.0,   0.1, 1.0, 0.3,   0.0, 0.0,
		0.0, 1.0, 0.0,   0.1, 0.2, 1.0,   0.0, 0.0,
	];
	
	#[rustfmt::skip]
	let indices = [
		0, 1, 2,
	];
	
	let vertex_buffer = gfx
        .create_vertex_buffer()
        .with_info(&vertex_info)
        .with_data(&vertices)
        .build()
        .unwrap();
	
	let index_buffer = gfx
		.create_index_buffer()
		.with_data(&indices)
		.build()
		.unwrap();
	
	let uniform_buffer = gfx
		.create_uniform_buffer(0, "Locals")
		.with_data(&mvp)
		.build()
		.unwrap();

	State {
		font,
		clear_options,
		pipeline,
		vbo: vertex_buffer,
		ibo: index_buffer,
		ubo: uniform_buffer,
		missing_texture: texture,
		texture: pending_texture,
		render_texture,
		object_offset: (0.0f32, 0.0f32),
	}
}

fn update(app: &mut App, state: &mut State) {
	//state.last_key = app.keyboard.last_key_released();
	
	if app.keyboard.is_down(KeyCode::W) {
		state.object_offset.1 -= MOVE_SPEED * app.timer.delta_f32();
	}
	
	if app.keyboard.is_down(KeyCode::A) {
		state.object_offset.0 -= MOVE_SPEED * app.timer.delta_f32();
	}
	
	if app.keyboard.is_down(KeyCode::S) {
		state.object_offset.1 += MOVE_SPEED * app.timer.delta_f32();
	}
	
	if app.keyboard.is_down(KeyCode::D) {
		state.object_offset.0 += MOVE_SPEED * app.timer.delta_f32();
	}
}

fn draw(app: &mut App, gfx: &mut Graphics, plugins: &mut Plugins, state: &mut State) {
	// Deferred UI stuff?
	let mut ui_output = plugins.egui(|ctx| {
        egui::Window::new("egui window").show(ctx, |ui| {
            //ui.image(state.tex_id, state.img_size);
        });
		
		egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Egui Plugin Example");

            ui.separator();
            if ui.button("Quit").clicked() {
                app.exit();
            }

            ui.separator();
			ui.horizontal(|ui|{
			
			});
            ui.label("Check the source code to learn more about how it works");
        });
    });
	
	// Updating textures:
	/*
	gfx.update_texture(&mut state.texture)
        .with_data(&state.bytes)
        .update()
        .unwrap();
	*/
	//let image_on_rt2 = some_method_that_binds_texture_and_returns_renderer(gfx, state, tex, None);
    //gfx.render_to(&state.render_texture2, &image_on_rt2);
	
	// 3D drawing:
	let mut renderer = gfx.create_renderer();
	renderer.begin(Some(&state.clear_options));
	renderer.set_pipeline(&state.pipeline);
	renderer.bind_texture(0, &state.missing_texture);
	//renderer.bind_buffer(&state.vbo);
	renderer.bind_buffers(&[
		&state.vbo,
		&state.ibo,
		&state.ubo,
	]);
	renderer.draw(0, 3);
	renderer.end();
	gfx.render(&renderer);

	// Basic 2D drawing:
	let mut draw = gfx.create_draw();
	//draw.clear(Color::BLACK);
	let x = state.object_offset.0.clone();
	let y = state.object_offset.1.clone();
	draw.circle(50.0)
		.position(x, y)
		.color(Color::RED);
	
	draw.text(&state.font, "Use WASD to move the circle")
		.position(10.0, 10.0)
		.size(20.0);
	
	draw.text(&state.font, &format!("Last key: key:?"))
		.position(10.0, 560.0)
		.size(20.0);
	
	//draw.image(&state.texture);
	
	gfx.render(&draw);
	
	/*
	if ui_output.needs_repaint() {
		gfx.render(&ui_output);
	}
	*/
	gfx.render(&ui_output);
}