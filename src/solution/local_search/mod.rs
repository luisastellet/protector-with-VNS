mod loop_move;
mod path;
mod reposition_unit;
mod section_moves;

use clap::ValueEnum;
use petgraph::graph::{EdgeIndex, NodeIndex};
use rand::Rng;

use crate::{Percentage, Solution, graph::vertex::Vertex};

// Local Search moves
use loop_move::add_loop;
use reposition_unit::reposition_unit;
use section_moves::expand_route;

use self::section_moves::RemovalParams;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Neighborhood {
    ExpandRoute,
    AddLoop,
    RepositionUnit,
}

impl std::fmt::Display for Neighborhood {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no skipped values")
            .get_name()
            .fmt(f)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EnabledNeighborhoods {
    pub expand_route: bool,
    pub add_loop: bool,
    pub reposition_unit: bool,
}

impl EnabledNeighborhoods {
    pub fn from_list(neighborhoods: &[Neighborhood]) -> Self {
        Self {
            expand_route: neighborhoods.contains(&Neighborhood::ExpandRoute),
            add_loop: neighborhoods.contains(&Neighborhood::AddLoop),
            reposition_unit: neighborhoods.contains(&Neighborhood::RepositionUnit),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Params {
    pub max_edges_between_loop: Percentage,
    pub max_section_size: Percentage,
    pub neighborhoods: EnabledNeighborhoods,
}

pub fn local_search(solution: &mut Solution<'_>, params: &Params, rng: &mut impl Rng) {
    let no_restriction = |_, _| false;
    let no_special = |_| false;
    let graph = solution.graph;
    loop {
        for (zone, routes) in solution.zones.iter_mut() {
            for route in routes.iter_mut() {
                let same_zone_restriction =
                    |_, node: NodeIndex| graph[node].weight.zone() != *zone;
                if params.neighborhoods.expand_route {
                    let mut removal_params = RemovalParams {
                        solution_coverage: &mut solution.coverage,
                        route,
                        path_restriction: &same_zone_restriction,
                        special_edge: &no_special,
                    };
                    expand_route(&mut removal_params, params.max_section_size, rng);
                }
                if params.neighborhoods.add_loop {
                    add_loop(
                        route,
                        &mut solution.coverage,
                        &same_zone_restriction,
                        params,
                        rng,
                    );
                }
            }
        }
        for route in solution.globals.iter_mut() {
            if params.neighborhoods.expand_route {
                let mut removal_params = RemovalParams {
                    solution_coverage: &mut solution.coverage,
                    route,
                    path_restriction: &no_restriction,
                    special_edge: &no_special,
                };
                expand_route(&mut removal_params, params.max_section_size, rng);
            }
            if params.neighborhoods.add_loop {
                add_loop(route, &mut solution.coverage, &no_restriction, params, rng);
            }
        }
        for (vips, route) in solution.vips.iter_mut() {
            let vip_restriction = |edge: EdgeIndex| {
                vips.edges.contains(&edge)
                    || graph[edge]
                        .weight
                        .reverse
                        .filter(|reverse| vips.edges.contains(reverse))
                        .is_some()
            };
            if params.neighborhoods.expand_route {
                let mut removal_params = RemovalParams {
                    solution_coverage: &mut solution.coverage,
                    route,
                    path_restriction: &no_restriction,
                    special_edge: &vip_restriction,
                };
                expand_route(&mut removal_params, params.max_section_size, rng);
            }
            if params.neighborhoods.add_loop {
                add_loop(route, &mut solution.coverage, &no_restriction, params, rng);
            }
        }
        if params.neighborhoods.reposition_unit {
            for unit in solution.fixed_units.iter_mut() {
                *unit = reposition_unit(solution.graph, &mut solution.coverage, *unit);
            }
        }
        let old_score = solution.score();
        solution.refresh_score();
        if old_score == solution.score() {
            break;
        }
    }
}

/// Identifica uma rota da solução sem manter empréstimo mutável sobre ela.
#[derive(Debug, Clone, Copy)]
enum RouteSlot {
    /// (índice da zona em `solution.zones`, índice da rota dentro da zona)
    Zone(usize, usize),
    Global(usize),
    Vip(usize),
}

/// Perturbação do VNS: tenta inserir `k` laços em rotas sorteadas.
///
/// Aplica as MESMAS restrições da busca local:
/// - rotas de zona usam `same_zone_restriction` (o laço não pode sair da zona);
/// - rotas globais e de VIP usam `no_restriction`.
pub fn shake(solution: &mut Solution<'_>, params: &Params, k: usize, rng: &mut impl Rng) {
    let no_restriction = |_, _| false;
    let graph = solution.graph;

    let mut slots = Vec::new();
    for (zi, (_, routes)) in solution.zones.iter().enumerate() {
        for ri in 0..routes.len() {
            slots.push(RouteSlot::Zone(zi, ri));
        }
    }
    for gi in 0..solution.globals.len() {
        slots.push(RouteSlot::Global(gi));
    }
    for vi in 0..solution.vips.len() {
        slots.push(RouteSlot::Vip(vi));
    }
    if slots.is_empty() {
        return;
    }

    let mut inserted = 0;
    let max_attempts = k * 10;
    for _ in 0..max_attempts {
        if inserted >= k {
            break;
        }
        let slot = slots[rng.gen_range(0..slots.len())];
        let ok = match slot {
            RouteSlot::Zone(zi, ri) => {
                let (zone, routes) = solution
                    .zones
                    .iter_mut()
                    .nth(zi)
                    .expect("zone index should be valid");
                let same_zone_restriction =
                    |_, node: NodeIndex| graph[node].weight.zone() != *zone;
                add_loop(
                    &mut routes[ri],
                    &mut solution.coverage,
                    &same_zone_restriction,
                    params,
                    rng,
                )
            }
            RouteSlot::Global(gi) => add_loop(
                &mut solution.globals[gi],
                &mut solution.coverage,
                &no_restriction,
                params,
                rng,
            ),
            RouteSlot::Vip(vi) => {
                let (_, route) = solution
                    .vips
                    .iter_mut()
                    .nth(vi)
                    .expect("vip index should be valid");
                add_loop(route, &mut solution.coverage, &no_restriction, params, rng)
            }
        };
        if ok {
            inserted += 1;
        }
    }
    solution.refresh_score();
}
