mod gl_bindings;
use gl_bindings::Gles2;
use gl_bindings::{self as gl, types};

use glutin::{self, PossiblyCurrent};
use std::ffi::{CStr, CString};

pub struct Gl {
    pub gl: Gles2,
}

pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

pub fn load(gl_context: &glutin::Context<PossiblyCurrent>) -> Gl {
    let gl = gl::Gles2::load_with(|ptr| gl_context.get_proc_address(ptr) as *const _);
    let version = unsafe {
        let data = CStr::from_ptr(gl.GetString(gl::VERSION) as *const _)
            .to_bytes()
            .to_vec();
        String::from_utf8(data).unwrap()
    };
    println!("OpenGL version {}", version);
    Gl { gl }
}

impl Gl {
    pub fn draw_frame(&self, color: Color) {
        unsafe {
            self.gl
                .ClearColor(color.red, color.green, color.blue, color.alpha);
            self.gl.Clear(gl::COLOR_BUFFER_BIT);
            self.gl.DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }

    pub fn viewport(&self) {
        unsafe {
            self.gl.Viewport(0, 0, 800, 600);
            self.gl.ClearColor(0., 0.5, 0., 1.0);
            self.gl.Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    fn get_shader_info_log(&self, shader: u32) -> String {
        let mut len = 0;
        unsafe {
            self.gl.GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
        }
        if !(len >= 0 && len < 10000) {
            panic!("info log size is wrong");
        }

        let buffer = vec![0u8; len as usize + 1];
        let error: CString = unsafe { CString::from_vec_unchecked(buffer) };

        // let data: Box<[i8]> = data.into_boxed_slice();
        // let data: *mut i8 = Box::into_raw(data) as *mut i8;
        // let length: Box<i32> = Box::new(0);
        // let length: *mut i32 = Box::into_raw(length);

        // let mut data: [i8; SIZE] = [0i8; SIZE];
        // let data: *mut i8 = &mut data[0];
        unsafe {
            self.gl
                .GetShaderInfoLog(shader, len, std::ptr::null_mut(), error.as_ptr() as *mut _);
        }
        error.to_string_lossy().into_owned()
    }

    pub fn compile_shader_source(
        &self,
        source: &'static [u8],
        shader_type: gl::types::GLenum,
    ) -> Result<u32, String> {
        let shader = unsafe { self.gl.CreateShader(shader_type) };

        unsafe {
            self.gl.ShaderSource(
                shader,
                1,
                [source.as_ptr() as *const _].as_ptr(),
                std::ptr::null(),
            );
            self.gl.CompileShader(shader);
        }

        let mut is_compiled = 0;
        unsafe {
            self.gl
                .GetShaderiv(shader, gl::COMPILE_STATUS, &mut is_compiled);
        }
        if is_compiled == 0 {
            Err(self.get_shader_info_log(shader))
        } else {
            Ok(shader)
        }
    }

    pub fn rect(&self) {
        const VS_SRC: &'static [u8] = b"
        #version 100
        precision mediump float;
        attribute vec2 position;
        void main() {xxx
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

        let program = unsafe { self.gl.CreateProgram() };
        if program == 0 {
            panic!("Cannot create the opengl program (CreateProgram)");
        }
        let vs = self
            .compile_shader_source(VS_SRC, gl::VERTEX_SHADER)
            .unwrap();
        let fs = self
            .compile_shader_source(FS_SRC, gl::FRAGMENT_SHADER)
            .unwrap();

        unsafe {
            self.gl.AttachShader(program, vs);
            self.gl.AttachShader(program, fs);
            self.gl.LinkProgram(program);
            self.gl.UseProgram(program);
        }

        unsafe {
            let mut vb = std::mem::zeroed();
            self.gl.GenBuffers(1, &mut vb);
            self.gl.BindBuffer(gl::ARRAY_BUFFER, vb);
            self.gl.BufferData(
                gl::ARRAY_BUFFER,
                (VERTEX_DATA.len() * std::mem::size_of::<f32>()) as isize,
                VERTEX_DATA.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            if self.gl.BindVertexArray.is_loaded() {
                let mut vao = std::mem::zeroed();
                self.gl.GenVertexArrays(1, &mut vao);
                self.gl.BindVertexArray(vao);
            }
        }

        let color_attrib = unsafe {
            self.gl
                .GetUniformLocation(program, b"uColor\0".as_ptr() as *const _)
        };

        unsafe {
            self.gl.Uniform4fv(
                color_attrib,
                1,
                &[0.0f32, 0.3f32, 0.5f32, 1.0f32] as *const _,
            );
        }

        let pos_attrib = unsafe {
            self.gl
                .GetAttribLocation(program, b"position\0".as_ptr() as *const _)
        };

        unsafe {
            self.gl
                .EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
            self.gl.VertexAttribPointer(
                pos_attrib as gl::types::GLuint,
                2,
                gl::FLOAT,
                0,
                0,
                std::ptr::null(),
            );

            self.gl.DrawArrays(gl::TRIANGLES, 0, 12);
        }
    }
}
