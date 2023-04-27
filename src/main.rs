use tokio_cron_scheduler::{JobScheduler, Job};
use serde_derive::{Serialize, Deserialize};
const JOBCONFIGFILE: &str = "jobconfig.json";

#[derive(Serialize, Deserialize, Debug)]
struct JobConfig {
    name: String,
    schedule: String,
    process: String,
    command: String,
}

fn read_jobs(job_config_file: String) -> Vec<JobConfig> {
    // let job_config = format!("{}", job_config_file);
    let job_config = include_str!("../jobconfig.json");
    let jobs: Vec<JobConfig> = serde_json::from_str(&job_config).unwrap_or_else(|_| {
        panic!("Error reading job config file {:?}", job_config_file);
    });
    jobs
}

#[tokio::main]
async fn main() {
    let mut cwd_path = std::env::current_dir().unwrap();
    cwd_path.push(JOBCONFIGFILE);
    let job_config_file = cwd_path.into_os_string().into_string().unwrap();
    let jobs = read_jobs(job_config_file);
    println!("Start Job Scheduler for {:?} jobs", jobs.len());
    let scheduler = JobScheduler::new().await;
    let mut sched = scheduler.unwrap();
    for job in jobs {

        // let mut job_to_run = Job::new_async("1/4 * * * * *", |uuid, mut l| {
        //     Box::pin(async move {
        //         println!("I run async every 4 seconds id {:?}", uuid);
        //         let next_tick = l.next_tick_for_job(uuid).await;
        //         match next_tick {
        //             Ok(Some(ts)) => println!("Next time for 4s is {:?}", ts),
        //             _ => println!("Could not get next tick for 4s job"),
        //         }
        //     })
        // })
        // .unwrap();
        let job_name = job.name;
        let job_schedule = job.schedule.as_str();
        let job_process = job.process;
        let job_args = job.command;
        let job_to_run = Job::new(job_schedule, move |_uuid, _l| {
                
                
                
                println!("Job {:?} is running", job_name);
                let mut command = std::process::Command::new(job_process.clone());
                command.args(job_args.clone().split_whitespace().collect::<Vec<&str>>());
                let output = command.output().expect("failed to execute process");
                println!("Job {:?} is finished", job_name);
                println!("Job {:?} stdout: {:?}", job_name, String::from_utf8_lossy(&output.stdout));
                println!("Job {:?} stderr: {:?}", job_name, String::from_utf8_lossy(&output.stderr));
            
        })
        .unwrap();
        sched.add(job_to_run).await.unwrap();
    }

    #[cfg(feature = "signal")]
    sched.shutdown_on_ctrl_c();

    sched.set_shutdown_handler(Box::new(|| {
        Box::pin(async move {
          println!("Shut down done");
        })
      }));
  
    sched.start().await.unwrap();

    // Wait a while so that the jobs actually run
    tokio::time::sleep(core::time::Duration::from_secs(100)).await;
}
