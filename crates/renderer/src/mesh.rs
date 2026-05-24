use std::fs;
use std::path::Path;

use anyhow::Context;

use crate::math::Vec3;

#[derive(Debug, Clone)]
pub struct Mesh {
    pub positions: Vec<Vertex>,
    pub indices: Vec<usize>,
}

#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub position: Vec3,
}

impl Default for Mesh {
    fn default() -> Self {
        Self { positions: Vec::new(), indices: Vec::new() }
    }
}

impl Mesh {
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path).with_context(|| format!("failed to read .obj file {}", path.display()))?;
        Self::parse(&contents)
    }

    pub fn parse(text: &str) -> anyhow::Result<Self> {
        let mut mesh = Mesh::default();

        for line in text.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with("#") {
                continue
            }

            let mut parts = line.split_whitespace();

            match parts.next() {
                Some("v") => {
                    parse_position(parts, &mut mesh);
                }
                Some("f") => {
                    parse_face(parts, &mut mesh);
                }
                _ => {}
            }
        }

        Ok(mesh)
    }
}

fn parse_position<'a>(mut parts: impl Iterator<Item = &'a str>, mesh: &mut Mesh) -> anyhow::Result<()> {
    let x = parts.next().ok_or(anyhow::anyhow!("missing data"))?.parse()?;
    let y = parts.next().ok_or(anyhow::anyhow!("missing data"))?.parse()?;
    let z = parts.next().ok_or(anyhow::anyhow!("missing data"))?.parse()?;

    mesh.positions.push(Vertex { position: Vec3::new(x, y, z) });

    Ok(())
}

fn parse_face<'a>(parts: impl Iterator<Item = &'a str>, mesh: &mut Mesh) -> anyhow::Result<()> {

    for vertex in parts {
        let mut segments = vertex.split("/");
        let position: usize = segments.next().ok_or(anyhow::anyhow!("missing data"))?.parse()?;

        mesh.indices.push(position - 1);
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_vertex() {
        let obj = r#"v 1.0 2.0 3.0"#;

        let mesh = Mesh::parse(obj);

        assert!(mesh.is_ok());

        let mesh = mesh.unwrap();

        assert_eq!(mesh.positions.len(), 1);
        assert_eq!(mesh.positions[0].position, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn parse_simple_face() {
        let obj = r#"
        v 0.0 0.0 0.0
        v 1.0 0.0 0.0
        v 0.0 1.0 0.0

        f 1 2 3
        "#;

        let mesh = Mesh::parse(obj);
        assert!(mesh.is_ok());

        let mesh = mesh.unwrap();

        assert_eq!(mesh.indices.len(), 3);
        assert_eq!(mesh.positions.len(), 3);

        assert_eq!(mesh.indices[0], 0);
        assert_eq!(mesh.indices[1], 1);
        assert_eq!(mesh.indices[2], 2);
    }

    #[test]
    fn parse_normals_face() {
        let obj = r#"
        v 0.0 0.0 0.0
        v 1.0 0.0 0.0
        v 0.0 1.0 0.0

        f 1//1 2//1 3//1
        "#;

        let mesh = Mesh::parse(obj);
        assert!(mesh.is_ok());

        let mesh = mesh.unwrap();

        assert_eq!(mesh.indices.len(), 3);
        assert_eq!(mesh.positions.len(), 3);

        assert_eq!(mesh.indices[0], 0);
        assert_eq!(mesh.indices[1], 1);
        assert_eq!(mesh.indices[2], 2);
    }

    #[test]
    fn ignores_comments() {
        let obj = r#"
        v 0.0 0.0 0.0
        # comment
        v 1.0 0.0 0.0
        v 0.0 1.0 0.0
        # another comment
        f 1 2 3
        "#;

        let mesh = Mesh::parse(obj);
        assert!(mesh.is_ok());

        let mesh = mesh.unwrap();

        assert_eq!(mesh.indices.len(), 3);
        assert_eq!(mesh.positions.len(), 3);

        assert_eq!(mesh.indices[0], 0);
        assert_eq!(mesh.indices[1], 1);
        assert_eq!(mesh.indices[2], 2);
    }
}
