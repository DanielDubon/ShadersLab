use nalgebra_glm::{Vec3, Vec4, Mat3, mat4_to_mat3};
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::fragment::Fragment;
use crate::color::Color;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;
use crate::planet_type::PlanetType;

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
  // Transform position
  let position = Vec4::new(
    vertex.position.x,
    vertex.position.y,
    vertex.position.z,
    1.0
  );
  let transformed = uniforms.projection_matrix * uniforms.view_matrix * uniforms.model_matrix * position;

  // Perform perspective division
  let w = transformed.w;
  let ndc_position = Vec4::new(
    transformed.x / w,
    transformed.y / w,
    transformed.z / w,
    1.0
  );

  // apply viewport matrix
  let screen_position = uniforms.viewport_matrix * ndc_position;

  // Transform normal
  let model_mat3 = mat4_to_mat3(&uniforms.model_matrix); 
  let normal_matrix = model_mat3.transpose().try_inverse().unwrap_or(Mat3::identity());

  let transformed_normal = normal_matrix * vertex.normal;

  // Create a new Vertex with transformed attributes
  Vertex {
    position: vertex.position,
    normal: vertex.normal,
    tex_coords: vertex.tex_coords,
    color: vertex.color,
    transformed_position: Vec3::new(screen_position.x, screen_position.y, screen_position.z),
    transformed_normal,
  }
}

pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms, planet_type: &PlanetType) -> Color {
    match planet_type {
        PlanetType::Sun => lava_shader(fragment, uniforms),
        PlanetType::Mercury => mercury_shader(fragment, uniforms),
        PlanetType::Venus => venus_shader(fragment, uniforms),
        PlanetType::Earth => {
            let earth_color = earth_shader(fragment, uniforms);
            let cloud_color = cloud_shader(fragment, uniforms);
            blend_layers(earth_color, cloud_color)
        },
        PlanetType::Moon => moon_shader(fragment, uniforms),
        PlanetType::Mars => mars_shader(fragment, uniforms),
        PlanetType::Jupiter => jupiter_shader(fragment, uniforms),
        PlanetType::Saturn => saturn_shader(fragment, uniforms),
        PlanetType::Uranus => uranus_shader(fragment, uniforms),
        PlanetType::Neptune => neptune_shader(fragment, uniforms),
        PlanetType::BlackHole => black_hole_shader(fragment, uniforms),
    }
}

fn blend_layers(base: Color, clouds: Color) -> Color {
    // Las nubes blancas se mezclan sobre la tierra
    // Si el color de la nube es más oscuro (cielo azul), se ignora
    let cloud_intensity = (
        clouds.get_red() as f32 + 
        clouds.get_green() as f32 + 
        clouds.get_blue() as f32
    ) / (3.0 * 255.0);

    if cloud_intensity > 0.3 { // Reducido el umbral para que más nubes sean visibles
        base.lerp(&clouds, 0.7) // Puedes ajustar la opacidad (0.7) según necesites
    } else {
        base
    }
}

fn random_color_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let seed = uniforms.time as u64;

  let mut rng = StdRng::seed_from_u64(seed);

  let r = rng.gen_range(0..=255);
  let g = rng.gen_range(0..=255);
  let b = rng.gen_range(0..=255);

  let random_color = Color::new(r, g, b);

  random_color * fragment.intensity
}

fn cloud_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 100.0;  // Reducido para nubes más grandes
    let ox = 100.0;
    let oy = 100.0;
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;
    let t = uniforms.time as f32 * 0.1;

    let noise_value = uniforms.noise.get_noise_2d(x * zoom + ox + t, y * zoom + oy);

    // Define cloud threshold and colors
    let cloud_threshold = 0.1; // Reducido para más cobertura
    let cloud_color = Color::new(255, 255, 255);

    let cloud_factor = if noise_value > cloud_threshold {
        ((noise_value - cloud_threshold) / (1.0 - cloud_threshold)).min(1.0)
    } else {
        0.0
    };

    cloud_color * (cloud_factor * fragment.intensity)
}


fn lava_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  // Colores más brillantes y solares
  let bright_color = Color::new(255, 255, 100); // Amarillo brillante casi blanco
  let dark_color = Color::new(255, 140, 0);    // Naranja más brillante

  // Get fragment position
  let position = Vec3::new(
    fragment.vertex_position.x,
    fragment.vertex_position.y,
    fragment.depth
  );

  // Ajustes para movimiento más rápido y dinámico
  let base_frequency = 0.4;  // Aumentado para más movimiento
  let pulsate_amplitude = 0.8;  // Aumentado para más contraste
  let t = uniforms.time as f32 * 0.02;  // Velocidad aumentada

  // Pulsate on the z-axis to change spot size
  let pulsate = (t * base_frequency).sin() * pulsate_amplitude;

  // Zoom reducido para patrones más grandes
  let zoom = 800.0;
  let noise_value1 = uniforms.noise.get_noise_3d(
    position.x * zoom,
    position.y * zoom,
    (position.z + pulsate) * zoom
  );
  let noise_value2 = uniforms.noise.get_noise_3d(
    (position.x + 1000.0) * zoom,
    (position.y + 1000.0) * zoom,
    (position.z + 1000.0 + pulsate) * zoom
  );
  // Ajuste del contraste del ruido
  let noise_value = ((noise_value1 + noise_value2) * 0.5 + 0.2).min(1.0);

  let color = dark_color.lerp(&bright_color, noise_value);

  // Aumentar la intensidad general
  color * fragment.intensity * 1.2
}

fn earth_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Colores más simples y definidos
    let ocean_color = Color::new(25, 80, 180);     // Azul más profundo para océanos
    let land_color = Color::new(50, 160, 80);      // Verde más vivo para continentes
    
    let position = Vec3::new(
        fragment.vertex_position.x,
        fragment.vertex_position.y,
        fragment.depth
    );

    // Un solo nivel de ruido para los continentes
    let zoom = 250.0;  // Ajustado para continentes más grandes
    let noise_value = uniforms.noise.get_noise_3d(
        position.x * zoom,
        position.y * zoom,
        position.z * zoom
    ).abs();  // Usar valor absoluto para evitar valores negativos

    // Umbral más definido para la separación tierra/agua
    let threshold = 0.5;
    let transition_width = 0.1;

    // Transición suave entre tierra y agua
    let land_factor = if noise_value < (threshold - transition_width) {
        0.0  // Océano
    } else if noise_value > (threshold + transition_width) {
        1.0  // Tierra
    } else {
        // Transición suave en los bordes
        ((noise_value - (threshold - transition_width)) / (transition_width * 2.0))
    };

    // Mezclar colores
    let base_color = ocean_color.lerp(&land_color, land_factor);

    // Efecto simple de atmósfera en los bordes
    let atmosphere_color = Color::new(150, 200, 255);
    let normal_dot = fragment.normal.dot(&Vec3::new(0.0, 0.0, 1.0));
    let atmosphere_factor = (1.0 - normal_dot.abs()).powf(2.0);
    
    let final_color = base_color.lerp(&atmosphere_color, atmosphere_factor * 0.4);
    
    final_color * fragment.intensity
}

fn mercury_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Colores grisáceos y marrones para el terreno rocoso
    let dark_color = Color::new(80, 75, 70);    // Gris oscuro
    let light_color = Color::new(170, 160, 150); // Gris claro
    let crater_color = Color::new(60, 55, 50);   // Gris más oscuro para cráteres
    
    let position = fragment.vertex_position;
    let zoom = 300.0;
    
    // Ruido base para el terreno
    let terrain = uniforms.noise.get_noise_3d(
        position.x * zoom,
        position.y * zoom,
        position.z * zoom
    ).abs();
    
    // Ruido adicional para cráteres
    let crater_zoom = 600.0;
    let craters = uniforms.noise.get_noise_3d(
        position.x * crater_zoom,
        position.y * crater_zoom,
        position.z * crater_zoom
    ).abs();
    
    let base_color = dark_color.lerp(&light_color, terrain);
    let final_color = if craters > 0.7 {
        base_color.lerp(&crater_color, 0.5)
    } else {
        base_color
    };
    
    final_color * fragment.intensity
}

fn venus_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Colores amarillentos y naranjas para la densa atmósfera
    let base_color = Color::new(230, 180, 50);    // Amarillo
    let cloud_color = Color::new(255, 198, 88);   // Naranja claro
    
    let position = fragment.vertex_position;
    let t = uniforms.time as f32 * 0.05;  // Movimiento lento de nubes
    
    // Patrones de nubes en movimiento
    let cloud_zoom = 150.0;
    let clouds = uniforms.noise.get_noise_3d(
        position.x * cloud_zoom + t,
        position.y * cloud_zoom,
        position.z * cloud_zoom
    ).abs();
    
    let final_color = base_color.lerp(&cloud_color, clouds);
    
    // Efecto de atmósfera densa
    let atmosphere_factor = (1.0 - fragment.normal.dot(&Vec3::new(0.0, 0.0, 1.0))).powf(0.5);
    let atmosphere_color = Color::new(255, 220, 150);
    
    final_color.lerp(&atmosphere_color, atmosphere_factor * 0.3) * fragment.intensity
}

fn mars_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Colores rojizos característicos de Marte
    let dark_red = Color::new(145, 50, 20);    // Rojo oscuro
    let light_red = Color::new(200, 80, 30);   // Rojo claro
    let dust_color = Color::new(230, 130, 50);  // Color polvo marciano
    
    let position = fragment.vertex_position;
    let zoom = 250.0;
    
    // Terreno base
    let terrain = uniforms.noise.get_noise_3d(
        position.x * zoom,
        position.y * zoom,
        position.z * zoom
    ).abs();
    
    // Patrones de polvo
    let dust_zoom = 400.0;
    let dust = uniforms.noise.get_noise_3d(
        position.x * dust_zoom,
        position.y * dust_zoom,
        position.z * dust_zoom
    ).abs();
    
    let base_color = dark_red.lerp(&light_red, terrain);
    let final_color = base_color.lerp(&dust_color, dust * 0.3);
    
    final_color * fragment.intensity
}

fn jupiter_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Colores para las bandas de Júpiter
    let light_band = Color::new(255, 220, 180);  // Banda clara
    let dark_band = Color::new(180, 140, 100);   // Banda oscura
    let storm_color = Color::new(255, 160, 120); // Color para la Gran Mancha Roja
    
    let position = fragment.vertex_position;
    let t = uniforms.time as f32 * 0.1;
    
    // Bandas horizontales
    let band_zoom = 100.0;
    let bands = uniforms.noise.get_noise_2d(
        position.y * band_zoom,
        t
    ).abs();
    
    // Turbulencia adicional
    let turb_zoom = 300.0;
    let turbulence = uniforms.noise.get_noise_3d(
        position.x * turb_zoom + t,
        position.y * turb_zoom,
        position.z * turb_zoom
    ).abs();
    
    let base_color = dark_band.lerp(&light_band, bands);
    let final_color = base_color.lerp(&storm_color, turbulence * 0.3);
    
    final_color * fragment.intensity
}

fn saturn_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Colores para Saturno y sus anillos
    let planet_light = Color::new(255, 240, 200);  // Color claro del planeta
    let planet_dark = Color::new(200, 180, 140);   // Color oscuro del planeta
    let ring_light = Color::new(210, 190, 170);    // Color claro del anillo
    let ring_dark = Color::new(160, 140, 120);     // Color oscuro del anillo
    
    let position = fragment.vertex_position;
    let normal = fragment.normal;
    
    // Calcular distancia desde el centro
    let radius = (position.x * position.x + position.z * position.z).sqrt();
    let y_abs = position.y.abs();
    
    // Definir los parámetros del anillo
    let ring_inner = 1.2;    // Donde comienza el anillo (justo fuera del planeta)
    let ring_outer = 2.5;    // Donde termina el anillo
    let ring_thickness = 0.1; // Grosor del anillo
    
    // Determinar si estamos en el anillo
    let in_ring = radius >= ring_inner && 
                  radius <= ring_outer && 
                  y_abs <= ring_thickness;
    
    if in_ring {
        // Patrón de anillos concéntricos
        let ring_pattern = ((radius * 20.0).sin() * 0.5 + 0.5).abs();
        
        // Variación adicional en los anillos
        let detail = uniforms.noise.get_noise_2d(
            radius * 15.0,
            position.z.atan2(position.x) * 5.0
        ).abs();
        
        // Combinar patrones
        let ring_factor = ring_pattern * 0.7 + detail * 0.3;
        
        // Color final del anillo
        let ring_color = ring_light.lerp(&ring_dark, ring_factor);
        
        // Aplicar sombreado basado en la normal
        let light_factor = normal.dot(&Vec3::new(0.0, 1.0, 0.0)).abs();
        ring_color * fragment.intensity * light_factor.max(0.2)
    } else {
        // Color del planeta con bandas
        let t = uniforms.time as f32 * 0.08;
        let bands = uniforms.noise.get_noise_2d(
            position.y * 120.0,
            t
        ).abs();
        
        planet_light.lerp(&planet_dark, bands) * fragment.intensity
    }
}

fn uranus_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Tonos azul-verdosos característicos
    let base_color = Color::new(150, 210, 230);  // Azul verdoso claro
    let cloud_color = Color::new(180, 230, 255); // Azul más claro
    
    let position = fragment.vertex_position;
    let t = uniforms.time as f32 * 0.03;
    
    // Patrones de nubes suaves
    let cloud_zoom = 200.0;
    let clouds = uniforms.noise.get_noise_3d(
        position.x * cloud_zoom + t,
        position.y * cloud_zoom,
        position.z * cloud_zoom
    ).abs();
    
    let final_color = base_color.lerp(&cloud_color, clouds * 0.4);
    
    final_color * fragment.intensity
}

fn neptune_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Azul intenso característico
    let base_color = Color::new(30, 100, 200);   // Azul profundo
    let storm_color = Color::new(100, 160, 255); // Azul más claro para tormentas
    
    let position = fragment.vertex_position;
    let t = uniforms.time as f32 * 0.06;
    
    // Patrones de tormentas
    let storm_zoom = 250.0;
    let storms = uniforms.noise.get_noise_3d(
        position.x * storm_zoom + t,
        position.y * storm_zoom,
        position.z * storm_zoom
    ).abs();
    
    // Bandas sutiles
    let band_zoom = 150.0;
    let bands = uniforms.noise.get_noise_2d(
        position.y * band_zoom,
        t
    ).abs();
    
    let final_color = base_color.lerp(&storm_color, (storms + bands * 0.5) * 0.4);
    
    final_color * fragment.intensity
}

fn moon_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Colores base para la luna
    let dark_color = Color::new(100, 100, 100);   // Gris oscuro
    let light_color = Color::new(200, 200, 200);  // Gris claro
    let crater_color = Color::new(80, 80, 80);    // Gris más oscuro para cráteres
    
    let position = fragment.vertex_position;
    let zoom = 400.0;
    
    // Ruido base para el terreno lunar
    let terrain = uniforms.noise.get_noise_3d(
        position.x * zoom,
        position.y * zoom,
        position.z * zoom
    ).abs();
    
    // Ruido adicional para cráteres
    let crater_zoom = 800.0;
    let craters = uniforms.noise.get_noise_3d(
        position.x * crater_zoom,
        position.y * crater_zoom,
        position.z * crater_zoom
    ).abs();
    
    let base_color = dark_color.lerp(&light_color, terrain);
    let final_color = if craters > 0.7 {
        base_color.lerp(&crater_color, 0.5)
    } else {
        base_color
    };
    
    final_color * fragment.intensity
}

fn black_hole_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let position = fragment.vertex_position;
    
    // Colores psicodélicos
    let core_color = Color::new(0, 0, 0);           // Centro negro
    let inner_color = Color::new(255, 0, 255);      // Magenta brillante
    let outer_color = Color::new(147, 0, 255);      // Morado
    let space_color = Color::new(75, 0, 130);       // Índigo oscuro
    
    // Calcular distancia desde el centro
    let radius = (position.x * position.x + position.z * position.z).sqrt();
    
    // Tiempo para animación
    let t = uniforms.time as f32 * 0.05;
    
    // Efecto de vórtice
    let angle = position.z.atan2(position.x) + t;
    let spiral = (angle * 5.0 + radius * 10.0 + t).sin() * 0.5 + 0.5;
    
    // Efecto de pulso
    let pulse = (t * 2.0).sin() * 0.5 + 0.5;
    
    // Distorsión del espacio
    let distortion = 1.0 / (radius + 0.5);
    
    // Patrones de ruido para más detalle
    let noise = uniforms.noise.get_noise_3d(
        position.x * 2.0 + t,
        position.y * 2.0,
        position.z * 2.0 - t
    ).abs();
    
    // Combinar efectos
    let effect = (spiral + noise + pulse) / 3.0;
    
    if radius < 0.3 {
        // Centro del agujero negro (siempre negro)
        core_color
    } else if radius < 1.0 {
        // Región interna psicodélica
        let factor = ((radius - 0.3) / 0.7).powf(0.5);
        let base = core_color.lerp(&inner_color, factor * effect);
        base * (distortion * 0.5)
    } else if radius < 2.0 {
        // Región externa con vórtice
        let factor = ((radius - 1.0) / 1.0).powf(0.5);
        let base = inner_color.lerp(&outer_color, factor * effect);
        base * ((3.0 - radius) * 0.5)
    } else {
        // Espacio exterior con distorsión
        let fade = (1.0 / (radius - 1.5)).min(1.0);
        outer_color.lerp(&space_color, fade) * (0.5 * fade)
    }
}