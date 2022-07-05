use tokio::process::Command;
use tar;
use bollard::{self, container::{CreateContainerOptions, UploadToContainerOptions}, models::Config};

pub async fn testing() {
    let docker = bollard::Docker::connect_with_socket_defaults().unwrap();

    let cco = CreateContainerOptions {
        name: "rust_docker_container"
    };

    let config = bollard::container::Config {
        image: Some("test_cont"),
        ..Default::default()
    };

    let created_cont = docker.create_container(Some(cco), config).await.unwrap();
    
    println!("CREATED = {:?}", created_cont);

    let file_cont = tokio::fs::read("/home/fuchczyk/Programming/alsit/src/crypto.rs")
        .await
        .unwrap();

    let mut header = tar::Header::new_gnu();
    header.set_path("testfile.rs").unwrap();
    header.set_size(file_cont.len().try_into().unwrap());
    header.set_cksum();

    let mut upload_cont: Vec<u8> = Vec::new();
    let mut builder = tar::Builder::new(&mut upload_cont);
    builder.append(&header, file_cont.as_slice()).unwrap();
   
    let data = builder.into_inner().unwrap();

    let upload_options = UploadToContainerOptions {
        path: "/udana_sciezka",
        ..Default::default()
    };
    
    let x = docker.upload_to_container("rust_docker_container", Some(upload_options), upload_cont.into()).await;
    x.unwrap();
}