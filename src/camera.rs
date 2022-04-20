use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

use crate::deg_to_rad;

pub struct Camera {
    pub eye: glam::Vec3,
    pub up: glam::Vec3,
    pub front: glam::Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn build_vp_matrix(&self) -> glam::Mat4 {
        let view = glam::Mat4::look_at_rh(self.eye, self.eye + self.front, self.up);
        let proj =
            glam::Mat4::perspective_rh(deg_to_rad(self.fovy), self.aspect, self.znear, self.zfar);

        proj * view
    }
}

pub struct CameraController {
    speed: f32,
    sensitivity: f32,

    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,

    yaw: f32,
    pitch: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity,

            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,

            yaw: 0.0,
            pitch: 0.0,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Q => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::E => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }

            _ => false,
        }
    }

    pub fn cursor_move(&mut self, position_delta: glam::Vec2) {
        self.yaw += position_delta.x * self.sensitivity;
        self.pitch -= position_delta.y * self.sensitivity;

        if self.pitch > 89.0 {
            self.pitch = 89.0;
        }
        if self.pitch < -89.0 {
            self.pitch = -89.0;
        }
    }

    pub fn update_camera(&self, camera: &mut Camera, delta_time: f32) {
        let new_front = glam::vec3(
            deg_to_rad(self.yaw).cos() * deg_to_rad(self.pitch).cos(),
            deg_to_rad(self.pitch).sin(),
            deg_to_rad(self.yaw).sin() * deg_to_rad(self.pitch).cos(),
        );
        camera.front = new_front.normalize();
        // println!("{:?}, ({}, {})", camera.front, self.yaw, self.pitch);
        let right = camera.front.cross(camera.up);

        if self.is_forward_pressed {
            camera.eye += camera.front * self.speed * delta_time;
        }
        if self.is_backward_pressed {
            camera.eye -= camera.front * self.speed * delta_time;
        }
        if self.is_right_pressed {
            camera.eye += right * self.speed * delta_time;
        }
        if self.is_left_pressed {
            camera.eye -= right * self.speed * delta_time;
        }
        if self.is_up_pressed {
            camera.eye += camera.up * self.speed * delta_time;
        }
        if self.is_down_pressed {
            camera.eye -= camera.up * self.speed * delta_time;
        }
    }
}
