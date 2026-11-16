# Police Route Optimization: Extending GRASP with Variable Neighborhood Search (VNS)

Esta implementação consiste em uma extensão e melhoria sobre o trabalho original apresentado no artigo [Optimizing Preventive Police Patrol Routes in Brazilian Cities Using a GRASP-Based Routing Heuristic](https://journals-sol.sbc.org.br/index.php/jisa/article/view/6805), cujo repositório base pode ser acessado em https://github.com/UFFeScience/protector. O projeto foi desenvolvido no âmbito da disciplina de **Pesquisa Operacional**.

---

## Contexto e Justificativa da Contribuição

Em problemas de roteamento para rondas de viaturas (*Police Patrol Routing Problem*), abordagens clássicas baseadas puramente em GRASP utilizam uma fase construtiva gulosa-aleatória seguida de uma busca local estática (como o operador 2-opt simples). Embora eficientes para encontrar soluções iniciais rápidas, esses métodos frequentemente estagnam em **mínimos locais**, especialmente em instâncias de alta densidade urbana com grafos complexos e múltiplos pontos de interesse crítico.

### O que foi alterado e qual a contribuição?
1. **Mecanismo de Vizinhança Múltipla (VNS):** Em vez de aplicar uma única rotina de busca local estática como no trabalho original, o algoritmo agora alterna sistematicamente entre diferentes estruturas de vizinhança ($N_k, k = 1, \dots, k_{max}$):
   * **$N_1$ (Swap):** Troca a ordem de dois pontos de patrulha na rota.
   * **$N_2$ (Relocate / Reinsertion):** Realoca um setor crítico de um ponto para outro da rota da viatura.
   * **$N_3$ (2-opt Inter-segment):** Inverte uma subsequência de nós para desfazer cruzamentos de rotas.
2. **Estratégia de Escape de Mínimos Locais (*Shake*):** Quando a busca local esgota a exploração de uma vizinhança sem ganho de aptidão, o operador de perturbação altera estocasticamente a solução atual com base no parâmetro $k$, permitindo explorar novas regiões promissoras do espaço de busca sem perder a viabilidade da rota.

---

## Estrutura do Repositório

- `model.py`: Definição do grafo viário, matriz de custos/distâncias e cálculo da função objetivo (cobertura de criticidade vs. custo de deslocamento).
- `grasp_vns.py`: Implementação do algoritmo híbrido (Construção Gulosa-Aleatória + VNS com múltiplas estruturas de vizinhança).
- `main.py`: Script de execução com instâncias de exemplo e logs de convergência.

## Como Executar

Certifique-se de ter o Python instalado!

Para facilitar a execução de todo o projeto (Rust + Python), utilize o Makefile incluído na raiz do repositório.

1. Configuração Inicial
Instala as dependências do Python e verifica o ambiente Rust/Cargo:

```Bash
make setup
```

2. Compilar o Projeto
Compila o motor de otimização em Rust no modo de liberação (release) para máxima performance:

```Bash
make build
```

3. Execução das Heurísticas para Coleta de Métricas
Você pode executar cada abordagem isoladamente ou rodar todas em sequência para gerar os dados comparativos:

- Rodar apenas o GRASP Tradicional:

```Bash
make run-grasp
```

- Rodar apenas o DM-Grasp (com Mineração de Dados):

```Bash
make run-dm
```

- Rodar apenas a nova abordagem VNS / VND híbrida:

```Bash
make run-vns
```

- Rodar o algoritmo guloso

```Bash
make run-greedy
```

- Rodar todas as três heurísticas em sequência de uma só vez:

```Bash
make run-all
````

1. Executar o Script Auxiliar em Python

```Bash
make run-py
````

5. Limpar Arquivos Temporários
Remove artefatos de compilação, caches e logs gerados:

```Bash
make clean
```

O sistema conta com um módulo de rastreamento de desempenho que registra o histórico de iterações, os melhores custos de solução encontrados e o tempo de execução decorrido, salvando-os em um arquivo `convergence_log.csv` para avaliação e geração de gráficos no seu relatório.
