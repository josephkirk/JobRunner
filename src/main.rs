use std::time::Duration;
use std::process::Stdio;
use tokio_cron_scheduler::{JobScheduler, Job};
use serde_derive::{Serialize, Deserialize};
use log::{info, warn, error, debug};
use tokio::{time, task};
use env_logger::Env;

const JOBCONFIGFILE: &str = "jobconfig.json";

#[derive(Serialize, Deserialize, Debug)]
struct JobConfig {
    name: String,
    schedule: String,
    process: String,
    command: String,
}

fn read_jobs(job_config_file: String) -> Vec<JobConfig> {
    // open the job_config_file in read-only mode and return the content as String  
    let job_config_content = std::fs::read_to_string(job_config_file).expect("Something went wrong reading the file");
    // parse the job_config_content as JSON and return a Vec<JobConfig>
    let jobs: Vec<JobConfig> = serde_json::from_str(&job_config_content).unwrap();
    return jobs;
}

#[tokio::main]
async fn main() {
    let env = Env::default()
    .filter_or("MY_LOG_LEVEL", "info")
    .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);
    let mut cwd_path = std::env::current_dir().unwrap();
    // let cwd = cwd_path.clone().into_os_string().into_string().unwrap();
    cwd_path.push(JOBCONFIGFILE);
    let job_config_file = cwd_path.clone().into_os_string().into_string().unwrap();
    let jobs = read_jobs(job_config_file);

    info!("Start Job Scheduler for {:?} jobs", jobs.len());
    let scheduler = JobScheduler::new().await;
    let mut sched = scheduler.unwrap();
    
    for job in jobs {
        //               sec  min   hour   day of month   month   day of week   year
        // let cron_expression = "0   30   9,12,15     1,15       May-Aug  Mon,Wed,Fri  2018/2";
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
        // job_args = job_args
        //     .replace("./",&format!("/{:?}",cwd))
        //     .replace(".\\", &format!("\\{:?}",cwd));

        let job_to_run = Job::new(job_schedule, move |_uuid, _l| {
                info!("Job {:?} is running with: {:?}", job_name, job_args);
                let mut command = std::process::Command::new(job_process.clone())
                    .args(job_args.clone().split_whitespace().collect::<Vec<&str>>())
                    // .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    // .stderr(Stdio::piped())
                    .spawn()
                    .expect("Fail to execute command");
                command.wait().expect("failed to wait on child");
                let output = command.wait_with_output().expect("failed to wait command");
                info!("Job {:?} is finished", job_name);
                debug!("Job {:?} stdout: {:?}", job_name, String::from_utf8_lossy(&output.stdout));
                if output.stderr.len() > 0 {
                    warn!("Job {:?} stderr: {:?}", job_name, String::from_utf8_lossy(&output.stderr));
                }            
        })
        .unwrap();
        sched.add(job_to_run).await.unwrap();
    }

    #[cfg(feature = "signal")]
    sched.shutdown_on_ctrl_c();

    sched.set_shutdown_handler(Box::new(|| {
        Box::pin(async move {
          info!("Shut down done");
        })
      }));
  
    if let Err(e) = sched.start().await {
        error!("Error starting scheduler: {:?}", e)
    };
    // sched.time_till_next_job().await;
    // Setup loop to keep the program alive
    let forever = task::spawn(async {
        let mut interval = time::interval(Duration::from_secs(500));

        loop {
            interval.tick().await;
        }
    });

    forever.await.unwrap();
}
