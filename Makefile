.PHONY: all setup build run-greedy run-grasp run-vns run-all compare clean

all: setup build

setup:
	@echo "==> Configurando dependências Python e ambiente..."
	python3 -m pip install --upgrade pip
	@if [ -f requirements.txt ]; then pip install -r requirements.txt; fi
	@echo "✔ Setup concluído!"

build:
	@echo "==> Compilando o núcleo de otimização em Rust (Modo Release)..."
	cargo build --release
	@echo "✔ Compilação concluída!"

# Execuções individuais
run-greedy:
	@echo "==> Executando Heurística Gulosa (Greedy)..."
	cargo run --release -- instances/sacoma-3745/graph --metaheuristic one-greedy-construction --executions 5 --max-section-size 0.3 --iterations 1

run-grasp:
	@echo "==> Executando GRASP puro..."
	cargo run --release -- instances/sacoma-3745/graph --metaheuristic grasp --iterations 200 --executions 5

run-vns:
	@echo "==> Executando VNS / VND híbrido (Sua Contribuição com mais iterações)..."
	cargo run --release -- instances/sacoma-3745/graph --metaheuristic vns --iterations 200 --executions 5 --max-section-size 0.3

# Executa todos os experimentos válidos em sequência
run-all: run-greedy run-grasp run-vns
	@echo "✔ Todos os experimentos foram concluídos!"

# Compara os resultados limpos
compare:
	@echo "==> Analisando e comparando os resultados..."
	python3 compare_results.py

clean:
	@echo "==> Limpando artefatos..."
	cargo clean
	rm -rf __pycache__ .pytest_cache *.pyc *.csv results/
	@echo "✔ Limpeza concluída!"