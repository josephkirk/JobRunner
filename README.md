# Simple Job runner

Simple job runner from json config file using tokio-cron-scheduler

json config file format:

```json
[
    {
        "name": "Job1",
        "schedule": "1/5 * * * * *",
        "process": "cmd",
        "command": "/C echo Hello World"
    },
]
```
**schedule is in cron expression format**
"sec  min   hour   day of month   month   day of week   year"