# Otimização de Rotas de Patrulha Policial: GRASP estendido com VNS

Este projeto estende o trabalho [Optimizing Preventive Police Patrol Routes in Brazilian Cities Using a GRASP-Based Routing Heuristic](https://journals-sol.sbc.org.br/index.php/jisa/article/view/6805), cujo repositório base está em https://github.com/UFFeScience/protector. Foi desenvolvido na disciplina de **Pesquisa Operacional** (UFF).

---

## Contexto e contribuição

Em problemas de roteamento de rondas de viaturas (*Police Patrol Routing Problem*), o GRASP combina uma construção gulosa-aleatória com uma busca local. Esse esquema gera boas soluções rapidamente, mas cada reinício é independente e a busca local tende a parar em **ótimos locais**, principalmente em grafos urbanos densos com muitos pontos críticos.

### O que foi adicionado

1. **VNS (*Variable Neighborhood Search*):** uma nova metaheurística, `vns_heuristic`, em `src/metaheuristic.rs`. Em vez de reiniciar a construção a cada iteração, ela parte de uma única solução inicial (construída e melhorada pela busca local) e a refina de forma iterativa.
2. **Operador de perturbação (*shake*):** dado o nível de vizinhança `k` (de 1 a `k_max = 3`), o `shake` perturba a solução corrente inserindo até `k` laços (`add_loop`) em rotas escolhidas aleatoriamente (zonais, globais e VIP). Quanto maior o `k`, mais forte a perturbação.
3. **Busca local reaproveitada:** após cada perturbação, aplica-se a mesma busca local do GRASP, com as vizinhanças `ExpandRoute`, `AddLoop` e `RepositionUnit`, que podem ser habilitadas individualmente.

### Lógica de aceitação

```
current ← construção + busca local
best ← current
enquanto não atingir o critério de parada:
    k ← 1
    enquanto k ≤ k_max:
        candidate ← shake(cópia de current, k)
        candidate ← busca_local(candidate)
        se score(candidate) > score(current):
            current ← candidate
            atualiza best, se for o caso
            k ← 1
        senão:
            k ← k + 1
```

Se o candidato melhora a solução corrente, a busca volta à perturbação mais fraca (`k = 1`). Caso contrário, a perturbação é intensificada (`k + 1`).

---

## Heurísticas disponíveis

| Comando | Heurística |
|---|---|
| `make run-greedy` | Construção gulosa (One-Greedy) |
| `make run-grasp` | GRASP multi-start tradicional |
| `make run-dm` | DM-GRASP (com mineração de dados) |
| `make run-vns` | VNS (GRASP estendido com *shake*) |
| `make run-all` | Todas as heurísticas em sequência |

---

## Estrutura do repositório

- `src/metaheuristic.rs`: `multistart_grasp`, `vns_heuristic`, `dm_grasp`, `one_greedy_construction` e `shake`.
- `src/solution/mod.rs`: representação da solução (rotas VIP, zonais, globais, unidades fixas, cobertura e score).
- `src/solution/local_search/`: busca local e seus movimentos (`section_moves`, `loop_move`, `reposition_unit`).
- `src/run.rs`: execução das heurísticas e exportação dos resultados.
- `Makefile`: automação de compilação e execução.

---

## Como executar

Pré-requisitos: **Rust/Cargo** e **Python 3**.

1. **Configuração inicial** (dependências Python e verificação do ambiente Rust):

   ```bash
   make setup
   ```

2. **Compilar** (modo release):

   ```bash
   make build
   ```

3. **Executar as heurísticas** (isoladamente ou todas de uma vez):

   ```bash
   make run-greedy   # guloso
   make run-grasp    # GRASP tradicional
   make run-dm       # DM-GRASP
   make run-vns      # VNS
   make run-all      # todas em sequência
   ```

4. **Script auxiliar em Python**:

   ```bash
   make run-py
   ```

5. **Limpar arquivos temporários** (artefatos de compilação, caches e logs):

   ```bash
   make clean
   ```

---

## Coleta de métricas

O módulo de rastreamento registra o histórico de iterações, o melhor custo encontrado e o tempo decorrido, salvando tudo em `convergence_log.csv`. Esse arquivo serve de base para os gráficos de convergência do relatório.

## Saídas dos experimentos

Cada execução de `vns`, `grasp`, `dm-grasp` ou `one-greedy-construction` grava os arquivos abaixo em `results/`.

### `results/resumo_<heurística>.csv`

Uma linha por execução, com a solução final:

| coluna | descrição |
|---|---|
| `execucao` | número da execução (a semente é `base_seed + execucao`) |
| `melhor_score` | score da solução final da execução |
| `tempo_segundos` | tempo total da execução, em segundos |
| `violacoes_zona` | número de arestas de rotas de zona que saem da zona (deve ser `0`) |

### `results/<heurística>/exec_N.csv`

Log de convergência da execução `N`. Uma linha por solução avaliada após a busca local:

| coluna | descrição |
|---|---|
| `iteracao` | contador de soluções avaliadas (a `0` é a solução inicial) |
| `score_candidata` | score da solução avaliada naquela iteração |
| `melhor_score` | melhor score encontrado **nesta execução** até aquela iteração |
| `tempo_segundos` | tempo decorrido desde o início da execução, em segundos |

Use `melhor_score` para curvas de convergência e `score_candidata` para ver a oscilação da busca. Na `one-greedy-construction` o `exec_N.csv` sai só com o cabeçalho, porque ela não tem busca iterativa.
