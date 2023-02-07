//! Implements Job structs used for processing data by scheduling algorithms.
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
use std::{fmt, cmp::max};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Job {
    pub delivery_time: u32,   // r
    pub processing_time: u32, // p
    pub cooldown_time: u32,   // q
}

impl Job {
    pub fn new(delivery_time: u32, processing_time: u32, cooldown_time: u32) -> Job {
        Job {
            delivery_time,
            processing_time,
            cooldown_time,
        }
    }

    #[allow(dead_code)]
    pub fn total_time(&self) -> u32 {
        self.delivery_time + self.processing_time + self.cooldown_time
    }
}

impl fmt::Display for Job {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}, {}, {})",
            self.delivery_time, self.processing_time, self.cooldown_time
        )
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JobList {
    pub jobs: Vec<Job>,
}

impl fmt::Display for JobList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in &self.jobs {
            writeln!(f, "{}", i)?;
        }
        Ok(())
    }
}

impl JobList {
    /// Creates a new [`JobList`].
    pub fn new(jobs: Vec<Job>) -> JobList {
        JobList { jobs }
    }

    /// Returns the Job List sorted by delivery time of this [`JobList`].
    #[allow(dead_code)]
    pub fn sorted_by_delivery_time(&self) -> Vec<Job> {
        let mut by_delivery_time = self.jobs.clone();
        by_delivery_time.sort_by_key(|a| a.delivery_time);
        by_delivery_time
    }

    /// Returns the Job List sorted by processing time of this [`JobList`].
    #[allow(dead_code)]
    pub fn sorted_by_processing_time(&self) -> Vec<Job> {
        let mut by_processing_time = self.jobs.clone();
        by_processing_time.sort_by_key(|a| a.processing_time);
        by_processing_time
    }

    /// Returns the Job List sorted by cooldown time of this [`JobList`].
    #[allow(dead_code)]
    pub fn sorted_by_cooldown_time(&self) -> Vec<Job> {
        let mut by_cooldown_time = self.jobs.clone();
        by_cooldown_time.sort_by_key(|a| a.cooldown_time);
        by_cooldown_time
    }

    /// Returns the makespan of this JobList (if all jobs are executed on a single machine).
    pub fn c_max(&self) -> u32 {
        let mut makespan = 0;
        let mut s = 0; // current time

        for job in self.jobs.iter() {
            if job.delivery_time > s {
                s = job.delivery_time + job.processing_time;
            } else {
                s += job.processing_time;
            }
            makespan = max(makespan, s + job.cooldown_time);
        }
        makespan
    }
}

/// A job execution schedule for a single machine with possible preemptions, assigning to every job one or multiple execution times.
/// If a job is assigned multiple execution times, then it was preempted by some other job in between.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JobSchedule {
    pub jobs: Vec<Job>,
    /// For every time a job is started or resumed, this contains an entry with the time and the job's position in [job_list].
    pub timetable: Vec<(u32, usize)>,
}

impl JobSchedule {
    /// compute the makespan of the schedule, i.e., the time at which all jobs are completed
    pub fn c_max(&self) -> u32 {
        let mut makespan = 0;
        let mut processing_times_remaining : Vec<u32> = self.jobs.iter().map(|job| job.processing_time).collect();
        let mut iter = self.timetable.iter();
        let (mut prev_time, mut prev_index) = match iter.next() {
            Some(x) => *x,
            None => return 0,
        };
        for (time, index) in iter {
            makespan = max(
                makespan,
                prev_time + processing_times_remaining[prev_index] + self.jobs[prev_index].cooldown_time
            );
            processing_times_remaining[prev_index] = 
                processing_times_remaining[prev_index].checked_sub(time - prev_time).unwrap_or(0);
            prev_time = *time;
            prev_index = *index;
        }
        makespan = max(
            makespan,
            prev_time + processing_times_remaining[prev_index] + self.jobs[prev_index].cooldown_time
        );
        makespan
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_c_max_ex1() {
        let js = JobList {
            jobs: vec![
                Job::new(10, 5, 7),  // 1
                Job::new(13, 6, 26), // 2
                Job::new(11, 7, 24), // 3
                Job::new(20, 4, 21), // 4
                Job::new(30, 3, 8),  // 5
                Job::new(0, 6, 17),  // 6
                Job::new(30, 2, 0),  // 7
            ],
        };
        let result = js.c_max();
        assert_eq!(result, 58);
    }

    #[test]
    fn test_c_max_ex2() {
        let js = JobList {
            jobs: vec![
                Job::new(0, 6, 17),  // 6
                Job::new(10, 5, 7),  // 1
                Job::new(13, 6, 26), // 2
                Job::new(11, 7, 24), // 3
                Job::new(20, 4, 21), // 4
                Job::new(30, 3, 8),  // 5
                Job::new(30, 2, 0),  // 7
            ],
        };
        let result = js.c_max();
        assert_eq!(result, 53);
    }

    #[test]
    fn test_c_max_ex3() {
        let js = JobList {
            jobs: vec![
                Job::new(0, 6, 17),  // 6
                Job::new(11, 7, 24), // 3
                Job::new(13, 6, 26), // 2
                Job::new(20, 4, 21), // 4
                Job::new(10, 5, 7),  // 1
                Job::new(30, 3, 8),  // 5
                Job::new(30, 2, 0),  // 7
            ],
        };
        let result = js.c_max();
        assert_eq!(result, 50);
    }

    #[test]
    fn test_c_max_ex4() {
        let js = JobList {
            jobs: vec![
                Job::new(2, 20, 88),   // 8
                Job::new(5, 14, 125),  // 4
                Job::new(8, 16, 114),  // 5
                Job::new(9, 28, 94),   // 10
                Job::new(70, 4, 93),   // 2
                Job::new(71, 7, 71),   // 6
                Job::new(52, 1, 56),   // 1
                Job::new(52, 20, 56),  // 9
                Job::new(112, 22, 79), // 3
                Job::new(90, 2, 13),   // 7
            ],
        };
        let result = js.c_max();
        assert_eq!(result, 213);
    }

    #[test]
    fn test_c_max_ex5() {
        let js = JobList {
            jobs: vec![
                Job::new(15, 86, 700),  // 5
                Job::new(51, 52, 403),  // 7
                Job::new(144, 73, 536), // 6
                Job::new(183, 17, 641), // 9
                Job::new(226, 5, 629),  // 15
                Job::new(162, 80, 575), // 16
                Job::new(103, 68, 470), // 2
                Job::new(394, 34, 400), // 4
                Job::new(35, 37, 386),  // 13
                Job::new(39, 38, 340),  // 3
                Job::new(162, 52, 241), // 1
                Job::new(556, 23, 79),  // 18
                Job::new(567, 71, 618), // 14
                Job::new(588, 45, 632), // 17
                Job::new(598, 45, 200), // 20
                Job::new(728, 18, 640), // 10
                Job::new(715, 8, 93),   // 19
                Job::new(667, 80, 92),  // 11
                Job::new(57, 21, 76),   // 12
                Job::new(233, 68, 23),  // 8
            ],
        };
        let result = js.c_max();
        assert_eq!(result, 1399);
    }

    #[test]
    fn test_schedule_c_max_with_gap() {
        let jobs = vec![
            Job::new(0, 14, 20), // 0
            Job::new(5, 8, 7),   // 1
            Job::new(42, 10, 5), // 2
        ];
        let timetable = vec![
            (0, 0),
            (5, 1),
            (13, 0),
            (42, 2),
        ];
        let schedule = JobSchedule{
            jobs,
            timetable,
        };
        assert_eq!(schedule.c_max(), 42+10+5);
    }

    #[test]
    fn test_schedule_c_max_with_preemption() {
        let jobs = vec![
            Job::new(3, 20, 0), // 0
            Job::new(5, 8, 7),   // 1
        ];
        let timetable = vec![
            (3, 0),
            (16, 1),
            (24, 0),
        ];
        let schedule = JobSchedule{
            jobs,
            timetable,
        };
        assert_eq!(schedule.c_max(), 16+8+7);
    }
}
