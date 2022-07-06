use bollard::{
    self,
    container::{CreateContainerOptions, StartContainerOptions, UploadToContainerOptions, WaitContainerOptions},
};
use deadpool_postgres::Pool;

use crate::ticket::{self, ExerciseId, Language, TicketError, TicketId};

const TESTING_IMAGE_NAME: &str = env!("TESTING_IMAGE_NAME");
const TESTS_PATH: &str = env!("TESTS_PATH");

use tokio_stream::StreamExt;

pub async fn test_program(ticket_id: TicketId, exercise_id: ExerciseId, db: Pool) {
    let (content, lang);

    loop {
        match ticket::get_content(ticket_id, &db).await {
            Ok(res) => {
                (content, lang) = res;
                break;
            }
            Err(TicketError::WrongTicketId) => {
                return;
            }
            Err(TicketError::DatabaseError) => {
                error!("Error occured at test_program() at point get_content(). Trying again. TicketId = {}", ticket_id);
            }
        }
    }

    let container_name = invoke_container(content, lang, ticket_id, exercise_id).await;
    let wait_container_options: WaitContainerOptions<&str> = WaitContainerOptions::default();

    //TODO: Need to check how exactly it works.
    let _exit_code = crate::DOCKER.wait_container(&container_name, Some(wait_container_options)).next().await;
    
}

async fn tarize_tests(exercise_id: ExerciseId) -> Vec<u8> {
    let tests;
    loop {
        match tokio::fs::read(format!("{}{}/tests.tar", TESTS_PATH, exercise_id)).await {
            Ok(content) => {
                tests = content;
                break;
            }
            Err(error) => {
                error!(
                    "Error occured while reading tar with tests. Trying again. ERROR = {}.",
                    error
                );
            }
        }
    }

    tests
}

async fn tarize_program(content: String, lang: Language) -> Vec<u8> {
    let program: Vec<u8> = content.into();
    let mut tar_program_header = async_tar::Header::new_gnu();
    let _ = tar_program_header.set_path(format!("main{}", lang.extension()));
    let _ = tar_program_header.set_size(program.len().try_into().unwrap());
    let _ = tar_program_header.set_cksum();

    let mut tar_content: Vec<u8> = Vec::new();

    let mut tar_builder = async_tar::Builder::new(&mut tar_content);
    tar_builder
        .append(&tar_program_header, program.as_slice())
        .await;

    // TODO: Can error occur here?
    let _ = tar_builder.into_inner().await;

    tar_content
}

async fn invoke_container(
    content: String,
    lang: Language,
    ticket_id: TicketId,
    exercise_id: ExerciseId,
) -> String {
    let container_name = format!("{}{}", TESTING_IMAGE_NAME, ticket_id);
    let container_name_config = CreateContainerOptions {
        name: &container_name,
    };

    let test_language_env = format!("TEST_LANGUAGE={}", lang.to_string());
    let config = bollard::container::Config {
        image: Some(TESTING_IMAGE_NAME),
        env: Some(vec![test_language_env.as_str()]),
        ..Default::default()
    };

    let created_container;

    loop {
        let _ = crate::DOCKER.remove_container(&container_name, None).await;

        match crate::DOCKER
            .create_container(Some(container_name_config.clone()), config.clone())
            .await
        {
            Ok(res) => {
                created_container = res;
                break;
            }
            Err(error) => {
                error!(
                    "Error occured while creating container. Trying again. ERROR = {}.",
                    error
                );
            }
        }
    }

    let tar_program = tarize_program(content, lang).await;
    let tar_tests = tarize_tests(exercise_id).await;

    let program_upload = UploadToContainerOptions {
        path: "/program",
        ..Default::default()
    };

    let tests_upload = UploadToContainerOptions {
        path: "/tests",
        ..Default::default()
    };

    loop {
        match crate::DOCKER
            .upload_to_container(
                &container_name,
                Some(program_upload.clone()),
                tar_program.clone().into(),
            )
            .await
        {
            Ok(_) => {
                break;
            }
            Err(error) => {
                error!("Error occured while uploading program to container. Trying again. TicketId = {}, Error = {}", ticket_id, error);
            }
        }
    }

    loop {
        match crate::DOCKER
            .upload_to_container(
                &container_name,
                Some(tests_upload.clone()),
                tar_tests.clone().into(),
            )
            .await
        {
            Ok(_) => {
                break;
            }
            Err(error) => {
                error!("Error occured while uploading tests to container. Trying again. TicketId = {}, Error = {}", ticket_id, error);
            }
        }
    }

    while let Err(error) = crate::DOCKER
        .start_container(&container_name, None::<StartContainerOptions<String>>)
        .await
    {
        error!(
            "Error occured while starting container. Trying again. TicketId = {}, Error = {}",
            ticket_id, error
        );
    }

    container_name
}
