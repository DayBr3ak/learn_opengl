use super::{gl, Gl, Gles2};
use std;
use std::ffi::CString;

pub struct Shader<'u> {
    id: gl::types::GLuint,
    glraw: &'u Gles2,
}

fn get_shader_info_log(gl: &Gles2, shader: u32) -> String {
    let mut len = 0;
    unsafe {
        gl.GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
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
        gl.GetShaderInfoLog(shader, len, std::ptr::null_mut(), error.as_ptr() as *mut _);
    }
    error.to_string_lossy().into_owned()
}

fn shader_from_source(
    gl: &Gles2,
    source: &'static [u8],
    shader_type: gl::types::GLenum,
) -> Result<u32, String> {
    let shader = unsafe { gl.CreateShader(shader_type) };

    unsafe {
        gl.ShaderSource(
            shader,
            1,
            [source.as_ptr() as *const _].as_ptr(),
            std::ptr::null(),
        );
        gl.CompileShader(shader);
    }

    let mut is_compiled = 0;
    unsafe {
        gl.GetShaderiv(shader, gl::COMPILE_STATUS, &mut is_compiled);
    }
    if is_compiled == 0 {
        Err(get_shader_info_log(gl, shader))
    } else {
        Ok(shader)
    }
}

impl<'u> Shader<'u> {
    fn from_source(
        gl: &'u Gl,
        source: &'static [u8],
        shader_type: gl::types::GLenum,
    ) -> Result<Shader<'u>, String> {
        let id = shader_from_source(&gl.glraw, source, shader_type)?;
        Ok(Shader {
            id,
            glraw: &gl.glraw,
        })
    }

    pub fn from_vert_source(gl: &'u Gl, source: &'static [u8]) -> Result<Shader<'u>, String> {
        Self::from_source(gl, source, gl::VERTEX_SHADER)
    }

    pub fn from_frag_source(gl: &'u Gl, source: &'static [u8]) -> Result<Shader<'u>, String> {
        Self::from_source(gl, source, gl::FRAGMENT_SHADER)
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl<'u> Drop for Shader<'u> {
    fn drop(&mut self) {
        unsafe {
            self.glraw.DeleteShader(self.id);
        }
    }
}
