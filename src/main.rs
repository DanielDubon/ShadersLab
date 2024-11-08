use nalgebra_glm::{Vec3, Mat4, look_at, perspective};
use minifb::{Key, Window, WindowOptions};
use std::f32::consts::PI;

mod framebuffer;
mod triangle;
mod vertex;
mod obj;
mod color;
mod fragment;
mod shaders;
mod camera;
mod planet_type;

use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use camera::Camera;
use triangle::triangle;
use shaders::{vertex_shader, fragment_shader};
use fastnoise_lite::{FastNoiseLite, NoiseType, FractalType};
use planet_type::PlanetType;

pub struct CelestialBody {
    position: Vec3,
    scale: f32,
    rotation: Vec3,
    shader_type: PlanetType,
}

pub struct Uniforms {
    model_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    viewport_matrix: Mat4,
    time: u32,
    noise: FastNoiseLite
}

fn create_noise() -> FastNoiseLite {
    create_cloud_noise() 
}

fn create_cloud_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise
}

fn create_cell_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::Cellular));
    noise.set_frequency(Some(0.1));
    noise
}

fn create_ground_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    
   
    noise.set_noise_type(Some(NoiseType::Cellular)); 
    noise.set_fractal_type(Some(FractalType::FBm)); 
    noise.set_fractal_octaves(Some(5));              
    noise.set_fractal_lacunarity(Some(2.0));         
    noise.set_fractal_gain(Some(0.5));               
    noise.set_frequency(Some(0.05));                 

    noise
}

fn create_lava_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(42);
    
 
    noise.set_noise_type(Some(NoiseType::Perlin));  
    noise.set_fractal_type(Some(FractalType::FBm)); 
    noise.set_fractal_octaves(Some(6));            
    noise.set_fractal_lacunarity(Some(2.0));       
    noise.set_fractal_gain(Some(0.5));              
    noise.set_frequency(Some(0.002));                
    
    noise
}

fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    let rotation_matrix_x = Mat4::new(
        1.0,  0.0,    0.0,   0.0,
        0.0,  cos_x, -sin_x, 0.0,
        0.0,  sin_x,  cos_x, 0.0,
        0.0,  0.0,    0.0,   1.0,
    );

    let rotation_matrix_y = Mat4::new(
        cos_y,  0.0,  sin_y, 0.0,
        0.0,    1.0,  0.0,   0.0,
        -sin_y, 0.0,  cos_y, 0.0,
        0.0,    0.0,  0.0,   1.0,
    );

    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0,
        sin_z,  cos_z, 0.0, 0.0,
        0.0,    0.0,  1.0, 0.0,
        0.0,    0.0,  0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    let transform_matrix = Mat4::new(
        scale, 0.0,   0.0,   translation.x,
        0.0,   scale, 0.0,   translation.y,
        0.0,   0.0,   scale, translation.z,
        0.0,   0.0,   0.0,   1.0,
    );

    transform_matrix * rotation_matrix
}


fn create_view_matrix(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    look_at(&eye, &center, &up)
}

fn create_perspective_matrix(window_width: f32, window_height: f32) -> Mat4 {
    let fov = 45.0 * PI / 180.0;
    let aspect_ratio = window_width / window_height;
    let near = 0.1;
    let far = 1000.0;

    perspective(fov, aspect_ratio, near, far)
}

fn create_viewport_matrix(width: f32, height: f32) -> Mat4 {
    Mat4::new(
        width / 2.0, 0.0, 0.0, width / 2.0,
        0.0, -height / 2.0, 0.0, height / 2.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    )
}

fn render(
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms,
    vertex_array: &[Vertex],
    planet_type: &PlanetType
) {
    // Vertex Shader Stage
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    // Primitive Assembly Stage
    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }

 
    let mut fragments = Vec::new();
    for tri in &triangles {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2]));
    }

    
    for fragment in fragments {
        let x = fragment.position.x as usize;
        let y = fragment.position.y as usize;
        if x < framebuffer.width && y < framebuffer.height {
            
            let shaded_color = fragment_shader(&fragment, &uniforms, planet_type);
            let color = shaded_color.to_hex();
            framebuffer.set_current_color(color);
            framebuffer.point(x, y, fragment.depth);
        }
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Rust Graphics - Renderer Example",
        window_width,
        window_height,
        WindowOptions::default(),
    )
        .unwrap();

    window.set_position(500, 500);
    window.update();

    framebuffer.set_background_color(0x333355);

    
    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0)
    );

    let obj = Obj::load("assets/models/esfera.obj").expect("Failed to load obj");
    let vertex_arrays = obj.get_vertex_array(); 
    let mut time = 0;

    let noise = create_noise();
    let projection_matrix = create_perspective_matrix(window_width as f32, window_height as f32);
    let viewport_matrix = create_viewport_matrix(framebuffer_width as f32, framebuffer_height as f32);
    let mut uniforms = Uniforms { 
        model_matrix: Mat4::identity(), 
        view_matrix: Mat4::identity(), 
        projection_matrix, 
        viewport_matrix, 
        time: 0, 
        noise
    };

    
    let celestial_bodies = vec![
        CelestialBody {
            position: Vec3::new(0.0, 0.0, 0.0),
            scale: 2.0,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::Sun,
        },
        CelestialBody {
            position: Vec3::new(6.0, 0.0, 0.0),
            scale: 0.4,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::Mercury,
        },
        CelestialBody {
            position: Vec3::new(12.0, 0.0, 0.0),
            scale: 0.6,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::Venus,
        },
        CelestialBody {
            position: Vec3::new(18.0, 0.0, 0.0),
            scale: 0.7,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::Earth,
        },
        CelestialBody {
            position: Vec3::new(24.0, 0.0, 0.0),
            scale: 0.5,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::Mars,
        },
        CelestialBody {
            position: Vec3::new(32.0, 0.0, 0.0),
            scale: 1.5,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::Jupiter,
        },
        CelestialBody {
            position: Vec3::new(40.0, 0.0, 0.0),
            scale: 1.3,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::Saturn,
        },
        CelestialBody {
            position: Vec3::new(48.0, 0.0, 0.0),
            scale: 0.9,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::Uranus,
        },
        CelestialBody {
            position: Vec3::new(56.0, 0.0, 0.0),
            scale: 0.9,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::Neptune,
        },
    ];

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        time += 1;

        handle_input(&window, &mut camera);

        framebuffer.clear();

        // Renderizar cada cuerpo celeste
        for body in &celestial_bodies {
            uniforms.model_matrix = create_model_matrix(
                body.position,
                body.scale,
                body.rotation + Vec3::new(0.0, time as f32 * 0.01, 0.0)
            );
            uniforms.view_matrix = create_view_matrix(camera.eye, camera.center, camera.up);
            uniforms.time = time;
            
            render(&mut framebuffer, &uniforms, &vertex_arrays, &body.shader_type);
        }

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();
    }
}

fn handle_input(window: &Window, camera: &mut Camera) {
    let movement_speed = 0.5;
    let rotation_speed = PI/50.0;
    let zoom_speed = 1.0;

    // Rotación de la cámara (mirando arriba/abajo)
    if window.is_key_down(Key::Up) {
        camera.rotate_pitch(-rotation_speed);
    }
    if window.is_key_down(Key::Down) {
        camera.rotate_pitch(rotation_speed);
    }

    // Movimiento WASD (adelante, izquierda, atrás, derecha)
    let forward = camera.get_forward() * movement_speed;
    let right = camera.get_right() * movement_speed;
    
    if window.is_key_down(Key::W) {
        camera.move_forward(movement_speed);
    }
    if window.is_key_down(Key::S) {
        camera.move_forward(-movement_speed);
    }
    if window.is_key_down(Key::A) {
        camera.move_right(-movement_speed);
    }
    if window.is_key_down(Key::D) {
        camera.move_right(movement_speed);
    }

    // Movimiento vertical (Q para subir, E para bajar)
    if window.is_key_down(Key::Q) {
        camera.move_up(movement_speed);
    }
    if window.is_key_down(Key::E) {
        camera.move_up(-movement_speed);
    }

    // Zoom (1 para acercar, 2 para alejar)
    if window.is_key_down(Key::Key1) {
        camera.zoom(zoom_speed);
    }
    if window.is_key_down(Key::Key2) {
        camera.zoom(-zoom_speed);
    }
}
