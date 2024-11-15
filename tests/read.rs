extern crate ply_rs;
use ply_rs::*;

type Ply = ply::Ply<ply::DefaultElement>;

async fn read_file(path: &str) -> Ply {
    let mut f = tokio::fs::File::open(path).await.unwrap();
    let p = parser::Parser::new();
    let ply = p.read_ply(&mut f).await;
    assert!(ply.is_ok(), "failed: {}", ply.err().unwrap());
    ply.unwrap()
}

#[tokio::test]
async fn read_empty() {
    let ply = read_file("example_plys/empty_ok_ascii.ply").await;
    assert_eq!(
        ply.header
            .elements
            .iter()
            .find(|x| x.name == "face")
            .unwrap()
            .count,
        0
    );
    assert!(ply.payload["vertex"].is_empty());
    assert!(ply.payload["face"].is_empty());
}
#[tokio::test]
async fn read_empy_equal() {
    let bin = read_file("example_plys/empty_2_ok_little_endian.ply").await;
    let ascii = read_file("example_plys/empty_2_ok_ascii.ply").await;
    assert_eq!(bin.header.elements, ascii.header.elements);
    assert_eq!(bin.payload, ascii.payload);
}
#[tokio::test]
async fn read_house() {
    let ply = read_file("example_plys/house_ok_ascii.ply").await;
    println!("Created ply: {:?}", ply);
    assert_eq!(
        ply.header
            .elements
            .iter()
            .find(|x| x.name == "face")
            .unwrap()
            .count,
        3
    );
    assert_eq!(ply.payload["vertex"].len(), 5);
    assert_eq!(ply.payload["face"].len(), 3);
}
#[tokio::test]
async fn read_house_equal() {
    let bin = read_file("example_plys/house_2_ok_little_endian.ply").await;
    let ascii = read_file("example_plys/house_2_ok_ascii.ply").await;
    assert_eq!(bin.header.elements, ascii.header.elements);
    assert_eq!(bin.payload, ascii.payload);
}
#[tokio::test]
async fn read_greg_turk_1() {
    let ply = read_file("example_plys/greg_turk_example1_ok_ascii.ply").await;
    println!("Created ply: {:?}", ply);
}
#[tokio::test]
async fn read_greg_turk_2() {
    let ply = read_file("example_plys/greg_turk_example2_ok_ascii.ply").await;
    println!("Created ply: {:?}", ply);
}
#[tokio::test]
async fn read_exponent_values_ok() {
    let ply = read_file("example_plys/exponent_values_ok_ascii.ply").await;
    println!("Created ply: {:?}", ply);
}
#[tokio::test]
async fn read_leading_spaces_ok() {
    let ply = read_file("example_plys/leading_spaces_ok_ascii.ply").await;
    println!("Created ply: {:?}", ply);
}
#[tokio::test]
async fn read_all_atomic_types_ok() {
    let ply = read_file("example_plys/all_atomic_types_ok_ascii.ply").await;
    println!("Created ply: {:?}", ply);
}

mod struct_test_1 {
    use super::parser::Parser;
    use super::ply;
    use super::read_file;

    #[derive(Debug)]
    struct Vertex {
        x: f32,
        y: f32,
        z: f32,
    }

    #[derive(Debug)]
    struct Face {
        vertex_index: Vec<i32>,
    }

    impl ply::PropertyAccess for Vertex {
        fn new() -> Self {
            Vertex {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }
        }

        fn set_property(&mut self, key: &str, property: ply::Property) {
            match (key, property) {
                ("x", ply::Property::Float(v)) => self.x = v,
                ("y", ply::Property::Float(v)) => self.y = v,
                ("z", ply::Property::Float(v)) => self.z = v,
                (k, _) => panic!("Vertex: Unexpected key/value combination: key: {}", k),
            }
        }
    }

    // same thing for Face
    impl ply::PropertyAccess for Face {
        fn new() -> Self {
            Face {
                vertex_index: Vec::new(),
            }
        }

        fn set_property(&mut self, key: &str, property: ply::Property) {
            match (key, property) {
                ("vertex_index", ply::Property::ListInt(vec)) => self.vertex_index = vec,
                (k, _) => panic!("Face: Unexpected key/value combination: key: {}", k),
            }
        }
    }

    /// Demonstrates simplest use case for reading from a file.
    #[tokio::test]
    async fn read_into_struct() {
        // set up a reader, in this a file.
        let path = "example_plys/greg_turk_example1_ok_ascii.ply";

        let f = tokio::fs::File::open(path).await.unwrap();
        // The header of a ply file consists of ascii lines, BufRead provides useful methods for that.
        let mut f = tokio::io::BufReader::new(f);

        // Create a parser for each struct. Parsers are cheap objects.
        let vertex_parser = Parser::<Vertex>::new();
        let face_parser = Parser::<Face>::new();

        // lets first consume the header
        // We also could use `face_parser`, The configuration is a parser's only state.
        // The reading position only depends on `f`.
        let header = vertex_parser.read_header(&mut f).await.unwrap();

        // Depending on the header, read the data into our structs..
        let mut vertex_list = Vec::new();
        let mut face_list = Vec::new();
        for element in &header.elements {
            // we could also just parse them in sequence, but the file format might change
            match element.name.as_ref() {
                "vertex" => {
                    vertex_list = vertex_parser
                        .read_payload_for_element(&mut f, element, &header)
                        .await
                        .unwrap();
                }
                "face" => {
                    face_list = face_parser
                        .read_payload_for_element(&mut f, element, &header)
                        .await
                        .unwrap();
                }
                _ => panic!("Enexpeced element!"),
            }
        }

        println!("header: {:#?}", header);
        println!("vertex list: {:#?}", vertex_list);
        println!("face list: {:#?}", face_list);

        let ply = read_file(path).await;

        for (i, vert) in vertex_list.iter().enumerate() {
            let x = match ply.payload["vertex"][i]["x"] {
                ply::Property::Float(v) => v,
                _ => panic!("Unexpected property."),
            };
            assert_eq!(vert.x, x);
            let y = match ply.payload["vertex"][i]["y"] {
                ply::Property::Float(v) => v,
                _ => panic!("Unexpected property."),
            };
            assert_eq!(vert.y, y);
            let z = match ply.payload["vertex"][i]["z"] {
                ply::Property::Float(v) => v,
                _ => panic!("Unexpected property."),
            };
            assert_eq!(vert.z, z);
        }

        for (i, face) in face_list.iter().enumerate() {
            let v = match ply.payload["face"][i]["vertex_index"] {
                ply::Property::ListInt(ref v) => v,
                _ => panic!("Unexpected property."),
            };
            assert_eq!(face.vertex_index, *v);
        }
    }
}
