mod render_gl;

use gl::Gles2;

use glutin::{self, PossiblyCurrent};
use std::ffi::CStr;
pub static mut GL_CONTEXT: GlContext = GlContext { gl: None };

pub struct GlContext {
    gl: Option<Gl>,
}

impl GlContext {
    pub fn take_context(&mut self) -> Gl {
        let ctx = self.gl.take();
        ctx.unwrap()
    }
    pub fn give_back_context(&mut self, ctx: Gl) {
        self.gl = Some(ctx);
    }
}

pub struct Gl {
    glraw: Gles2,
}

pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

pub fn load(gl_context: &glutin::Context<PossiblyCurrent>) {
    let gl = gl::Gles2::load_with(|ptr| gl_context.get_proc_address(ptr) as *const _);

    let get_string = |param_id: gl::types::GLenum| unsafe {
        let data = CStr::from_ptr(gl.GetString(param_id) as *const _)
            .to_bytes()
            .to_vec();
        String::from_utf8(data).unwrap()
    };

    println!("Vendor graphic card {}", get_string(gl::VENDOR));
    println!("Renderer {}", get_string(gl::RENDERER));
    println!("OpenGL version {}", get_string(gl::VERSION));
    println!("Glsl version {}", get_string(gl::SHADING_LANGUAGE_VERSION));
    unsafe { GL_CONTEXT.gl = Some(Gl { glraw: gl }) };
}

impl Gl {
    pub fn draw_frame(&self, color: Color) {
        let gl = &self.glraw;
        unsafe {
            gl.ClearColor(color.red, color.green, color.blue, color.alpha);
            gl.Clear(gl::COLOR_BUFFER_BIT);
            gl.DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }

    pub fn viewport(&self) {
        let gl = &self.glraw;
        unsafe {
            gl.Viewport(0, 0, 800, 600);
            gl.ClearColor(0., 0.5, 0., 1.0);
            gl.Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    pub fn rect(&self) -> Result<(), String> {
        const VS_SRC: &'static [u8] = b"
        #version 100
        precision mediump float;
        attribute vec2 position;
        void main() {
            gl_Position = vec4(position, 0.0, 1.0);
        }
        \0";

        const FS_SRC: &'static [u8] = b"
        #version 100
        precision mediump float;
        uniform vec4 uColor;
        void main() {
            gl_FragColor = uColor;
        }
        \0";

        const VAL: f32 = 0.5;
        const ASPECT: f32 = 1.0;

        #[rustfmt::skip]
        const VERTEX_DATA: [f32; 12] = [
            -VAL, VAL * ASPECT, VAL,  VAL* ASPECT, VAL, -VAL* ASPECT, // triangle 1
            -VAL, VAL* ASPECT, VAL, -VAL* ASPECT, -VAL, -VAL* ASPECT, // trangle 2
        ];

        let vs = render_gl::Shader::from_vert_source(&self, VS_SRC)?;
        let fs = render_gl::Shader::from_frag_source(&self, FS_SRC)?;
        let program = render_gl::Program::from_shaders(&self, &[vs, fs])?;
        program.set_used();

        let gl = &self.glraw;
        unsafe {
            let mut vb = std::mem::zeroed();
            gl.GenBuffers(1, &mut vb);
            gl.BindBuffer(gl::ARRAY_BUFFER, vb);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (VERTEX_DATA.len() * std::mem::size_of::<f32>()) as isize,
                VERTEX_DATA.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            if gl.BindVertexArray.is_loaded() {
                let mut vao = std::mem::zeroed();
                gl.GenVertexArrays(1, &mut vao);
                gl.BindVertexArray(vao);
            }
        }

        let color_attrib =
            unsafe { gl.GetUniformLocation(program.id(), b"uColor\0".as_ptr() as *const _) };
        let pos_attrib =
            unsafe { gl.GetAttribLocation(program.id(), b"position\0".as_ptr() as *const _) };

        unsafe {
            gl.Uniform4fv(
                color_attrib,
                1,
                &[0.0f32, 0.3f32, 0.5f32, 1.0f32] as *const _,
            );
        }

        unsafe {
            gl.EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
            gl.VertexAttribPointer(
                pos_attrib as gl::types::GLuint,
                2,
                gl::FLOAT,
                0,
                0,
                std::ptr::null(),
            );

            gl.DrawArrays(gl::TRIANGLES, 0, 12);
        }
        Ok(())
    }
}
