use crate::error::ManagerError;
use crate::pipelines::pipeline::PipelineJob;
use crate::pipelines::pipeline::TransformationStep;
use k8s_openapi::api::batch::v1::Job;
use serde_json::json;
use std::env;

pub fn create_job_template(
    pipeline_job: &PipelineJob,
    step: &TransformationStep,
) -> Result<Job, ManagerError> {
    let name = format!("{}-{}", pipeline_job.pipeline_hash, step.name);
    let outputbucket = format!("{}-output", &name);

    let job: Job = serde_json::from_value(json!({
        "apiVersion": "batch/v1",
        "kind": "Job",
        "metadata": { "name": name, "labels": {"pipeline_hash": pipeline_job.pipeline_hash} },
        "spec": {
            "parallelism": 1,
            "template": {
                "metadata": {
                    "name": name
                },
                "spec": {
                    "volumes": [
                        {"name": "data-volume", "emptyDir": {}}
                    ],
                    "containers": [{
                        "name": "sidecar",
                        "image": env::var("SIDECAR_IMAGE").unwrap(),
                        "env": [
                            {"name": "DATA_VOLUME_PATH", "value": "/data-volume"},
                            {"name": "ITERUM_NAME", "value": &pipeline_job.name},
                            {"name": "PIPELINE_HASH", "value": &pipeline_job.pipeline_hash},

                            {"name": "MINIO_URL", "value": env::var("MINIO_URL").unwrap()},
                            {"name": "MINIO_ACCESS_KEY", "value": env::var("MINIO_ACCESS_KEY").unwrap()},
                            {"name": "MINIO_SECRET_KEY", "value": env::var("MINIO_SECRET_KEY").unwrap()},
                            {"name": "MINIO_USE_SSL", "value": env::var("MINIO_USE_SSL").unwrap()},
                            {"name": "MINIO_OUTPUT_BUCKET", "value": &outputbucket},

                            {"name": "MQ_BROKER_URL", "value": env::var("MQ_BROKER_URL").unwrap()},
                            {"name": "MQ_OUTPUT_QUEUE", "value": &step.output_channel},
                            {"name": "MQ_INPUT_QUEUE", "value": &step.input_channel},

                            {"name": "TRANSFORMATION_STEP_INPUT", "value": "tts.sock"},
                            {"name": "TRANSFORMATION_STEP_OUTPUT", "value": "fts.sock"},
                        ],
                        "volumeMounts": [{
                            "name": "data-volume",
                            "mountPath": "/data-volume"
                        }]
                    },
                    {
                        "name": "transformation-step",
                        "image": step.image,
                        "env": [
                            {"name": "DATA_VOLUME_PATH", "value": "/data-volume"},
                            {"name": "TRANSFORMATION_STEP_INPUT", "value": "/data-volume/tts.sock"},
                            {"name": "TRANSFORMATION_STEP_OUTPUT", "value": "/data-volume/fts.sock"},
                        ],
                        "volumeMounts": [{
                            "name": "data-volume",
                            "mountPath": "/data-volume"
                        }]
                    }],
                    "restartPolicy": "OnFailure"
                }
            }
        }
    }))?;
    Ok(job)
}