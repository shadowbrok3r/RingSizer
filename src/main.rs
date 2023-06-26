extern crate wavefront_obj;

use std::io;
use std::path::{Path, PathBuf};
use wavefront_obj::obj::{parse, ObjSet};
use wavefront_obj::obj::Object;

fn main() {
    // Prompt the user for the path to the .obj file
    println!("Enter the path to the .obj file:");
    let mut obj_path = String::new();
    io::stdin().read_line(&mut obj_path).expect("Failed to read input");
    let obj_path = obj_path.trim();

    // Read the .obj file
    let input_data = std::fs::read_to_string(&obj_path).expect("Failed to read file");

    // Parse the .obj file
    let mut obj_set: ObjSet = parse(input_data).expect("Failed to parse .obj file");

    // Access the parsed data
    let object = &mut obj_set.objects[0]; // Assuming there's only one object in the file
    let vertices = &mut object.vertices;

    // Prompt the user for the target US ring size
    println!("Enter the target US ring size (e.g., 7.25):");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");

    // Parse the input as a floating-point number
    let target_ring_size: f32 = match input.trim().parse() {
        Ok(size) => size,
        Err(_) => {
            println!("Invalid input. Please enter a valid target US ring size (e.g., 7.25).");
            return;
        }
    };

    // Ensure only one mesh exists in the .obj file
    if obj_set.objects.len() != 1 {
        println!("Error: The .obj file should contain only one mesh.");
        return;
    }

    // Access the parsed data
    let object = &obj_set.objects[0];
    let vertices = &object.vertices;

    // Check if the mesh is non-manifold or contains holes
    let is_non_manifold = check_non_manifold(vertices);
    let has_holes = check_holes(vertices);

    if is_non_manifold {
        println!("Error: The mesh is non-manifold.");
        return;
    }

    if has_holes {
        println!("Error: The mesh contains holes.");
        return;
    }


    // Calculate the current inside diameter
    let current_diameter_mm = calculate_diameter(&vertices);

    // Calculate the scaling factor
    let scaling_factor = target_ring_size / current_diameter_mm;

    // Scale the vertices
    for vertex in vertices.iter_mut() {
        vertex.x *= scaling_factor.into();
        vertex.y *= scaling_factor.into();
        vertex.z *= scaling_factor.into();
    }

    // Construct the output file path
    let output_dir = Path::new(&obj_path).parent().unwrap_or_else(|| Path::new("."));
    let file_name = Path::new(&obj_path).file_name().unwrap().to_string_lossy();
    let output_file_name = format!(
        "{}_size-{:.2}.obj",
        file_name,
        target_ring_size
    );
    let output_path = output_dir.join(output_file_name);

    // Update the .obj file with the scaled vertices
    let updated_data = obj_set;

    // Save the updated .obj file
    std::fs::write(&output_path, updated_data).expect("Failed to write output file");

    println!("Object scaled and saved to: {:?}", output_path);
}

fn calculate_diameter(vertices: &[wavefront_obj::obj::Vertex]) -> f32 {
    // Find the minimum and maximum radii
    let mut min_radius = f32::MAX;
    let mut max_radius = f32::MIN;

    // Process the vertices
    for vertex in vertices.iter() {
        let radius = (vertex.x.powi(2) + vertex.y.powi(2)).sqrt();
        if radius < min_radius.into() {
            min_radius = radius.floor();
        }
        if radius > max_radius.into() {
            max_radius = radius.into();
        }
    }

    // Calculate the inside diameter
    let inside_diameter_mm = (max_radius - min_radius) * 2.0;

    inside_diameter_mm
}

fn check_non_manifold(vertices: &[wavefront_obj::obj::Vertex]) -> bool {
    // Check for non-manifold conditions
    // Iterate over the faces and check if any vertex is shared by more than two faces
    for i in 0..vertices.len() {
        let vertex = &vertices[i];
        let mut face_count = 0;
        for face in vertices.iter() {
            if face.x == vertex.x && face.y == vertex.y && face.z == vertex.z {
                face_count += 1;
            }
        }
        if face_count > 2 {
            return true; // Non-manifold condition found
        }
    }
    false // No non-manifold condition found
}

fn check_holes(object: &Object) -> bool {
    let faces = &object.geometry[0].shapes;
    let mut visited: Vec<bool> = vec![false; object.vertices.len()];

    // Perform depth-first search on the mesh to check for holes
    for face in faces.iter() {
        match face {
            wavefront_obj::obj::Primitive::Triangle((v1, _, _), (v2, _, _), (v3, _, _)) => {
                visited[v1.0] = true;
                visited[v2.0] = true;
                visited[v3.0] = true;
            }
            _ => {
                println!("Error: Unsupported face type.");
                return false;
            }
        }
    }

    // Check if any vertices were not visited, indicating a hole
    for i in 0..visited.len() {
        if !visited[i] {
            return true; // Hole found
        }
    }
    false // No holes found
}