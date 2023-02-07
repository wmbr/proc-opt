//! Implements Schrage's algorithm (with and without preemptions).
//!
//! Schrage's algorithm is a heuristic algorithm that tries to optimize
//! scheduling of tasks on a single machine. Using Graham's notation the problem
//! can be written as:
//! $$ 1|r_{j}, q_{j}|C_{max} $$
//!
//! It uses a greedy strategy of always scheduling jobs as early as possible.
//! If multiple jobs are available to be scheduled, then it gives priority to jobs 
//! with higher cooldown times (`q`) (or higher processing times (`p`) in case of ties).
//!
//! In the case where preemptions are allowed, the output can be proven to always be an optimal solution.
//!
//! <div align="center">
//!
//! |$j$  |1    |2    |3    |4    |5    |6    |7    |
//! |:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
//! |$r_j$|10   |13   |11   |20   |30   |0    |30   |
//! |$p_j$|5    |6    |7    |4    |3    |6    |2    |
//! |$q_j$|7    |26   |24   |21   |8    |17   |0    |
//!
//! </div>
//!
//! Natural arrangement of jobs:
//!
#![doc=include_str!("../../img/schrage_natural.svg")]
//!
//! Schrage sub-optimized arrangement gives the value of $53$ for the $C_{max}$ value:
//!
#![doc=include_str!("../../img/Schrage.svg")]
//!
//! Compared to the $50$ which is the optimal solution to this particular set of jobs:
//!
#![doc=include_str!("../../img/optimal.svg")]
//!
//!
//!
//!
//!
//!

use crate::jobs::{Job, JobList, JobSchedule};
use std::cmp;
use std::collections::BinaryHeap;

#[derive(Eq)]
struct SchrageJob {
    pub job: Job,
}

impl Ord for SchrageJob {
    // Order according to ascending priority,
    // i.e. by ascending cooldown time, using processing time as tiebreaker.
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        if self.job.cooldown_time == other.job.cooldown_time {
            self.job.processing_time.cmp(&other.job.processing_time)
        } else {
            self.job.cooldown_time.cmp(&other.job.cooldown_time)
        }
    }
}

impl PartialOrd for SchrageJob {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for SchrageJob {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == cmp::Ordering::Equal
    }
}

/// Schrage's algorithm.
/// Schedules jobs on a single machine by a heuristic that attempts to minimze the makespan.
/// Runs in O(n log n) time for n jobs.
///
/// # Arguments
///
/// * `jobs`: A vector of jobs.
///
/// returns: JobList
///
/// # Examples
///
/// ```rust
/// use proc_opt::jobs::{JobList, Job};
/// use proc_opt::schrage::schrage;
/// let expected_result = JobList {
///     jobs: vec![
///         Job::new(0, 6, 17),  // 6
///         Job::new(10, 5, 7),  // 1
///         Job::new(13, 6, 26), // 2
///         Job::new(11, 7, 24), // 3
///         Job::new(20, 4, 21), // 4
///         Job::new(30, 3, 8),  // 5
///         Job::new(30, 2, 0),  // 7
///     ],
/// };
/// let js = vec![
///     Job::new(10, 5, 7),  // 1
///     Job::new(13, 6, 26), // 2
///     Job::new(11, 7, 24), // 3
///     Job::new(20, 4, 21), // 4
///     Job::new(30, 3, 8),  // 5
///     Job::new(0, 6, 17),  // 6
///     Job::new(30, 2, 0),  // 7
/// ];
/// let result = schrage(js);
/// assert_eq!(result, expected_result);
/// assert_eq!(result.c_max(), 53);
/// ```
pub fn schrage(mut jobs: Vec<Job>) -> JobList {
    // sort by descending delivery time
    // because we want to pop the jobs with lowest delivery time first
    jobs.sort_unstable_by_key(|x| cmp::Reverse(x.delivery_time));
    // A list of jobs that in a current moment are ready to run, sorted by descending priority
    let mut ready_to_run = BinaryHeap::new();
    // Time tracking variable
    let mut t: u32 = 0;
    // The final sequence in which the jobs should be run
    let mut pi: JobList = JobList::new(Vec::new());

    // Iterate over all of the jobs until we ran out of them
    while !jobs.is_empty() || !ready_to_run.is_empty() {
        // Find all jobs that are available
        while !jobs.is_empty()
            && jobs.last().unwrap().delivery_time <= t
        {
            ready_to_run.push(
                SchrageJob{ job: jobs.pop().unwrap() }
            );
        }
        // If there are jobs that are ready to run schedule them
        match ready_to_run.pop() {
            Some(sjob) => {
                // Add a job to the final sequence
                pi.jobs.push(sjob.job);
                t += sjob.job.processing_time;
            },
            None => {
                // If there aren't any jobs that can be run,
                // skip to when the nearest job is available.
                // Note that ready_to_run cannot be empty at this point.
                t = jobs.last().unwrap().delivery_time;
            }
        };
    }
    pi
}

/// Schrage's algorithm with preemptions.
/// Schedules jobs on a single machine with preemptions in a way which minimzes the makespan.
/// Runs in O(n log n) time for n jobs.
///
/// # Example
///
/// ```
/// use proc_opt::jobs::{JobSchedule, Job};
/// use proc_opt::schrage::schrage_preemptive;
/// let js = vec![
///     Job::new(0, 27, 78),
///     Job::new(140, 7, 67),
///     Job::new(14, 36, 54),
///     Job::new(133, 76, 5),
/// ];
/// let expected_result = JobSchedule{
///     jobs: vec![
///         Job::new(0, 27, 78),  // 0
///         Job::new(14, 36, 54), // 1
///         Job::new(133, 76, 5), // 2
///         Job::new(140, 7, 67), // 3
///     ],
///     timetable: vec![
///         (0, 0),   // start job 0 at time 0
///         (27, 1),  // start job 1 at time 27
///         (133, 2), // start job 2 at time 133
///         (140, 3), // start job 3 at time 140 (preempting job 2)
///         (147, 2), // continue job 2
///     ],
/// };
/// let result = schrage_preemptive(js);
/// assert_eq!(result, expected_result)
/// ```
pub fn schrage_preemptive(mut jobs: Vec<Job>) -> JobSchedule {
    // sort by ascending delivery time
    jobs.sort_unstable_by_key(|x| x.delivery_time);
    // A list of jobs that in a current moment are ready to run, sorted by descending priority
    // Together with each job we store its index (in `jobs`).
    let mut ready_to_run = BinaryHeap::new();
    // Time tracking variable
    let mut t: u32 = 0;
    // The final timetable
    let mut timetable: Vec<(u32, usize)> = Vec::new();
    // index of the next job to become available
    let mut job_index = 0;
    // Iterate over all of the jobs until we ran out of them
    while job_index < jobs.len() || !ready_to_run.is_empty() {
        // Find all jobs that are available
        while job_index < jobs.len()
            && jobs[job_index].delivery_time <= t
        {
            ready_to_run.push((
                SchrageJob{ job: jobs[job_index] },
                job_index,
            ));
            job_index += 1;
        }
        // If there are jobs that are ready to run schedule them
        match ready_to_run.pop() {
            Some((mut sjob, i)) => {
                // Schedule that job unless it is already scheduled
                if timetable.is_empty() || timetable.last().unwrap().1 != i {
                    timetable.push((t, i));
                }
                t += sjob.job.processing_time;
                // check if a new job arrives before this one is done
                if job_index < jobs.len() {
                    let next_delivery = jobs[job_index].delivery_time;
                    if next_delivery < t {
                        // add this job back to the heap with the remaining processing time
                        sjob.job.processing_time = t - next_delivery;
                        ready_to_run.push((sjob, i));
                        t = next_delivery;
                    }
                }
            },
            None => {
                // If there aren't any jobs that can be run,
                // skip to when the nearest job is available
                // note that job_index < jobs.len() is guaranteed here
                t = jobs[job_index].delivery_time;
            }
        };
    }
    JobSchedule{
        jobs,
        timetable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schrage_ex1() {
        let expected_result = JobList {
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
        let js = vec![
            Job::new(10, 5, 7),  // 1
            Job::new(13, 6, 26), // 2
            Job::new(11, 7, 24), // 3
            Job::new(20, 4, 21), // 4
            Job::new(30, 3, 8),  // 5
            Job::new(0, 6, 17),  // 6
            Job::new(30, 2, 0),  // 7
        ];
        let result = schrage(js);
        assert_eq!(result, expected_result);
        assert_eq!(result.c_max(), 53);
    }

    #[test]
    fn test_schrage_ex2() {
        let expected_result = JobList {
            jobs: vec![
                Job::new(1, 5, 9), // 1
                Job::new(3, 6, 8), // 5
                Job::new(1, 4, 6), // 3
                Job::new(4, 5, 4), // 2
                Job::new(7, 3, 3), // 4
                Job::new(4, 7, 1), // 6
            ],
        };
        let js = vec![
            Job::new(1, 5, 9), // 1
            Job::new(4, 5, 4), // 2
            Job::new(1, 4, 6), // 3
            Job::new(7, 3, 3), // 4
            Job::new(3, 6, 8), // 5
            Job::new(4, 7, 1), // 6
        ];
        let result = schrage(js);
        assert_eq!(result, expected_result);
        assert_eq!(result.c_max(), 32);
    }

    #[test]
    fn test_schrage_ex3() {
        let expected_result = JobList {
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
        let js = vec![
            Job::new(162, 52, 241), // 1
            Job::new(103, 68, 470), // 2
            Job::new(39, 38, 340),  // 3
            Job::new(394, 34, 400), // 4
            Job::new(15, 86, 700),  // 5
            Job::new(144, 73, 536), // 6
            Job::new(51, 52, 403),  // 7
            Job::new(233, 68, 23),  // 8
            Job::new(183, 17, 641), // 9
            Job::new(728, 18, 640), // 10
            Job::new(667, 80, 92),  // 11
            Job::new(57, 21, 76),   // 12
            Job::new(35, 37, 386),  // 13
            Job::new(567, 71, 618), // 14
            Job::new(226, 5, 629),  // 15
            Job::new(162, 80, 575), // 16
            Job::new(588, 45, 632), // 17
            Job::new(556, 23, 79),  // 18
            Job::new(715, 8, 93),   // 19
            Job::new(598, 45, 200), // 20
        ];
        let result = schrage(js);
        assert_eq!(result, expected_result);
        assert_eq!(result.c_max(), 1399);
    }

    #[test]
    fn test_schrage_ex4() {
        let expected_result = JobList {
            jobs: vec![
                Job::new(2, 20, 88),   // 8
                Job::new(5, 14, 125),  // 4
                Job::new(8, 16, 114),  // 5
                Job::new(9, 28, 94),   // 10
                Job::new(70, 4, 93),   // 2
                Job::new(71, 7, 71),   // 6
                Job::new(52, 20, 56),  // 9
                Job::new(52, 1, 56),   // 1
                Job::new(112, 22, 79), // 3
                Job::new(90, 2, 13),   // 7
            ],
        };
        let js = vec![
            Job::new(52, 1, 56),   // 1
            Job::new(70, 4, 93),   // 2
            Job::new(112, 22, 79), // 3
            Job::new(5, 14, 125),  // 4
            Job::new(8, 16, 114),  // 5
            Job::new(71, 7, 71),   // 6
            Job::new(90, 2, 13),   // 7
            Job::new(2, 20, 88),   // 8
            Job::new(52, 20, 56),  // 9
            Job::new(9, 28, 94),   // 10
        ];
        let result = schrage(js);
        assert_eq!(result, expected_result);
        assert_eq!(result.c_max(), 213);
    }

    #[test]
    fn test_sort() {
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
        let expected = vec![
                Job::new(30, 2, 0),  // 7
                Job::new(10, 5, 7),  // 1
                Job::new(30, 3, 8),  // 5
                Job::new(0, 6, 17),  // 6
                Job::new(20, 4, 21), // 4
                Job::new(11, 7, 24), // 3
                Job::new(13, 6, 26), // 2
        ];
        assert_eq!(js.sorted_by_cooldown_time(), expected);
    }

    #[test]
    fn test_schrage_preemptive1() {
        let js = vec![
            Job::new(0, 27, 78),
            Job::new(140, 7, 67),
            Job::new(14, 36, 54),
            Job::new(133, 76, 5),
        ];
        let result = schrage_preemptive(js);
        assert_eq!(result.c_max(), 221)
    }

    #[test]
    fn test_schrage_preemptive2() {
        let js = vec![
            Job::new(8, 68, 984),
            Job::new(747, 60, 1241),
            Job::new(811, 78, 56),
            Job::new(1760, 58, 1558),
            Job::new(860, 16, 319),
            Job::new(1549, 28, 927),
            Job::new(1010, 96, 749),
            Job::new(738, 37, 844),
            Job::new(599, 20, 1170),
            Job::new(446, 53, 1509),
            Job::new(1363, 36, 19),
            Job::new(1277, 14, 685),
            Job::new(1574, 98, 1472),
            Job::new(1886, 3, 1571),
            Job::new(591, 21, 1587),
            Job::new(714, 25, 1490),
            Job::new(1881, 43, 1647),
            Job::new(983, 62, 514),
            Job::new(858, 8, 1215),
            Job::new(634, 7, 587),
            Job::new(784, 14, 1897),
            Job::new(1893, 22, 1878),
            Job::new(308, 89, 1039),
            Job::new(1892, 91, 1815),
            Job::new(1024, 75, 1602),
            Job::new(1467, 59, 378),
            Job::new(1830, 3, 1173),
            Job::new(167, 25, 702),
            Job::new(357, 3, 416),
            Job::new(1739, 68, 71),
            Job::new(1810, 58, 1220),
            Job::new(453, 62, 393),
            Job::new(462, 60, 22),
            Job::new(332, 25, 1512),
            Job::new(845, 96, 1176),
            Job::new(522, 80, 513),
            Job::new(1110, 61, 1854),
            Job::new(484, 32, 570),
            Job::new(545, 91, 274),
            Job::new(64, 67, 74),
            Job::new(90, 9, 1423),
            Job::new(1013, 67, 1567),
            Job::new(1509, 86, 878),
            Job::new(238, 12, 285),
            Job::new(1226, 23, 1767),
            Job::new(83, 35, 22),
            Job::new(626, 97, 63),
            Job::new(6, 24, 707),
            Job::new(507, 31, 1294),
            Job::new(638, 98, 1528),
        ];
        let result = schrage_preemptive(js);
        assert_eq!(result.c_max(), 3820);
    }

    #[test]
    fn test_schrage_preemptive3() {
        let js = vec![
            Job::new(162, 52, 241),
            Job::new(103, 68, 470),
            Job::new(39, 38, 340),
            Job::new(394, 34, 400),
            Job::new(15, 86, 700),
            Job::new(144, 73, 536),
            Job::new(51, 52, 403),
            Job::new(233, 68, 23),
            Job::new(183, 17, 641),
            Job::new(728, 18, 640),
            Job::new(667, 80, 92),
            Job::new(57, 21, 76),
            Job::new(35, 37, 386),
            Job::new(567, 71, 618),
            Job::new(226, 5, 629),
            Job::new(162, 80, 575),
            Job::new(588, 45, 632),
            Job::new(556, 23, 79),
            Job::new(715, 8, 93),
            Job::new(598, 45, 200),
        ];
        let result = schrage_preemptive(js);
        assert_eq!(result.c_max(), 1386);
    }

    #[test]
    fn test_schrage_preemptive4() {
        let js = vec![
            Job::new(219, 5, 276),
            Job::new(84, 13, 103),
            Job::new(336, 35, 146),
            Job::new(271, 62, 264),
            Job::new(120, 33, 303),
            Job::new(299, 14, 328),
            Job::new(106, 46, 91),
            Job::new(181, 93, 97),
            Job::new(263, 13, 168),
            Job::new(79, 60, 235),
        ];
        let result = schrage_preemptive(js);
        assert_eq!(result.c_max(), 641);
    }
}
