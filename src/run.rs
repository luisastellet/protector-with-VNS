use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    time::{Duration, Instant},
};

use anyhow::{anyhow, ensure};
use ordered_float::NotNan;
use rand::SeedableRng;

use crate::{
    Instance, Solution,
    data_mining::MiningParams,
    metaheuristic::{self, DmParams},
    shared::stop_criterion::{IterCriterion, StopCriterion, TimeCriterion},
};

#[derive(Debug, Clone)]
pub struct RunOpts {
    pub executions: u32,
    pub base_seed: u64,
    /// Determines the folder where intermediate solutions are saved
    pub intermediates_folder: PathBuf,
    pub metaheuristic: Metaheuristic,
    pub grasp: metaheuristic::Params,
    pub mining: MiningParams,
    pub max_time: Option<Duration>,
    pub max_iter: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
pub enum Metaheuristic {
    Grasp,
    DmGrasp,
    Greedy,
    Vns, 
}

impl FromStr for Metaheuristic {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "grasp" => Metaheuristic::Grasp,
            "dm-grasp" => Metaheuristic::DmGrasp,
            "one-greedy-construction" => Metaheuristic::Greedy,
            "vns" => Metaheuristic::Vns,
            _ => return Err("failed to parse metaheuristic"),
        })
    }
}

impl Display for Metaheuristic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Metaheuristic::Grasp => f.write_str("grasp"),
            Metaheuristic::DmGrasp => f.write_str("dm-grasp"),
            Metaheuristic::Greedy => f.write_str("one-greedy-construction"),
            Metaheuristic::Vns => f.write_str("vns"),
        }
    }
}

#[derive(Debug)]
pub struct Run<'a> {
    solutions: Vec<(u32, Solution<'a>, Duration)>,
    opts: RunOpts,
}

impl<'a> Run<'a> {
    pub fn new(instance: &'a Instance, opts: RunOpts) -> anyhow::Result<Self> {
        ensure!(
            opts.max_iter.xor(opts.max_time.map(|_| 0)).is_some(),
            "You must set exactly one stop criterion: time or iter."
        );

        let mut best = NotNan::<f64>::default();
        let _ = std::fs::remove_dir_all(&opts.intermediates_folder);
        let _ = std::fs::remove_dir_all("tmp");

        let mut export_intermediate = |solution: &Solution| {
            if solution.score() > best {
                best = solution.score();
                export(solution, &opts.intermediates_folder, best)?;
            }
            Some(())
        };

        let solutions = (1..=opts.executions)
            .flat_map(|exec_number| {
                let start = Instant::now();

                let seed = opts.base_seed + exec_number as u64;
                let mut rng = rand_pcg::Pcg64::seed_from_u64(seed);

                let (mut time_criterion, mut iter_criterion);

                let stop_criterion: &mut dyn StopCriterion = if let Some(max_time) = opts.max_time {
                    time_criterion = TimeCriterion::new(max_time);
                    &mut time_criterion
                } else {
                    iter_criterion = IterCriterion::new(opts.max_iter.unwrap());
                    &mut iter_criterion
                };

                let mut historico_convergencia: Vec<(usize, f64, Duration)> = Vec::new();
                let mut iter_contador = 0;

                let mut export_intermediate = |solution: &Solution| {
                    if solution.score() > best {
                        best = solution.score();
                        export(solution, &opts.intermediates_folder, best)?;
                    }
                    
                    let elapsed = start.elapsed();
                    historico_convergencia.push((iter_contador, *solution.score().as_ref(), elapsed));
                    iter_contador += 1;

                    Some(())
                };

                let solution = match opts.metaheuristic {
                    Metaheuristic::Grasp => metaheuristic::multistart_grasp(
                        instance,
                        &opts.grasp,
                        stop_criterion,
                        &mut rng,
                        &mut export_intermediate,
                    ),
                    Metaheuristic::DmGrasp => metaheuristic::dm_grasp(
                        instance,
                        DmParams {
                            grasp: opts.grasp,
                            mining: opts.mining,
                        },
                        stop_criterion,
                        &mut rng,
                        &mut export_intermediate,
                    ),
                    Metaheuristic::Greedy => {
                        metaheuristic::one_greedy_construction(instance, &opts.grasp, &mut rng)
                    }
                    Metaheuristic::Vns => metaheuristic::vns_heuristic(
                        instance,
                        &opts.grasp,
                        stop_criterion,
                        &mut rng,
                        &mut export_intermediate,
                    ),
                }?;

                let duration = start.elapsed();

                let _ = salvar_log_convergencia(&historico_convergencia, &opts.metaheuristic.to_string(), exec_number);

                log::info!(
                    "Execution {} done. Time: {:?}, Score: {:.2}",
                    exec_number,
                    duration,
                    solution.score(),
                );

                Some((exec_number, solution, duration))
            })
            .collect::<Vec<_>>();
        
        let _ = fs::create_dir_all("results");
        let resumo_path = format!("results/resumo_{}.csv", opts.metaheuristic);
        let mut resumo_file = File::create(&resumo_path)?;
        writeln!(resumo_file, "execucao,melhor_score,tempo_segundo")?;
        for (exec_num, sol, dur) in &solutions {
            writeln!(resumo_file, "{},{},{:.4}", exec_num, sol.score(), dur.as_secs_f64())?;
        }
        println!("Resumo salvo com sucesso em: {}", resumo_path);

        if solutions.is_empty() {
            return Err(anyhow!("Couldn't build any solution"));
        }

        Ok(Self { solutions, opts })
    }

    pub fn solutions(&self) -> &[(u32, Solution<'a>, Duration)] {
        &self.solutions
    }

    pub fn opts(&self) -> &RunOpts {
        &self.opts
    }
}

use std::fs::File;
use std::io::Write;

fn salvar_log_convergencia(
    historico: &[(usize, f64, Duration)],
    metaheuristic_name: &str,
    exec_number: u32,
) -> std::io::Result<()> {
    let dir_path = format!("results/{}", metaheuristic_name);
    fs::create_dir_all(&dir_path)?;

    let arquivo_path = format!("{}/exec_{}.csv", dir_path, exec_number);
    println!("Salvando log em: {:?}", std::path::Path::new(&arquivo_path).canonicalize().unwrap_or_else(|_| PathBuf::from(&arquivo_path)));

    let mut file = File::create(&arquivo_path)?;
    writeln!(file, "iteracao,melhor_score,tempo_segundos")?;

    let mut tempo_acumulado = 0.0;
    for (it, score, duration) in historico {
        tempo_acumulado += duration.as_secs_f64();
        writeln!(file, "{},{},{:.4}", it, score, tempo_acumulado)?;
    }
    Ok(())
}

fn export(solution: &Solution, folder: &Path, best: NotNan<f64>) -> Option<()> {
    let _ = fs::create_dir(folder);
    let file_name = format!("{:.2}.txt", best.as_ref());
    let mut file = fs::File::create(folder.join(file_name)).ok()?;
    solution.export(&mut file).ok()?;
    Some(())
}
