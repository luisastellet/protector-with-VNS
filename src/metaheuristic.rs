use log::debug;
use ordered_float::NotNan;
use rand::Rng;

use crate::{
    Instance, Percentage, Solution,
    data_mining::{self, MiningParams},
    local_search,
    shared::{
        elite_set::{EliteSet, Score},
        stop_criterion::StopCriterion,
    },
    solution,
};

#[derive(Debug, Clone, Copy)]
pub struct Params {
    pub construction: solution::Params,
    pub local_search: local_search::Params,
}

pub fn multistart_grasp<'a>(
    instance: &'a Instance,
    params: &Params,
    stop_criterion: &mut dyn StopCriterion,
    rng: &mut impl Rng,
    mut after_local_search: impl FnMut(&Solution) -> Option<()>,
) -> Option<Solution<'a>> {
    let mut best = None::<Solution>;
    for number in 0.. {
        if stop_criterion.should_stop() {
            break;
        }

        let mut solution = match Solution::new(instance, &params.construction, rng) {
            Some(s) => s,
            None => continue,
        };

        debug!("Construction score: {}", solution.score());
        local_search(&mut solution, &params.local_search, rng);
        debug!("Local search score: {}", solution.score());

        after_local_search(&solution)?;

        if let Some(ref current) = best {
            if solution.score() > current.score() {
                debug!("Better solution found ({}): {}", number, solution.score());
                best = Some(solution);
            }
        } else {
            best = Some(solution);
        };
        stop_criterion.update();
    }
    best
}


pub fn vns_heuristic<'a>(
    instance: &'a Instance,
    params: &Params,
    stop_criterion: &mut dyn StopCriterion,
    rng: &mut impl Rng,
    mut after_local_search: impl FnMut(&Solution) -> Option<()>,
) -> Option<Solution<'a>> {
    let mut current = Solution::new(instance, &params.construction, rng)?;

    local_search(&mut current, &params.local_search, rng);

    after_local_search(&current)?;

    let mut best = current.clone();

    let k_max = 3;
    let mut iteration = 0;

    while !stop_criterion.should_stop() {
        let mut k = 1;

        while k <= k_max && !stop_criterion.should_stop() {
            iteration += 1;

            let mut candidate = current.clone();

            local_search::shake(
                &mut candidate,
                &params.local_search,
                k,
                rng,
            );

            local_search(
                &mut candidate,
                &params.local_search,
                rng,
            );

            debug!(
                "VNS iteration {} | k = {} | candidate score = {}",
                iteration,
                k,
                candidate.score()
            );

            after_local_search(&candidate)?;

            if candidate.score() > current.score() {
                current = candidate;

                if current.score() > best.score() {
                    best = current.clone();

                    debug!(
                        "New best solution found: {}",
                        best.score()
                    );
                }

                k = 1;
            } else {
                k += 1;
            }

            stop_criterion.update();
        }
    }

    Some(best)
}


impl Score for Solution<'_> {
    type Value = NotNan<f64>;

    fn score(&self) -> Self::Value {
        self.score()
    }
}

impl PartialEq for Solution<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.score() == other.score()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DmParams {
    pub grasp: Params,
    pub mining: MiningParams,
}

pub fn dm_grasp<'a>(
    instance: &'a Instance,
    DmParams {
        grasp: params,
        mining: mining_params,
    }: DmParams,
    stop_criterion: &mut dyn StopCriterion,
    rng: &mut impl Rng,
    mut after_local_search: impl FnMut(&Solution) -> Option<()>,
) -> Option<Solution<'a>> {
    let mut elite_set = EliteSet::<Solution>::new(10, NotNan::default());
    let mut current_best_score = NotNan::default();

    let half = Percentage::try_from(0.5).expect("percentage should be valid");
    for number in 0.. {
        if stop_criterion.progress() >= half {
            break;
        }

        let Some(mut solution) = Solution::new(instance, &params.construction, rng) else {
            continue;
        };

        debug!("Construction score: {}", solution.score());
        local_search(&mut solution, &params.local_search, rng);
        debug!("Local search score: {}", solution.score());

        after_local_search(&solution)?;

        let _ = elite_set.try_insert(solution);

        if let Some(best) = elite_set.best() {
            if best.score() > current_best_score {
                debug!("Better solution found ({}): {}", number, best.score());
                current_best_score = best.score();
            }
        }
        stop_criterion.update();
    }

    let patterns = data_mining::mine_elites(&elite_set, mining_params);

    let mut best = elite_set.best().cloned();

    for number in stop_criterion.current_iter().. {
        if stop_criterion.should_stop() {
            break;
        }

        let Some(mut solution) =
            Solution::new_with_patterns(instance, &params.construction, &patterns, rng)
        else {
            continue;
        };

        debug!("Construction score: {}", solution.score());
        local_search(&mut solution, &params.local_search, rng);
        debug!("Local search score: {}", solution.score());

        after_local_search(&solution)?;

        if let Some(ref current) = best {
            if solution.score() > current.score() {
                debug!("Better solution found ({}): {}", number, solution.score());
                best = Some(solution);
            }
        } else {
            best = Some(solution);
        };
        stop_criterion.update();
    }

    best
}

pub fn one_greedy_construction<'a>(
    instance: &'a Instance,
    params: &Params,
    rng: &mut impl Rng,
) -> Option<Solution<'a>> {
    Solution::new(instance, &params.construction, rng)
}