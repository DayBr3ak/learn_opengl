use super::resources::{Error as ResourcesError, Resources};
use super::Gl;

use gl::Gles;
use std;
use std::ffi::CString;

#[derive(Debug)]
pub enum Error {
    ResourceLoad { name: String, inner: ResourcesError },
    CanNotDetermineShaderTypeForResource { name: String },
    CompileError { name: String, message: String },
    LinkError { name: String, message: String },
}

pub struct Shader {
    id: gl::types::GLuint,
    gl: Gles,
}

fn get_shader_info_log(gl: &Gles, shader: u32) -> String {
    let mut len = 0;
    unsafe {
        gl.GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
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
    gl: &Gles,
    source: &CString,
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

impl Shader {
    fn from_source(
        gl: &Gl,
        source: &CString,
        shader_type: gl::types::GLenum,
    ) -> Result<Shader, String> {
        let id = shader_from_source(&gl.inner, source, shader_type)?;
        Ok(Shader {
            id,
            gl: gl.inner.clone(),
        })
    }

    pub fn from_res(gl: &Gl, res: &Resources, name: &str) -> Result<Shader, Error> {
        const POSSIBLE_EXT: [(&str, gl::types::GLenum); 2] =
            [(".vert", gl::VERTEX_SHADER), (".frag", gl::FRAGMENT_SHADER)];

        let shader_kind = POSSIBLE_EXT
            .iter()
            .find(|&&(file_extension, _)| name.ends_with(file_extension))
            .map(|&(_, kind)| kind)
            .ok_or_else(|| Error::CanNotDetermineShaderTypeForResource { name: name.into() })?;

        let source = res.load_cstring(name).map_err(|e| Error::ResourceLoad {
            name: name.into(),
            inner: e,
        })?;

        Shader::from_source(gl, &source, shader_kind).map_err(|message| Error::CompileError {
            name: name.into(),
            message,
        })
    }

    pub fn from_vert_source(gl: &Gl, source: &CString) -> Result<Shader, String> {
        Self::from_source(gl, source, gl::VERTEX_SHADER)
    }

    pub fn from_frag_source(gl: &Gl, source: &CString) -> Result<Shader, String> {
        Self::from_source(gl, source, gl::FRAGMENT_SHADER)
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteShader(self.id);
        }
    }
}

pub struct Program {
    id: gl::types::GLuint,
    gl: Gles,
}
impl Program {
    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
    fn from_shaders(gl: &Gl, shaders: &[Shader]) -> Result<Program, String> {
        let gl = &gl.inner;
        let program_id = unsafe { gl.CreateProgram() };
        if program_id == 0 {
            return Err(String::from(
                "Cannot create the opengl program (CreateProgram)",
            ));
        }

        for shader in shaders {
            unsafe {
                gl.AttachShader(program_id, shader.id());
            }
        }

        unsafe {
            gl.LinkProgram(program_id);
        }

        // continue with error handling here
        let mut success = 0;
        unsafe {
            gl.GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        }
        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl.GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let buffer = vec![0u8; len as usize + 1];
            let error: CString = unsafe { CString::from_vec_unchecked(buffer) };
            unsafe {
                gl.GetProgramInfoLog(
                    program_id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut _,
                );
            }

            return Err(error.to_string_lossy().into_owned());
        }

        for shader in shaders {
            unsafe {
                gl.DetachShader(program_id, shader.id());
            }
        }

        Ok(Program {
            id: program_id,
            gl: gl.clone(),
        })
    }

    pub fn from_res(gl: &Gl, res: &Resources, name: &str) -> Result<Program, Error> {
        const POSSIBLE_EXT: [&str; 2] = [".vert", ".frag"];

        let resource_names = POSSIBLE_EXT
            .iter()
            .map(|file_extension| format!("{}{}", name, file_extension))
            .collect::<Vec<String>>();

        let shaders = resource_names
            .iter()
            .map(|resource_name| Shader::from_res(gl, res, resource_name))
            .collect::<Result<Vec<Shader>, Error>>()?;

        Program::from_shaders(gl, &shaders[..]).map_err(|message| Error::LinkError {
            name: name.into(),
            message,
        })
    }

    pub fn set_used(&self) {
        unsafe { self.gl.UseProgram(self.id) }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.id);
        }
    }
}
